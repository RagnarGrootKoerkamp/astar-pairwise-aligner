#![feature(let_chains)]

use astarpa::{make_aligner, stats::AstarStats};
use clap::Parser;
use pa_bin::cli::Cli;
use pa_types::*;
use std::{
    io::{BufWriter, Write},
    ops::ControlFlow,
};

fn main() {
    let args = Cli::parse();

    let mut avg_stats = AstarStats::default();

    #[cfg(not(features = "vis"))]
    let aligner = make_aligner(args.diagonal_transition, &args.heuristic);

    #[cfg(features = "vis")]
    let aligner = args
        .vis
        .astar_aligner(args.diagonal_transition, &args.heuristic);

    let mut out_file = args
        .output
        .as_ref()
        .map(|o| BufWriter::new(std::fs::File::create(o).unwrap()));

    // Process the input.
    args.process_input_pairs(|a: Seq, b: Seq| {
        // Run the pair.
        let ((cost, cigar), stats) = aligner.align(a, b);

        // Record and print stats.
        if args.silent <= 1 {
            print!("\r");
            if args.silent == 0 {
                stats.print();
            }
        }
        avg_stats += stats;
        if args.silent <= 1 {
            avg_stats.print_no_newline();
        }

        if let Some(f) = &mut out_file {
            writeln!(f, "{cost},{}", cigar.to_string()).unwrap();
        }
        ControlFlow::Continue(())
    });

    if avg_stats.sample_size > 0 {
        print!("\r");
        avg_stats.print();
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn cli_test() {
        <super::Cli as clap::CommandFactory>::command().debug_assert();
    }
}
