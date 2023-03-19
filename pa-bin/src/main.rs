#![feature(let_chains)]

use astarpa::{make_aligner, make_aligner_with_visualizer, stats::AstarStats, AstarStatsAligner};
use clap::Parser;
use pa_bin::cli::Cli;
use pa_types::*;
use std::{
    io::{BufWriter, Write},
    ops::ControlFlow,
};

pub fn astar_aligner(args: &Cli) -> Box<dyn AstarStatsAligner> {
    #[cfg(not(feature = "vis"))]
    {
        make_aligner(args.diagonal_transition, &args.heuristic)
    }

    #[cfg(feature = "vis")]
    {
        use pa_vis::cli::VisualizerType;
        match args.vis.make_visualizer() {
            VisualizerType::NoVisualizer => {
                make_aligner(args.diagonal_transition, &args.heuristic)
            }
            VisualizerType::Visualizer(vis) => {
                eprintln!("vis!");
                make_aligner_with_visualizer(args.diagonal_transition, &args.heuristic, vis)
            }
        }
    }
}

fn main() {
    let args = Cli::parse();

    let mut avg_stats = AstarStats::default();

    let aligner = astar_aligner(&args);

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
