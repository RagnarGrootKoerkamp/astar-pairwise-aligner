#![feature(test)]
#![cfg(test)]

extern crate test;
use astar_pairwise_aligner::{
    aligners::{
        diagonal_transition::{DiagonalTransition, GapCostHeuristic},
        nw::NW,
        triple_accel::TripleAccel,
    },
    cost_model::LinearCost,
    heuristic::{NoCost, Pruning, SH},
    matches::MatchConfig,
    prelude::CostModel,
    visualizer::NoVisualizer,
};
use pa_generate::uniform_seeded;
use test::Bencher;

const N: usize = 30000;
const E: f32 = 0.05;

fn run_aligner(
    mut aligner: impl astar_pairwise_aligner::aligners::Aligner,
    n: usize,
    e: f32,
    seed: &mut u64,
) {
    let (ref a, ref b) = uniform_seeded(n, e, *seed);
    *seed += 1;
    aligner.cost(a, b);
}

#[bench]
fn triple_accel(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(TripleAccel::new(false, CostModel::Levenshtein), N, E, seed));
}
#[bench]
fn triple_accel_exp(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(TripleAccel::new(false, CostModel::Levenshtein), N, E, seed));
}

fn make_nw(use_gap_cost_heuristic: bool) -> NW<0, NoVisualizer, NoCost> {
    NW::new(
        LinearCost::new_unit(),
        use_gap_cost_heuristic,
        use_gap_cost_heuristic,
    )
}

#[bench]
fn nw_simple(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(make_nw(false), N, E, seed));
}
#[bench]
fn nw_gapcost(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(make_nw(true), N, E, seed));
}

fn make_nw_sh() -> NW<0, NoVisualizer, SH> {
    NW {
        cm: LinearCost::new_unit(),
        use_gap_cost_heuristic: false,
        exponential_search: true,
        local_doubling: false,
        h: SH {
            match_config: MatchConfig::exact(10),
            pruning: Pruning::default(),
        },
        v: NoVisualizer,
    }
}

#[bench]
fn nw_sh(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(make_nw_sh(), N, E, seed));
}

fn make_dt(
    use_gap_cost_heuristic: GapCostHeuristic,
    dc: bool,
) -> DiagonalTransition<0, NoVisualizer, NoCost> {
    DiagonalTransition::new(
        LinearCost::new_unit(),
        use_gap_cost_heuristic,
        NoCost,
        dc,
        NoVisualizer,
    )
}

#[bench]
fn dt_simple(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(make_dt(GapCostHeuristic::Disable, false), N, E, seed));
}
#[bench]
fn dt_gapcost(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(make_dt(GapCostHeuristic::Enable, false), N, E, seed));
}
#[bench]
fn dt_dc(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(make_dt(GapCostHeuristic::Disable, true), N, E, seed));
}

fn make_dt_sh(use_gap_cost_heuristic: GapCostHeuristic) -> DiagonalTransition<0, NoVisualizer, SH> {
    DiagonalTransition::new(
        LinearCost::new_unit(),
        use_gap_cost_heuristic,
        SH {
            match_config: MatchConfig::exact(10),
            pruning: Pruning::default(),
        },
        false,
        NoVisualizer,
    )
}

#[bench]
fn dt_sh(bench: &mut Bencher) {
    let ref mut seed = 0;
    bench.iter(|| run_aligner(make_dt_sh(GapCostHeuristic::Enable), N, E, seed));
}
