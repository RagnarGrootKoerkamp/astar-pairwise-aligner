#![feature(let_chains)]

use astar_pairwise_aligner::{
    cli::{
        heuristic_params::{Algorithm, AlgorithmParams, HeuristicRunner},
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
    time::{self, Duration},
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
    algorithm: AlgorithmParams,

    /// Parameters and settings for the heuristic.
    #[clap(flatten)]
    params: HeuristicArgs,

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
    #[clap(long, parse(try_from_str = parse_duration::parse))]
    timeout: Option<Duration>,
}

/// Wrapper function to run on each heuristic.
struct AlignWithHeuristic<'a, 'b> {
    a: Seq<'a>,
    b: Seq<'a>,
    params: &'b Cli,
}

impl HeuristicRunner for AlignWithHeuristic<'_, '_> {
    type R = AlignResult;

    fn call<H: Heuristic>(&self, h: H) -> Self::R {
        self.params.visualizer.run_on_visualizer(
            self.a,
            self.b,
            <Cli as clap::CommandFactory>::command().get_matches(),
            VisRunner { aligner: &self, h },
        )
    }
}

/// Wrapper function to run on each visualizer.
struct VisRunner<'a, 'b, 'c, H: Heuristic> {
    aligner: &'c AlignWithHeuristic<'a, 'b>,
    h: H,
}

impl<H: Heuristic> VisualizerRunner for VisRunner<'_, '_, '_, H> {
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
            !self.aligner.params.algorithm.no_greedy_matching,
            self.aligner.params.algorithm.dt,
            &mut v,
        )
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
            let dist = match args.algorithm.algorithm {
                Algorithm::Nw => bio::alignment::distance::levenshtein(a, b),
                Algorithm::NwSimd => bio::alignment::distance::simd::levenshtein(a, b),
                _ => unreachable!(),
            };
            AlignResult {
                sample_size: 1,
                input: InputStats {
                    len_a: a.len(),
                    len_b: b.len(),
                    ..Default::default()
                },
                edit_distance: dist as Cost,
                ..Default::default()
            }
        } else {
            args.params.run_on_heuristic(AlignWithHeuristic {
                a,
                b,
                params: &args,
            })
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
