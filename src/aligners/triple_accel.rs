use triple_accel::levenshtein::EditCosts;

use crate::{
    cost_model::{CostModel, UnitCost},
    prelude::Cost,
};

use super::{cigar::Cigar, Aligner, Seq};

/// NW aligner for unit costs (Levenshtein distance) only, using library functions.
#[derive(Debug)]
pub struct TripleAccel<C: CostModel> {
    pub exp_search: bool,
    pub cost_model: C,
}

impl Aligner for TripleAccel<UnitCost> {
    type CostModel = UnitCost;

    fn cost_model(&self) -> &Self::CostModel {
        &UnitCost
    }

    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        if self.exp_search {
            triple_accel::levenshtein_exp(a, b)
        } else {
            triple_accel::levenshtein(a, b)
        }
    }

    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Cigar) {
        let (cost, edits) = if self.exp_search {
            triple_accel::levenshtein::levenshtein_exp_with_opts(
                a,
                b,
                true,
                EditCosts::new(1, 0, 1, None),
            )
        } else {
            triple_accel::levenshtein::levenshtein_simd_k_with_opts(
                a,
                b,
                u32::MAX,
                true,
                EditCosts::new(1, 0, 1, None),
            )
            .unwrap()
        };
        (cost, Cigar::from_edits(&edits.unwrap()))
    }

    fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, f_max: Cost) -> Option<Cost> {
        triple_accel::levenshtein::levenshtein_simd_k(a, b, f_max)
    }

    fn align_for_bounded_dist(&mut self, a: Seq, b: Seq, f_max: Cost) -> Option<(Cost, Cigar)> {
        triple_accel::levenshtein::levenshtein_simd_k_with_opts(
            a,
            b,
            f_max,
            true,
            EditCosts::new(1, 0, 1, None),
        )
        .map(|(cost, edits)| (cost, Cigar::from_edits(&edits.unwrap())))
    }
}
