use std::marker::PhantomData;

use pa_types::{Cigar, Cost, Seq};
use serde::{Deserialize, Serialize};

use crate::heuristic::Heuristic;
use crate::stats::AstarStats;
use crate::visualizer_trait::*;
use crate::{
    astar::astar,
    astar_dt::astar_dt,
    cli::heuristic_params::{HeuristicArgs, HeuristicType},
    heuristic::{GapCost, NoCost, Pruning, ZeroCost, CSH, SH},
    prelude::{BruteForceContour, HintContours},
};

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
        let h = &self.heuristic;
        match h.heuristic {
            HeuristicType::None => self.align_with_h(a, b, &NoCost),
            HeuristicType::Zero => self.align_with_h(a, b, &ZeroCost),
            HeuristicType::Gap => self.align_with_h(a, b, &GapCost),
            HeuristicType::SH => self.align_with_h(
                a,
                b,
                &SH {
                    match_config: h.match_config(false),
                    pruning: Pruning {
                        enabled: !h.no_prune,
                        skip_prune: h.skip_prune,
                    },
                },
            ),
            HeuristicType::CSH => self.align_with_h(
                a,
                b,
                &CSH {
                    match_config: h.match_config(h.gap_cost),
                    pruning: Pruning {
                        enabled: !h.no_prune,
                        skip_prune: h.skip_prune,
                    },
                    use_gap_cost: h.gap_cost,
                    c: PhantomData::<HintContours<BruteForceContour>>,
                },
            ),
        }
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
