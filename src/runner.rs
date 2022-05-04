use crate::prelude::*;
use contour::central::CentralContour;
use heuristic::seed_heuristic::SH;
use std::{marker::PhantomData, process::exit};
use structopt::StructOpt;
use strum_macros::EnumString;

#[derive(EnumString, Default, Debug, PartialEq, Eq, strum_macros::Display)]
#[strum(ascii_case_insensitive)]
pub enum CostFunction {
    #[default]
    Zero,
    Gap,
    Max,
    Count,
    BiCount,
}

#[derive(EnumString, Default, Debug, strum_macros::Display)]
#[strum(ascii_case_insensitive)]
pub enum Contour {
    #[default]
    BruteForce,
    Central,
}

#[derive(EnumString, Debug, Default, strum_macros::Display)]
#[strum(ascii_case_insensitive)]
pub enum Contours {
    BruteForce,
    #[default]
    Hint,
}

#[derive(EnumString, Debug, PartialEq, Default, strum_macros::Display)]
#[strum(ascii_case_insensitive)]
#[allow(non_camel_case_types)]
pub enum Algorithm {
    // The basic n^2 DP
    Naive,
    // Naive, but with SIMD
    Simd,
    // Dijkstra
    Dijkstra,
    // Slow CSH implementation with the provided --cost function.
    BruteForceCSH,
    // ChainedSeedsHeuristic with/without gapcost
    CSH,
    CSH_GapCost,
    // SeedHeuristic
    #[default]
    SH,
}

#[derive(StructOpt, Debug)]
pub struct Params {
    #[structopt(short, long, default_value)]
    algorithm: Algorithm,

    #[structopt(short)]
    k: Option<I>,

    #[structopt(long)]
    kmin: Option<I>,

    #[structopt(long)]
    kmax: Option<I>,

    // Either k or e must be specified.
    #[structopt(long)]
    error_rate: Option<f32>,

    #[structopt(long)]
    max_matches: Option<usize>,

    #[structopt(short, default_value = "0")]
    max_seed_cost: MatchCost,

    #[structopt(long, default_value)]
    match_algorithm: MatchAlgorithm,

    #[structopt(long, default_value)]
    cost: CostFunction,

    #[structopt(short = "C", long, default_value)]
    contours: Contours,

    #[structopt(short = "c", long, default_value)]
    contour: Contour,

    #[structopt(long)]
    no_prune: bool,

    #[structopt(long)]
    no_greedy_matching: bool,

    // Do not run anything, but only print the automatically chosen parameters.
    #[structopt(long)]
    only_print_parameters: bool,
}

impl Params {
    // Returns a pair (m,k).
    fn determine_mk(&self, _a: &Sequence, b: &Sequence) -> (MatchCost, I) {
        if let Some(k) = self.k {
            return (self.max_seed_cost, k);
        }
        let e = self
            .error_rate
            .expect("At least one of k and e must be specified!");
        let n = b.len();

        // For SH and CSH, use a fixed mapping:
        if self.algorithm == Algorithm::SH || self.algorithm == Algorithm::CSH {
            return match self.error_rate.unwrap() {
                e if e < 0.025 => (0, 31),
                e if e < 0.06 => (0, 14),
                e if e < 0.14 => (1, 16),
                e if e < 0.25 => (1, 11),
                _ => todo!("Error rate too high!"),
            };
        }

        // True error rate:
        //  1% => 0.90%
        //  5% => 4.4%
        // 10% => 8.5%
        // 20% => 15.7%
        // Approximation:
        // 10% of mutations doesn't do anything, and e/2 of mutations cancels.
        let _e_real = 0.9 * e * (1. - e / 2.);

        // Use inexact matches for error rates more than 15%, and more than 7% when n is large.
        let m = if e > 0.15 || (e > 0.07 && n > 500_000) {
            1
        } else {
            0
        };

        // We need at least log_4(n) for unique matches, and a bit extra when matches are inexact.
        let k_min = (n as f32).log(4f32) + if m == 1 { 1.5 } else { 0. };
        // Maximal k that can handle the given error rate.
        let k_max = (m + 1) as f32 / e;
        // Choose the middle between the two bounds, leaning toward the lower bound.
        // Usually, you only need to exceed the lowerbound a bit for
        // good performance, while you want to be as low (far away from the upperbound) as possible.
        let k = k_min + 1. / 3. * (k_max - k_min);
        //println!("kmin {k_min}, kmax {k_max}, k{k}");
        // k can be at most 31.
        let k = min(k.round() as I, 31);
        if self.only_print_parameters {
            println!("m = {m}  k = {k}");
            exit(0);
        }
        (m, k)
    }
}

pub fn run(a: &Sequence, b: &Sequence, params: &Params) -> AlignResult {
    fn match_config(
        params: &Params,
        a: &Sequence,
        b: &Sequence,
        window_filter: bool,
    ) -> matches::MatchConfig {
        let (m, k) = params.determine_mk(a, b);
        MatchConfig {
            length: if let Some(max) = params.max_matches {
                LengthConfig::Max(MaxMatches {
                    max_matches: max,
                    k_min: params.kmin.unwrap_or(k),
                    k_max: params.kmax.unwrap_or(k),
                })
            } else {
                Fixed(k)
            },
            max_match_cost: m,
            algorithm: params.match_algorithm,
            window_filter,
        }
    }

    match params.algorithm {
        Algorithm::Naive => {
            let dist = bio::alignment::distance::levenshtein(a, b);
            AlignResult {
                sample_size: 1,
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
            AlignResult {
                sample_size: 1,
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

            // Greedy matching is disabled for Dijkstra to have more consistent runtimes.
            align_advanced(
                a,
                b,
                &alphabet,
                sequence_stats,
                heuristic,
                !params.no_greedy_matching,
            )
        }
        Algorithm::BruteForceCSH => {
            fn run_cost<C: Distance>(a: &Sequence, b: &Sequence, params: &Params) -> AlignResult
            where
                for<'a> C::DistanceInstance<'a>: HeuristicInstance<'a>,
            {
                let heuristic = BruteForceCSH {
                    match_config: match_config(params, a, b, false),
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

                align_advanced(
                    a,
                    b,
                    &alphabet,
                    sequence_stats,
                    heuristic,
                    !params.no_greedy_matching,
                )
            }

            match params.cost {
                CostFunction::Zero => run_cost::<ZeroCost>(a, b, params),
                CostFunction::Gap => run_cost::<GapCost>(a, b, params),
                CostFunction::Max => run_cost::<MaxCost>(a, b, params),
                CostFunction::Count => run_cost::<CountCost>(a, b, params),
                CostFunction::BiCount => run_cost::<BiCountCost>(a, b, params),
            }
        }
        Algorithm::CSH | Algorithm::CSH_GapCost => {
            assert!(
                params.cost == CostFunction::Zero,
                "Use --algorithm CSH_gapcost instead."
            );
            fn run_contours<C: 'static + crate::contour::Contours>(
                a: &Sequence,
                b: &Sequence,
                params: &Params,
            ) -> AlignResult {
                assert!(params.cost == CostFunction::Zero || params.cost == CostFunction::Gap);
                let heuristic = CSH {
                    match_config: match_config(params, a, b, params.cost == CostFunction::Gap),
                    pruning: !params.no_prune,
                    use_gap_cost: params.algorithm == Algorithm::CSH_GapCost,
                    c: PhantomData::<C>,
                };

                let alphabet = Alphabet::new(b"ACTG");
                let sequence_stats = SequenceStats {
                    len_a: a.len(),
                    len_b: b.len(),
                    error_rate: 0.,
                    source: Source::Extern,
                };

                align_advanced(
                    a,
                    b,
                    &alphabet,
                    sequence_stats,
                    heuristic,
                    !params.no_greedy_matching,
                )
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
        Algorithm::SH => {
            let heuristic = SH {
                match_config: match_config(params, a, b, false),
                pruning: !params.no_prune,
            };

            let alphabet = Alphabet::new(b"ACTG");
            let sequence_stats = SequenceStats {
                len_a: a.len(),
                len_b: b.len(),
                error_rate: 0.,
                source: Source::Extern,
            };

            align_advanced(
                a,
                b,
                &alphabet,
                sequence_stats,
                heuristic,
                !params.no_greedy_matching,
            )
        }
    }
}
