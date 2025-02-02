use perf_examples::*;

fn main() {
    let start = std::time::Instant::now();

    loop {
        do_mystery_work(9001);
        if start.elapsed() > std::time::Duration::from_secs(10) {
            break;
        }
    }
}
