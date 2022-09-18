use std::marker::PhantomData;

use crate::{
    heuristic::{Heuristic, Pruning, ZeroCost, CSH, SH},
    matches::{LengthConfig, MatchConfig},
    prelude::{BruteForceContour, HintContours, MatchCost, I},
};
use clap::{Parser, ValueEnum};

/// TODO: Add other aligners here as well.
#[derive(Debug, PartialEq, Default, Clone, Copy, ValueEnum)]
#[allow(non_camel_case_types)]
pub enum Algorithm {
    // The basic n^2 DP.
    Nw,
    // Naive, but with SIMD and O(ns) exponential search.
    NwSimd,
    Dijkstra,
    #[default]
    SH,
    CSH,
}

impl Algorithm {
    pub fn has_heuristic(self) -> bool {
        use Algorithm::*;
        match self {
            Nw | NwSimd => false,
            Dijkstra | SH | CSH => true,
        }
    }
}

/// TODO: Add separate --dt and --gap-cost flags.
#[derive(Parser, Debug)]
#[clap(help_heading = "PARAMS")]
pub struct HeuristicParams {
    /// nw, nw-simd, dijkstra, sh, csh
    ///
    /// More values:
    /// brute-force-csh, csh-gap-cost
    /// With diagonal transition:
    /// {dijkstra,sh,csh,csh-gap-cost}-dt
    #[clap(
        short,
        long,
        default_value_t,
        value_enum,
        hide_possible_values = true,
        display_order = 10
    )]
    pub algorithm: Algorithm,

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

    /// Disable greedy matching
    #[clap(long, hide_short_help = true)]
    pub no_greedy_matching: bool,

    /// Use diagonal-transition based methods.
    #[clap(long, hide_short_help = true)]
    pub dt: bool,

    /// Use gap-cost for CSH.
    #[clap(long, hide_short_help = true)]
    pub gap_cost: bool,

    /// First find a candidate path and then proof its correctness.
    #[clap(long, hide_short_help = true)]
    pub proof_path: bool,

    /// Save last frame as image.
    // TODO: Move setting elsewhere?
    #[clap(long, hide_short_help = true)]
    pub save_last: Option<String>,
}

pub trait HeuristicRunner {
    type R;
    fn call<H: Heuristic>(&self, h: H) -> Self::R;
}

impl HeuristicParams {
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
        assert!(self.algorithm.has_heuristic());
        match self.algorithm {
            Algorithm::Nw | Algorithm::NwSimd => unreachable!(),
            Algorithm::Dijkstra => f.call(ZeroCost),
            Algorithm::CSH => {
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
            Algorithm::SH => {
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
