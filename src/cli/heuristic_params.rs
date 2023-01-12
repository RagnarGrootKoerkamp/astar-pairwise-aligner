use std::marker::PhantomData;

use crate::{
    heuristic::{GapCost, Heuristic, NoCost, Pruning, ZeroCost, CSH, SH},
    matches::{LengthConfig, MatchConfig},
    prelude::{BruteForceContour, HintContours, MatchCost, I},
};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

/// TODO: Add other aligners here as well.
#[derive(Debug, PartialEq, Default, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum Algorithm {
    // See HeuristicArgs for configuration.
    #[default]
    Astar,

    // Internal reimplementations
    NW,
    DT,

    // External methods
    TripleAccel,
    Edlib,
    Biwfa,
}

impl Algorithm {
    pub fn external(&self) -> bool {
        match self {
            Algorithm::TripleAccel | Algorithm::Edlib | Algorithm::Biwfa => true,
            Algorithm::NW | Algorithm::DT | Algorithm::Astar => false,
        }
    }
    pub fn internal(&self) -> bool {
        !self.external()
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum HeuristicType {
    None,
    Zero,
    Gap,
    #[default]
    SH,
    CSH,
}

#[derive(Parser, Debug, Serialize, Deserialize)]
#[clap(help_heading = "ALGORITHM")]
pub struct AlgorithmArgs {
    #[clap(short, long, default_value_t, value_enum, display_order = 10)]
    pub algorithm: Algorithm,

    /// Use diagonal-transition based A*.
    #[clap(long, hide_short_help = true)]
    pub dt: bool,

    /// Use exponential search in NW algorithm (like edlib).
    #[clap(long, hide_short_help = true)]
    pub exp_search: bool,

    /// Use local doubling in NW/DT.
    #[clap(long, hide_short_help = true)]
    pub local_doubling: bool,

    /// Use divide and conquer for diagonal transition (like BiWFA).
    #[clap(long, hide_short_help = true)]
    pub dc: bool,
}

/// Convert to a title string for the visualizer.
impl ToString for AlgorithmArgs {
    fn to_string(&self) -> String {
        match self.algorithm {
            Algorithm::NW => {
                let mut s = "Needleman-Wunsch".to_string();
                if self.exp_search {
                    s += " + Doubling";
                }
                if self.local_doubling {
                    s += " + Local Doubling";
                }
                s
            }
            Algorithm::DT => {
                let mut s = "Diagonal Transition".to_string();
                if self.dc {
                    s += " + Divide & Conquer";
                }
                if self.local_doubling {
                    s += " + Local Doubling";
                }
                s
            }
            Algorithm::Astar => {
                let mut s = "A*".to_string();
                if self.dt {
                    s += " + Diagonal Transition";
                }
                s
            }
            Algorithm::TripleAccel => {
                if self.exp_search {
                    "Needleman-Wunsch + Doubling (triple-accel)".into()
                } else {
                    "Needleman-Wunsch (triple-accel)".into()
                }
            }
            Algorithm::Edlib => "Edlib".into(),
            Algorithm::Biwfa => "BiWFA".into(),
        }
    }
}

fn default_match_cost() -> MatchCost {
    2
}
fn default_seed_length() -> I {
    15
}

/// TODO: Add separate --dt and --gap-cost flags.
#[derive(Parser, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[clap(help_heading = "HEURISTIC")]
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

pub fn comment(alg: &AlgorithmArgs, h: &HeuristicArgs) -> Option<String> {
    match alg.algorithm {
        Algorithm::NW => {
            assert!(
                !(alg.exp_search && alg.local_doubling),
                "Cannot not do both exponential search and local doubling at the same time."
            );

            if !alg.exp_search && !alg.local_doubling {
                Some("Visit all states ordered by i".into())
            } else if alg.exp_search {
                match h.heuristic {
                    HeuristicType::None => {
                        if !h.gap_cost {
                            Some("Visit Gap(s, u) ≤ Fmax ordered by i".into())
                        } else {
                            Some("Visit Gap(s, u) + Gap(u, t) ≤ Fmax ordered by i".into())
                        }
                    }
                    HeuristicType::Zero => Some("Visit g ≤ Fmax ordered by i".into()),
                    HeuristicType::Gap => Some("Visit g + Gap(u, t) ≤ Fmax ordered by i".into()),
                    HeuristicType::SH => Some("Visit g + SH(u) ≤ Fmax ordered by i".into()),
                    HeuristicType::CSH => Some("Visit g + CSH(u) ≤ Fmax ordered by i".into()),
                }
            } else {
                assert!(alg.local_doubling);
                match h.heuristic {
                    HeuristicType::None => panic!("Local doubling requires a heuristic!"),
                    HeuristicType::Zero => Some("Visit g ≤ Fmax(i) ordered by i".into()),
                    HeuristicType::Gap => Some("Visit g + Gap(u, t) ≤ Fmax(i) ordered by i".into()),
                    HeuristicType::SH => Some("Visit g + SH(u) ≤ Fmax(i) ordered by i".into()),
                    HeuristicType::CSH => Some("Visit g + CSH(u) ≤ Fmax(i) ordered by i".into()),
                }
            }
        }
        Algorithm::DT => {
            if !alg.local_doubling {
                Some("Visit fr. states by g".into())
            } else {
                assert!(alg.local_doubling);
                match h.heuristic {
                    HeuristicType::None => panic!("Local doubling requires a heuristic!"),
                    HeuristicType::Zero => Some("Visit fr. states g ≤ Fmax(i) ordered by g".into()),
                    HeuristicType::Gap => {
                        Some("Visit fr. states g + Gap(u, t) ≤ Fmax(i) ordered by g".into())
                    }
                    HeuristicType::SH => {
                        Some("Visit fr. states g + SH(u) ≤ Fmax(i) ordered by g".into())
                    }
                    HeuristicType::CSH => {
                        Some("Visit fr. states g + CSH(u) ≤ Fmax(i) ordered by g".into())
                    }
                }
            }
        }
        Algorithm::Astar => {
            if !alg.dt {
                match h.heuristic {
                    HeuristicType::None | HeuristicType::Zero => {
                        Some("A* without heuristic is simply Dijkstra".into())
                    }
                    HeuristicType::Gap => Some("Visit states ordered by f = g + Gap(u, t)".into()),
                    HeuristicType::SH => Some("Visit states ordered by f = g + SH(u)".into()),
                    HeuristicType::CSH => Some("Visit states ordered by f = g + CSH(u)".into()),
                }
            } else {
                match h.heuristic {
                    HeuristicType::None | HeuristicType::Zero => {
                        Some("A* without heuristic is simply Dijkstra".into())
                    }
                    HeuristicType::Gap => {
                        Some("Visit fr. states ordered by f = g + Gap(u, t)".into())
                    }
                    HeuristicType::SH => Some("Visit fr. states ordered by f = g + SH(u)".into()),
                    HeuristicType::CSH => Some("Visit fr. states ordered by f = g + CSH(u)".into()),
                }
            }
        }

        _ => None,
    }
}

pub trait HeuristicRunner {
    type R;
    fn call<H: Heuristic>(&self, h: H) -> Self::R;
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

    pub fn run_on_heuristic<F: HeuristicRunner>(&self, f: F) -> F::R {
        match self.heuristic {
            HeuristicType::None => f.call(NoCost),
            HeuristicType::Zero => f.call(ZeroCost),
            HeuristicType::Gap => f.call(GapCost),
            HeuristicType::CSH => {
                let heuristic = CSH {
                    match_config: self.match_config(self.gap_cost),
                    pruning: Pruning {
                        enabled: !self.no_prune,
                        skip_prune: self.skip_prune,
                    },
                    use_gap_cost: self.gap_cost,
                    c: PhantomData::<HintContours<BruteForceContour>>,
                };

                f.call(heuristic)
            }
            HeuristicType::SH => {
                let heuristic = SH {
                    match_config: self.match_config(false),
                    pruning: Pruning {
                        enabled: !self.no_prune,
                        skip_prune: self.skip_prune,
                    },
                };

                f.call(heuristic)
            }
        }
    }
}
