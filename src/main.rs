#![feature(derive_default_enum)]
use itertools::Itertools;
use pairwise_aligner::{align, prelude::*, SequenceStats, Source};
use std::{marker::PhantomData, path::PathBuf};
use structopt::StructOpt;
use strum_macros::EnumString;

#[derive(EnumString, Default)]
#[strum(ascii_case_insensitive)]
enum Cost {
    Zero,
    #[default]
    Gap,
    Max,
    Count,
    BiCount,
}

#[derive(EnumString, Default)]
#[strum(ascii_case_insensitive)]
enum Contour {
    BruteForce,
    #[default]
    LogQuery,
    Set,
}

#[derive(EnumString)]
#[strum(ascii_case_insensitive)]
enum Contours {
    BruteForce,
    Naive,
}

impl Default for Contours {
    fn default() -> Self {
        Self::Naive
    }
}

#[derive(EnumString)]
#[strum(ascii_case_insensitive)]
enum Algorithm {
    // The basic n^2 DP
    Naive,
    // Naive, but with SIMD
    Simd,
    // SeedHeuristic, with the provided --cost
    Seed,
    // GapSeedHeuristic, using an efficient implementation from contours
    GapSeed,
}

fn run(a: &Sequence, b: &Sequence, args: &Cli) {
    //println!("{}\n{}", to_string(&a), to_string(&b));

    match args.algorithm {
        Algorithm::Naive => {
            let dist = bio::alignment::distance::levenshtein(&a, &b);
            println!("SIMD {:>8} {:>8} {:>6}", a.len(), b.len(), dist);
        }
        Algorithm::Simd => {
            let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
            println!("SIMD {:>8} {:>8} {:>6}", a.len(), b.len(), dist);
        }
        Algorithm::Seed => {
            fn run_cost<C: Distance>(a: &Sequence, b: &Sequence, args: &Cli)
            where
                for<'a> C::DistanceInstance<'a>: HeuristicInstance<'a, Pos = Pos>,
            {
                let heuristic = SeedHeuristic {
                    match_config: pairwise_aligner::prelude::MatchConfig {
                        length: Fixed(args.l),
                        max_match_cost: args.max_seed_cost,
                        ..Default::default()
                    },
                    distance_function: C::default(),
                    pruning: !args.no_prune,
                    prune_fraction: args.prune_fraction,
                };
                println!("Heuristic:\n{:?}", heuristic);

                let alphabet = Alphabet::new(b"ACTG");
                let sequence_stats = SequenceStats {
                    len_a: a.len(),
                    len_b: b.len(),
                    error_rate: 0.,
                    source: Source::Extern,
                };

                align(&a, &b, &alphabet, sequence_stats, heuristic).print();
            }

            match args.cost {
                Cost::Zero => run_cost::<ZeroCost>(a, b, args),
                Cost::Gap => run_cost::<GapCost>(a, b, args),
                Cost::Max => run_cost::<MaxCost>(a, b, args),
                Cost::Count => run_cost::<CountCost>(a, b, args),
                Cost::BiCount => run_cost::<BiCountCost>(a, b, args),
            }
        }
        Algorithm::GapSeed => {
            fn run_contours<C: pairwise_aligner::prelude::Contours>(
                a: &Sequence,
                b: &Sequence,
                args: &Cli,
            ) {
                let heuristic = GapSeedHeuristic {
                    match_config: pairwise_aligner::prelude::MatchConfig {
                        length: Fixed(args.l),
                        max_match_cost: args.max_seed_cost,
                        ..Default::default()
                    },
                    pruning: !args.no_prune,
                    prune_fraction: args.prune_fraction,
                    c: PhantomData::<NaiveContours<BruteForceContour>>,
                };
                println!("Heuristic:\n{:?}", heuristic);

                let alphabet = Alphabet::new(b"ACTG");
                let sequence_stats = SequenceStats {
                    len_a: a.len(),
                    len_b: b.len(),
                    error_rate: 0.,
                    source: Source::Extern,
                };

                align(&a, &b, &alphabet, sequence_stats, heuristic).print();
            }

            match args.contours {
                Contours::BruteForce => run_contours::<BruteForceContours>(a, b, args),
                Contours::Naive => match args.contour {
                    Contour::BruteForce => {
                        run_contours::<NaiveContours<BruteForceContour>>(a, b, args)
                    }
                    Contour::LogQuery => run_contours::<NaiveContours<LogQueryContour>>(a, b, args),
                    Contour::Set => run_contours::<NaiveContours<SetContour>>(a, b, args),
                },
            }
        }
    };

    //align(&a, &b, &alphabet, sequence_stats, heuristic).print();
}

#[derive(StructOpt)]
#[structopt(
    name = "A* Pairwise Aligner",
    about = "Exact pairwise alignment using A*",
    author = "Ragnar Groot Koerkamp, Pesho Ivanov"
)]
struct Cli {
    #[structopt(short, long, parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short, conflicts_with = "input", required_unless = "input")]
    n: Option<usize>,

    #[structopt(short, default_value = "0.2")]
    e: f32,

    #[structopt(short, default_value = "7")]
    l: usize,

    #[structopt(short, long, default_value = "GapSeed")]
    algorithm: Algorithm,

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

    #[structopt(long, default_value = "0.5")]
    prune_fraction: f32,
}

fn main() {
    let args = Cli::from_args();

    // Read the input
    if let Some(input) = &args.input {
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
            run(&a, &b, &args);
        }
    } else {
        // Generate random input.
        // TODO: Propagate stats.
        let (a, b, _, _) = setup(args.n.unwrap(), args.e);
        run(&a, &b, &args);
    }
}
