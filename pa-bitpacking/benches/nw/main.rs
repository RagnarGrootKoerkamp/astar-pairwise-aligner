//! TODO:
//! - Apply `profile` once at the start, instead of inside each benchmarked function.
#![allow(incomplete_features)]
#![feature(
    let_chains,
    int_roundings,
    test,
    array_chunks,
    iter_array_chunks,
    array_windows,
    split_array,
    portable_simd,
    generic_const_exprs,
    concat_idents,
    bigint_helper_methods,
    core_intrinsics
)]

use bio::alignment::distance::simd::levenshtein;
use criterion::measurement::Measurement;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, BenchmarkId, Criterion};
use criterion_perf_events::Perf;
use pa_vis_types::NoVis;
use perfcnt::linux::HardwareEventType as Hardware;
use perfcnt::linux::PerfCounterBuilderLinux as Builder;

use pa_bitpacking::*;
use strum::IntoEnumIterator;

mod scalar;
mod simd;
use scalar::*;
use simd::*;

fn h_bench<H: HEncoding, M: Measurement>(
    t: &str,
    a: CompressedSeq,
    bp: ProfileSlice,
    c: &mut BenchmarkGroup<M>,
    d: D,
) {
    for dir in Order::iter() {
        c.bench_function(BenchmarkId::new("Local", format!("{t}/{dir}")), |bb| {
            bb.iter(|| assert_eq!(nw::<H>(a, bp, dir, &NoVis), d))
        });
    }
    for dir in Direction::iter() {
        c.bench_function(BenchmarkId::new("Diag", format!("{t}/{dir}")), |bb| {
            bb.iter(|| assert_eq!(nw_diag::<H>(a, bp, dir, &NoVis), d))
        });
    }

    for dir in Direction::iter() {
        c.bench_function(BenchmarkId::new("Striped", format!("{t}/{dir}/1")), |bb| {
            bb.iter(|| assert_eq!(nw_striped_col::<1, H>(a, bp, dir, &NoVis), d))
        });
        c.bench_function(BenchmarkId::new("Striped", format!("{t}/{dir}/2")), |bb| {
            bb.iter(|| assert_eq!(nw_striped_col::<2, H>(a, bp, dir, &NoVis), d))
        });
        c.bench_function(BenchmarkId::new("Striped", format!("{t}/{dir}/3")), |bb| {
            bb.iter(|| assert_eq!(nw_striped_col::<3, H>(a, bp, dir, &NoVis), d))
        });
        c.bench_function(BenchmarkId::new("Striped", format!("{t}/{dir}/4")), |bb| {
            bb.iter(|| assert_eq!(nw_striped_col::<4, H>(a, bp, dir, &NoVis), d))
        });
    }
}

fn simd_bench<M: Measurement>(
    a: CompressedSeq,
    b: CompressedSeq,
    bp: ProfileSlice,
    c: &mut BenchmarkGroup<M>,
    d: D,
) {
    // Functions only output the difference along the bottom row, so we correct
    // for that here.
    let d = d - b.len() as D;

    //let bp = padded_profile(b, 1 * 4 - 1);
    c.bench_function(BenchmarkId::new("Simd", format!("1")), |bb| {
        let bp = padded_profile(b, 1 * 4 - 1);
        bb.iter(|| assert_eq!(nw_simd_striped_col::<1>(a, &bp, &NoVis), d))
    });
    c.bench_function(BenchmarkId::new("Simd", format!("2")), |bb| {
        let bp = padded_profile(b, 2 * 4 - 1);
        bb.iter(|| assert_eq!(nw_simd_striped_col::<2>(a, &bp, &NoVis), d))
    });
    c.bench_function(BenchmarkId::new("Simd", format!("3")), |bb| {
        let bp = padded_profile(b, 3 * 4 - 1);
        bb.iter(|| assert_eq!(nw_simd_striped_col::<3>(a, &bp, &NoVis), d))
    });
    c.bench_function(BenchmarkId::new("Simd", format!("4")), |bb| {
        let bp = padded_profile(b, 4 * 4 - 1);
        bb.iter(|| assert_eq!(nw_simd_striped_col::<4>(a, &bp, &NoVis), d))
    });
    c.bench_function(BenchmarkId::new("SimdRow", format!("1")), |bb| {
        bb.iter(|| assert_eq!(nw_simd_striped_row_wrapper::<1>(a, bp), d))
    });
    c.bench_function(BenchmarkId::new("SimdRow", format!("2")), |bb| {
        bb.iter(|| assert_eq!(nw_simd_striped_row_wrapper::<2>(a, bp), d))
    });
    c.bench_function(BenchmarkId::new("SimdRow", format!("3")), |bb| {
        bb.iter(|| assert_eq!(nw_simd_striped_row_wrapper::<3>(a, bp), d))
    });
    c.bench_function(BenchmarkId::new("SimdRow", format!("4")), |bb| {
        bb.iter(|| assert_eq!(nw_simd_striped_row_wrapper::<4>(a, bp), d))
    });
}

fn bench<M: Measurement>(unit: &str, c: &mut Criterion<M>) {
    let (a, _) = &pa_generate::uniform_fixed(256, 0.);
    let (b, _) = &pa_generate::uniform_fixed(4096, 0.);

    let (_, bp) = profile(a, b);
    let (ref a, ref b) = compress(a, b);

    let d = levenshtein(a, b) as D;

    let c = &mut c.benchmark_group(unit);
    c.bench_function("TripleAccel", |bb| {
        bb.iter(|| assert_eq!(levenshtein(a, b) as D, d))
    });
    h_bench::<i8, M>("i8", a.into(), &bp, c, d);
    h_bench::<(u8, u8), M>("u8", a.into(), &bp, c, d);
    h_bench::<(B, B), M>("B", a.into(), &bp, c, d);
    simd_bench(a.into(), b.into(), &bp, c, d);
}

fn bench_time<M: Measurement>(c: &mut Criterion<M>) {
    bench("time", c);
}
fn bench_instr<M: Measurement>(c: &mut Criterion<M>) {
    bench("instr", c);
}
fn bench_cycles<M: Measurement>(c: &mut Criterion<M>) {
    bench("cycles", c);
}

criterion_group!(benches, bench_time);
criterion_group!(
    name = instructions_bench;
    config = Criterion::default().with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)));
    targets = bench_instr
);
criterion_group!(
    name = cycles_bench;
    config = Criterion::default().with_measurement(Perf::new(Builder::from_hardware_event(Hardware::CPUCycles)));
    targets = bench_cycles
);
criterion_main!(benches, cycles_bench, instructions_bench);
