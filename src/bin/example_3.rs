use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    sync::{atomic::AtomicU64, Arc},
};

fn main() {
    let mut args = std::env::args().skip(1);
    let mode = args.next().expect("want address argument");

    println!("{}", mode);

    if let Ok(addr) = mode.parse() {
        let num_conns = args
            .next()
            .expect("want num_conns argument")
            .parse()
            .expect("want number");
        client(addr, num_conns);
        return;
    }

    if let Ok(port) = mode.parse() {
        server(port);
        return;
    }
}

fn client(addr: SocketAddr, num_conns: u32) {
    let mut conns = Vec::new();
    const WORK_AMT: u32 = 500_000;
    for t in 0..num_conns {
        let start = t * num_conns;
        let end = t * num_conns + (WORK_AMT / num_conns);
        let cn = TcpStream::connect(addr).expect("connect");
        conns.push((start, end, cn));
    }

    let mut jhs = Vec::new();
    for (start, end, mut cn) in conns {
        let jh = std::thread::spawn(move || {
            let mut durs = Vec::with_capacity((WORK_AMT / num_conns) as usize);
            let mut buf = [0u8; std::mem::size_of::<u64>()];
            for i in start..end {
                let req_start = std::time::Instant::now();
                cn.write_all(&i.to_le_bytes()).expect("write");
                cn.read_exact(&mut buf).expect("read");
                let req_end = std::time::Instant::now();
                let req_duration = req_end - req_start;
                durs.push(req_duration);
            }

            durs.sort();
            let p95 = durs[durs.len() * 95 / 100];
            (u64::from_le_bytes(buf), p95)
        });
        jhs.push(jh);
    }

    let mut sum = 0;
    let mut p95s = Vec::with_capacity(num_conns as _);
    for c in jhs {
        let (conn_sum, p95) = c.join().expect("join");
        sum = conn_sum.max(sum);
        p95s.push(p95);
    }

    p95s.sort();
    let median_conn_p95 = p95s[p95s.len() / 2];
    println!(
        "sum: {} median per-connection p95 latency: {:?}",
        sum, median_conn_p95
    );
}

fn server(port: u16) {
    let srv = TcpListener::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port)))
        .expect("bind");

    let sum = Arc::new(AtomicU64::new(0));
    for cn in srv.incoming() {
        let mut cn = cn.expect("accept new connection");
        //let peer = cn.peer_addr().expect("cn peer addr");
        let sum = Arc::clone(&sum);
        //println!("connected to {:?}", peer);

        std::thread::spawn(move || {
            let mut buf = [0u8; 4];
            loop {
                match cn.read_exact(&mut buf) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("exiting on error {:?}", e);
                        break;
                    }
                }

                let curr = sum.fetch_add(
                    u32::from_le_bytes(buf) as u64,
                    std::sync::atomic::Ordering::AcqRel,
                );
                cn.write_all(&curr.to_le_bytes()).expect("write");
            }

            // 2: the original one and the clone in this thread
            if Arc::strong_count(&sum) == 2 {
                println!("exiting");
                std::process::exit(0);
            }
        });
    }
}
