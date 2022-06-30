#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


use crate::cost_model::{Cost, LinearCost};

use super::{cigar::Cigar, nw::Path, Aligner, Seq};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct WFA;

lazy_static! {
    static ref COST_MODEL: LinearCost = LinearCost::new_unit();
}
impl Aligner for WFA {
    type CostModel = LinearCost;

    fn cost_model(&self) -> &Self::CostModel {
        &COST_MODEL
    }

    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        unsafe {
            // Configure alignment attributes
            let mut attributes = wavefront_aligner_attr_default;
            attributes.distance_metric = distance_metric_t_gap_affine;
            attributes.affine_penalties.mismatch = 4;
            attributes.affine_penalties.gap_opening = 6;
            attributes.affine_penalties.gap_extension = 2;
            // Initialize Wavefront Aligner
            let wf_aligner = wavefront_aligner_new(&mut attributes);
            let au8 = &*(a as *const _ as *const [i8]);
            let bu8 = &*(b as *const _ as *const [i8]);
            return wavefront_align(
                wf_aligner,
                au8.as_ptr(),
                a.len() as i32,
                bu8.as_ptr(),
                b.len() as i32,
            ) as Cost;
            // Align
        }
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

    fn cost_exponential_search(&mut self, a: Seq, b: Seq) -> Cost {
        bio::alignment::distance::simd::levenshtein(a, b)
    }
}
