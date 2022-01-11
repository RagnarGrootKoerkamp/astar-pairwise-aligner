#![feature(derive_default_enum)]
use itertools::Itertools;
use pairwise_aligner::{align, prelude::*, SequenceStats, Source};
use std::{marker::PhantomData, path::PathBuf};
use structopt::StructOpt;
use strum_macros::EnumString;

#[derive(StructOpt)]
struct Input {
    #[structopt(short, long, parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short, conflicts_with = "input", required_unless = "input")]
    n: Option<usize>,

    #[structopt(short, default_value = "0.2")]
    e: f32,
}

#[derive(StructOpt)]
#[structopt(
    name = "A* Pairwise Aligner",
    about = "Exact pairwise alignment using A*",
    author = "Ragnar Groot Koerkamp, Pesho Ivanov"
)]
struct Cli {
    #[structopt(flatten)]
    input: Input,

    #[structopt(flatten)]
    params: Params,
}

fn main() {
    let args = Cli::from_args();

    // Read the input
    if let Some(input) = &args.input.input {
        let data = std::fs::read(&input).unwrap();
        let pairs = data
            .split(|c| *c == '\n' as u8)
            .tuples()
            .map(|(a, b)| {
                assert!(a[0] == '>' as u8);
                assert!(b[0] == '<' as u8);
                (a[1..].to_vec(), b[1..].to_vec())
            })
            .collect_vec();

        for (a, b) in pairs {
            run(&a, &b, &args.params);
        }
    } else {
        // Generate random input.
        // TODO: Propagate stats.
        let (a, b, _, _) = setup(args.input.n.unwrap(), args.input.e);
        run(&a, &b, &args.params);
    }
}
