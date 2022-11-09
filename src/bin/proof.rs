#![feature(let_chains)]

use astar_pairwise_aligner::{
    aligners::diagonal_transition::{DiagonalTransition, GapCostHeuristic},
    cli::{
        heuristic_params::{AlgorithmArgs, HeuristicRunner},
        input::Input,
        visualizer::{VisualizerArgs, VisualizerRunner},
    },
    heuristic::path::PathHeuristic,
    prelude::*,
};
use clap::Parser;
use cli::heuristic_params::HeuristicArgs;
use itertools::Itertools;
use std::{ops::ControlFlow, path::PathBuf, time::Duration};

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
    #[clap(long, parse(try_from_str = parse_duration::parse))]
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
            VisRunner { aligner: &self, h },
            Some(&self.args.algorithm),
            Some(&self.args.heuristic),
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

    fn call<V: visualizer::VisualizerT>(&self, v: V) -> Self::R {
        let h = PathHeuristic { h: self.h };
        let start_time = instant::Instant::now();
        let (cost, ref mut hi) = h.build_with_cost(self.aligner.a, self.aligner.b);
        let precomputation = start_time.elapsed().as_secs_f32();

        let start_time = instant::Instant::now();
        let mut dt = DiagonalTransition::new(
            LinearCost::new_unit(),
            GapCostHeuristic::Disable,
            h,
            false,
            v,
        );
        let (cost, _) = dt
            .align_for_bounded_dist_with_h(self.aligner.a, self.aligner.b, Some(cost), hi)
            .unwrap();
        let total = start_time.elapsed().as_secs_f32();

        AlignResult {
            input: InputStats {
                len_a: self.aligner.a.len(),
                len_b: self.aligner.b.len(),
                error_rate: 0.,
            },
            timing: TimingStats {
                total,
                total_sum_squares: total * total,
                precomputation,
                ..Default::default()
            },
            edit_distance: cost,
            sample_size: 1,
            ..Default::default()
        }
    }
}

fn main() {
    let args = Cli::parse();

    // Read the input
    let mut avg_result = AlignResult::default();
    let start = instant::Instant::now();

    args.input.process_input_pairs(|a: Seq, b: Seq| {
        // Run the pair.
        let r = args
            .heuristic
            .run_on_heuristic(AlignWithHeuristic { a, b, args: &args });

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
