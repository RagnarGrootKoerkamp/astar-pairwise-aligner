use crate::prelude::*;
use contour::central::CentralContour;
use std::marker::PhantomData;
use structopt::StructOpt;
use strum_macros::EnumString;

#[derive(EnumString, Default, Debug)]
#[strum(ascii_case_insensitive)]
pub enum CostFunction {
    Zero,
    #[default]
    Gap,
    Max,
    Count,
    BiCount,
}

#[derive(EnumString, Default, Debug)]
#[strum(ascii_case_insensitive)]
pub enum Contour {
    #[default]
    BruteForce,
    Central,
}

#[derive(EnumString, Debug, Default)]
#[strum(ascii_case_insensitive)]
pub enum Contours {
    BruteForce,
    Naive,
    #[default]
    Hint,
}

#[derive(EnumString, Debug)]
#[strum(ascii_case_insensitive)]
pub enum Algorithm {
    // The basic n^2 DP
    Naive,
    // Naive, but with SIMD
    Simd,
    // Dijkstra
    Dijkstra,
    // SeedHeuristic, with the provided --cost
    Seed,
    // GapSeedHeuristic, using an efficient implementation from contours
    GapSeed,
}

#[derive(StructOpt, Debug)]
pub struct Params {
    #[structopt(short, long, default_value = "GapSeed")]
    algorithm: Algorithm,

    #[structopt(short)]
    k: I,

    #[structopt(short, default_value = "0")]
    max_seed_cost: Cost,

    #[structopt(long, default_value = "Gap")]
    cost: CostFunction,

    #[structopt(short = "C", long, default_value = "Hint")]
    contours: Contours,

    #[structopt(short = "c", long, default_value = "BruteForce")]
    contour: Contour,

    #[structopt(long)]
    no_prune: bool,

    #[structopt(long, default_value = "1.0")]
    prune_fraction: f32,
}

pub fn run(a: &Sequence, b: &Sequence, params: &Params) -> AlignResult {
    match params.algorithm {
        Algorithm::Naive => {
            let dist = bio::alignment::distance::levenshtein(a, b);
            AlignResult {
                input: SequenceStats {
                    len_a: a.len(),
                    len_b: b.len(),
                    ..Default::default()
                },
                edit_distance: dist as Cost,
                ..Default::default()
            }
        }
        Algorithm::Simd => {
            let dist = bio::alignment::distance::simd::levenshtein(a, b);
            println!("SIMD {:>8} {:>8} {:>6}", a.len(), b.len(), dist);
            AlignResult {
                input: SequenceStats {
                    len_a: a.len(),
                    len_b: b.len(),
                    ..Default::default()
                },
                edit_distance: dist as Cost,
                ..Default::default()
            }
        }
        Algorithm::Dijkstra => {
            let heuristic = ZeroCost;
            //println!("Heuristic:\n{:?}", heuristic);

            let alphabet = Alphabet::new(b"ACTG");
            let sequence_stats = SequenceStats {
                len_a: a.len(),
                len_b: b.len(),
                error_rate: 0.,
                source: Source::Extern,
            };

            align(a, b, &alphabet, sequence_stats, heuristic)
        }
        Algorithm::Seed => {
            fn run_cost<C: Distance>(a: &Sequence, b: &Sequence, params: &Params) -> AlignResult
            where
                for<'a> C::DistanceInstance<'a>: HeuristicInstance<'a, Pos = Pos>,
            {
                let heuristic = SeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(params.k),
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

                align(a, b, &alphabet, sequence_stats, heuristic)
            }

            match params.cost {
                CostFunction::Zero => run_cost::<ZeroCost>(a, b, params),
                CostFunction::Gap => run_cost::<GapCost>(a, b, params),
                CostFunction::Max => run_cost::<MaxCost>(a, b, params),
                CostFunction::Count => run_cost::<CountCost>(a, b, params),
                CostFunction::BiCount => run_cost::<BiCountCost>(a, b, params),
            }
        }
        Algorithm::GapSeed => {
            fn run_contours<C: 'static + crate::contour::Contours>(
                a: &Sequence,
                b: &Sequence,
                params: &Params,
            ) -> AlignResult {
                let heuristic = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(params.k),
                        max_match_cost: params.max_seed_cost,
                        ..Default::default()
                    },
                    pruning: !params.no_prune,
                    prune_fraction: params.prune_fraction,
                    c: PhantomData::<C>,
                };
                //println!("Heuristic:\n{:?}", heuristic);

                let alphabet = Alphabet::new(b"ACTG");
                let sequence_stats = SequenceStats {
                    len_a: a.len(),
                    len_b: b.len(),
                    error_rate: 0.,
                    source: Source::Extern,
                };

                align(a, b, &alphabet, sequence_stats, heuristic)
            }

            match params.contours {
                Contours::BruteForce => run_contours::<BruteForceContours>(a, b, params),
                Contours::Naive => match params.contour {
                    Contour::BruteForce => {
                        run_contours::<NaiveContours<BruteForceContour>>(a, b, params)
                    }
                    Contour::Central => run_contours::<NaiveContours<CentralContour>>(a, b, params),
                },
                Contours::Hint => match params.contour {
                    Contour::BruteForce => {
                        run_contours::<HintContours<BruteForceContour>>(a, b, params)
                    }
                    Contour::Central => run_contours::<HintContours<CentralContour>>(a, b, params),
                },
            }
        }
    }
}
