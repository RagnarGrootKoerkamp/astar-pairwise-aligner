//! Benchmarks run in CI.

#![feature(test)]
#![cfg(test)]

use astarpa::astar;
use pa_generate::uniform_fixed;
use pa_heuristic::*;
use pa_vis::*;

extern crate test;

use test::Bencher;

#[bench]
fn base_100(bench: &mut Bencher) {
    let n = 100;
    let e = 0.2;
    let h = GCSH::new(MatchConfig::inexact(6), Pruning::start());

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn base_1000(bench: &mut Bencher) {
    let n = 1000;
    let e = 0.2;
    let h = GCSH::new(MatchConfig::inexact(7), Pruning::start());

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn base_10000(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.2;
    let h = GCSH::new(MatchConfig::inexact(8), Pruning::start());

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn base_50000_similar(bench: &mut Bencher) {
    let n = 50000;
    let e = 0.05;
    let h = GCSH::new(MatchConfig::inexact(10), Pruning::start());

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn fast_100(bench: &mut Bencher) {
    let n = 100;
    let e = 0.2;
    let h = GCSH::new(MatchConfig::inexact(6), Pruning::start());

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn fast_1000(bench: &mut Bencher) {
    let n = 1000;
    let e = 0.2;
    let h = GCSH::new(MatchConfig::inexact(7), Pruning::start());

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn fast_10000(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.2;
    let h = GCSH::new(MatchConfig::inexact(8), Pruning::start());

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}

#[bench]
fn fast_50000_similar(bench: &mut Bencher) {
    let n = 50000;
    let e = 0.05;
    let h = GCSH::new(MatchConfig::inexact(10), Pruning::start());

    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| astar(&a, &b, &h, &NoVis));
}
