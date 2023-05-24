#![feature(test)]
#![cfg(test)]

use std::hint::black_box;

use pa_heuristic::matches::{find_matches_qgram_hash_exact, MatchConfig};

extern crate test;

use pa_generate::uniform_fixed;
use pa_types::I;
use test::Bencher;

const E: f32 = 0.02;
const K: I = 14;

#[bench]
fn n100(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| {
        for _ in 0..1000000 {
            black_box(find_matches_qgram_hash_exact(
                &a,
                &b,
                MatchConfig::exact(K),
                false,
            ));
        }
    });
}

#[bench]
fn n10k(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| {
        for _ in 0..10000 {
            black_box(find_matches_qgram_hash_exact(
                &a,
                &b,
                MatchConfig::exact(K),
                false,
            ));
        }
    });
}

#[allow(non_snake_case)]
#[bench]
fn n1M(bench: &mut Bencher) {
    let n = 1000000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| {
        for _ in 0..100 {
            black_box(find_matches_qgram_hash_exact(
                &a,
                &b,
                MatchConfig::exact(K),
                false,
            ));
        }
    });
}
