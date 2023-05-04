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

fn h_bench<H: HEncoding, M: Measurement>(t: &str, a: Seq, b: Seq, c: &mut BenchmarkGroup<M>) {
    for d in Order::iter() {
        c.bench_function(BenchmarkId::new("Local", format!("{t}/{d}")), |bb| {
            bb.iter(|| nw::<H>(a, b, d, &NoVis))
        });
    }
    for d in Direction::iter() {
        c.bench_function(BenchmarkId::new("Diag", format!("{t}/{d}")), |bb| {
            bb.iter(|| nw_diag::<H>(a, b, d, &NoVis))
        });
    }

    for d in Direction::iter() {
        c.bench_function(BenchmarkId::new("Striped", format!("{t}/{d}/1")), |bb| {
            bb.iter(|| nw_striped_col::<1, H>(a, b, d, &NoVis))
        });
        c.bench_function(BenchmarkId::new("Striped", format!("{t}/{d}/2")), |bb| {
            bb.iter(|| nw_striped_col::<2, H>(a, b, d, &NoVis))
        });
        c.bench_function(BenchmarkId::new("Striped", format!("{t}/{d}/3")), |bb| {
            bb.iter(|| nw_striped_col::<3, H>(a, b, d, &NoVis))
        });
        c.bench_function(BenchmarkId::new("Striped", format!("{t}/{d}/4")), |bb| {
            bb.iter(|| nw_striped_col::<4, H>(a, b, d, &NoVis))
        });
    }
}

fn simd_bench<M: Measurement>(a: CompressedSeq, b: CompressedSeq, c: &mut BenchmarkGroup<M>) {
    c.bench_function(BenchmarkId::new("Simd", format!("1")), |bb| {
        bb.iter(|| nw_simd_striped_col::<1>(a, b, &NoVis))
    });
    c.bench_function(BenchmarkId::new("Simd", format!("2")), |bb| {
        bb.iter(|| nw_simd_striped_col::<2>(a, b, &NoVis))
    });
    c.bench_function(BenchmarkId::new("Simd", format!("3")), |bb| {
        bb.iter(|| nw_simd_striped_col::<3>(a, b, &NoVis))
    });
    c.bench_function(BenchmarkId::new("Simd", format!("4")), |bb| {
        bb.iter(|| nw_simd_striped_col::<4>(a, b, &NoVis))
    });
    c.bench_function(BenchmarkId::new("SimdRow", format!("1")), |bb| {
        bb.iter(|| nw_simd_striped_row_wrapper::<1>(a, b))
    });
    c.bench_function(BenchmarkId::new("SimdRow", format!("2")), |bb| {
        bb.iter(|| nw_simd_striped_row_wrapper::<2>(a, b))
    });
    c.bench_function(BenchmarkId::new("SimdRow", format!("3")), |bb| {
        bb.iter(|| nw_simd_striped_row_wrapper::<3>(a, b))
    });
    c.bench_function(BenchmarkId::new("SimdRow", format!("4")), |bb| {
        bb.iter(|| nw_simd_striped_row_wrapper::<4>(a, b))
    });
}

fn bench<M: Measurement>(unit: &str, c: &mut Criterion<M>) {
    let (b, a) = &pa_generate::uniform_fixed(1024, 0.1);
    let (ref ca, ref cb) = compress(a, b);
    let d = levenshtein(a, b) as D;

    let c = &mut c.benchmark_group(unit);
    c.bench_function("TripleAccel", |bb| {
        bb.iter(|| assert_eq!(levenshtein(a, b) as D, d))
    });
    h_bench::<i8, M>("i8", a, b, c);
    h_bench::<(u8, u8), M>("u8", a, b, c);
    h_bench::<(B, B), M>("B", a, b, c);
    simd_bench(ca, cb, c);
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
