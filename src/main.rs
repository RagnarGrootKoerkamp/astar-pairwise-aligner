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
            AstarViz { aligner: &self, h },
        )
    }
}

/// Wrapper function to run on each visualizer.
struct AstarViz<'a, 'b, 'c, H: Heuristic> {
    aligner: &'c AlignWithHeuristic<'a, 'b>,
    h: H,
}

impl<H: Heuristic> VisualizerRunner for AstarViz<'_, '_, '_, H> {
    type R = AlignResult;

    fn call<V: visualizer::VisualizerT>(&self, mut v: V) -> Self::R {
        let sequence_stats = InputStats {
            len_a: self.aligner.a.len(),
            len_b: self.aligner.b.len(),
            error_rate: 0.,
        };
        align_advanced(
            self.aligner.a,
            self.aligner.b,
            sequence_stats,
            self.h,
            !self.aligner.args.algorithm.no_greedy_matching,
            self.aligner.args.algorithm.dt,
            &mut v,
        )
    }
}

struct NwViz<'a> {
    a: Seq<'a>,
    b: Seq<'a>,
    exponential_search: bool,
}

impl VisualizerRunner for NwViz<'_> {
    type R = Cost;

    fn call<V: visualizer::VisualizerT>(&self, v: V) -> Self::R {
        aligners::nw::NW {
            cm: LinearCost::new_unit(),
            use_gap_cost_heuristic: false,
            exponential_search: self.exponential_search,
            h: ZeroCost,
            v,
        }
        .align(self.a, self.b)
        .0
    }
}

struct DtViz<'a> {
    a: Seq<'a>,
    b: Seq<'a>,
    dc: bool,
}

impl VisualizerRunner for DtViz<'_> {
    type R = Cost;

    fn call<V: visualizer::VisualizerT>(&self, v: V) -> Self::R {
        aligners::diagonal_transition::DiagonalTransition::new(
            LinearCost::new_unit(),
            GapCostHeuristic::Disable,
            ZeroCost,
            self.dc,
            v,
        )
        .align(self.a, self.b)
        .0
    }
}

fn main() {
    let args = Cli::parse();

    // Read the input
    let mut avg_result = AlignResult::default();
    let start = time::Instant::now();

    args.input.process_input_pairs(|a: Seq, b: Seq| {
        // Run the pair.
        let r = if args.algorithm.algorithm != Algorithm::AStar {
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
                Algorithm::NW => args.visualizer.run_on_visualizer(
                    a,
                    b,
                    <Cli as clap::CommandFactory>::command().get_matches(),
                    NwViz {
                        a,
                        b,
                        exponential_search: args.algorithm.exp_search,
                    },
                ),
                Algorithm::DT => args.visualizer.run_on_visualizer(
                    a,
                    b,
                    <Cli as clap::CommandFactory>::command().get_matches(),
                    DtViz {
                        a,
                        b,
                        dc: args.algorithm.dc,
                    },
                ),
                Algorithm::AStar => unreachable!(),
            };
            let total_duration = start.elapsed().as_secs_f32();
            AlignResult {
                sample_size: 1,
                input: InputStats {
                    len_a: a.len(),
                    len_b: b.len(),
                    ..Default::default()
                },
                edit_distance: cost as Cost,
                timing: TimingStats {
                    total: total_duration,
                    total_sum_squares: total_duration * total_duration,
                    ..Default::default()
                },
                ..Default::default()
            }
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
