use std::marker::PhantomData;

use crate::astar::AstarStats;
use crate::heuristic::Heuristic;
use crate::{
    astar::astar,
    astar_dt::astar_dt,
    cli::{
        heuristic_params::{HeuristicArgs, HeuristicType},
        visualizer::{VisualizerArgs, VisualizerType},
    },
    heuristic::{GapCost, NoCost, Pruning, ZeroCost, CSH, SH},
    prelude::{BruteForceContour, HintContours},
    visualizer::{NoVisualizer, VisualizerConfig},
};

use super::Aligner;

/// The main entrypoint for running A* with some parameters.
pub struct AstarParams {
    pub diagonal_transition: bool,
    pub heuristic: HeuristicArgs,
}

/// Generic A* instance for the chosen heuristic and visualizer.
pub struct Astar<V: VisualizerConfig, H: Heuristic> {
    pub dt: bool,
    /// The heuristic to use.
    pub h: H,
    /// The visualizer to use.
    pub v: V,
}

impl AstarParams {
    pub fn aligner_with_visualizer(&self, v_args: &VisualizerArgs) -> Box<dyn AstarAligner> {
        match v_args.make_visualizer() {
            VisualizerType::NoVizualizer => self.generic_algner(NoVisualizer),
            #[cfg(any(feature = "vis", feature = "wasm"))]
            VisualizerType::Visualizer(config) => self.generic_aligner(config),
        }
    }

    pub fn aligner(&self) -> Box<dyn AstarAligner> {
        self.generic_algner(NoVisualizer)
    }

    fn generic_algner<'a, V: VisualizerConfig + 'a + 'static>(
        &self,
        v: V,
    ) -> Box<dyn AstarAligner> {
        let &AstarParams {
            diagonal_transition: dt,
            heuristic: ref h,
        } = self;
        match h.heuristic {
            HeuristicType::None => Box::new(Astar::new(dt, NoCost, v)),
            HeuristicType::Zero => Box::new(Astar::new(dt, ZeroCost, v)),
            HeuristicType::Gap => Box::new(Astar::new(dt, GapCost, v)),
            HeuristicType::SH => Box::new(Astar::new(
                dt,
                SH {
                    match_config: h.match_config(false),
                    pruning: Pruning {
                        enabled: !h.no_prune,
                        skip_prune: h.skip_prune,
                    },
                },
                v,
            )),
            HeuristicType::CSH => Box::new(Astar::new(
                dt,
                CSH {
                    match_config: h.match_config(h.gap_cost),
                    pruning: Pruning {
                        enabled: !h.no_prune,
                        skip_prune: h.skip_prune,
                    },
                    use_gap_cost: h.gap_cost,
                    c: PhantomData::<HintContours<BruteForceContour>>,
                },
                v,
            )),
        }
    }
}

impl<V: VisualizerConfig, H: Heuristic> Astar<V, H> {
    fn new(dt: bool, h: H, v: V) -> Self {
        Astar { dt, h, v }
    }
}

pub trait AstarAligner: Aligner {
    fn align_with_stats(
        &mut self,
        a: super::Seq,
        b: super::Seq,
    ) -> ((crate::cost_model::Cost, super::cigar::Cigar), AstarStats);
}

impl<V: VisualizerConfig, H: Heuristic> AstarAligner for Astar<V, H> {
    fn align_with_stats(
        &mut self,
        a: super::Seq,
        b: super::Seq,
    ) -> ((crate::cost_model::Cost, super::cigar::Cigar), AstarStats) {
        if self.dt {
            astar_dt(a, b, &self.h, &self.v)
        } else {
            astar(a, b, &self.h, &self.v)
        }
    }
}

impl<V: VisualizerConfig, H: Heuristic> std::fmt::Debug for Astar<V, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Astar")
            .field("dt", &self.dt)
            .field("h", &self.h)
            .finish()
    }
}

impl<V: VisualizerConfig, H: Heuristic> Aligner for Astar<V, H> {
    fn align(
        &mut self,
        a: super::Seq,
        b: super::Seq,
    ) -> (crate::cost_model::Cost, super::cigar::Cigar) {
        if self.dt {
            astar_dt(a, b, &self.h, &self.v).0
        } else {
            astar(a, b, &self.h, &self.v).0
        }
    }
}
