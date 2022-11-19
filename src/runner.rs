use std::{path::PathBuf, time::Duration};

use crate::{
    aligners::{diagonal_transition::GapCostHeuristic, Aligner},
    cli::{
        heuristic_params::{Algorithm, AlgorithmArgs, HeuristicArgs, HeuristicRunner},
        input::Input,
        visualizer::{VisualizerArgs, VisualizerRunner},
    },
    prelude::*,
};
use clap::{ArgMatches, Parser};
use serde::{Deserialize, Serialize};

#[derive(Parser, Serialize, Deserialize)]
#[clap(author, about)]
pub struct Cli {
    #[clap(flatten)]
    pub input: Input,

    /// Where to write optional statistics.
    #[clap(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Parameters and settings for the algorithm.
    #[clap(flatten)]
    pub algorithm: AlgorithmArgs,

    /// Parameters and settings for the heuristic.
    #[clap(flatten)]
    pub heuristic: HeuristicArgs,

    /// Parameters and settings for the visualizer.
    #[clap(flatten)]
    pub visualizer: VisualizerArgs,

    /// Print less. Pass twice for summary line only.
    ///
    /// Do not print a new line per alignment, but instead overwrite the previous one.
    /// Pass twice to only print a summary line and avoid all terminal clutter, e.g. for benchmarking.
    #[clap(short, long, parse(from_occurrences))]
    pub silent: u8,

    /// Stop aligning new pairs after this timeout.
    #[clap(long, parse(try_from_str = parse_duration::parse), hide_short_help = true)]
    pub timeout: Option<Duration>,
}

/// Wrapper function to run on each heuristic.
pub struct AlignWithHeuristic<'a, 'b> {
    pub a: Seq<'a>,
    pub b: Seq<'a>,
    pub args: &'b Cli,
}

impl HeuristicRunner for AlignWithHeuristic<'_, '_> {
    type R = AlignResult;

    fn call<H: Heuristic>(&self, h: H) -> Self::R {
        self.args.visualizer.run_on_visualizer(
            self.a,
            self.b,
            if cfg!(feature = "wasm") {
                ArgMatches::default()
            } else {
                <Cli as clap::CommandFactory>::command().get_matches()
            },
            AstarViz {
                a: &self.a,
                b: &self.b,
                h,
                args: &self.args,
            },
            Some(&self.args.algorithm),
            Some(&self.args.heuristic),
        )
    }
}

/// Wrapper function to run on each visualizer.
struct AstarViz<'a, 'd, H: Heuristic> {
    a: Seq<'a>,
    b: Seq<'a>,
    args: &'d Cli,
    h: H,
}

impl<H: Heuristic> VisualizerRunner for AstarViz<'_, '_, H> {
    type R = AlignResult;

    fn call<V: visualizer::VisualizerT>(&self, mut v: V) -> Self::R {
        match self.args.algorithm.algorithm {
            Algorithm::Astar => {
                let sequence_stats = InputStats {
                    len_a: self.a.len(),
                    len_b: self.b.len(),
                    error_rate: 0.,
                };
                align_advanced(
                    self.a,
                    self.b,
                    sequence_stats,
                    self.h,
                    self.args.algorithm.dt,
                    &mut v,
                )
            }
            Algorithm::NW => {
                let start = instant::Instant::now();
                let cost = aligners::nw::NW {
                    cm: LinearCost::new_unit(),
                    use_gap_cost_heuristic: self.args.heuristic.gap_cost,
                    exponential_search: self.args.algorithm.exp_search,
                    local_doubling: self.args.algorithm.local_doubling,
                    h: self.h,
                    v,
                }
                .align(self.a, self.b)
                .0;
                AlignResult::new(self.a, self.b, cost, start.elapsed().as_secs_f32())
            }
            Algorithm::DT => {
                let start = instant::Instant::now();
                let mut dt = aligners::diagonal_transition::DiagonalTransition::new(
                    LinearCost::new_unit(),
                    GapCostHeuristic::Disable,
                    self.h,
                    self.args.algorithm.dc,
                    v,
                );
                dt.local_doubling = self.args.algorithm.local_doubling;
                let cost = dt.align(self.a, self.b).0;
                AlignResult::new(self.a, self.b, cost, start.elapsed().as_secs_f32())
            }
            _ => panic!(),
        }
    }
}
