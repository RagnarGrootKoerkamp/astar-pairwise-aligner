use crate::{
    dt::{DiagonalTransition, GapCostHeuristic},
    nw::NW,
};
use itertools::Itertools;
use pa_affine_types::*;
use pa_generate::ErrorModel;
use pa_heuristic::*;
use pa_types::*;
use pa_vis_types::NoVis;
use rand::{seq::IteratorRandom, thread_rng, Rng};

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

fn test_aligner_on_input<const N: usize, A: AffineAligner>(
    a: Seq,
    b: Seq,
    aligner: &mut impl AffineAligner,
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
        eprintln!("a {}\nb {}", seq_to_string(a), seq_to_string(b));
    }
    let nw = NW::new(cm.clone(), false, false);
    let nw_cost = nw.cost(a, b);
    let cost = aligner.align_affine(a, b).0;
    // Rerun the alignment with the visualizer enabled.
    if D && nw_cost != cost && let Some(viz_aligner) = viz_aligner {
        eprintln!("{params}\na: {}\nb: {}\nnw_cost: {nw_cost}\ntest_cost: {cost}\n", seq_to_string(a), seq_to_string(b));
        viz_aligner(a, b).align_affine(a, b);
    }
    // Test the cost reported by all aligners.
    assert_eq!(
        nw_cost,
        cost,
        "\n{params}\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\nNW cigar: {}\nAligner\n{aligner:?}",
        seq_to_string(&a),
        seq_to_string(&b),
        nw.align(a, b).1.unwrap().to_string()
    );
    if test_path {
        let (cost, Some(cigar)) = aligner.align_affine(a, b) else { panic!() };
        if cost != nw_cost {
            eprintln!("\n================= TEST CIGAR ======================\n");
            eprintln!(
                "{params}\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\ncigar: {}\nnwcig: {}",
                seq_to_string(a),
                seq_to_string(b),
                cigar.to_string(),
                nw.align(a, b).1.unwrap().to_string()
            );
        }
        assert_eq!(cost, nw_cost);
        cigar.verify(cm, a, b);
    }
}

/// Test that:
/// - the aligner gives the same cost as NW, both for `cost` and `align` members.
/// - the `Cigar` is valid and of the correct cost.
fn test_aligner_on_cost_model_with_viz<const N: usize, A: AffineAligner>(
    cm: AffineCost<N>,
    mut aligner: impl AffineAligner,
    mut viz_aligner: Option<&mut dyn FnMut(Seq, Seq) -> A>,
    test_path: bool,
) {
    for (((n, e), error_model), seed) in test_sequences() {
        let (ref a, ref b) = pa_generate::generate_model(n, e, error_model, seed);
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

fn test_aligner_on_cost_model<const N: usize, A: AffineAligner>(
    cm: AffineCost<N>,
    aligner: A,
    test_path: bool,
) {
    let a: Option<&mut dyn FnMut(Seq, Seq) -> A> = None;
    test_aligner_on_cost_model_with_viz(cm, aligner, a, test_path);
}

macro_rules! test_functions_macro {
    () => {
        #[test]
        fn lcs_cost() {
            // sub=infinity, indel=1
            test(AffineCost::lcs());
        }

        #[test]
        fn unit_cost() {
            // sub=indel=1
            test(AffineCost::unit());
        }

        #[test]
        fn linear_cost() {
            // sub=1, indel=2
            test(AffineCost::linear(1, 2));
        }

        #[test]
        fn linear_cost_3() {
            // sub=1, indel=3
            test(AffineCost::linear(1, 3));
        }

        #[test]
        fn linear_asymmetric_cost() {
            // sub=1, insert=2, deletion=3
            test(AffineCost::linear_asymmetric(1, 2, 3));
        }

        #[test]
        fn affine_cost() {
            // sub=1
            // open=2, extend=1
            test(AffineCost::affine(1, 2, 1));
        }

        #[test]
        fn linear_affine_cost() {
            // sub=1, indel=3
            // open=2, extend=1
            test(AffineCost::linear_affine(1, 3, 2, 1));
        }

        #[ignore = "broken -- fix in the future"]
        #[test]
        fn double_affine_cost() {
            // sub=1
            // Gap cost is min(4+2*l, 10+1*l).
            test(AffineCost::double_affine(1, 4, 2, 10, 1));
        }

        #[test]
        fn asymmetric_affine_cost() {
            // sub=1
            // insert: open=2, extend=2
            // deletion: open=3, extend=1
            test(AffineCost::affine_asymmetric(1, 2, 2, 3, 1));
        }

        #[test]
        fn ins_asymmetric_affine_cost() {
            test(AffineCost::new(
                Some(1),
                Some(1),
                None,
                [AffineLayerCosts {
                    affine_type: AffineLayerType::DeleteLayer,
                    open: 2,
                    extend: 2,
                }],
            ));
        }

        #[test]
        fn del_asymmetric_affine_cost() {
            test(AffineCost::new(
                Some(1),
                None,
                Some(1),
                [AffineLayerCosts {
                    affine_type: AffineLayerType::InsertLayer,
                    open: 2,
                    extend: 2,
                }],
            ));
        }
    };
}

// TODO: Replace the duplication below by macros.
mod nw {

    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(cm.clone(), NW::new(cm, false, false), true);
    }

    test_functions_macro!();
}

mod exp_band_simple {
    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(cm.clone(), NW::new(cm.clone(), false, false), true);
    }

    test_functions_macro!();
}

mod exp_band_gap_heuristic {
    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(cm.clone(), NW::new(cm.clone(), true, true), true);
    }

    test_functions_macro!();
}

mod diagonal_transition_simple {
    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            DiagonalTransition::new(cm, GapCostHeuristic::Disable, NoCost, false, NoVis),
            true,
        );
    }

    test_functions_macro!();
}

mod diagonal_transition_gap_heuristic {

    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            DiagonalTransition::new(cm, GapCostHeuristic::Enable, NoCost, false, NoVis),
            true,
        );
    }

    test_functions_macro!();
}

// FIXME: Enable diagonal transition + divide & conquer tests once they are
// actually passing. For now, affine cost is not working yet.
mod diagonal_transition_dc {
    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            DiagonalTransition::new(cm, GapCostHeuristic::Disable, NoCost, true, NoVis),
            true,
        );
    }

    test_functions_macro!();
}

mod nw_sh {

    use super::*;
    use crate::nw::AffineFront;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            NW {
                cm,
                strategy: crate::Strategy::band_doubling(),
                domain: crate::Domain::Astar(SH {
                    match_config: MatchConfig::exact(5),
                    pruning: Pruning::disabled(),
                }),
                block_width: 1,
                v: NoVis,
                front: AffineFront,
                trace: true,
                sparse_h: true,
                prune: true,
            },
            // test `align` as well?
            true,
        );
    }

    #[ignore = "broken; fix in the future"]
    #[test]
    fn unit_cost() {
        // sub=indel=1
        test(AffineCost::unit());
    }
}

mod diagonal_transition_sh {

    use crate::dt::{DiagonalTransition, GapCostHeuristic};

    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            DiagonalTransition::new(
                cm,
                GapCostHeuristic::Disable,
                SH {
                    match_config: MatchConfig::exact(5),
                    pruning: Pruning::disabled(),
                },
                false,
                NoVis,
            ),
            false,
        );
    }

    #[test]
    fn unit_cost() {
        // sub=indel=1
        test(AffineCost::unit());
    }
}
