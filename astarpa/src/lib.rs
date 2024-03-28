//!
//! # A*PA library
//!
//! This crate is the entrypoint of the A*PA library.
//! It can be used in a few ways. From simple to generic:
//! - `astarpa(a,b)`
//! - `astarpa_gcsh(a,b,r,k,Prune)`
//! - `make_aligner(dt: bool, h: HeuristicParams).align(a,b)`
//! - `AstarPa{ dt: bool, h: Heuristic, v: VisualizerT}.align(a,b)`
//! The last 2 methods create an aligner object that can be reused.
//!
#![feature(
    test,
    duration_constants,
    step_trait,
    int_roundings,
    iter_intersperse,
    slice_as_chunks,
    let_chains,
    is_sorted,
    exclusive_range_pattern,
    associated_type_defaults,
    // If you get compile errors here, make sure to be on at least rust 1.72.
    // Before, this was called `drain_filter`.
    extract_if
)]

mod alignment_graph;
mod astar;
mod astar_dt;
mod bucket_queue;
mod config;
#[cfg(test)]
mod tests;

pub mod stats;

mod prelude {
    pub use pa_types::*;
    pub use rustc_hash::FxHashMap as HashMap;

    pub use crate::config::*;
}

use pa_heuristic::seeds::MatchCost;
use pa_heuristic::{Heuristic, HeuristicMapper, Prune};
use pa_heuristic::{MatchConfig, Pruning, GCSH};
use pa_types::{Aligner, Cigar, Cost, Seq, I};
use pa_vis::{NoVis, VisualizerT};
use stats::AstarStats;

// ------------ Root alignment interface follows from here ------------

pub use astar::{astar, astar_with_vis};
pub use astar_dt::astar_dt;
pub use pa_heuristic::HeuristicParams;

/// Align using default settings:
/// - Gap-cost chaining seed heuristic (GCSH)
/// - with diagonal transition (DT)
/// - inexact matches (r=2)
/// - seed length k=15
/// - prune by start only.
pub fn astarpa(a: Seq, b: Seq) -> (Cost, Cigar) {
    astarpa_gcsh(a, b, 2, 15, Prune::Start)
}

/// Align using GCSH with DT, with custom parameters.
/// - r=1 instead of r=2 can be used when the error rate is low.
/// - pruning by start *and* end (`Prune::Both`) can help for higher error rates where there are not many spurious matches.
pub fn astarpa_gcsh(a: Seq, b: Seq, r: MatchCost, k: I, pruning: Prune) -> (Cost, Cigar) {
    astar_dt::astar_dt(
        a,
        b,
        &GCSH::new(MatchConfig::new(k, r), Pruning::new(pruning)),
        &NoVis,
    )
    .0
}

/// Build an `AstarStatsAligner` instance from
pub fn make_aligner(dt: bool, h: &HeuristicParams) -> Box<dyn AstarStatsAligner> {
    make_aligner_with_visualizer(dt, h, NoVis)
}

/// Build a type-erased aligner object from parameters.
pub fn make_aligner_with_visualizer<V: VisualizerT + 'static>(
    dt: bool,
    h: &HeuristicParams,
    v: V,
) -> Box<dyn AstarStatsAligner> {
    struct Mapper<V: VisualizerT> {
        dt: bool,
        v: V,
    }
    impl<V: VisualizerT + 'static> HeuristicMapper for Mapper<V> {
        type R = Box<dyn AstarStatsAligner>;
        fn call<H: Heuristic + 'static>(self, h: H) -> Box<dyn AstarStatsAligner> {
            Box::new(AstarPa {
                dt: self.dt,
                h,
                v: self.v,
            })
        }
    }

    h.map(Mapper { dt, v })
}

/// Align using a reusable object containing all parameters.
#[derive(Debug)]
pub struct AstarPa<V: VisualizerT, H: Heuristic> {
    pub dt: bool,
    pub h: H,
    pub v: V,
}

impl<H: Heuristic> AstarPa<NoVis, H> {
    pub fn new(dt: bool, h: H) -> Self {
        AstarPa { dt, h, v: NoVis }
    }
}
impl<V: VisualizerT, H: Heuristic> AstarPa<V, H> {
    pub fn align(&self, a: Seq, b: Seq) -> ((Cost, Cigar), AstarStats) {
        if self.dt {
            astar_dt(a, b, &self.h, &self.v)
        } else {
            astar(a, b, &self.h, &self.v)
        }
    }
}

/// Helper trait to erase the type of the heuristic that additionally returns alignment statistics.
pub trait AstarStatsAligner: Aligner {
    fn align(&self, a: Seq, b: Seq) -> ((Cost, Cigar), AstarStats);
}

// Implement aligner traits.
impl<V: VisualizerT, H: Heuristic> AstarStatsAligner for AstarPa<V, H> {
    fn align(&self, a: Seq, b: Seq) -> ((Cost, Cigar), AstarStats) {
        self.align(a, b)
    }
}

/// A simple aligner interface.
impl<V: VisualizerT, H: Heuristic> Aligner for AstarPa<V, H> {
    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Option<Cigar>) {
        let ((cost, cigar), _stats) = AstarPa::align(self, a, b);
        (cost, Some(cigar))
    }
}
