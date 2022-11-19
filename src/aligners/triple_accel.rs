use triple_accel::levenshtein::EditCosts;

use crate::{cost_model::CostModel, prelude::Cost};

use super::{cigar::Cigar, Aligner, Seq};

/// NW aligner for unit costs (Levenshtein distance) only, using library functions.
#[derive(Debug)]
pub struct TripleAccel {
    exp_search: bool,
    cm: EditCosts,
}

impl TripleAccel {
    pub fn new(exp_search: bool, cm: CostModel) -> Self {
        let cm = match cm.to_affine() {
            CostModel::Affine { sub, open, extend } => {
                EditCosts::new(sub as u8, extend as u8, open as u8, None)
            }
            cm => panic!("TripleAccel does not support {cm:?}"),
        };

        Self { exp_search, cm }
    }
}

impl Aligner for TripleAccel {
    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        if self.exp_search {
            triple_accel::levenshtein::levenshtein_exp_with_opts(a, b, false, self.cm).0
        } else {
            triple_accel::levenshtein::levenshtein_simd_k_with_opts(a, b, u32::MAX, false, self.cm)
                .unwrap()
                .0
        }
    }

    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Cigar) {
        let (cost, edits) = if self.exp_search {
            triple_accel::levenshtein::levenshtein_exp_with_opts(a, b, true, self.cm)
        } else {
            triple_accel::levenshtein::levenshtein_simd_k_with_opts(a, b, u32::MAX, true, self.cm)
                .unwrap()
        };
        (cost, Cigar::from_triple_accel_edits(&edits.unwrap()))
    }

    fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, f_max: Cost) -> Option<Cost> {
        triple_accel::levenshtein::levenshtein_simd_k_with_opts(a, b, f_max, false, self.cm)
            .map(|r| r.0)
    }

    fn align_for_bounded_dist(&mut self, a: Seq, b: Seq, f_max: Cost) -> Option<(Cost, Cigar)> {
        triple_accel::levenshtein::levenshtein_simd_k_with_opts(a, b, f_max, true, self.cm)
            .map(|(cost, edits)| (cost, Cigar::from_triple_accel_edits(&edits.unwrap())))
    }
}
