pub fn do_mystery_work(amt: usize) {
    let k = 2350845.545;
    for i in 0..amt {
        std::hint::black_box(f64::sqrt(k * i as f64));
    }
}
