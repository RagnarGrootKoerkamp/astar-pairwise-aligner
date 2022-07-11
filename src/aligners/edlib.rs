use std::intrinsics::transmute;

use crate::{
    aligners::edlib::edlib::{
        edlibAlign, edlibDefaultAlignConfig, edlibFreeAlignResult, EDLIB_STATUS_OK,
    },
    cost_model::{Cost, LinearCost},
};

use super::{cigar::Cigar, diagonal_transition::Direction, Aligner, Path, Seq};

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused)]

mod edlib {
    include!(concat!(env!("OUT_DIR"), "/bindings_edlib.rs"));
}

pub struct Edlib;

impl std::fmt::Debug for Edlib {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("edlib").finish()
    }
}

fn unit_cost(a: Seq, b: Seq) -> Cost {
    unsafe {
        let a: &[i8] = transmute(a);
        let b: &[i8] = transmute(b);
        let result = edlibAlign(
            a.as_ptr(),
            a.len() as i32,
            b.as_ptr(),
            b.len() as i32,
            edlibDefaultAlignConfig(),
        );
        let distance = result.editDistance as Cost;
        assert!(result.status == EDLIB_STATUS_OK as i32);
        edlibFreeAlignResult(result);
        distance
    }
}

impl Aligner for Edlib {
    type CostModel = LinearCost;

    fn cost_model(&self) -> &Self::CostModel {
        unimplemented!()
    }

    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        unit_cost(a, b)
    }

    fn align(&mut self, _a: Seq, _b: Seq) -> (Cost, Path, Cigar) {
        unimplemented!()
    }

    fn cost_for_bounded_dist(&mut self, _a: Seq, _b: Seq, _s_bound: Option<Cost>) -> Option<Cost> {
        unimplemented!();
    }

    fn align_for_bounded_dist(
        &mut self,
        _a: Seq,
        _b: Seq,
        _s_bound: Option<Cost>,
    ) -> Option<(Cost, Path, Cigar)> {
        unimplemented!();
    }

    type Fronts = ();

    type State = ();

    fn parent(
        &self,
        _a: Seq,
        _b: Seq,
        _fronts: &Self::Fronts,
        _st: Self::State,
        _direction: Direction,
    ) -> Option<(Self::State, super::edit_graph::CigarOps)> {
        unimplemented!()
    }
}
