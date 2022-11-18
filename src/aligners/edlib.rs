use super::{cigar::Cigar, diagonal_transition::Direction, Aligner, Seq};
use crate::{
    aligners::edlib::edlib::{
        edlibAlign, edlibDefaultAlignConfig, edlibFreeAlignResult, EDLIB_STATUS_OK,
    },
    cost_model::{Cost, UnitCost},
};
use std::intrinsics::transmute;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused)]
mod edlib {
    include!(concat!(env!("OUT_DIR"), "/bindings_edlib.rs"));
}

#[derive(Debug)]
pub struct Edlib;

impl Aligner for Edlib {
    type CostModel = UnitCost;

    fn cost_model(&self) -> &Self::CostModel {
        &UnitCost
    }

    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
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
}
