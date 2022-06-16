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
    for i in 1..20 {
        // Test section
        let (ref a, ref b, ref _alphabet, _stats) = setup(i, 0.3);
        println!("{i}");
        print!("s1 == {}\ns2 == {}\n", to_string(&a), to_string(&b));
        assert_eq!(tmp.align(a, b).0, tmp2.align(a, b).0);
    }
}
