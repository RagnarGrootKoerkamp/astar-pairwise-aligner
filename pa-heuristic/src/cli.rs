use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::heuristic::*;
use crate::matches::*;
use pa_types::*;

/// The type of the heuristic. Defaults to SH.
#[derive(Debug, PartialEq, Eq, Default, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum HeuristicType {
    /// No heuristic.
    None,
    /// A heuristic that returns 0.
    Zero,
    /// Gap-cost to the target.
    Gap,
    /// Seed heuristic.
    SH,
    /// Chaining seed heuristic.
    CSH,
    /// Gap-cost chaining seed heuristic.
    #[default]
    GCSH,

    // For testing
    /// Bruteforce GapCost
    GapCost,
    /// Affine gap costs
    Affine,
}

fn default_match_cost() -> MatchCost {
    2
}
fn default_seed_length() -> I {
    15
}
fn default_prune() -> Prune {
    Prune::Both
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
    #[clap(long, action = clap::ArgAction::Set, default_value = "none")]
    #[serde(default = "default_prune")]
    pub prune: Prune,

    /// Skip pruning every Nth match.
    ///
    /// This is only used for CSH where skipping can give a speedup, not for SH.
    #[clap(long, hide_short_help = true)]
    #[serde(default)]
    pub skip_prune: Option<usize>,
}

impl Default for HeuristicArgs {
    fn default() -> Self {
        Self {
            heuristic: HeuristicType::GCSH,
            r: 2,
            k: 15,
            prune: Prune::Start,
            kmin: None,
            kmax: None,
            max_matches: None,
            skip_prune: None,
        }
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
                if self.prune.is_enabled() {
                    s += " + Pruning"
                } else {
                    s += " (no pruning)"
                }
                s
            }
            HeuristicType::CSH => {
                let mut s = format!("Chaining Seed Heuristic (r={}, k={})", self.r, self.k);
                if self.prune.is_enabled() {
                    s += " + Pruning"
                } else {
                    s += " (no pruning)"
                }
                s
            }
            HeuristicType::GCSH => {
                let mut s = format!(
                    "Gap-cost chaining Seed Heuristic (r={}, k={})",
                    self.r, self.k
                );
                if self.prune.is_enabled() {
                    s += " + Pruning"
                } else {
                    s += " (no pruning)"
                }
                s
            }
            _ => panic!(),
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
            HeuristicType::SH => f.call(SH {
                match_config: self.match_config(false),
                pruning: Pruning {
                    enabled: self.prune,
                    skip_prune: self.skip_prune,
                },
            }),
            HeuristicType::CSH => f.call(CSH::new(
                self.match_config(false),
                Pruning {
                    enabled: self.prune,
                    skip_prune: self.skip_prune,
                },
            )),
            HeuristicType::GCSH => f.call(GCSH::new(
                self.match_config(true),
                Pruning {
                    enabled: self.prune,
                    skip_prune: self.skip_prune,
                },
            )),
            // bruteforce variants
            HeuristicType::GapCost => f.call(BruteForceGCSH {
                match_config: self.match_config(false),
                pruning: Pruning {
                    enabled: self.prune,
                    skip_prune: self.skip_prune,
                },
                distance_function: GapCost,
            }),
            HeuristicType::Affine => f.call(BruteForceGCSH {
                match_config: self.match_config(false),
                pruning: Pruning {
                    enabled: self.prune,
                    skip_prune: self.skip_prune,
                },
                distance_function: AffineGapCost { k: self.k },
            }),
        }
    }
}
