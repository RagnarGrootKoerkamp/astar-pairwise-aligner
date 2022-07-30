use crate::prelude::*;
use clap::{Parser, ValueEnum};
use contour::central::CentralContour;
use std::{marker::PhantomData, process::exit};

#[derive(Default, Debug, PartialEq, Eq, ValueEnum, Clone, Copy)]
pub enum CostFunction {
    #[default]
    Zero,
    Gap,
    Max,
    Count,
    BiCount,
}

#[derive(Default, Debug, ValueEnum, Clone, Copy)]
pub enum Contour {
    #[default]
    BruteForce,
    Central,
}

#[derive(Debug, Default, ValueEnum, Clone, Copy)]
pub enum Contours {
    BruteForce,
    #[default]
    Hint,
}

#[derive(Debug, PartialEq, Default, Clone, Copy, ValueEnum)]
#[allow(non_camel_case_types)]
pub enum Algorithm {
    // The basic n^2 DP
    Nw,
    // Naive, but with SIMD
    NwSimd,
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
    // Heuristic variants based with Diagonal Transition
    Dijkstra_DT,
    CSH_DT,
    CSH_GapCost_DT,
    SH_DT,
}

impl Algorithm {
    fn diagonal_transition(self) -> bool {
        match self {
            Algorithm::Dijkstra_DT
            | Algorithm::CSH_DT
            | Algorithm::CSH_GapCost_DT
            | Algorithm::SH_DT => true,
            _ => false,
        }
    }
}

#[derive(Parser, Debug)]
pub struct Params {
    /// nw, nw-simd, dijkstra, sh, csh
    ///
    /// More values:
    /// brute-force-csh, csh-gap-cost
    /// With diagonal transition:
    /// {dijkstra,sh,csh,csh-gap-cost}-dt
    #[clap(
        short,
        long,
        default_value_t,
        value_enum,
        hide_possible_values = true,
        display_order = 10
    )]
    algorithm: Algorithm,

    /// Seed potential
    ///
    /// 1 for exact matches,
    /// 2 for inexact matches.
    #[clap(
        short = 'r',
        default_value_t = 1,
        value_name = "r",
        display_order = 10,
        requires = "k"
    )]
    r: MatchCost,

    /// Seed length
    #[clap(short, value_name = "k", display_order = 10, requires = "r")]
    k: Option<I>,

    /// Error rate of input to infer r and k.
    ///
    /// Copied from GenerateArgs.
    #[clap(skip)]
    pub error_rate: Option<f32>,

    /// Minimal seed length
    #[clap(long, hide_short_help = true)]
    kmin: Option<I>,

    /// Maximal seed length
    #[clap(long, hide_short_help = true)]
    kmax: Option<I>,

    /// The maximal number of matches per seed
    #[clap(long, hide_short_help = true)]
    max_matches: Option<usize>,

    /// Algorithm to use to find all matches
    #[clap(long, value_enum, default_value_t, hide_short_help = true)]
    match_algorithm: MatchAlgorithm,

    /// The cost function to use in BruteForceCsh.
    #[clap(long, default_value_t, value_enum, hide_short_help = true)]
    cost: CostFunction,

    /// The type of contours to use
    #[clap(short = 'C', long, default_value_t, value_enum, hide_short_help = true)]
    contours: Contours,

    /// The type of inner-contour to use
    #[clap(short = 'c', long, default_value_t, value_enum, hide_short_help = true)]
    contour: Contour,

    /// Disable pruning
    #[clap(long, hide_short_help = true)]
    no_prune: bool,

    /// Disable greedy matching
    #[clap(long, hide_short_help = true)]
    no_greedy_matching: bool,

    /// Do not run anything, but print inferred parameters
    #[clap(long)]
    print_parameters: bool,
}

impl Params {
    // Returns a pair (m,k).
    fn determine_mk(&self, _a: Seq, b: Seq) -> (MatchCost, I) {
        if let Some(k) = self.k {
            return (self.r - 1, k);
        }

        // New simpler version.
        if true {
            return match self.error_rate.expect("--error-rate is needed!") {
                e if e < 0.09 => (0, 15),
                e if e <= 1. => (1, 15),
                _ => todo!("Error rate not in [0,1]!"),
            };
        }

        let e = self
            .error_rate
            .expect("At least one of k and e must be specified!");
        let n = b.len();

        // For SH and CSH, use a fixed mapping:
        if self.algorithm == Algorithm::SH || self.algorithm == Algorithm::CSH {
            // V1
            if false {
                return match self.error_rate.unwrap() {
                    e if e < 0.025 => (0, 31),
                    e if e < 0.06 => (0, 14),
                    e if e < 0.14 => (1, 16),
                    e if e < 0.25 => (1, 11),
                    _ => todo!("Error rate too high!"),
                };
            }
            return match self.error_rate.unwrap() {
                e if e < 0.09 => (0, 14),
                e if e <= 1. => (1, 14),
                _ => todo!("Error rate not in [0,1]!"),
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
        if self.print_parameters {
            println!("m = {m}  k = {k}");
            exit(0);
        }
        (m, k)
    }
}

pub fn run(a: Seq, b: Seq, params: &Params) -> AlignResult {
    fn match_config(params: &Params, a: Seq, b: Seq, window_filter: bool) -> matches::MatchConfig {
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
        Algorithm::Nw => {
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
        Algorithm::NwSimd => {
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
        Algorithm::Dijkstra | Algorithm::Dijkstra_DT => {
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
                params.algorithm.diagonal_transition(),
            )
        }
        Algorithm::BruteForceCSH => {
            fn run_cost<C: Distance>(a: Seq, b: Seq, params: &Params) -> AlignResult
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
                    params.algorithm.diagonal_transition(),
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
        Algorithm::CSH | Algorithm::CSH_GapCost | Algorithm::CSH_DT | Algorithm::CSH_GapCost_DT => {
            assert!(
                params.cost == CostFunction::Zero,
                "Use --algorithm CSH_gapcost instead."
            );
            fn run_contours<C: 'static + crate::contour::Contours>(
                a: Seq,
                b: Seq,
                params: &Params,
            ) -> AlignResult {
                assert!(params.cost == CostFunction::Zero || params.cost == CostFunction::Gap);
                let heuristic = CSH {
                    match_config: match_config(params, a, b, params.cost == CostFunction::Gap),
                    pruning: !params.no_prune,
                    use_gap_cost: params.algorithm == Algorithm::CSH_GapCost
                        || params.algorithm == Algorithm::CSH_GapCost_DT,
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
                    params.algorithm.diagonal_transition(),
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
        Algorithm::SH | Algorithm::SH_DT => {
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
                params.algorithm.diagonal_transition(),
            )
        }
    }
}
