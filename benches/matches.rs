#![feature(test)]
#![cfg(test)]
use pairwise_aligner::{
    prelude::*,
    seeds::{find_matches_qgram_hash, find_matches_qgramindex, find_matches_trie},
};

extern crate test;

use test::Bencher;

const E: f32 = 0.02;
const K: I = 8;

#[bench]
fn n100_exact_qgramindex(bench: &mut Bencher) {
    let n = 100;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        find_matches_qgramindex(
            &a,
            &b,
            &alph,
            MatchConfig {
                length: Fixed(K),
                max_match_cost: 0,
                ..Default::default()
            },
        )
    });
}

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
//                 length: Fixed(6),
//                 max_match_cost: 1,
//                 ..Default::default()
//             },
//         )
//     });
// }

#[bench]
fn n10000_exact_qgramindex(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        find_matches_qgramindex(
            &a,
            &b,
            &alph,
            MatchConfig {
                length: Fixed(K),
                max_match_cost: 0,
                ..Default::default()
            },
        )
    });
}

// #[bench]
// fn n10000_inexact_qgramindex(bench: &mut Bencher) {
//     let n = 10000;
//     let e = 0.20;
//     let (a, b, alph, _) = setup(n, e);
//     bench.iter(|| {
//         find_matches_qgramindex(
//             &a,
//             &b,
//             &alph,
//             MatchConfig {
//                 length: Fixed(9),
//                 max_match_cost: 1,
//                 ..Default::default()
//             },
//         )
//     });
// }

#[bench]
fn n100_exact_trie(bench: &mut Bencher) {
    let n = 100;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        find_matches_trie(
            &a,
            &b,
            &alph,
            MatchConfig {
                length: Fixed(K),
                max_match_cost: 0,
                ..Default::default()
            },
        )
    });
}

// #[bench]
// fn n100_inexact_trie(bench: &mut Bencher) {
//     let n = 100;
//     let e = 0.10;
//     let (a, b, alph, _) = setup(n, e);
//     bench.iter(|| {
//         find_matches_trie(
//             &a,
//             &b,
//             &alph,
//             MatchConfig {
//                 length: Fixed(6),
//                 max_match_cost: 1,
//                 ..Default::default()
//             },
//         )
//     });
// }

#[bench]
fn n10000_exact_trie(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        find_matches_trie(
            &a,
            &b,
            &alph,
            MatchConfig {
                length: Fixed(K),
                max_match_cost: 0,
                ..Default::default()
            },
        )
    });
}

// #[bench]
// fn n10000_inexact_trie(bench: &mut Bencher) {
//     let n = 10000;
//     let e = 0.20;
//     let (a, b, alph, _) = setup(n, e);
//     bench.iter(|| {
//         find_matches_trie(
//             &a,
//             &b,
//             &alph,
//             MatchConfig {
//                 length: Fixed(9),
//                 max_match_cost: 1,
//                 ..Default::default()
//             },
//         )
//     });
// }

#[bench]
fn n100_exact_hash(bench: &mut Bencher) {
    let n = 100;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        find_matches_qgram_hash(
            &a,
            &b,
            &alph,
            MatchConfig {
                length: Fixed(K),
                max_match_cost: 0,
                ..Default::default()
            },
        )
    });
}

#[bench]
fn n10000_exact_hash(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        find_matches_qgram_hash(
            &a,
            &b,
            &alph,
            MatchConfig {
                length: Fixed(K),
                max_match_cost: 0,
                ..Default::default()
            },
        )
    });
}

#[bench]
fn n100_aho_corasick(bench: &mut Bencher) {
    let n = 100;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| {
        matches::aho_corasick(&a, &b, K);
    });
}
#[bench]
fn n10000_aho_corasick(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| {
        matches::aho_corasick(&a, &b, K);
    });
}

#[bench]
fn n100_lookup_seeds_in_qgram_hashmap(bench: &mut Bencher) {
    let n = 100;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| {
        matches::lookup_seeds_in_qgram_hashmap(&a, &b, K);
    });
}
#[bench]
fn n10000_lookup_seeds_in_qgram_hashmap(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| {
        matches::lookup_seeds_in_qgram_hashmap(&a, &b, K);
    });
}

#[bench]
fn n100_lookup_suffixes_in_qgram_hashmap(bench: &mut Bencher) {
    let n = 100;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| {
        matches::lookup_suffixes_in_qgram_hashmap(&a, &b, K);
    });
}
#[bench]
fn n10000_lookup_suffixes_in_qgram_hashmap(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, _, _) = setup(n, E);
    bench.iter(|| {
        matches::lookup_suffixes_in_qgram_hashmap(&a, &b, K);
    });
}
