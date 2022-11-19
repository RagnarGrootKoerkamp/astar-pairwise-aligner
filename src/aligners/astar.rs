use std::marker::PhantomData;

use crate::heuristic::Heuristic;
use crate::{
    astar::astar_wrap,
    astar_dt::astar_dt_wrap,
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

    // fn align_with_stats(
    //     &mut self,
    //     a: super::Seq,
    //     b: super::Seq,
    // ) -> ((crate::cost_model::Cost, super::cigar::Cigar), AlignResult) {
    //     if self.dt {
    //         astar_dt_wrap(a, b, &self.h, &self.v).0
    //     } else {
    //         astar_wrap(a, b, &self.h, &self.v).0
    //     }
    // }
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
    fn from_args_with_v<'a, V: VisualizerConfig + 'a>(
        dt: bool,
        h: &HeuristicArgs,
        v: V,
    ) -> Box<dyn Aligner + 'a> {
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
    ) -> Box<dyn Aligner> {
        match v_args.make_visualizer() {
            VisualizerType::NoVizualizer => Self::from_args_with_v(dt, h_args, NoVisualizer),
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
            astar_dt_wrap(a, b, &self.h, &self.v).0
        } else {
            astar_wrap(a, b, &self.h, &self.v).0
        }
    }
}
