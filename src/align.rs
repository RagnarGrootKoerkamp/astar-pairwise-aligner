use pa_types::{Cigar, Cost, Seq};
use serde::{Deserialize, Serialize};

use crate::cli::heuristic_params::HeuristicRunner;
use crate::heuristic::Heuristic;
use crate::stats::AstarStats;
use crate::visualizer::*;
use crate::{astar::astar, astar_dt::astar_dt, cli::heuristic_params::HeuristicArgs};

/// The main entrypoint for running A* with some parameters.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AstarPaParams<V: Visualizer> {
    pub diagonal_transition: bool,
    pub heuristic: HeuristicArgs,
    #[serde(default)]
    pub visualizer: V,
}

#[derive(Debug)]
pub struct AstarPa<V: Visualizer, H: Heuristic> {
    pub dt: bool,
    pub h: H,
    pub v: V,
}

impl<V: Visualizer> AstarPaParams<V> {
    pub fn align(&self, a: Seq, b: Seq) -> ((Cost, Cigar), AstarStats) {
        struct Runner<'a, V: Visualizer> {
            params: &'a AstarPaParams<V>,
            a: Seq<'a>,
            b: Seq<'a>,
        }
        impl<V: Visualizer> HeuristicRunner for Runner<'_, V> {
            type R = ((Cost, Cigar), AstarStats);
            fn call<H: Heuristic>(&self, h: H) -> Self::R {
                self.params.align_with_h(self.a, self.b, &h)
            }
        }

        self.heuristic
            .run_on_heuristic(Runner { params: self, a, b })
    }

    fn align_with_h<H: Heuristic>(&self, a: Seq, b: Seq, h: &H) -> ((Cost, Cigar), AstarStats) {
        if self.diagonal_transition {
            astar_dt(a, b, h, &self.visualizer)
        } else {
            astar(a, b, h, &self.visualizer)
        }
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
