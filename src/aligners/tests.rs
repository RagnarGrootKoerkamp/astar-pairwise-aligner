use itertools::Itertools;

use super::{
    cigar::test::verify_cigar,
    diagonal_transition::{DiagonalTransition, Direction, GapCostHeuristic, HistoryCompression},
    nw::NW,
    Aligner,
};
use crate::{
    generate::setup_sequences,
    heuristic::ZeroCost,
    prelude::{to_string, AffineCost, AffineLayerCosts, AffineLayerType},
    visualizer::NoVisualizer,
};

fn test_sequences(
) -> itertools::Product<std::slice::Iter<'static, usize>, std::slice::Iter<'static, f32>> {
    let ns = &[
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 30, 40, 50, 100,
        200, /*500, 1000*/
    ];
    let es = &[0.0, 0.01, 0.05, 0.10, 0.20, 0.30, 0.50, 1.0];
    ns.iter().cartesian_product(es)
}

/// Test that:
/// - the aligner gives the same cost as NW, both for `cost` and `align` members.
/// - the `Cigar` is valid and of the correct cost.
fn test_aligner_on_cost_model<const N: usize>(
    cm: AffineCost<N>,
    mut aligner: impl Aligner,
    test_path: bool,
) {
    let mut nw = NW::new(cm.clone(), false);
    for (&n, &e) in test_sequences() {
        let (ref a, ref b) = setup_sequences(n, e);
        //println!("a {} b {}", to_string(a), to_string(b));
        let nw_cost = nw.cost(a, b);

        let cost = aligner.cost(a, b);

        // Test the cost reported by all aligners.
        assert_eq!(
            nw_cost,
            cost,
            "{n} {e}\na == {}\nb == {}\n",
            to_string(&a),
            to_string(&b)
        );

        if test_path {
            let (cost, _path, cigar) = aligner.align(a, b);
            assert_eq!(cost, nw_cost);
            verify_cigar(&cm, a, b, &cigar);
        }
    }
}

mod nw_lib {
    use crate::aligners::nw_lib::NWLib;

    use super::*;

    #[test]
    fn unit_cost_simple() {
        // sub=indel=1
        let cm = AffineCost::new_unit();
        test_aligner_on_cost_model(cm.clone(), NWLib { simd: false }, false);
    }

    #[test]
    fn unit_cost_simd_exponential_search() {
        // sub=indel=1
        let cm = AffineCost::new_unit();
        test_aligner_on_cost_model(cm.clone(), NWLib { simd: true }, false);
    }
}

// TODO: Replace the duplication below by macros.
mod nw {

    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(cm.clone(), NW::new(cm, false), true);
    }

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
}

macro_rules! test_exp_band {
    ($use_gap_cost_heuristic:expr, $name:ident) => {
        paste::paste! {
            mod [<exp_band_ $name>] {
                use super::*;

                fn test<const N: usize>(cm: AffineCost<N>) {
                    test_aligner_on_cost_model(
                        cm.clone(),
                        NW::new(cm.clone(), $use_gap_cost_heuristic),
                        true,
                    );
                }

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
            }
        }
    };
}

test_exp_band!(false, simple);
test_exp_band!(true, gap_heuristic);

macro_rules! test_diagonal_transition {
    ($use_gap_cost_heuristic:expr, $history_compression:expr, $name:ident) => {
        paste::paste! {
            mod [<diagonal_transition_ $name>] {
                use super::*;

                fn test<const N: usize>(cm: AffineCost<N>) {
                    test_aligner_on_cost_model(
                        cm.clone(),
                        DiagonalTransition::new_variant(cm, $use_gap_cost_heuristic, ZeroCost, $history_compression, Direction::Forward, NoVisualizer),
                        true);
                }

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

            }
        }
    };
}

test_diagonal_transition!(
    GapCostHeuristic::Disable,
    HistoryCompression::Disable,
    simple
);
test_diagonal_transition!(
    GapCostHeuristic::Enable,
    HistoryCompression::Disable,
    gap_heuristic
);
test_diagonal_transition!(
    GapCostHeuristic::Disable,
    HistoryCompression::Disable,
    exp_search_simple
);
test_diagonal_transition!(
    GapCostHeuristic::Enable,
    HistoryCompression::Disable,
    exp_search_gap_heuristic
);

mod nw_sh {

    use crate::{heuristic::SH, matches::MatchConfig};

    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            NW {
                cm,
                use_gap_cost_heuristic: false,
                h: SH {
                    match_config: MatchConfig::exact(5),
                    pruning: false,
                },
                v: NoVisualizer,
            },
            // test `align` as well?
            true,
        );
    }

    #[test]
    fn unit_cost() {
        // sub=indel=1
        test(AffineCost::new_unit());
    }
}
mod diagonal_transition_sh {
    use crate::{heuristic::SH, matches::MatchConfig};

    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            DiagonalTransition::new_variant(
                cm,
                GapCostHeuristic::Disable,
                SH {
                    match_config: MatchConfig::exact(5),
                    pruning: false,
                },
                HistoryCompression::Disable,
                Direction::Forward,
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
        aligners::{nw::NW, Aligner},
        prelude::{
            AffineCost, AffineLayerCosts,
            AffineLayerType::{HomoPolymerDelete, HomoPolymerInsert},
        },
    };

    #[test]
    fn test_1() {
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
        let nw = NW { cm: cm.clone() };
        assert_eq!(nw.cost(b"ABC", b"AC"), 4);
    }
}
