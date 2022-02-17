use crate::prelude::*;
use contour::central::CentralContour;
use heuristic::unordered::UnorderedHeuristic;
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
    // UnorderedHeuristic
    Unordered,
}

#[derive(StructOpt, Debug)]
pub struct Params {
    #[structopt(short, long, default_value = "GapSeed")]
    algorithm: Algorithm,

    #[structopt(short)]
    k: Option<I>,

    #[structopt(long)]
    kmin: Option<I>,

    #[structopt(long)]
    kmax: Option<I>,

    #[structopt(long)]
    max_matches: Option<usize>,

    #[structopt(short, default_value = "0")]
    max_seed_cost: MatchCost,

    #[structopt(long, default_value = "Hash")]
    match_algorithm: MatchAlgorithm,

    #[structopt(long, default_value = "Gap")]
    cost: CostFunction,

    #[structopt(short = "C", long, default_value = "Hint")]
    contours: Contours,

    #[structopt(short = "c", long, default_value = "BruteForce")]
    contour: Contour,

    #[structopt(long)]
    no_prune: bool,
}

pub fn run(a: &Sequence, b: &Sequence, params: &Params) -> AlignResult {
    fn match_config(params: &Params) -> matches::MatchConfig {
        MatchConfig {
            length: if let Some(max) = params.max_matches {
                LengthConfig::Max(MaxMatches {
                    max_matches: max,
                    k_min: params.kmin.unwrap(),
                    k_max: params.kmax.unwrap(),
                })
            } else {
                Fixed(params.k.unwrap())
            },
            max_match_cost: params.max_seed_cost,
            algorithm: params.match_algorithm,
        }
    }

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
                for<'a> C::DistanceInstance<'a>: HeuristicInstance<'a>,
            {
                let heuristic = SeedHeuristic {
                    match_config: match_config(params),
                    distance_function: C::default(),
                    pruning: !params.no_prune,
                };

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
                    match_config: match_config(params),
                    pruning: !params.no_prune,
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
                Contours::Hint => match params.contour {
                    Contour::BruteForce => {
                        run_contours::<HintContours<BruteForceContour>>(a, b, params)
                    }
                    Contour::Central => run_contours::<HintContours<CentralContour>>(a, b, params),
                },
            }
        }
        Algorithm::Unordered => {
            let heuristic = UnorderedHeuristic {
                match_config: match_config(params),
                pruning: !params.no_prune,
            };

            let alphabet = Alphabet::new(b"ACTG");
            let sequence_stats = SequenceStats {
                len_a: a.len(),
                len_b: b.len(),
                error_rate: 0.,
                source: Source::Extern,
            };

            align(a, b, &alphabet, sequence_stats, heuristic)
        }
    }
}
