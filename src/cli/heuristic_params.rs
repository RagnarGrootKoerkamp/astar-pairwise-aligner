use std::marker::PhantomData;

use crate::{
    heuristic::{Heuristic, NoCost, Pruning, ZeroCost, CSH, SH},
    matches::{LengthConfig, MatchConfig},
    prelude::{BruteForceContour, HintContours, MatchCost, I},
};
use clap::{Parser, ValueEnum};

/// TODO: Add other aligners here as well.
#[derive(Debug, PartialEq, Default, Clone, Copy, ValueEnum)]
pub enum Algorithm {
    // Internal methods
    NW,
    DT,
    // See HeuristicArgs for configuration.
    #[default]
    Astar,

    // External methods

    // The basic n^2 DP.
    NwLib,
    // SIMD and O(ns) band-doubling.
    NwLibSimd,
    Edlib,
    Wfa,
    Biwfa,
}

impl Algorithm {
    pub fn external(&self) -> bool {
        match self {
            Algorithm::NwLib
            | Algorithm::NwLibSimd
            | Algorithm::Edlib
            | Algorithm::Wfa
            | Algorithm::Biwfa => true,
            Algorithm::NW | Algorithm::DT | Algorithm::Astar => false,
        }
    }
}

#[derive(Debug, PartialEq, Default, Clone, Copy, ValueEnum)]
pub enum HeuristicType {
    None,
    Zero,
    #[default]
    SH,
    CSH,
}

#[derive(Parser, Debug)]
#[clap(help_heading = "ALGORITHM")]
pub struct AlgorithmArgs {
    #[clap(short, long, default_value_t, value_enum, display_order = 10)]
    pub algorithm: Algorithm,

    /// Disable greedy matching
    #[clap(long, hide_short_help = true)]
    pub no_greedy_matching: bool,

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

/// TODO: Add separate --dt and --gap-cost flags.
#[derive(Parser, Debug)]
#[clap(help_heading = "HEURISTIC")]
pub struct HeuristicArgs {
    #[clap(short = 'H', long, default_value_t, value_enum, display_order = 10)]
    pub heuristic: HeuristicType,

    /// Seed potential
    ///
    /// 2 for inexact matches.
    #[clap(short = 'r', default_value_t = 2, value_name = "r", display_order = 10)]
    pub r: MatchCost,

    /// Seed length
    #[clap(short, value_name = "k", display_order = 10, default_value_t = 15)]
    pub k: I,

    /// Minimal seed length
    #[clap(long, hide_short_help = true)]
    pub kmin: Option<I>,

    /// Maximal seed length
    #[clap(long, hide_short_help = true)]
    pub kmax: Option<I>,

    /// The maximal number of matches per seed
    #[clap(long, hide_short_help = true)]
    pub max_matches: Option<usize>,

    /// Disable pruning
    #[clap(long, hide_short_help = true)]
    pub no_prune: bool,

    /// Skip pruning every Nth match. 0 to disable
    ///
    /// This is only used for CSH, not for SH.
    #[clap(long, hide_short_help = true, default_value_t = 0)]
    pub skip_prune: usize,

    /// Use gap-cost for CSH.
    #[clap(long, hide_short_help = true)]
    pub gap_cost: bool,
}

pub trait HeuristicRunner {
    type R;
    fn call<H: Heuristic>(&self, h: H) -> Self::R;
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

    pub fn run_on_heuristic<F: HeuristicRunner>(&self, f: F) -> F::R {
        match self.heuristic {
            HeuristicType::None => f.call(NoCost),
            HeuristicType::Zero => f.call(ZeroCost),
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
