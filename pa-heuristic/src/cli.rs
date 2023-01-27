use std::marker::PhantomData;

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::contour::*;
use crate::heuristic::*;
use crate::matches::*;
use pa_types::*;

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
fn default_prune() -> bool {
    true
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
    #[clap(long, action = clap::ArgAction::Set, default_value = "true")]
    #[serde(default = "default_prune")]
    pub prune: bool,

    /// Skip pruning every Nth match.
    ///
    /// This is only used for CSH where skipping can give a speedup, not for SH.
    #[clap(long, hide_short_help = true)]
    #[serde(default)]
    pub skip_prune: Option<usize>,

    /// Use gap-cost for CSH.
    #[clap(long, hide_short_help = true)]
    pub gap_cost: bool,
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
                if self.prune {
                    s += " + Pruning"
                } else {
                    s += " (no pruning)"
                }
                s
            }
            HeuristicType::CSH => {
                let mut s = format!("Chaining Seed Heuristic (r={}, k={})", self.r, self.k);
                if self.prune {
                    s += " + Pruning"
                } else {
                    s += " (no pruning)"
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
    pub fn match_config(&self, window_filter: bool) -> MatchConfig {
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
                    enabled: self.prune,
                    skip_prune: self.skip_prune,
                },
                use_gap_cost: self.gap_cost,
                c: PhantomData::<HintContours<BruteForceContour>>,
            }),
            HeuristicType::SH => f.call(SH {
                match_config: self.match_config(false),
                pruning: Pruning {
                    enabled: self.prune,
                    skip_prune: self.skip_prune,
                },
            }),
        }
    }
}
