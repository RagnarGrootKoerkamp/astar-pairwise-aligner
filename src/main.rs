#![feature(derive_default_enum)]
use bio::alphabets::dna::alphabet;
use itertools::Itertools;
use pairwise_aligner::{align, prelude::*, SequenceStats, Source};
use std::{marker::PhantomData, path::PathBuf};
use structopt::StructOpt;
use strum_macros::EnumString;

#[derive(EnumString, Default)]
enum Cost {
    Zero,
    #[default]
    Gap,
    Max,
    Count,
    BiCount,
}

#[derive(EnumString, Default)]
enum Contour {
    BruteForce,
    #[default]
    LogQuery,
    Set,
}

#[derive(EnumString)]
enum Contours {
    BruteForce,
    Naive(Contour),
}

impl Default for Contours {
    fn default() -> Self {
        Self::Naive(Contour::default())
    }
}

#[derive(EnumString)]
enum Algorithm {
    // The basic n^2 DP
    Naive,
    // Naive, but with SIMD
    Simd,
    // SeedHeuristic, with the provided --cost
    Seed(Cost),
    // GapSeedHeuristic, using an efficient implementation from contours
    GapSeed(Contours),
}

#[derive(StructOpt)]
#[structopt(
    name = "A* Pairwise Aligner",
    about = "Exact pairwise alignment using A*",
    author = "Ragnar Groot Koerkamp, Pesho Ivanov"
)]
struct Cli {
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    #[structopt(short, default_value = "7")]
    l: usize,

    #[structopt(short, default_value = "GapSeed")]
    h: Algorithm,

    #[structopt(long, default_value = "Gap")]
    cost: Cost,

    #[structopt(short = "-C", long, default_value = "Naive")]
    contours: Contours,

    #[structopt(short = "-c", long, default_value = "LogQuery")]
    contour: Contour,

    #[structopt(short, default_value = "1")]
    max_seed_cost: usize,

    #[structopt(long)]
    no_prune: bool,

    #[structopt(long)]
    no_incremental_pruning: bool,

    #[structopt(long, default_value = "1.0")]
    prune_fraction: f32,
}

fn main() {
    let args = Cli::from_args();

    // Read the input
    let data = std::fs::read(args.input).unwrap();
    let pairs = data
        .split(|c| *c == '\n' as u8)
        .tuples()
        .map(|(a, b)| {
            assert!(a[0] == '>' as u8);
            assert!(b[0] == '<' as u8);
            (a[1..].to_vec(), b[1..].to_vec())
        })
        .collect_vec();

    let heuristic = GapSeedHeuristic {
        match_config: pairwise_aligner::prelude::MatchConfig {
            length: Fixed(args.l),
            max_match_cost: args.max_seed_cost,
            ..Default::default()
        },
        pruning: !args.no_prune,
        prune_fraction: args.prune_fraction,
        incremental_pruning: !args.no_incremental_pruning,
        c: PhantomData::<NaiveContours<BruteForceContour>>,
    };

    for (a, b) in pairs {
        let sequence_stats = SequenceStats {
            len_a: a.len(),
            len_b: b.len(),
            error_rate: 0.,
            source: Source::Extern,
        };

        align(&a, &b, &alphabet(), sequence_stats, heuristic);
    }
}
