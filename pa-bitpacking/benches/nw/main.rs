#![feature(portable_simd)]
use bio::alignment::distance::simd::levenshtein;
use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion,
};
use pa_generate::{get_rng, random_sequence};
use pa_types::Cost;
use std::time::Duration;

use pa_bitpacking::{bit_profile::Bits, *};

const TEST: bool = true;

fn bench_scalar<P: Profile, H: HEncoding>(
    c: &mut BenchmarkGroup<WallTime>,
    pa: &[P::A],
    pb: &[P::B],
    d: Cost,
    prefix: &str,
) {
    let mut h = vec![H::one(); pa.len()];
    let mut v = vec![V::one(); pb.len()];
    let mut test = |name: &str, f: fn(&[P::A], &[P::B], &mut [H], &mut [V]) -> Cost| {
        h.fill(H::one());
        v.fill(V::one());
        let d2 = f(pa, pb, &mut h, &mut v);
        if d2 == 0 {
            return;
        }
        c.bench_function(&format!("{prefix}/{name}"), |bb| {
            bb.iter(|| {
                h.fill(H::one());
                v.fill(V::one());
                let d2 = f(pa, pb, &mut h, &mut v);
                if TEST && d2 != d && d2 != 0 {
                    panic!("{} != {}", d, d2)
                }
            })
        });
    };

    use scalar::*;
    test("col", col::<P, H>);
    test("row", row::<P, H>);
    // test("diagru", diag_ru::<P, H>);
    // test("diagld", diag_ld::<P, H>);
    // test("colsru/1", cols_ru::<1, P, H>);
    // test("colsru/2", cols_ru::<2, P, H>);
    // test("colsru/3", cols_ru::<3, P, H>);
    // test("colsru/4", cols_ru::<4, P, H>);
    // test("colsld/1", cols_ld::<1, P, H>);
    // test("colsld/2", cols_ld::<2, P, H>);
    // test("colsld/3", cols_ld::<3, P, H>);
    // test("colsld/4", cols_ld::<4, P, H>);
    // test("rowsru/1", rows_ru::<1, P, H>);
    // test("rowsru/2", rows_ru::<2, P, H>);
    // test("rowsru/3", rows_ru::<3, P, H>);
    // test("rowsru/4", rows_ru::<4, P, H>);
    // test("rowsld/1", rows_ld::<1, P, H>);
    // test("rowsld/2", rows_ld::<2, P, H>);
    // test("rowsld/3", rows_ld::<3, P, H>);
    // test("rowsld/4", rows_ld::<4, P, H>);

    // let mut test_local = |name: &str, f: fn(&[P::A], &[P::B], &mut [V]) -> Cost| {
    //     v.fill(V::one());
    //     let d2 = f(pa, pb, &mut v);
    //     if d2 == 0 {
    //         return;
    //     }
    //     c.bench_function(&format!("{prefix}/{name}"), |bb| {
    //         bb.iter(|| {
    //             h.fill(H::one());
    //             v.fill(V::one());
    //             let d2 = f(pa, pb, &mut v);
    //             if TEST && d2 != d && d2 != 0 {
    //                 panic!("{} != {}", d, d2)
    //             }
    //         })
    //     });
    // };

    // test_local("col/lh", col_local_h::<P, H>);
    // test_local("colsru/lh/1", cols_ru_local_h::<1, P, H>);
    // test_local("colsru/lh/2", cols_ru_local_h::<2, P, H>);
    // test_local("colsru/lh/3", cols_ru_local_h::<3, P, H>);
    // test_local("colsru/lh/4", cols_ru_local_h::<4, P, H>);
    // test_local("colsld/lh/1", cols_ld_local_h::<1, P, H>);
    // test_local("colsld/lh/2", cols_ld_local_h::<2, P, H>);
    // test_local("colsld/lh/3", cols_ld_local_h::<3, P, H>);
    // test_local("colsld/lh/4", cols_ld_local_h::<4, P, H>);
}

fn bench_simd<H: HEncoding>(
    c: &mut BenchmarkGroup<WallTime>,
    pa: &[Bits],
    pb: &[Bits],
    d: Cost,
    prefix: &str,
) {
    let mut h = vec![H::one(); pa.len()];
    let mut v = vec![V::one(); pb.len()];
    let mut test =
        |name: &str, f: fn(&[Bits], &[Bits], &mut [H], &mut [V], bool) -> Cost, exact: bool| {
            h.fill(H::one());
            v.fill(V::one());
            let d2 = f(pa, pb, &mut h, &mut v, exact);
            if d2 == 0 {
                return;
            }
            c.bench_function(&format!("{prefix}/{name}"), |bb| {
                bb.iter(|| {
                    h.fill(H::one());
                    v.fill(V::one());
                    let d2 = f(pa, pb, &mut h, &mut v, exact);
                    if TEST && d2 != d && d2 != 0 {
                        panic!("{} != {}", d, d2)
                    }
                })
            });
        };

    use simd::*;

    // test("simd1rowx/1", row::<1, H, 1>, true);
    // test("simd1rowx/2", row::<2, H, 1>, true);
    test("simd2rowx/1", compute::<1, H, 2>, true);
    // test("simd2rowx/2", row::<2, H, 2>, true);
    test("simd4rowx/1", compute::<1, H, 4>, true);
    test("simd4rowx/2", compute::<2, H, 4>, true);

    // test("simd1rowp/1", row::<1, H, 1>, false);
    // test("simd1rowp/2", row::<2, H, 1>, false);
    test("simd2rowp/1", compute::<1, H, 2>, false);
    // test("simd2rowp/2", row::<2, H, 2>, false);
    test("simd4rowp/1", compute::<1, H, 4>, false);
    test("simd4rowp/2", compute::<2, H, 4>, false);
}

fn bench(c: &mut Criterion) {
    for height in (64..=512).step_by(64) {
        let c = &mut c.benchmark_group(&format!("{}", height));
        let rng = &mut get_rng(Some(31415));
        let a = &random_sequence(256, rng);
        let b = &random_sequence(height, rng);
        let d = if TEST {
            levenshtein(a, b) as Cost - b.len() as Cost
        } else {
            0
        };

        let (ref pa, ref pb) = ScatterProfile::build(a, b);
        bench_scalar::<ScatterProfile, (u64, u64)>(c, pa, pb, d, "scat/u64");

        let (ref pa, ref pb) = BitProfile::build(a, b);
        bench_scalar::<BitProfile, (u64, u64)>(c, pa, pb, d, "bit/u64");

        // bench_simd::<(u8, u8)>(c, pa, pb, d, "bit/u8");
        bench_simd::<(u64, u64)>(c, pa, pb, d, "bit/u64");
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_millis(500)).warm_up_time(Duration::from_millis(100));
    targets = bench
);
criterion_main!(benches);
