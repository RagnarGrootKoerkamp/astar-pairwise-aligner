//! Benchmarks run in CI.

#![feature(test)]
#![cfg(test)]
use astar_pairwise_aligner::{prelude::*, visualizer::NoVis};

extern crate test;

use test::Bencher;

#[bench]
fn base_100(bench: &mut Bencher) {
    let n = 100;
    let e = 0.2;
    let h = CSH {
        match_config: MatchConfig::inexact(6),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn base_1000(bench: &mut Bencher) {
    let n = 1000;
    let e = 0.2;
    let h = CSH {
        match_config: MatchConfig::inexact(7),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn base_10000(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.2;
    let h = CSH {
        match_config: MatchConfig::inexact(8),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn base_50000_similar(bench: &mut Bencher) {
    let n = 50000;
    let e = 0.05;
    let h = CSH {
        match_config: MatchConfig::inexact(10),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn fast_100(bench: &mut Bencher) {
    let n = 100;
    let e = 0.2;
    let h = CSH {
        match_config: MatchConfig::inexact(6),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn fast_1000(bench: &mut Bencher) {
    let n = 1000;
    let e = 0.2;
    let h = CSH {
        match_config: MatchConfig::inexact(7),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn fast_10000(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.2;
    let h = CSH {
        match_config: MatchConfig::inexact(8),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn fast_50000_similar(bench: &mut Bencher) {
    let n = 50000;
    let e = 0.05;
    let h = CSH {
        match_config: MatchConfig::inexact(10),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}
