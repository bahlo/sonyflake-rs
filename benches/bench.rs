use bencher::{Bencher, benchmark_group, benchmark_main};
use sonyflake::{Sonyflake, decompose};

fn bench_new(b: &mut Bencher) {
    b.iter(Sonyflake::new);
}

fn bench_next_id(b: &mut Bencher) {
    let sf = Sonyflake::new().expect("Could not create Sonyflake");
    b.iter(|| sf.next_id());
}

fn bench_decompose(b: &mut Bencher) {
    let sf = Sonyflake::new().expect("Could not create Sonyflake");
    let next_id = sf.next_id().expect("Could not get next id");

    b.iter(|| decompose(next_id));
}

benchmark_group!(sonyflake_perf, bench_new, bench_next_id, bench_decompose);

benchmark_main!(sonyflake_perf);
