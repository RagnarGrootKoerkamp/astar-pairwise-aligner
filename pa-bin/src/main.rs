#![feature(let_chains, trait_upcasting)]

use clap::Parser;
use pa_bin::Cli;
use pa_types::*;
use std::{
    io::{BufWriter, Write},
    ops::ControlFlow,
};

fn main() {
    let args = Cli::parse();

    let mut aligner = args.aligner.build();

    let mut out_file = args
        .output
        .as_ref()
        .map(|o| BufWriter::new(std::fs::File::create(o).unwrap()));

    let mut done = 0;

    eprint!("Done: {done:>3}\r");

    // Process the input.
    args.process_input_pairs(|a: Seq, b: Seq| {
        // Run the pair.
        let (cost, cigar) = aligner.align(a, b);

        done += 1;
        eprint!("Done: {done:>3}\r");

        if let Some(f) = &mut out_file {
            writeln!(f, "{cost},{}", cigar.unwrap().to_string()).unwrap();
        }
        ControlFlow::Continue(())
    });
    eprintln!();
}

#[cfg(test)]
mod test {
    #[test]
    fn cli_test() {
        <super::Cli as clap::CommandFactory>::command().debug_assert();
    }
}
