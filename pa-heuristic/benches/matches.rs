use criterion::{criterion_group, criterion_main, Criterion};
use instant::Duration;
use pa_generate::uniform_fixed;
use pa_heuristic::{
    matches::{exact::*, Matches},
    MatchConfig,
};

fn bench(c: &mut Criterion) {
    for n in [500000] {
        let mut c = c.benchmark_group(format!("{n}"));
        for k in [8, 10, 12, 14] {
            for e in [0.05] {
                let (_, b) = &uniform_fixed(n, e);
                let (a, _) = &uniform_fixed(n, e);
                for transform_filter in [true] {
                    let mut test = |name: &str, f: &dyn Fn() -> Matches| {
                        c.bench_function(&format!("{k}/{e}/{transform_filter}/{name}"), |bb| {
                            bb.iter(|| f())
                        });
                    };

                    let config = MatchConfig::exact(k);
                    test("a_1", &|| hash_a(a, b, config, transform_filter));
                    test("a_2", &|| hash_a_single(a, b, config, transform_filter));
                    // test("a_3", &|| {
                    //     hash_a_qgram_index(a, b, config, transform_filter)
                    // });
                    test("a_sw", &|| {
                        hash_a_sliding_window(a, b, config, transform_filter)
                    });

                    test("b_1", &|| hash_b(a, b, config, transform_filter));
                    test("b_2", &|| hash_b_single(a, b, config, transform_filter));
                    // test("b_3", &|| {
                    //     hash_b_qgram_index(a, b, config, transform_filter)
                    // });
                }
            }
        }
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_millis(1000)).warm_up_time(Duration::from_millis(1000)).sample_size(10);
    targets = bench
);
criterion_main!(benches);
