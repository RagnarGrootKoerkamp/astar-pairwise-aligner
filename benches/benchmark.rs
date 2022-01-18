#![feature(test)]
#![cfg(test)]
use pairwise_aligner::prelude::*;

extern crate test;

use test::Bencher;

#[bench]
fn stable_hundred(bench: &mut Bencher) {
    let n = 100;
    let e = 0.2;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(6),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        prune_fraction: 1.0,
        c: PhantomData::<NaiveContours<LogQueryContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}

#[bench]
fn stable_thousand(bench: &mut Bencher) {
    let n = 1000;
    let e = 0.2;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(7),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        prune_fraction: 1.0,
        c: PhantomData::<NaiveContours<LogQueryContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}

#[bench]
fn stable_ten_k(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.2;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(8),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        prune_fraction: 1.0,
        c: PhantomData::<NaiveContours<LogQueryContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}

#[bench]
fn stable_ten_k_prune_less(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.2;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(8),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        prune_fraction: 0.6,
        c: PhantomData::<NaiveContours<LogQueryContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}

#[bench]
fn stable_hundred_k_similar(bench: &mut Bencher) {
    let n = 100000;
    let e = 0.05;
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(10),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        prune_fraction: 1.0,
        c: PhantomData::<NaiveContours<LogQueryContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    bench.iter(|| align(&a, &b, &alphabet, stats, h));
}

// #[bench]
// fn optimal_hundred(bench: &mut Bencher) {
//     let n = 100;
//     let e = 0.2;
//     let h = GapSeedHeuristic {
//         match_config: MatchConfig {
//             length: Fixed(6),
//             max_match_cost: 1,
//             ..MatchConfig::default()
//         },
//         pruning: true,
//         prune_fraction: 1.0,
//         c: PhantomData::<NaiveContours<CentralContour>>,
//         ..GapSeedHeuristic::default()
//     };

//     let (a, b, alphabet, stats) = setup(n, e);
//     bench.iter(|| align(&a, &b, &alphabet, stats, h));
// }

// #[bench]
// fn optimal_thousand(bench: &mut Bencher) {
//     let n = 1000;
//     let e = 0.2;
//     let h = GapSeedHeuristic {
//         match_config: MatchConfig {
//             length: Fixed(7),
//             max_match_cost: 1,
//             ..MatchConfig::default()
//         },
//         pruning: true,
//         prune_fraction: 1.0,
//         c: PhantomData::<NaiveContours<CentralContour>>,
//         ..GapSeedHeuristic::default()
//     };

//     let (a, b, alphabet, stats) = setup(n, e);
//     bench.iter(|| align(&a, &b, &alphabet, stats, h));
// }

// #[bench]
// fn optimal_ten_k(bench: &mut Bencher) {
//     let n = 10000;
//     let e = 0.2;
//     let h = GapSeedHeuristic {
//         match_config: MatchConfig {
//             length: Fixed(8),
//             max_match_cost: 1,
//             ..MatchConfig::default()
//         },
//         pruning: true,
//         prune_fraction: 1.0,
//         c: PhantomData::<NaiveContours<CentralContour>>,
//         ..GapSeedHeuristic::default()
//     };

//     let (a, b, alphabet, stats) = setup(n, e);
//     bench.iter(|| align(&a, &b, &alphabet, stats, h));
// }

// #[bench]
// fn optimal_ten_k_prune_less(bench: &mut Bencher) {
//     let n = 10000;
//     let e = 0.2;
//     let h = GapSeedHeuristic {
//         match_config: MatchConfig {
//             length: Fixed(8),
//             max_match_cost: 1,
//             ..MatchConfig::default()
//         },
//         pruning: true,
//         prune_fraction: 0.6,
//         c: PhantomData::<NaiveContours<CentralContour>>,
//         ..GapSeedHeuristic::default()
//     };

//     let (a, b, alphabet, stats) = setup(n, e);
//     bench.iter(|| align(&a, &b, &alphabet, stats, h));
// }

// #[bench]
// fn optimal_hundred_k_similar(bench: &mut Bencher) {
//     let n = 100000;
//     let e = 0.05;
//     let h = GapSeedHeuristic {
//         match_config: MatchConfig {
//             length: Fixed(10),
//             max_match_cost: 1,
//             ..MatchConfig::default()
//         },
//         pruning: true,
//         prune_fraction: 1.0,
//         c: PhantomData::<NaiveContours<CentralContour>>,
//         ..GapSeedHeuristic::default()
//     };

//     let (a, b, alphabet, stats) = setup(n, e);
//     bench.iter(|| align(&a, &b, &alphabet, stats, h));
// }
