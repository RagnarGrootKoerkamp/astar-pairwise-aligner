use pa_heuristic::{Heuristic, HeuristicMapper};
use pa_types::{Cigar, Cost, Seq};
use pa_vis_types::{NoVis, Visualizer};
use serde::{Deserialize, Serialize};

use crate::stats::AstarStats;
use crate::{astar, astar_dt};
use pa_heuristic::HeuristicArgs;

/// The main entrypoint for running A* with some parameters.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AstarPaParams<V: Visualizer> {
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
pub struct AstarPa<V: Visualizer, H: Heuristic> {
    pub dt: bool,
    pub h: H,
    pub v: V,
}

impl<V: Visualizer + 'static> AstarPaParams<V> {
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
        struct Runner<'a, V: Visualizer> {
            params: &'a AstarPaParams<V>,
        }
        impl<V: Visualizer + 'static> HeuristicMapper for Runner<'_, V> {
            type R = Box<dyn AstarPaAligner>;
            fn call<H: Heuristic + 'static>(&self, h: H) -> Box<dyn AstarPaAligner> {
                Box::new(AstarPa {
                    dt: self.params.diagonal_transition,
                    h,
                    v: self.params.visualizer,
                })
            }
        }

        self.heuristic.map(Runner { params: self })
    }

    pub fn align(&self, a: Seq, b: Seq) -> ((Cost, Cigar), AstarStats) {
        self.aligner().align(a, b)
    }
}

impl<V: Visualizer, H: Heuristic> AstarPa<V, H> {
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

impl<V: Visualizer, H: Heuristic> AstarPaAligner for AstarPa<V, H> {
    fn align(&self, a: Seq, b: Seq) -> ((Cost, Cigar), AstarStats) {
        self.align(a, b)
    }
}
