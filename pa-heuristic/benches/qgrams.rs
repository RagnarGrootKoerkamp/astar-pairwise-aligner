use criterion::{criterion_group, criterion_main, Criterion};
use instant::Duration;
use pa_generate::random_sequence;
use pa_heuristic::matches::qgrams::QGrams;

fn bench(c: &mut Criterion) {
    for n in [1000, 10000, 100000] {
        let c = &mut c.benchmark_group(&format!("{}", n));
        let a = &random_sequence(n, &mut rand::thread_rng());
        let qgrams = QGrams::new(a, a);
        let mut test = |name, f: &dyn Fn() -> usize| {
            c.bench_function(&format!("{name}"), |bb| bb.iter(|| f()));
        };
        test("a_qgrams", &|| qgrams.a_qgrams(10).map(|(_i, q)| q).sum());
        test("a_qgrams_rev", &|| {
            qgrams.a_qgrams_rev(10).map(|(_i, q)| q).sum()
        });
        test("b_qgrams", &|| qgrams.b_qgrams(10).map(|(_i, q)| q).sum());
        test("b_qgrams_rev", &|| {
            qgrams.b_qgrams_rev(10).map(|(_i, q)| q).sum()
        });
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_millis(500)).warm_up_time(Duration::from_millis(100));
    targets = bench
);
criterion_main!(benches);
