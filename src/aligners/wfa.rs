#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::intrinsics::transmute;
use std::ffi::CString;
use crate::{cost_model::{Cost, LinearCost, AffineCost, CostModel, AffineLayerType}, aligners::wfa::wfa::{distance_metric_t_gap_linear, affine_penalties_t}};

use super::{cigar::Cigar, Aligner, Path, Seq};

mod wfa {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub struct WFA<CostModel> {
    cm: CostModel,
}

lazy_static! {
    static ref COST_MODEL: LinearCost = LinearCost::new_unit();
}

fn unit_cost( a: Seq, b: Seq) -> Cost {
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

fn linear_cost( a: Seq, b: Seq, sub: Cost, indel: Cost) -> Cost {
    unsafe {
        // Configure alignment attributes
        let mut attributes = wfa::wavefront_aligner_attr_default;
        attributes.distance_metric = distance_metric_t_gap_linear;
        attributes.alignment_scope = wfa::alignment_scope_t_compute_score;
        attributes.linear_penalties.indel = indel as i32;
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

fn affine_cost(a: Seq, b: Seq, mismatch: Cost, gap_open: Cost, gap_extend: Cost) -> Cost {
    // Configure alignment attributes
    unsafe {
        let mut attributes = wfa::wavefront_aligner_attr_default;
        attributes.distance_metric = wfa::distance_metric_t_gap_affine;
        attributes.affine_penalties.mismatch = mismatch as i32;
        attributes.affine_penalties.gap_opening = gap_open as i32;
        attributes.affine_penalties.gap_extension = gap_extend as i32;
        let a: &[i8] = transmute(a);
        let b: &[i8] = transmute(b);
        let wf_aligner = wfa::wavefront_aligner_new(&mut attributes);
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

impl<const N: usize> Aligner for WFA<AffineCost<N>> {
    type CostModel = LinearCost;

    fn cost_model(&self) -> &Self::CostModel {
        &COST_MODEL
    }

    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        if N == 0 {
            // unit cost
            if self.cm.sub == Some(1) && self.cm.ins == Some(1) && self.cm.del == Some(1){
                return unit_cost(a, b);
                // linear cost
            } else if let Some(sub) = self.cm.sub
                      && let Some(ins) = self.cm.ins
                      && let Some(del) = self.cm.del 
                      && ins == del {     
                return linear_cost(a, b, sub, ins);
            }
            
        } else if N == 2 {
            if let Some(mismatch) = self.cm.sub {   
                if (self.cm.affine[0].affine_type == AffineLayerType::InsertLayer && self.cm.affine[1].affine_type == AffineLayerType::DeleteLayer) || (self.cm.affine[1].affine_type == AffineLayerType::InsertLayer && self.cm.affine[0].affine_type == AffineLayerType::DeleteLayer) {
                    if (self.cm.affine[0].affine_type == self.cm.affine[1].affine_type) || (self.cm.affine[1].affine_type == self.cm.affine[0].affine_type){
                    if let Some(mism) = self.cm.sub {
                            return affine_cost(a, b, mism, self.cm.affine[0].open, self.cm.affine[0].extend);
                    }
                } 
            }
        }
    }
    todo!()
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
