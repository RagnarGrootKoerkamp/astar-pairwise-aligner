use ::triple_accel::levenshtein;
use itertools::Itertools;
use pa_generate::ErrorModel;
use pa_types::{Cost, CostModel, Seq};
use rand::{seq::IteratorRandom, thread_rng, Rng};

use crate::{
    align::AstarPa,
    heuristic::Heuristic,
    prelude::to_string,
    visualizer::{NoVis, Visualizer},
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

fn test_aligner_on_input<H: Heuristic, V: Visualizer>(
    a: Seq,
    b: Seq,
    aligner: AstarPa<V, H>,
    test_path: bool,
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
    let nw_cost = levenshtein(a, b) as Cost;
    let cost = aligner.align(a, b).0 .0;
    // Rerun the alignment with the visualizer enabled.
    // if D && nw_cost != cost && let Some(mut viz_aligner) = viz_aligner {
    //     eprintln!("{params}\na: {}\nb: {}\nnw_cost: {nw_cost}\ntest_cost: {cost}\n", to_string(a), to_string(b));
    //     viz_aligner().align(a, b);
    // }
    // Test the cost reported by all aligners.
    assert_eq!(
        nw_cost,
        cost,
        "\n{params}\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\nAligner\n{aligner:?}",
        to_string(&a),
        to_string(&b),
    );
    if test_path {
        let (cost, cigar) = aligner.align(a, b).0;
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
        cigar.verify(&CostModel::unit(), a, b);
    }
}

mod astar {
    use std::marker::PhantomData;

    use crate::{
        heuristic::{Heuristic, Pruning, CSH, SH},
        matches::MatchConfig,
        prelude::{BruteForceContour, HintContours},
    };

    use super::*;

    fn test_heuristic<H: Heuristic + 'static>(h: H, dt: bool) {
        // Greedy matching doesn't really matter much.
        // To speed up tests, we choose it randomly.
        {
            for (((n, e), error_model), seed) in test_sequences() {
                let (ref a, ref b) = pa_generate::generate_model(n, e, error_model, seed);
                test_aligner_on_input(
                    a,
                    b,
                    AstarPa { dt, h, v: NoVis },
                    true,
                    &format!("seed {seed} n {n} e {e} error_model {error_model:?}"),
                );
            }
        };
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
