#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::intrinsics::transmute;

use crate::cost_model::{Cost, LinearCost};

use super::{cigar::Cigar, Aligner, Path, Seq};

mod wfa {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

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
            let mut attributes = wfa::wavefront_aligner_attr_default;
            attributes.distance_metric = wfa::distance_metric_t_edit;
            attributes.alignment_scope = wfa::alignment_scope_t_compute_score;
            // Initialize Wavefront Aligner
            let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);
            let a: &[i8] = transmute(a);
            let b: &[i8] = transmute(b);
            let status = wfa::wavefront_align(
                wf_aligner,
                a.as_ptr(),
                a.len() as i32,
                b.as_ptr(),
                b.len() as i32,
            );
            assert_eq!(status, 0);
            -(*wf_aligner).cigar.score as Cost
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

    type Fronts = ();

    type State = ();

    fn parent(
        &self,
        _a: Seq,
        _b: Seq,
        _fronts: &Self::Fronts,
        _st: Self::State,
    ) -> Option<(Self::State, super::edit_graph::CigarOps)> {
        unimplemented!()
    }
}
