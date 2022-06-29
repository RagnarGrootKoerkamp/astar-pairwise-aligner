use crate::prelude::{Cost, LinearCost};

use super::{cigar::Cigar, edit_graph::State, Aligner, Path, Seq};

/// NW aligner for unit costs (Levenshtein distance) only, using library functions.
pub struct NWLib {
    pub simd: bool,
}

lazy_static! {
    static ref COST_MODEL: LinearCost = LinearCost::new_unit();
}

/// NWLib aligner only implements `cost()`.
impl Aligner for NWLib {
    type CostModel = LinearCost;

    type Fronts = ();

    fn cost_model(&self) -> &Self::CostModel {
        &COST_MODEL
    }

    fn parent(&self, _a: Seq, _b: Seq, _fronts: Self::Fronts, _state: State) -> Option<State> {
        unimplemented!()
    }

    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        if self.simd {
            bio::alignment::distance::simd::levenshtein(a, b)
        } else {
            bio::alignment::distance::levenshtein(a, b)
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
}
