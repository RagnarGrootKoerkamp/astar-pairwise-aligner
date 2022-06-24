use itertools::Itertools;

use super::{cigar::test::verify_cigar, nw::NW, Aligner, NoVisualizer};
use crate::{
    generate::setup_sequences,
    prelude::{to_string, AffineCost, AffineLayerCosts, AffineLayerType},
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
    exponential_search: bool,
) {
    let mut nw = NW {
        cm: cm.clone(),
        use_gap_cost_heuristic: false,
        v: &mut NoVisualizer,
    };
    for (&n, &e) in test_sequences() {
        let (ref a, ref b) = setup_sequences(n, e);
        let nw_cost = nw.cost(a, b);

        let cost = if exponential_search {
            aligner.cost_exponential_search(a, b)
        } else {
            aligner.cost(a, b)
        };

        // Test the cost reported by all aligners.
        assert_eq!(
            nw_cost,
            cost,
            "{n} {e}\na == {}\nb == {}\n",
            to_string(&a),
            to_string(&b)
        );

        if test_path {
            let (cost, _path, cigar) = if exponential_search {
                aligner.align_exponential_search(a, b)
            } else {
                aligner.align(a, b)
            };
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
        test_aligner_on_cost_model(cm.clone(), NWLib, false, false);
    }

    #[test]
    fn unit_cost_simd_exponential_search() {
        // sub=indel=1
        let cm = AffineCost::new_unit();
        test_aligner_on_cost_model(cm.clone(), NWLib, false, true);
    }
}

// TODO: Replace the duplication below by macros.
mod nw {

    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            NW {
                cm,
                use_gap_cost_heuristic: false,
                v: &mut NoVisualizer,
            },
            true,
            false,
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

macro_rules! test_exp_band {
    ($use_gap_cost_heuristic:expr, $name:ident) => {
        paste::paste! {
        mod [<exp_band_ $name>] {
            use super::*;

            fn test<const N: usize>(cm: AffineCost<N>) {
                test_aligner_on_cost_model(
                    cm.clone(),
                    NW {
                        cm: cm.clone(),
                        use_gap_cost_heuristic: $use_gap_cost_heuristic,
                        v: &mut NoVisualizer ,
                    },
                    true,
                    /*exponential_search=*/true
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

mod dt {
    use crate::aligners::diagonal_transition::DiagonalTransition;

    use super::*;

    fn test<const N: usize>(cm: AffineCost<N>) {
        test_aligner_on_cost_model(
            cm.clone(),
            DiagonalTransition::new(cm, &mut NoVisualizer),
            true,
            false,
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
