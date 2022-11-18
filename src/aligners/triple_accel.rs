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

/// TripleAccel only implements `cost()`.
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
    fn align(&mut self, _a: Seq, _b: Seq) -> (Cost, Cigar) {
        unimplemented!()
    }
    fn cost_for_bounded_dist(&mut self, _a: Seq, _b: Seq, _f_max: Option<Cost>) -> Option<Cost> {
        unimplemented!();
    }

    fn align_for_bounded_dist(
        &mut self,
        _a: Seq,
        _b: Seq,
        _f_max: Option<Cost>,
    ) -> Option<(Cost, Cigar)> {
        unimplemented!();
    }
}
