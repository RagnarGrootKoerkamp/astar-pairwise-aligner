use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

use crate::*;
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
    /// Char frequencies to the target.
    Frequency,
    /// Seed heuristic.
    SH,
    /// Chaining seed heuristic.
    CSH,
    /// Gap-cost chaining seed heuristic.
    #[default]
    GCSH,

    // For testing
    /// Bruteforce GapCost
    BruteForceGapCost,
    /// Affine gap costs
    BruteForceAffineGapCost,
}

fn default_match_cost() -> MatchCost {
    2
}
fn default_seed_length() -> I {
    15
}
fn default_local_prune() -> usize {
    0
}
fn default_prune() -> Prune {
    Prune::Start
}

/// Heuristic arguments.
#[derive(Parser, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[clap(next_help_heading = "Heuristic")]
#[serde(deny_unknown_fields)]
pub struct HeuristicParams {
    #[clap(short = 'H', long, default_value_t, value_enum)]
    #[serde(rename = "type")]
    pub heuristic: HeuristicType,

    /// Seed potential
    ///
    /// 2 for inexact matches.
    #[clap(short = 'r', default_value_t = 2, value_name = "r")]
    #[serde(default = "default_match_cost")]
    pub r: MatchCost,

    /// Seed length
    #[clap(short, value_name = "k", default_value_t = 15)]
    #[serde(default = "default_seed_length")]
    pub k: I,

    /// Local-pruning length.
    #[clap(short, value_name = "p", default_value_t = 0)]
    #[serde(default = "default_local_prune")]
    pub p: usize,

    #[clap(long)]
    #[clap(long, action = clap::ArgAction::Set, default_value = "start")]
    #[serde(default = "default_prune")]
    pub prune: Prune,

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

    /// Skip pruning every Nth match.
    ///
    /// This is not useful for SH, where pruning is always efficient.
    #[clap(long, hide_short_help = true)]
    #[serde(default)]
    pub skip_prune: Option<usize>,
}

impl Default for HeuristicParams {
    fn default() -> Self {
        Self {
            heuristic: HeuristicType::GCSH,
            r: 2,
            k: 15,
            p: 0,
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
impl ToString for HeuristicParams {
    fn to_string(&self) -> String {
        match self.heuristic {
            HeuristicType::None => "".into(),
            HeuristicType::Zero => "Zero".into(),
            HeuristicType::Gap => "Gap-cost to end".into(),
            HeuristicType::Frequency => "Frequency".into(),
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
    fn call<H: Heuristic + 'static>(self, h: H) -> Self::R;
}

impl HeuristicParams {
    /// Apply a generic function F to the instantiated heuristic.
    pub fn map<F: HeuristicMapper>(&self, f: F) -> F::R {
        let match_config = MatchConfig {
            length: if let Some(max) = self.max_matches {
                LengthConfig::Max(crate::matches::MaxMatches {
                    max_matches: max,
                    k_min: self.kmin.unwrap_or(self.k),
                    k_max: self.kmax.unwrap_or(self.k),
                })
            } else {
                LengthConfig::Fixed(self.k)
            },
            r: self.r,
            local_pruning: self.p,
        };
        let pruning = Pruning {
            enabled: self.prune,
            skip_prune: self.skip_prune,
        };
        match self.heuristic {
            HeuristicType::None => f.call(NoCost),
            HeuristicType::Zero => f.call(ZeroCost),
            HeuristicType::Gap => f.call(GapCost),
            HeuristicType::Frequency => f.call(CountCost),
            HeuristicType::SH => f.call(SH::new(match_config, pruning)),
            HeuristicType::CSH => f.call(CSH::new(match_config, pruning)),
            HeuristicType::GCSH => f.call(GCSH::new(match_config, pruning)),
            // bruteforce variants
            HeuristicType::BruteForceGapCost => f.call(BruteForceGCSH {
                match_config,
                pruning,
                distance_function: GapCost,
            }),
            HeuristicType::BruteForceAffineGapCost => f.call(BruteForceGCSH {
                match_config,
                pruning,
                distance_function: AffineGapCost { k: self.k },
            }),
        }
    }
}
