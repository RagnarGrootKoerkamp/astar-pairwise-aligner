use crate::prelude::{Cost, LinearCost};

use super::{cigar::Cigar, diagonal_transition::Direction, edit_graph::CigarOps, Aligner, Seq};

/// NW aligner for unit costs (Levenshtein distance) only, using library functions.
#[derive(Debug)]
pub struct TripleAccel {
    pub exp_search: bool,
}

lazy_static! {
    static ref COST_MODEL: LinearCost = LinearCost::new_unit();
}

/// TripleAccel only implements `cost()`.
impl Aligner for TripleAccel {
    type CostModel = LinearCost;

    type Fronts = ();

    type State = ();

    fn cost_model(&self) -> &Self::CostModel {
        &COST_MODEL
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
