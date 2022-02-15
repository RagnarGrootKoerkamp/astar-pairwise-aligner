//! Benchmarks run in CI.

#![feature(test)]
#![cfg(test)]
use pairwise_aligner::prelude::*;

extern crate test;

use test::Bencher;

#[bench]
fn base_100(bench: &mut Bencher) {
    let n = 100;
    let e = 0.2;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(6),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        c: PhantomData::<HintContours<CentralContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}

#[bench]
fn base_1000(bench: &mut Bencher) {
    let n = 1000;
    let e = 0.2;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(7),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        c: PhantomData::<HintContours<CentralContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}

#[bench]
fn base_10000(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.2;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(8),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        c: PhantomData::<HintContours<CentralContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}

#[bench]
fn base_50000_similar(bench: &mut Bencher) {
    let n = 50000;
    let e = 0.05;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(10),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        c: PhantomData::<HintContours<CentralContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}

#[bench]
fn fast_100(bench: &mut Bencher) {
    let n = 100;
    let e = 0.2;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(6),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}

#[bench]
fn fast_1000(bench: &mut Bencher) {
    let n = 1000;
    let e = 0.2;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(7),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}

#[bench]
fn fast_10000(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.2;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(8),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}

#[bench]
fn fast_50000_similar(bench: &mut Bencher) {
    let n = 50000;
    let e = 0.05;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(10),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}
