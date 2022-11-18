use crate::{
    cost_model::{CostModel, UnitCost},
    prelude::Cost,
};

use super::{cigar::Cigar, diagonal_transition::Direction, edit_graph::CigarOps, Aligner, Seq};

/// NW aligner for unit costs (Levenshtein distance) only, using library functions.
#[derive(Debug)]
pub struct TripleAccel<C: CostModel> {
    pub exp_search: bool,
    pub cost_model: C,
}

/// TripleAccel only implements `cost()`.
impl Aligner for TripleAccel<UnitCost> {
    type CostModel = UnitCost;

    type Fronts = ();

    type State = ();

    fn cost_model(&self) -> &Self::CostModel {
        &UnitCost
    }

    fn parent(
        &self,
        _a: Seq,
        _b: Seq,
        _fronts: &Self::Fronts,
        _state: Self::State,
        _direction: Direction,
    ) -> Option<(Self::State, CigarOps)> {
        unimplemented!()
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
