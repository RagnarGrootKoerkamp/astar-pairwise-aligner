//!
//! # A*PA library
//!
//! This crate is the entrypoint of the A*PA library.
//! It can be used in a few ways:
//! - Call `astar` or `astar_dt` directly using a heuristic and visualizer.
//! - Create a reusable `AstarPa` `Aligner` object.
//! - Create a simpler `AstarPaParams` object.
//!
//! The difference between `AstarPa` and `AstarPaParams` is that the first
//! requires an instantiated heuristic type, whereas the letter can be
//! configured using `HeuristicArgs` and instantiates the heuristic for you.
//!
#![feature(
    test,
    array_methods,
    duration_constants,
    step_trait,
    int_roundings,
    iter_intersperse,
    slice_as_chunks,
    let_chains,
    is_sorted,
    exclusive_range_pattern,
    associated_type_defaults,
    hash_drain_filter,
    drain_filter
)]

mod alignment_graph;
mod astar;
mod astar_dt;
mod bucket_queue;
mod config;
#[cfg(test)]
mod tests;

pub mod cli;
pub mod stats;

// The main alignment functions.
pub use astar::astar;
pub use astar_dt::astar_dt;

mod prelude {
    pub use pa_types::*;
    pub use rustc_hash::FxHashMap as HashMap;
    pub use rustc_hash::FxHashSet as HashSet;
    pub use std::cmp::{max, min};

    pub use crate::config::*;
}

// ------------ Root alignment interface follows from here ------------

use pa_affine_types::{AffineAligner, AffineCigar};
use pa_heuristic::HeuristicArgs;
use pa_heuristic::{Heuristic, HeuristicMapper};
use pa_types::{Cigar, Cost, Seq};
use pa_vis_types::{NoVis, VisualizerT};
use serde::{Deserialize, Serialize};
use stats::AstarStats;

/// The main entrypoint for running A* with some parameters.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AstarPaParams<V: VisualizerT> {
    pub diagonal_transition: bool,
    pub heuristic: HeuristicArgs,
    #[serde(default)]
    pub visualizer: V,
}

pub type AstarPaParamsNoVis = AstarPaParams<NoVis>;

impl AstarPaParams<NoVis> {
    pub fn new(diagonal_transition: bool, heuristic: HeuristicArgs) -> Self {
        Self {
            diagonal_transition,
            heuristic,
            visualizer: NoVis,
        }
    }
}

/// Alternative configuration using a typed `Heuristic` instance instead of a fixed config.
#[derive(Debug)]
pub struct AstarPa<V: VisualizerT, H: Heuristic> {
    pub dt: bool,
    pub h: H,
    pub v: V,
}

impl<V: VisualizerT + 'static> AstarPaParams<V> {
    pub fn new_with_vis(
        diagonal_transition: bool,
        heuristic: HeuristicArgs,
        visualizer: V,
    ) -> Self {
        Self {
            diagonal_transition,
            heuristic,
            visualizer,
        }
    }

    pub fn aligner(&self) -> Box<dyn AstarPaAligner> {
        struct Runner<'a, V: VisualizerT> {
            params: &'a AstarPaParams<V>,
        }
        impl<V: VisualizerT + 'static> HeuristicMapper for Runner<'_, V> {
            type R = Box<dyn AstarPaAligner>;
            fn call<H: Heuristic + 'static>(&self, h: H) -> Box<dyn AstarPaAligner> {
                Box::new(AstarPa {
                    dt: self.params.diagonal_transition,
                    h,
                    v: self.params.visualizer.clone(),
                })
            }
        }

        self.heuristic.map(Runner { params: self })
    }

    pub fn align(&self, a: Seq, b: Seq) -> ((Cost, Cigar), AstarStats) {
        self.aligner().align(a, b)
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

/// Helper trait to work with a `Box<dyn AstarPaAligner>` where the type of the
/// heuristic is hidden.
pub trait AstarPaAligner {
    fn align(&self, a: Seq, b: Seq) -> ((Cost, Cigar), AstarStats);
}

impl<V: VisualizerT, H: Heuristic> AstarPaAligner for AstarPa<V, H> {
    fn align(&self, a: Seq, b: Seq) -> ((Cost, Cigar), AstarStats) {
        self.align(a, b)
    }
}

impl<V: VisualizerT, H: Heuristic> AffineAligner for AstarPa<V, H> {
    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Option<AffineCigar>) {
        let ((cost, ref cigar), _stats) = AstarPa::align(self, a, b);
        (cost, Some(cigar.into()))
    }
}
