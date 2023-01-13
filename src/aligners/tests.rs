use ::triple_accel::levenshtein;
use itertools::Itertools;
use rand::{seq::IteratorRandom, thread_rng, Rng};

use super::{cigar::test::verify_cigar, Aligner, Seq};
use crate::{
    generate::{generate_model, ErrorModel},
    prelude::{to_string, AffineCost},
    visualizer::NoVisualizer,
};

fn test_sequences() -> impl Iterator<Item = (((usize, f32), ErrorModel), u64)> {
    let rng = &mut thread_rng();
    // Each run picks a random sample of the numbers below to speed things up.
    let ns = [
        0usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 30, 40, 50,
        60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180, 190, 200, 210, 220, 230, 240,
        250, 260, 270, 280, 290, 300, 500,
    ];
    let ns = ns.into_iter().choose_multiple(rng, ns.len() / 2);
    let es = [
        0.0f32, 0.01, 0.02, 0.03, 0.05, 0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 1.0,
    ];
    let es = es.into_iter().choose_multiple(rng, es.len() / 2);
    let models = [
        ErrorModel::Uniform,
        ErrorModel::NoisyInsert,
        ErrorModel::NoisyDelete,
        ErrorModel::SymmetricRepeat,
    ];
    // Run each test on a new random seed for increased coverage over time.
    let seeds = [rng.gen_range(0..u64::MAX)];
    ns.into_iter()
        .cartesian_product(es)
        .cartesian_product(models)
        .cartesian_product(seeds)
}

fn test_aligner_on_input<const N: usize, A: Aligner>(
    a: Seq,
    b: Seq,
    aligner: &mut impl Aligner,
    viz_aligner: &mut Option<&mut dyn FnMut(&[u8], &[u8]) -> A>,
    test_path: bool,
    cm: &AffineCost<N>,
    params: &str,
) {
    // Set to true for local debugging.
    const D: bool = false;

    // useful in case of panics inside the alignment code.
    eprintln!("{params}");
    if D {
        eprintln!("a {}\nb {}", to_string(a), to_string(b));
    }
    //let mut nw = NW::new(cm.clone(), false, false);
    let nw_cost = levenshtein(a, b) as _;
    let cost = aligner.cost(a, b);
    // Rerun the alignment with the visualizer enabled.
    if D && nw_cost != cost && let Some(viz_aligner) = viz_aligner {
        eprintln!("{params}\na: {}\nb: {}\nnw_cost: {nw_cost}\ntest_cost: {cost}\n", to_string(a), to_string(b));
        viz_aligner(a, b).align(a, b);
    }
    // Test the cost reported by all aligners.
    assert_eq!(
        nw_cost,
        cost,
        "\n{params}\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\nAligner\n{aligner:?}",
        to_string(&a),
        to_string(&b),
    );
    if test_path {
        let (cost, cigar) = aligner.align(a, b);
        if cost != nw_cost {
            eprintln!("\n================= TEST CIGAR ======================\n");
            eprintln!(
                "{params}\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\ncigar: {}",
                to_string(a),
                to_string(b),
                cigar.to_string(),
            );
        }
        assert_eq!(cost, nw_cost);
        verify_cigar(cm, a, b, &cigar);
    }
}

/// Test that:
/// - the aligner gives the same cost as NW, both for `cost` and `align` members.
/// - the `Cigar` is valid and of the correct cost.
fn test_aligner_on_cost_model_with_viz<const N: usize, A: Aligner>(
    cm: AffineCost<N>,
    mut aligner: impl Aligner,
    mut viz_aligner: Option<&mut dyn FnMut(Seq, Seq) -> A>,
    test_path: bool,
) {
    for (((n, e), error_model), seed) in test_sequences() {
        let (ref a, ref b) = generate_model(n, e, error_model, seed);
        test_aligner_on_input(
            a,
            b,
            &mut aligner,
            &mut viz_aligner,
            test_path,
            &cm,
            &format!("seed {seed} n {n} e {e} error_model {error_model:?}"),
        );
    }
}

mod astar {
    use std::marker::PhantomData;

    use crate::{
        aligners::astar::AstarPA,
        cost_model::LinearCost,
        heuristic::{Heuristic, Pruning, CSH, SH},
        matches::MatchConfig,
        prelude::{BruteForceContour, HintContours},
    };

    use super::*;

    fn test_heuristic<H: Heuristic>(h: H, dt: bool) {
        // Greedy matching doesn't really matter much.
        // To speed up tests, we choose it randomly.
        test_aligner_on_cost_model_with_viz(
            LinearCost::new_unit(),
            AstarPA {
                dt,
                h,
                v: NoVisualizer,
            },
            Some(&mut |_a, _b| AstarPA {
                dt,
                h,
                v: {
                    #[cfg(feature = "vis")]
                    {
                        use crate::visualizer::{Config, VisualizerStyle};
                        Config::new(VisualizerStyle::Test)
                    }
                    #[cfg(not(feature = "vis"))]
                    {
                        NoVisualizer
                    }
                },
            }),
            true,
        );
    }

    macro_rules! make_test {
        // h is a function (exact: bool, pruning: bool) -> Heuristic.
        ($name:ident, $h:expr) => {
            mod $name {
                use super::*;
                // large k variants with mostly linear matches
                #[test]
                fn exact_noprune() {
                    super::test_heuristic($h(true, false, false), false);
                }
                #[test]
                fn exact_prune() {
                    super::test_heuristic($h(true, true, false), false);
                }
                #[test]
                fn inexact_noprune() {
                    super::test_heuristic($h(false, false, false), false);
                }
                #[test]
                fn inexact_prune() {
                    super::test_heuristic($h(false, true, false), false);
                }
                #[test]
                fn exact_noprune_dt() {
                    super::test_heuristic($h(true, false, false), true);
                }
                #[test]
                fn exact_prune_dt() {
                    super::test_heuristic($h(true, true, false), true);
                }
                #[test]
                fn inexact_noprune_dt() {
                    super::test_heuristic($h(false, false, false), true);
                }
                #[test]
                fn inexact_prune_dt() {
                    super::test_heuristic($h(false, true, false), true);
                }

                // small k variants with many matches, to stress the contours
                #[test]
                fn exact_noprune_smallk() {
                    super::test_heuristic($h(true, false, true), false);
                }
                #[test]
                fn exact_prune_smallk() {
                    super::test_heuristic($h(true, true, true), false);
                }
                #[test]
                fn inexact_noprune_smallk() {
                    super::test_heuristic($h(false, false, true), false);
                }
                #[test]
                fn inexact_prune_smallk() {
                    super::test_heuristic($h(false, true, true), false);
                }
                #[test]
                fn exact_noprune_dt_smallk() {
                    super::test_heuristic($h(true, false, true), true);
                }
                #[test]
                fn exact_prune_dt_smallk() {
                    super::test_heuristic($h(true, true, true), true);
                }
                #[test]
                fn inexact_noprune_dt_smallk() {
                    super::test_heuristic($h(false, false, true), true);
                }
                #[test]
                fn inexact_prune_dt_smallk() {
                    super::test_heuristic($h(false, true, true), true);
                }
            }
        };
    }

    mod dijkstra {
        use crate::heuristic::NoCost;

        #[test]
        fn exact_noprune() {
            super::test_heuristic(NoCost, false);
        }
        #[test]
        fn exact_noprune_dt() {
            super::test_heuristic(NoCost, true);
        }
    }

    fn match_config(exact: bool, small_k: bool) -> MatchConfig {
        match (exact, small_k) {
            (true, false) => MatchConfig::exact(5),
            (true, true) => MatchConfig::exact(2),
            (false, false) => MatchConfig::inexact(9),
            (false, true) => MatchConfig::inexact(3),
        }
    }

    // normal k with few matches
    make_test!(sh, |exact, prune, small_k| SH {
        match_config: match_config(exact, small_k),
        pruning: Pruning::new(prune)
    });
    make_test!(csh, |exact, prune, small_k| CSH {
        match_config: match_config(exact, small_k),
        pruning: Pruning::new(prune),
        use_gap_cost: false,
        c: PhantomData::<HintContours<BruteForceContour>>,
    });
    make_test!(gch, |exact, prune, small_k| CSH {
        match_config: match_config(exact, small_k),
        pruning: Pruning::new(prune),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    });
}
