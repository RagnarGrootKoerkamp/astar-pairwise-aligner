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

pub struct Astar<V: VisualizerConfig, H: Heuristic> {
    pub dt: bool,

    /// The heuristic to use.
    pub h: H,

    /// The visualizer to use.
    pub v: V,
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

impl Astar<NoVisualizer, ZeroCost> {
    /// FIXME: FIGURE OUT WHY +'static IS NEEDED HERE??
    fn from_args_with_v<'a, V: VisualizerConfig + 'a + 'static>(
        dt: bool,
        h: &HeuristicArgs,
        v: V,
    ) -> Box<dyn AstarAligner> {
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

    pub fn from_args(
        dt: bool,
        h_args: &HeuristicArgs,
        v_args: &VisualizerArgs,
    ) -> Box<dyn AstarAligner> {
        match v_args.make_visualizer() {
            VisualizerType::NoVizualizer => Self::from_args_with_v(dt, h_args, NoVisualizer),
            #[cfg(any(feature = "sdl2", feature = "wasm"))]
            VisualizerType::Visualizer(config) => Self::from_args_with_v(dt, h_args, config),
        }
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
