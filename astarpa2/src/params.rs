use super::*;
use pa_heuristic::{GapCost, HeuristicMapper, HeuristicParams, NoCost};
use pa_vis::NoVis;
use serde::{Deserialize, Serialize};
use Domain::*;

/// Flat, untyped parameters for A*PA2 that can be used for CLI or pa-bench.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct AstarPa2Params {
    /// An optional name for the parameter set.
    #[serde(default)]
    pub name: String,

    /// The domain to compute.
    pub domain: Domain<()>,

    /// Heuristic to use for A* domain.
    pub heuristic: HeuristicParams,

    /// The strategy to use to compute the given domain.
    pub doubling: band::DoublingType,

    /// Compute `block_width` columns at a time, to reduce overhead of metadata
    /// computations.
    pub block_width: I,

    /// The front type to use.
    pub front: BlockParams,

    /// When true, `j_range` skips querying `h` when it can assuming consistency.
    #[serde(default)]
    pub sparse_h: bool,

    /// Whether pruning is enabled.
    #[serde(default)]
    pub prune: bool,

    /// Whether the visualizer is enabled.
    #[serde(default)]
    pub viz: bool,
}

impl AstarPa2Params {
    pub fn simple() -> Self {
        Self {
            name: "simple".into(),
            domain: Astar(()),
            heuristic: HeuristicParams {
                heuristic: pa_heuristic::HeuristicType::Gap,
                ..Default::default()
            },
            doubling: band::DoublingType::BandDoubling {
                start: DoublingStart::H0,
                factor: 2.0,
            },
            block_width: 256,
            front: BlockParams {
                sparse: true,
                simd: true,
                no_ilp: false,
                incremental_doubling: false,
                dt_trace: true,
                max_g: 40,
                fr_drop: 10,
            },
            sparse_h: true,
            prune: false,
            viz: false,
        }
    }

    pub fn full() -> Self {
        Self {
            name: "full".into(),
            domain: Astar(()),
            heuristic: HeuristicParams {
                heuristic: pa_heuristic::HeuristicType::GCSH,
                prune: pa_heuristic::Prune::Start,
                k: 12,
                r: 1,
                p: 14,
                ..Default::default()
            },
            doubling: band::DoublingType::BandDoubling {
                start: DoublingStart::H0,
                factor: 2.0,
            },
            block_width: 256,
            front: BlockParams {
                sparse: true,
                simd: true,
                no_ilp: false,
                incremental_doubling: true,
                dt_trace: true,
                max_g: 40,
                fr_drop: 10,
            },
            sparse_h: true,
            prune: true,
            viz: false,
        }
    }

    /// Convert to a typed `AstarPa2` `Aligner` instance, using a visualizer is
    /// if the `pa-vis` feature is enabled.
    pub fn make_aligner(&self, trace: bool) -> Box<dyn AstarPa2StatsAligner> {
        #[cfg(feature = "example")]
        if self.viz {
            use pa_vis::visualizer::{Gradient, When};
            use pa_vis::canvas::RED;
            use std::time::{Duration, SystemTime};

            let mut config = pa_vis::visualizer::Config::default();
            config.draw = When::StepBy(1);
            config.save = When::None; //When::LayersStepBy(30);
            config.save_last = false;
            config.delay = Duration::from_secs_f32(0.0001);
            config.cell_size = 0;
            config.downscaler = 0;
            config.style.bg_color = (255, 255, 255, 128);
            config.style.expanded = Gradient::TurboGradient(0.25..0.90);
            config.style.path_width = None;
            config.layer_drawing = false;
            config.style.draw_dt = false;
            config.style.draw_heuristic = false;
            config.style.draw_f = false;
            config.style.draw_h_calls = true;
            config.style.draw_labels = false;
            config.transparent_bmp = true;
            config.draw_old_on_top = false;
            config.paused = true;

            config.style.pruned_match = RED;
            config.style.match_width = 1;
            config.style.draw_matches = false;
            config.filepath = format!(
                "imgs/vis/{}-{}",
                self.name,
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            )
            .into();
            self.make_aligner_with_visualizer(trace, config)
        } else {
            self.make_aligner_with_visualizer(trace, NoVis)
        }
        #[cfg(not(feature = "example"))]
        self.make_aligner_with_visualizer(trace, NoVis)
    }

    /// Convert to a typed `AstarPa2` `Aligner` instance, with a given visualizer.
    pub fn make_aligner_with_visualizer<V: VisualizerT + 'static>(
        &self,
        trace: bool,
        v: V,
    ) -> Box<dyn AstarPa2StatsAligner> {
        struct Mapper<V: VisualizerT> {
            params: AstarPa2Params,
            trace: bool,
            v: V,
        }
        impl<V: VisualizerT + 'static> HeuristicMapper for Mapper<V> {
            type R = Box<dyn AstarPa2StatsAligner>;
            fn call<H: Heuristic + 'static>(self, h: H) -> Box<dyn AstarPa2StatsAligner> {
                Box::new(AstarPa2 {
                    domain: Domain::Astar(h),
                    doubling: self.params.doubling,
                    block_width: self.params.block_width,
                    v: self.v,
                    block: self.params.front,
                    trace: self.trace,
                    sparse_h: self.params.sparse_h,
                    prune: self.params.prune,
                })
            }
        }
        match self.domain {
            Domain::Astar(()) => self.heuristic.map(Mapper {
                params: self.clone(),
                trace,
                v,
            }),
            d => Box::new(AstarPa2 {
                domain: d.into(),
                doubling: self.doubling,
                block_width: self.block_width,
                v,
                block: self.front,
                trace,
                sparse_h: self.sparse_h,
                prune: self.prune,
            }),
        }
    }
}

/// Enum for the various computational domain types.
/// See Ukkonen, Scrooge, O(NP), Papamichail, A*PA
///
/// Distance from start can be none, gap, or g*
/// Distance to end can be none, gap, h
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Domain<H> {
    /// Compute the entire rectangle
    Full,
    /// States with gap(s, u) <= f
    GapStart,
    /// States with gap(s, u) + gap(u, t) <= f
    GapGap,
    /// States with g(u) + h(u) <= f, for some arbitrary h.
    /// For Dijkstra, use H=NoCost.
    /// For GapCost to end, use H=GapCost.
    Astar(H),
}

impl Default for Domain<()> {
    fn default() -> Self {
        Astar(())
    }
}

impl Domain<()> {
    pub fn into(self) -> Domain<NoCost> {
        match self {
            Full => Full,
            GapStart => GapStart,
            GapGap => GapGap,
            Astar(_) => panic!(),
        }
    }
}

impl Domain<NoCost> {
    pub fn full() -> Self {
        Full
    }
    pub fn gap_start() -> Self {
        GapStart
    }
    pub fn gap_gap() -> Self {
        GapGap
    }
    pub fn dijkstra() -> Self {
        Astar(NoCost)
    }
}

impl Domain<GapCost> {
    pub fn dist_gap() -> Self {
        Astar(GapCost)
    }
}

impl<H> Domain<H> {
    pub fn astar(h: H) -> Self {
        Astar(h)
    }

    pub fn h(&self) -> Option<&H> {
        match self {
            Astar(h) => Some(&h),
            _ => None,
        }
    }
    pub fn h_mut(&mut self) -> Option<&mut H> {
        match self {
            Astar(h) => Some(h),
            _ => None,
        }
    }
}
