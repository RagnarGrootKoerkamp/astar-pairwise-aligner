//! This file is currently disabled in lib.rs because everything here is broken currently.
use crate::{
    dt::{DiagonalTransition, GapCostHeuristic},
    nw::NW,
};
use pa_affine_types::*;
use pa_heuristic::*;
use pa_types::*;
use pa_vis::NoVis;

fn test_aligner_on_input<const N: usize>(
    a: Seq,
    b: Seq,
    aligner: &mut impl AffineAligner,
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
    // Test the cost reported by all aligners.
    assert_eq!(
        nw_cost,
        cost,
        "\n{params}\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\nNW cigar: {}\nAligner\n{aligner:?}",
        seq_to_string(&a),
        seq_to_string(&b),
        nw.align(a, b).1.unwrap().to_string()
    );
    let (cost, Some(cigar)) = aligner.align_affine(a, b) else {
        panic!()
    };
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

/// Test that:
/// - the aligner gives the same cost as NW, both for `cost` and `align` members.
/// - the `Cigar` is valid and of the correct cost.
fn test_aligner_on_cost_model<const N: usize>(cm: AffineCost<N>, mut aligner: impl AffineAligner) {
    for ((a, b), (n, e, error_model, seed)) in pa_test::gen_seqs() {
        test_aligner_on_input(
            &a,
            &b,
            &mut aligner,
            &cm,
            &format!("seed {seed} n {n} e {e} error_model {error_model:?}"),
        );
    }
}

macro_rules! test_cost_models {
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
        test_aligner_on_cost_model(cm.clone(), NW::new(cm, false, false));
    }

    test_cost_models!();
}

mod nw_band_doubling {
    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(cm.clone(), NW::new(cm, false, true));
    }

    test_cost_models!();
}

mod nw_band_doubling_gapcost {
    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(cm.clone(), NW::new(cm, true, true));
    }

    test_cost_models!();
}

mod nw_band_doubling_sh {

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
        );
    }

    #[ignore = "broken; fix in the future"]
    #[test]
    fn unit_cost() {
        // sub=indel=1
        test(AffineCost::unit());
    }
}

mod diagonal_transition_simple {
    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            DiagonalTransition::new(cm, GapCostHeuristic::Disable, NoCost, false, NoVis),
        );
    }

    test_cost_models!();
}

mod diagonal_transition_gap_heuristic {

    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            DiagonalTransition::new(cm, GapCostHeuristic::Enable, NoCost, false, NoVis),
        );
    }

    test_cost_models!();
}

// FIXME: Enable diagonal transition + divide & conquer tests once they are
// actually passing. For now, affine cost is not working yet.
mod diagonal_transition_dc {
    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            DiagonalTransition::new(cm, GapCostHeuristic::Disable, NoCost, true, NoVis),
        );
    }

    test_cost_models!();
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
        );
    }

    #[test]
    fn unit_cost() {
        // sub=indel=1
        test(AffineCost::unit());
    }
}
