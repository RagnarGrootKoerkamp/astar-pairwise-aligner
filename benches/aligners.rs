#![feature(test)]
#![cfg(test)]

extern crate test;
use pairwise_aligner::{
    aligners::{
        diagonal_transition::{
            DiagonalTransition, Direction, GapCostHeuristic, HistoryCompression,
        },
        nw::NW,
        nw_lib::NWLib,
    },
    cost_model::LinearCost,
    generate::setup_sequences_with_seed,
    heuristic::{ZeroCost, SH},
    matches::MatchConfig,
    visualizer::NoVisualizer,
};
use test::Bencher;

const N: usize = 30000;
const E: f32 = 0.05;

fn run_aligner(
    mut aligner: impl pairwise_aligner::aligners::Aligner,
    n: usize,
    e: f32,
    exponential_search: bool,
    seed: &mut u64,
) {
    let (ref a, ref b) = setup_sequences_with_seed(*seed, n, e);
    *seed += 1;
    if exponential_search {
        aligner.cost_exponential_search(a, b);
    } else {
        aligner.cost(a, b);
    }
}

#[bench]
fn nw_lib(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(NWLib, N, E, false, seed));
}
#[bench]
fn nw_lib_exp(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(NWLib, N, E, true, seed));
}

fn make_nw(use_gap_cost_heuristic: bool) -> NW<LinearCost, NoVisualizer, ZeroCost> {
    NW::new(LinearCost::new_unit(), use_gap_cost_heuristic)
}

#[bench]
fn nw_simple(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(make_nw(false), N, E, false, seed));
}
#[bench]
fn nw_gapcost(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(make_nw(true), N, E, true, seed));
}

fn make_nw_sh() -> NW<LinearCost, NoVisualizer, SH> {
    NW {
        cm: LinearCost::new_unit(),
        use_gap_cost_heuristic: false,
        h: SH {
            match_config: MatchConfig::exact(10),
            pruning: false,
        },
        v: NoVisualizer,
    }
}

#[bench]
fn nw_sh(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(make_nw_sh(), N, E, true, seed));
}

fn make_dt(
    use_gap_cost_heuristic: GapCostHeuristic,
) -> DiagonalTransition<LinearCost, NoVisualizer, ZeroCost> {
    DiagonalTransition::new(
        LinearCost::new_unit(),
        use_gap_cost_heuristic,
        ZeroCost,
        NoVisualizer,
    )
}

#[bench]
fn dt_simple(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(make_dt(GapCostHeuristic::Disable), N, E, false, seed));
}
#[bench]
fn dt_gapcost(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(make_dt(GapCostHeuristic::Enable), N, E, true, seed));
}

fn make_dt_sh(
    use_gap_cost_heuristic: GapCostHeuristic,
    history_compression: HistoryCompression,
) -> DiagonalTransition<LinearCost, NoVisualizer, SH> {
    DiagonalTransition::new_variant(
        LinearCost::new_unit(),
        use_gap_cost_heuristic,
        SH {
            match_config: MatchConfig::exact(10),
            pruning: false,
        },
        history_compression,
        Direction::Forward,
        NoVisualizer,
    )
}

#[bench]
fn dt_sh(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| {
        run_aligner(
            make_dt_sh(GapCostHeuristic::Enable, HistoryCompression::Disable),
            N,
            E,
            true,
            seed,
        )
    });
}
