fn test_aligner_on_cost_model<const N: usize, A: Aligner>(
    cm: AffineCost<N>,
    aligner: A,
    test_path: bool,
) {
    let a: Option<&mut dyn FnMut(Seq, Seq) -> A> = None;
    test_aligner_on_cost_model_with_viz(cm, aligner, a, test_path);
}

mod triple_accel {
    use crate::{aligners::triple_accel::TripleAccel, cost_model::CostModel::Levenshtein};

    use super::*;

    #[test]
    fn unit_cost() {
        let cm = AffineCost::new_unit();
        test_aligner_on_cost_model(cm.clone(), TripleAccel::new(false, Levenshtein), false);
    }

    #[test]
    fn unit_cost_exp() {
        let cm = AffineCost::new_unit();
        test_aligner_on_cost_model(cm.clone(), TripleAccel::new(false, Levenshtein), false);
    }
}

macro_rules! test_functions_macro {
    () => {
        #[test]
        fn lcs_cost() {
            // sub=infinity, indel=1
            test(AffineCost::new_lcs());
        }

        #[test]
        fn unit_cost() {
            // sub=indel=1
            test(AffineCost::new_unit());
        }

        #[test]
        fn linear_cost() {
            // sub=1, indel=2
            test(AffineCost::new_linear(1, 2));
        }

        #[test]
        fn linear_cost_3() {
            // sub=1, indel=3
            test(AffineCost::new_linear(1, 3));
        }

        #[test]
        fn linear_asymmetric_cost() {
            // sub=1, insert=2, deletion=3
            test(AffineCost::new_linear_asymmetric(1, 2, 3));
        }

        #[test]
        fn affine_cost() {
            // sub=1
            // open=2, extend=1
            test(AffineCost::new_affine(1, 2, 1));
        }

        #[test]
        fn linear_affine_cost() {
            // sub=1, indel=3
            // open=2, extend=1
            test(AffineCost::new_linear_affine(1, 3, 2, 1));
        }

        #[ignore = "broken -- fix in the future"]
        #[test]
        fn double_affine_cost() {
            // sub=1
            // Gap cost is min(4+2*l, 10+1*l).
            test(AffineCost::new_double_affine(1, 4, 2, 10, 1));
        }

        #[test]
        fn asymmetric_affine_cost() {
            // sub=1
            // insert: open=2, extend=2
            // deletion: open=3, extend=1
            test(AffineCost::new_affine_asymmetric(1, 2, 2, 3, 1));
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

        #[ignore = "homopolmer"]
        #[test]
        fn ins_homopolymer_cost() {
            test(AffineCost::new(
                Some(2),
                None,
                Some(3),
                [AffineLayerCosts {
                    affine_type: AffineLayerType::HomoPolymerInsert,
                    open: 2,
                    extend: 2,
                }],
            ));
        }

        #[ignore = "homopolymer"]
        #[test]
        fn del_homopolymer_cost() {
            test(AffineCost::new(
                Some(2),
                Some(3),
                None,
                [AffineLayerCosts {
                    affine_type: AffineLayerType::HomoPolymerDelete,
                    open: 2,
                    extend: 2,
                }],
            ));
        }

        #[ignore = "homopolymer"]
        #[test]
        fn indel_homopolymer_cost() {
            test(AffineCost::new(
                Some(2),
                None,
                None,
                [
                    AffineLayerCosts {
                        affine_type: AffineLayerType::HomoPolymerInsert,
                        open: 3,
                        extend: 1,
                    },
                    AffineLayerCosts {
                        affine_type: AffineLayerType::HomoPolymerDelete,
                        open: 3,
                        extend: 1,
                    },
                ],
            ));
        }

        #[ignore = "homopolymer"]
        #[test]
        fn indel_homopolymer_plus_affine_cost() {
            test(AffineCost::new(
                Some(2),
                None,
                None,
                [
                    AffineLayerCosts {
                        affine_type: AffineLayerType::InsertLayer,
                        open: 2,
                        extend: 2,
                    },
                    AffineLayerCosts {
                        affine_type: AffineLayerType::DeleteLayer,
                        open: 2,
                        extend: 2,
                    },
                    AffineLayerCosts {
                        affine_type: AffineLayerType::HomoPolymerInsert,
                        open: 3,
                        extend: 1,
                    },
                    AffineLayerCosts {
                        affine_type: AffineLayerType::HomoPolymerDelete,
                        open: 3,
                        extend: 1,
                    },
                ],
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
            DiagonalTransition::new(cm, GapCostHeuristic::Disable, NoCost, false, NoVisualizer),
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
            DiagonalTransition::new(cm, GapCostHeuristic::Enable, NoCost, false, NoVisualizer),
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
            DiagonalTransition::new(cm, GapCostHeuristic::Disable, NoCost, true, NoVisualizer),
            true,
        );
    }

    test_functions_macro!();
}

mod nw_sh {

    use crate::{
        heuristic::{Pruning, SH},
        matches::MatchConfig,
    };

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
                v: NoVisualizer,
            },
            // test `align` as well?
            true,
        );
    }

    #[ignore = "broken; fix in the future"]
    #[test]
    fn unit_cost() {
        // sub=indel=1
        test(AffineCost::new_unit());
    }
}

mod diagonal_transition_sh {
    use crate::{
        heuristic::{Pruning, SH},
        matches::MatchConfig,
    };

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
                NoVisualizer,
            ),
            false,
        );
    }

    #[test]
    fn unit_cost() {
        // sub=indel=1
        test(AffineCost::new_unit());
    }
}

mod homopolymer {
    use crate::{
        aligners::{cigar::test::verify_cigar, nw::NW, Aligner},
        cost_model::AffineLayerType::{DeleteLayer, InsertLayer},
        heuristic::NoCost,
        prelude::{
            AffineCost, AffineLayerCosts,
            AffineLayerType::{HomoPolymerDelete, HomoPolymerInsert},
        },
        visualizer::NoVisualizer,
    };

    #[ignore = "homopolymer"]
    #[test]
    fn homo_polymer() {
        let cm = AffineCost::new(
            Some(1),
            Some(10),
            Some(10),
            [
                AffineLayerCosts {
                    affine_type: HomoPolymerInsert,
                    open: 1,
                    extend: 1,
                },
                AffineLayerCosts {
                    affine_type: HomoPolymerDelete,
                    open: 1,
                    extend: 1,
                },
            ],
        );
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: false,
            exponential_search: false,
            local_doubling: false,
            h: NoCost,
            v: NoVisualizer,
        };
        assert_eq!(nw.cost(b"ABC", b"AC"), 2);
        assert_eq!(nw.cost(b"ABC", b""), 6);
        assert_eq!(nw.cost(b"ABBBC", b"AC"), 4);
        assert_eq!(nw.cost(b"ABCABCABC", b"BBBBBBBBB"), 6);
        assert_eq!(nw.cost(b"BBBBBBBBB", b"ABCABCABC"), 6);
        assert_eq!(nw.cost(b"", b"CCCC"), 5);
        assert_eq!(nw.cost(b"", b"ABC"), 6);
        assert_eq!(nw.cost(b"ABBB", b"CBBA"), 2);
        assert_eq!(nw.cost(b"BBBBBBBBB", b"CCCCCCC"), 10);
        assert_eq!(nw.cost(b"AAAAAAAAA", b""), 10);
    }

    #[ignore = "homopolymer"]
    #[test]
    fn homo_polymer_plus_affine_and_cigar() {
        let cm = AffineCost::new(
            Some(1),
            Some(10),
            Some(10),
            [
                AffineLayerCosts {
                    affine_type: InsertLayer,
                    open: 2,
                    extend: 2,
                },
                AffineLayerCosts {
                    affine_type: DeleteLayer,
                    open: 2,
                    extend: 2,
                },
                AffineLayerCosts {
                    affine_type: HomoPolymerInsert,
                    open: 3,
                    extend: 1,
                },
                AffineLayerCosts {
                    affine_type: HomoPolymerDelete,
                    open: 3,
                    extend: 1,
                },
            ],
        );
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: false,
            exponential_search: false,
            local_doubling: false,
            v: NoVisualizer,
            h: NoCost,
        };
        assert_eq!(nw.cost(b"ABC", b"AC"), 4);
        assert_eq!(nw.cost(b"ABC", b""), 8);
        assert_eq!(nw.cost(b"ABBBC", b"AC"), 6);
        assert_eq!(nw.cost(b"ABCABCABC", b"BBBBBBBBB"), 6);
        assert_eq!(nw.cost(b"BBBBBBBBB", b"ABCABCABC"), 6);
        assert_eq!(nw.align(b"", b"CCCC").0, 7);
        assert_eq!(nw.cost(b"", b"ABC"), 8);
        assert_eq!(nw.cost(b"ABBB", b"CBBA"), 2);
        assert_eq!(nw.cost(b"BBBBBBBBB", b"CCCCCCC"), 12);
        assert_eq!(nw.cost(b"AAAAAAAAA", b""), 12);
        let a = b"ABC";
        let b = b"AC";
        let cigar = nw.align(a, b).1;
        verify_cigar(&cm, a, b, &cigar);

        assert_eq!(nw.cost(b"ABC", b""), 8);
        assert_eq!(nw.cost(b"ABBBC", b"AC"), 6);
        assert_eq!(nw.cost(b"ABCABCABC", b"BBBBBBBBB"), 6);
        assert_eq!(nw.cost(b"BBBBBBBBB", b"ABCABCABC"), 6);
        assert_eq!(nw.align(b"", b"CCCC").0, 7);
        assert_eq!(nw.cost(b"", b"ABC"), 8);
        assert_eq!(nw.cost(b"ABBB", b"CBBA"), 2);
        assert_eq!(nw.cost(b"BBBBBBBBB", b"CCCCCCC"), 12);
        assert_eq!(nw.cost(b"AAAAAAAAA", b""), 12);
    }
}

// Interesting csae:
// sub: 1
// indel: 3
// G CA A TCGGG
// A CA   TCGGG
// will be found with cost 5=3+2 before finding the cost 4 path, which requires iterating up to s=4+3.

#[cfg(feature = "edlib")]
mod edlib {
    use crate::{aligners::edlib::Edlib, cost_model::LinearCost};

    use super::*;

    #[test]
    fn unit_cost() {
        // sub=indel=1
        test_aligner_on_cost_model(LinearCost::new_unit(), Edlib, false);
    }
}

#[cfg(feature = "biwfa")]
mod biwfa {
    use crate::aligners::wfa::WFA;

    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(cm.clone(), WFA { cm }, false);
    }

    #[test]
    fn unit_cost() {
        // sub=indel=1
        test(AffineCost::new_unit());
    }

    #[test]
    fn lcs_cost() {
        // sub=infinity, indel=1
        test(AffineCost::new_lcs());
    }

    #[test]
    fn linear_cost() {
        // sub=1, indel=2
        test(AffineCost::new_linear(1, 2));
    }

    #[test]
    fn linear_cost_3() {
        // sub=1, indel=3
        test(AffineCost::new_linear(1, 3));
    }

    #[test]
    fn affine_cost() {
        // sub=1
        // open=2, extend=1
        test(AffineCost::new_affine(1, 2, 1));
    }

    #[test]
    fn double_affine_cost() {
        // sub=1
        // Gap cost is min(4+2*l, 10+1*l).
        test(AffineCost::new_double_affine(1, 4, 2, 10, 1));
    }
}
