#![feature(let_chains)]

use astar_pairwise_aligner::{
    aligners::{diagonal_transition::GapCostHeuristic, nw_lib::NWLib, Aligner},
    cli::{
        heuristic_params::{Algorithm, AlgorithmArgs, HeuristicRunner},
        input::Input,
        visualizer::{VisualizerArgs, VisualizerRunner},
    },
    prelude::*,
};
use clap::Parser;
use cli::heuristic_params::HeuristicArgs;
use itertools::Itertools;
use std::{
    ops::ControlFlow,
    path::PathBuf,
    time::{self, Duration, Instant},
};

#[derive(Parser)]
#[clap(author, about)]
struct Cli {
    #[clap(flatten)]
    input: Input,

    /// Where to write optional statistics.
    #[clap(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Parameters and settings for the algorithm.
    #[clap(flatten)]
    algorithm: AlgorithmArgs,

    /// Parameters and settings for the heuristic.
    #[clap(flatten)]
    heuristic: HeuristicArgs,

    /// Parameters and settings for the visualizer.
    #[clap(flatten)]
    visualizer: VisualizerArgs,

    /// Print less. Pass twice for summary line only.
    ///
    /// Do not print a new line per alignment, but instead overwrite the previous one.
    /// Pass twice to only print a summary line and avoid all terminal clutter, e.g. for benchmarking.
    #[clap(short, long, parse(from_occurrences))]
    silent: u8,

    /// Stop aligning new pairs after this timeout.
    #[clap(long, parse(try_from_str = parse_duration::parse), hide_short_help = true)]
    timeout: Option<Duration>,
}

/// Wrapper function to run on each heuristic.
struct AlignWithHeuristic<'a, 'b> {
    a: Seq<'a>,
    b: Seq<'a>,
    args: &'b Cli,
}

impl HeuristicRunner for AlignWithHeuristic<'_, '_> {
    type R = AlignResult;

    fn call<H: Heuristic>(&self, h: H) -> Self::R {
        self.args.visualizer.run_on_visualizer(
            self.a,
            self.b,
            <Cli as clap::CommandFactory>::command().get_matches(),
            AstarViz {
                a: &self.a,
                b: &self.b,
                h,
                args: &self.args,
            },
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
            Algorithm::AStar => {
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
                    !self.args.algorithm.no_greedy_matching,
                    self.args.algorithm.dt,
                    &mut v,
                )
            }
            Algorithm::NW => {
                let start = Instant::now();
                let cost = aligners::nw::NW {
                    cm: LinearCost::new_unit(),
                    use_gap_cost_heuristic: false,
                    exponential_search: self.args.algorithm.exp_search,
                    local_doubling: self.args.algorithm.local_doubling,
                    h: NoCost,
                    v,
                }
                .align(self.a, self.b)
                .0;
                AlignResult::new(self.a, self.b, cost, start.elapsed().as_secs_f32())
            }
            Algorithm::DT => {
                let start = Instant::now();
                let mut dt = aligners::diagonal_transition::DiagonalTransition::new(
                    LinearCost::new_unit(),
                    GapCostHeuristic::Disable,
                    NoCost,
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

fn main() {
    let args = Cli::parse();

    // Read the input
    let mut avg_result = AlignResult::default();
    let start = time::Instant::now();

    args.input.process_input_pairs(|a: Seq, b: Seq| {
        // Run the pair.
        let r = if args.algorithm.algorithm.external() {
            let start = Instant::now();
            let cost = match args.algorithm.algorithm {
                Algorithm::NwLib => NWLib { simd: false }.cost(a, b),
                Algorithm::NwLibSimd => NWLib { simd: true }.cost(a, b),
                Algorithm::Edlib => {
                    #[cfg(not(feature = "edlib"))]
                    panic!("Enable the edlib feature flag to use edlib.");
                    #[cfg(feature = "edlib")]
                    aligners::edlib::Edlib.cost(a, b)
                }
                Algorithm::Wfa => {
                    #[cfg(not(feature = "wfa"))]
                    panic!("Enable the wfa feature flag to use WFA.");
                    #[cfg(feature = "wfa")]
                    aligners::wfa::WFA {
                        cm: LinearCost::new_unit(),
                        biwfa: false,
                    }
                    .cost(a, b)
                }
                Algorithm::Biwfa => {
                    #[cfg(not(feature = "wfa"))]
                    panic!("Enable the wfa feature flag to use BiWFA.");
                    #[cfg(feature = "wfa")]
                    aligners::wfa::WFA {
                        cm: LinearCost::new_unit(),
                        biwfa: true,
                    }
                    .cost(a, b)
                }
                _ => unreachable!(),
            };
            let total_duration = start.elapsed().as_secs_f32();
            AlignResult::new(a, b, cost, total_duration)
        } else {
            args.heuristic
                .run_on_heuristic(AlignWithHeuristic { a, b, args: &args })
        };

        // Record and print stats.
        avg_result.add_sample(&r);
        if args.silent <= 1 {
            print!("\r");
            if args.silent == 0 {
                r.print();
            }
            avg_result.print_no_newline();
        }

        if let Some(d) = args.timeout && start.elapsed() > d {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });

    if avg_result.sample_size > 0 {
        print!("\r");
        avg_result.print();

        if let Some(output) = args.output {
            let (header, vals) = avg_result.values();

            std::fs::write(
                output,
                format!(
                    "{}\n{}\n",
                    header.iter().map(|x| x.trim()).join("\t"),
                    vals.iter().map(|x| x.trim()).join("\t")
                ),
            )
            .unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::Cli;

    #[test]
    fn cli_test() {
        <Cli as clap::CommandFactory>::command().debug_assert();
    }
}
