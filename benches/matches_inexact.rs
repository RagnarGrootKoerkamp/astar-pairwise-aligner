//! Conclusions:
//! - bm is much larger than am and b, so never even consider matching a with bm.
//! - am is 2.5x larger than b, so build a datastructure over b
#![feature(test)]
#![cfg(test)]
use pairwise_aligner::{
    prelude::*,
    seeds::{find_matches_qgram_hash_inexact, find_matches_trie},
};

extern crate test;

use test::Bencher;

const E: f32 = 0.20;
const K: I = 10;

// #[bench]
// fn n100_inexact_qgramindex(bench: &mut Bencher) {
//     let n = 100;
//     let (a, b, alph, _) = setup(n, E);
//     bench.iter(|| {
//         find_matches_qgramindex(
//             &a,
//             &b,
//             &alph,
//             MatchConfig {
//                 length: Fixed(K),
//                 max_match_cost: 1,
//                 ..Default::default()
//             },
//         )
//     });
// }

// #[bench]
// fn n10000_inexact_qgramindex(bench: &mut Bencher) {
//     let n = 10000;
//     let (a, b, alph, _) = setup(n, E);
//     bench.iter(|| {
//         find_matches_qgramindex(
//             &a,
//             &b,
//             &alph,
//             MatchConfig {
//                 length: Fixed(K),
//                 max_match_cost: 1,
//                 ..Default::default()
//             },
//         )
//     });
// }

#[bench]
fn n100_inexact_trie(bench: &mut Bencher) {
    let n = 100;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        find_matches_trie(
            &a,
            &b,
            &alph,
            MatchConfig {
                length: Fixed(K),
                max_match_cost: 1,
                ..Default::default()
            },
        )
    });
}

#[bench]
fn n10000_inexact_trie(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        find_matches_trie(
            &a,
            &b,
            &alph,
            MatchConfig {
                length: Fixed(K),
                max_match_cost: 1,
                ..Default::default()
            },
        )
    });
}

#[bench]
fn n100_inexact_hash(bench: &mut Bencher) {
    let n = 100;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        find_matches_trie(
            &a,
            &b,
            &alph,
            MatchConfig {
                length: Fixed(K),
                max_match_cost: 1,
                ..Default::default()
            },
        )
    });
}

#[bench]
fn n10000_inexact_hash(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        find_matches_qgram_hash_inexact(
            &a,
            &b,
            &alph,
            MatchConfig {
                length: Fixed(K),
                max_match_cost: 1,
                ..Default::default()
            },
        )
    });
}

#[bench]
fn n100000_inexact_hash(bench: &mut Bencher) {
    let n = 100000;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        find_matches_qgram_hash_inexact(
            &a,
            &b,
            &alph,
            MatchConfig {
                length: Fixed(K),
                max_match_cost: 1,
                ..Default::default()
            },
        )
    });
}

#[bench]
fn n100_am_in_b(bench: &mut Bencher) {
    let n = 100;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| matches_inexact::lookup_am_in_b_hashmap(&a, &b, K));
}

#[bench]
fn n10000_am_in_b(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| matches_inexact::lookup_am_in_b_hashmap(&a, &b, K));
}

#[bench]
fn n100000_am_in_b(bench: &mut Bencher) {
    let n = 100000;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| matches_inexact::lookup_am_in_b_hashmap(&a, &b, K));
}

#[bench]
fn n100_am_in_b_dedup(bench: &mut Bencher) {
    let n = 100;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| matches_inexact::lookup_am_in_b_hashmap_dedup(&a, &b, K));
}

#[bench]
fn n10000_am_in_b_dedup(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| matches_inexact::lookup_am_in_b_hashmap_dedup(&a, &b, K));
}

#[bench]
fn n100000_am_in_b_dedup(bench: &mut Bencher) {
    let n = 100000;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| matches_inexact::lookup_am_in_b_hashmap_dedup(&a, &b, K));
}

#[bench]
fn n100_b_in_am(bench: &mut Bencher) {
    let n = 100;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| matches_inexact::lookup_b_in_am_hashmap(&a, &b, K));
}

#[bench]
fn n10000_b_in_am(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| matches_inexact::lookup_b_in_am_hashmap(&a, &b, K));
}

// #[bench]
// fn n100000_b_in_am(bench: &mut Bencher) {
//     let n = 100000;
//     let (a, b, _, _) = setup(n, E);
//     bench.iter(|| matches_inexact::lookup_b_in_am_hashmap(&a, &b, K));
// }

// #[bench]
// fn n100_a_in_bm(bench: &mut Bencher) {
//     let n = 100;
//     let (a, b, _, _) = setup(n, E);
//     bench.iter(|| matches_inexact::lookup_a_in_bm_hashmap(&a, &b, K));
// }

// #[bench]
// fn n10000_a_in_bm(bench: &mut Bencher) {
//     let n = 10000;
//     let (a, b, _, _) = setup(n, E);
//     bench.iter(|| matches_inexact::lookup_a_in_bm_hashmap(&a, &b, K));
// }

// #[bench]
// fn n100000_a_in_bm(bench: &mut Bencher) {
//     let n = 100000;
//     let (a, b, _, _) = setup(n, E);
//     bench.iter(|| matches_inexact::lookup_a_in_bm_hashmap(&a, &b, K));
// }

// #[bench]
// fn n100_bm_in_a(bench: &mut Bencher) {
//     let n = 100;
//     let (a, b, _, _) = setup(n, E);
//     bench.iter(|| matches_inexact::lookup_bm_in_a_hashmap(&a, &b, K));
// }

// #[bench]
// fn n10000_bm_in_a(bench: &mut Bencher) {
//     let n = 10000;
//     let (a, b, _, _) = setup(n, E);
//     bench.iter(|| matches_inexact::lookup_bm_in_a_hashmap(&a, &b, K));
// }

// #[bench]
// fn n100000_bm_in_a(bench: &mut Bencher) {
//     let n = 100000;
//     let (a, b, _, _) = setup(n, E);
//     bench.iter(|| matches_inexact::lookup_bm_in_a_hashmap(&a, &b, K));
// }
