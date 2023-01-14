use pa_types::{Cigar, Cost, Seq};
use serde::{Deserialize, Serialize};

use crate::stats::AstarStats;
use crate::visualizer::*;
use crate::{
    alignment_graph::*, astar, astar_dt, contour::*, heuristic::*, matches::*, prelude::*,
};
use clap::{Parser, ValueEnum};
use std::marker::PhantomData;

/// The type of the heuristic. Defaults to SH.
#[derive(Debug, PartialEq, Eq, Default, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum HeuristicType {
    None,
    Zero,
    Gap,
    #[default]
    SH,
    CSH,
}

fn default_match_cost() -> MatchCost {
    2
}
fn default_seed_length() -> I {
    15
}

/// Heuristic arguments.
#[derive(Parser, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[clap(next_help_heading = "Heuristic")]
#[serde(deny_unknown_fields)]
pub struct HeuristicArgs {
    #[clap(short = 'H', long, default_value_t, value_enum, display_order = 10)]
    #[serde(rename = "type")]
    pub heuristic: HeuristicType,

    /// Seed potential
    ///
    /// 2 for inexact matches.
    #[clap(short = 'r', default_value_t = 2, value_name = "r", display_order = 10)]
    #[serde(default = "default_match_cost")]
    pub r: MatchCost,

    /// Seed length
    #[clap(short, value_name = "k", display_order = 10, default_value_t = 15)]
    #[serde(default = "default_seed_length")]
    pub k: I,

    /// Minimal seed length
    #[clap(long, hide_short_help = true)]
    #[serde(default)]
    pub kmin: Option<I>,

    /// Maximal seed length
    #[clap(long, hide_short_help = true)]
    #[serde(default)]
    pub kmax: Option<I>,

    /// The maximal number of matches per seed
    #[clap(long, hide_short_help = true)]
    #[serde(default)]
    pub max_matches: Option<usize>,

    /// Disable pruning
    #[clap(long, hide_short_help = true)]
    #[serde(default)]
    pub no_prune: bool,

    /// Skip pruning every Nth match. 0 to disable
    ///
    /// This is only used for CSH, not for SH.
    #[clap(long, hide_short_help = true, default_value_t = 0)]
    #[serde(default)]
    pub skip_prune: usize,

    /// Use gap-cost for CSH.
    #[clap(long, hide_short_help = true)]
    #[serde(default)]
    pub gap_cost: bool,
}

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

/// A summary string for the visualizer.
/// Only includes parameters that change the type of algorithm, not numerical values.
impl ToString for HeuristicArgs {
    fn to_string(&self) -> String {
        match self.heuristic {
            HeuristicType::None => "".into(),
            HeuristicType::Zero => "Zero".into(),
            HeuristicType::Gap => "Gap-cost to end".into(),
            HeuristicType::SH => {
                let mut s = format!("Seed Heuristic (r={}, k={})", self.r, self.k);
                if self.no_prune {
                    s += " (no pruning)"
                } else {
                    s += " + Pruning"
                }
                s
            }
            HeuristicType::CSH => {
                let mut s = format!("Chaining Seed Heuristic (r={}, k={})", self.r, self.k);
                if self.no_prune {
                    s += " (no pruning)"
                } else {
                    s += " + Pruning"
                }
                if self.gap_cost {
                    s += " + Gap Cost"
                }
                s
            }
        }
    }
}

pub trait HeuristicMapper {
    type R;
    fn call<H: Heuristic + 'static>(&self, h: H) -> Self::R;
}

impl HeuristicArgs {
    fn match_config(&self, window_filter: bool) -> MatchConfig {
        let r = self.r;
        let k = self.k;
        MatchConfig {
            length: if let Some(max) = self.max_matches {
                LengthConfig::Max(crate::matches::MaxMatches {
                    max_matches: max,
                    k_min: self.kmin.unwrap_or(k),
                    k_max: self.kmax.unwrap_or(k),
                })
            } else {
                LengthConfig::Fixed(k)
            },
            max_match_cost: r - 1,
            window_filter,
        }
    }

    /// Apply a generic function F to the instantiated heuristic.
    pub fn map<F: HeuristicMapper>(&self, f: F) -> F::R {
        match self.heuristic {
            HeuristicType::None => f.call(NoCost),
            HeuristicType::Zero => f.call(ZeroCost),
            HeuristicType::Gap => f.call(GapCost),
            HeuristicType::CSH => f.call(CSH {
                match_config: self.match_config(self.gap_cost),
                pruning: Pruning {
                    enabled: !self.no_prune,
                    skip_prune: self.skip_prune,
                },
                use_gap_cost: self.gap_cost,
                c: PhantomData::<HintContours<BruteForceContour>>,
            }),
            HeuristicType::SH => f.call(SH {
                match_config: self.match_config(false),
                pruning: Pruning {
                    enabled: !self.no_prune,
                    skip_prune: self.skip_prune,
                },
            }),
        }
    }
}
