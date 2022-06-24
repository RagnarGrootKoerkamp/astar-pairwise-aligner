#![feature(test)]
#![cfg(test)]

extern crate test;
use pairwise_aligner::{
    aligners::{diagonal_transition::DiagonalTransition, nw::NW, nw_lib::NWLib, NoVisualizer},
    cost_model::LinearCost,
    generate::setup_sequences,
};
use test::Bencher;

const N: usize = 8000;
const E: f32 = 0.05;

fn run_aligner(
    mut aligner: impl pairwise_aligner::aligners::Aligner,
    n: usize,
    e: f32,
    exponential_search: bool,
) {
    let (ref a, ref b) = setup_sequences(n, e);
    if exponential_search {
        aligner.cost_exponential_search(a, b);
    } else {
        aligner.cost(a, b);
    }
}

#[bench]
fn nw_lib(bench: &mut Bencher) {
    bench.iter(|| run_aligner(NWLib, N, E, false));
}
#[bench]
fn nw_lib_exp(bench: &mut Bencher) {
    bench.iter(|| run_aligner(NWLib, N, E, true));
}

fn make_nw(use_gap_cost_heuristic: bool) -> NW<LinearCost, NoVisualizer> {
    NW {
        cm: LinearCost::new_unit(),
        use_gap_cost_heuristic,
        v: NoVisualizer,
    }
}

#[bench]
fn nw_simple(bench: &mut Bencher) {
    bench.iter(|| run_aligner(make_nw(false), N, E, false));
}
#[bench]
fn nw_exp_h(bench: &mut Bencher) {
    bench.iter(|| run_aligner(make_nw(true), N, E, true));
}

fn make_dt(use_gap_cost_heuristic: bool) -> DiagonalTransition<LinearCost, NoVisualizer> {
    DiagonalTransition::new(LinearCost::new_unit(), use_gap_cost_heuristic, NoVisualizer)
}

#[bench]
fn dt_simple(bench: &mut Bencher) {
    bench.iter(|| run_aligner(make_dt(false), N, E, false));
}
#[bench]
fn dt_exp_h(bench: &mut Bencher) {
    bench.iter(|| run_aligner(make_dt(true), N, E, true));
}
