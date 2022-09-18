#![feature(let_chains)]

use astar_pairwise_aligner::{
    cli::{
        heuristic_params::{Algorithm, HeuristicRunner},
        input::Input,
    },
    prelude::*,
};
use clap::Parser;
use cli::heuristic_params::HeuristicParams;
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
    #[clap(flatten, help_heading = "PARAMETERS")]
    params: HeuristicParams,

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

struct AlignWithHeuristic<'a, 'b> {
    a: Seq<'a>,
    b: Seq<'a>,
    params: &'b HeuristicParams,
}

impl HeuristicRunner for AlignWithHeuristic<'_, '_> {
    type R = AlignResult;

    fn call<H: Heuristic>(&self, h: H) -> Self::R {
        let sequence_stats = InputStats {
            len_a: self.a.len(),
            len_b: self.b.len(),
            error_rate: 0.,
        };

        // Greedy matching is disabled for Dijkstra to have more consistent runtimes.
        align_advanced(
            self.a,
            self.b,
            sequence_stats,
            h,
            !self.params.no_greedy_matching,
            self.params.dt,
            self.params.save_last.as_ref(),
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
        let r = if !args.params.algorithm.has_heuristic() {
            let dist = match args.params.algorithm {
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
                params: &args.params,
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
