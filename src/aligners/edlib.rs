use super::{cigar::Cigar, Aligner, Seq};
use crate::{
    aligners::edlib::edlib::{
        edlibAlign, edlibDefaultAlignConfig, edlibFreeAlignResult, EdlibAlignTask_EDLIB_TASK_PATH,
        EDLIB_STATUS_OK,
    },
    cost_model::{Cost, UnitCost},
};
use std::{intrinsics::transmute, ptr::slice_from_raw_parts};

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused)]
mod edlib {
    include!(concat!(env!("OUT_DIR"), "/bindings_edlib.rs"));
}

#[derive(Debug)]
pub struct Edlib;

fn edlib_align(a: Seq, b: Seq, trace: bool, f_max: Option<Cost>) -> Option<(u32, Option<Cigar>)> {
    unsafe {
        let a: &[i8] = transmute(a);
        let b: &[i8] = transmute(b);
        let mut config = edlibDefaultAlignConfig();
        if trace {
            config.task = EdlibAlignTask_EDLIB_TASK_PATH;
        }
        if let Some(f_max) = f_max {
            config.k = f_max as i32;
        }
        let result = edlibAlign(
            a.as_ptr(),
            a.len() as i32,
            b.as_ptr(),
            b.len() as i32,
            config,
        );
        assert!(result.status == EDLIB_STATUS_OK as i32);
        if result.editDistance == -1 {
            return None;
        }
        let distance = result.editDistance as Cost;

        let cigar = trace.then(|| {
            Cigar::from_edlib_alignment(&*slice_from_raw_parts(
                result.alignment,
                result.alignmentLength as usize,
            ))
        });
        edlibFreeAlignResult(result);
        Some((distance, cigar))
    }
}

impl Aligner for Edlib {
    type CostModel = UnitCost;

    fn cost_model(&self) -> &Self::CostModel {
        &UnitCost
    }

    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        edlib_align(a, b, false, None).unwrap().0
    }

    fn align(&mut self, a: Seq, b: Seq) -> (u32, Cigar) {
        let (d, c) = edlib_align(a, b, false, None).unwrap();
        (d, c.unwrap())
    }

    fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, f_max: Cost) -> Option<Cost> {
        Some(edlib_align(a, b, false, Some(f_max))?.0)
    }

    fn align_for_bounded_dist(&mut self, a: Seq, b: Seq, f_max: Cost) -> Option<(Cost, Cigar)> {
        let (d, c) = edlib_align(a, b, false, Some(f_max))?;
        Some((d, c.unwrap()))
    }
}
