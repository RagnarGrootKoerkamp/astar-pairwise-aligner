use crate::prelude::*;
use std::marker::PhantomData;
use structopt::StructOpt;
use strum_macros::EnumString;

#[derive(EnumString, Default)]
#[strum(ascii_case_insensitive)]
pub enum Cost {
    Zero,
    #[default]
    Gap,
    Max,
    Count,
    BiCount,
}

#[derive(EnumString, Default)]
#[strum(ascii_case_insensitive)]
pub enum Contour {
    BruteForce,
    #[default]
    LogQuery,
    Set,
}

#[derive(EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Contours {
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
pub enum Algorithm {
    // The basic n^2 DP
    Naive,
    // Naive, but with SIMD
    Simd,
    // SeedHeuristic, with the provided --cost
    Seed,
    // GapSeedHeuristic, using an efficient implementation from contours
    GapSeed,
}

#[derive(StructOpt)]
pub struct Params {
    #[structopt(short, long, default_value = "GapSeed")]
    algorithm: Algorithm,

    #[structopt(short, default_value = "7")]
    l: usize,

    #[structopt(short, default_value = "1")]
    max_seed_cost: usize,

    #[structopt(long, default_value = "Gap")]
    cost: Cost,

    #[structopt(short = "-C", long, default_value = "Naive")]
    contours: Contours,

    #[structopt(short = "-c", long, default_value = "LogQuery")]
    contour: Contour,

    #[structopt(long)]
    no_prune: bool,

    #[structopt(long, default_value = "0.5")]
    prune_fraction: f32,
}

pub fn run(a: &Sequence, b: &Sequence, params: &Params) {
    //println!("{}\n{}", to_string(&a), to_string(&b));

    match params.algorithm {
        Algorithm::Naive => {
            let dist = bio::alignment::distance::levenshtein(&a, &b);
            println!("SIMD {:>8} {:>8} {:>6}", a.len(), b.len(), dist);
        }
        Algorithm::Simd => {
            let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
            println!("SIMD {:>8} {:>8} {:>6}", a.len(), b.len(), dist);
        }
        Algorithm::Seed => {
            fn run_cost<C: Distance>(a: &Sequence, b: &Sequence, params: &Params)
            where
                for<'a> C::DistanceInstance<'a>: HeuristicInstance<'a, Pos = Pos>,
            {
                let heuristic = SeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(params.l),
                        max_match_cost: params.max_seed_cost,
                        ..Default::default()
                    },
                    distance_function: C::default(),
                    pruning: !params.no_prune,
                    prune_fraction: params.prune_fraction,
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

            match params.cost {
                Cost::Zero => run_cost::<ZeroCost>(a, b, params),
                Cost::Gap => run_cost::<GapCost>(a, b, params),
                Cost::Max => run_cost::<MaxCost>(a, b, params),
                Cost::Count => run_cost::<CountCost>(a, b, params),
                Cost::BiCount => run_cost::<BiCountCost>(a, b, params),
            }
        }
        Algorithm::GapSeed => {
            fn run_contours<C: crate::contour::Contours>(
                a: &Sequence,
                b: &Sequence,
                params: &Params,
            ) {
                let heuristic = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(params.l),
                        max_match_cost: params.max_seed_cost,
                        ..Default::default()
                    },
                    pruning: !params.no_prune,
                    prune_fraction: params.prune_fraction,
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

            match params.contours {
                Contours::BruteForce => run_contours::<BruteForceContours>(a, b, params),
                Contours::Naive => match params.contour {
                    Contour::BruteForce => {
                        run_contours::<NaiveContours<BruteForceContour>>(a, b, params)
                    }
                    Contour::LogQuery => {
                        run_contours::<NaiveContours<LogQueryContour>>(a, b, params)
                    }
                    Contour::Set => run_contours::<NaiveContours<SetContour>>(a, b, params),
                },
            }
        }
    };

    //align(&a, &b, &alphabet, sequence_stats, heuristic).print();
}
