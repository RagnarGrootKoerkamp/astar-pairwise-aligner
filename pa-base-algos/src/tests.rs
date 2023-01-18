use crate::{
    dt::{DiagonalTransition, GapCostHeuristic},
    nw::NW,
};
use pa_affine_types::*;
use pa_heuristic::*;
use pa_types::*;
use pa_vis_types::NoVis;

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

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            NW {
                cm,
                use_gap_cost_heuristic: false,
                exponential_search: true,
                local_doubling: false,
                h: SH {
                    match_config: MatchConfig::exact(5),
                    pruning: Pruning::default(),
                },
                v: NoVis,
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
                    pruning: Pruning::default(),
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
