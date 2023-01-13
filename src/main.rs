#![feature(let_chains)]

use astar_pairwise_aligner::{cli::Cli, prelude::*};
use clap::Parser;
use itertools::Itertools;
use std::ops::ControlFlow;

fn main() {
    let args = Cli::parse();
    //println!("{}", serde_json::to_string_pretty(&args).unwrap());

    // Read the input
    let mut avg_result = AstarStats::default();
    let start = instant::Instant::now();

    args.input.process_input_pairs(|a: Seq, b: Seq| {
        // Run the pair.
        let r = aligners::astar::AstarPAParams {
            diagonal_transition: args.algorithm.dt,
            heuristic: args.heuristic.clone(),
        }
        .aligner_with_visualizer(&args.visualizer)
        .align_with_stats(a, b)
        .1;

        // Record and print stats.
        if args.silent <= 1 {
            print!("\r");
            if args.silent == 0 {
                r.print();
            }
        }
        avg_result += r;
        if args.silent <= 1 {
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
