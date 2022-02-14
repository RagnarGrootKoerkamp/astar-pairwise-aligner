#![feature(test, bench_black_box)]
#![cfg(test)]

use std::hint::black_box;

use pairwise_aligner::{matches::find_matches_qgram_hash_exact, prelude::*};

extern crate test;

use test::Bencher;

const E: f32 = 0.02;
const K: I = 14;

#[bench]
fn n100(bench: &mut Bencher) {
    let n = 100;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        for _ in 0..1000000 {
            black_box(find_matches_qgram_hash_exact(
                &a,
                &b,
                &alph,
                MatchConfig {
                    length: Fixed(K),
                    max_match_cost: 0,
                    ..Default::default()
                },
            ));
        }
    });
}

#[bench]
fn n10k(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        for _ in 0..10000 {
            black_box(find_matches_qgram_hash_exact(
                &a,
                &b,
                &alph,
                MatchConfig {
                    length: Fixed(K),
                    max_match_cost: 0,
                    ..Default::default()
                },
            ));
        }
    });
}

#[allow(non_snake_case)]
#[bench]
fn n1M(bench: &mut Bencher) {
    let n = 1000000;
    let (a, b, alph, _) = setup(n, E);
    bench.iter(|| {
        for _ in 0..100 {
            black_box(find_matches_qgram_hash_exact(
                &a,
                &b,
                &alph,
                MatchConfig {
                    length: Fixed(K),
                    max_match_cost: 0,
                    ..Default::default()
                },
            ));
        }
    });
}
