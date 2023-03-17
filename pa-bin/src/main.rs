#![feature(let_chains)]

use astarpa::stats::AstarStats;
use clap::Parser;
use pa_types::*;
use pa_vis::cli::VisualizerArgs;
use std::ops::ControlFlow;

#[derive(Parser)]
pub struct Cli {
    #[clap(flatten)]
    args: astarpa::cli::Cli,
    #[clap(flatten)]
    vis: VisualizerArgs,
}

fn main() {
    let Cli { args, vis } = Cli::parse();

    let mut avg_result = AstarStats::default();

    let aligner = vis.astar_aligner(&args);

    // Process the input.
    args.input.process_input_pairs(|a: Seq, b: Seq| {
        // Run the pair.
        let r = aligner.align(a, b).1;

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

        ControlFlow::Continue(())
    });

    if avg_result.sample_size > 0 {
        print!("\r");
        avg_result.print();
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn cli_test() {
        <super::Cli as clap::CommandFactory>::command().debug_assert();
    }
}
