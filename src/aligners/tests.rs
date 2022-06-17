use itertools::Itertools;

use crate::{
    generate::setup,
    prelude::{to_string, AffineCost},
};

use super::{diagonal_transition::DiagonalTransition, nw::NW, Aligner};

//test draft
#[test]
fn test1() {
    let cm = AffineCost::new_affine(1, 2, 1);
    let tmp: NW<AffineCost<2>> = NW { cm: cm.clone() };
    let tmp2: DiagonalTransition<AffineCost<2>> = DiagonalTransition::new(cm);
    for len in (0..500).step(5) {
        // Test section
        for k in [0., 0.01, 0.05, 0.10, 0.20, 0.30, 0.50, 1.0] {
            let (ref a, ref b, ref _alphabet, _stats) = setup(len, 0.2 * k as f32);
            assert_eq!(
                tmp.align(a, b).0,
                tmp2.align(a, b).0,
                "{len}\ns1 == {}\ns2 == {}\n",
                to_string(&a),
                to_string(&b)
            );
        }
    }
}
