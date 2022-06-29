//! This module contains implementations of other alignment algorithms.

use self::{cigar::Cigar, edit_graph::State};
use crate::prelude::{Cost, CostModel, Pos};
use std::cmp::max;

pub mod cigar;
pub mod diagonal_transition;
pub mod front;
pub mod nw;
pub mod nw_lib;

pub mod edit_graph;
#[cfg(test)]
mod tests;

/// An owned sequence.
pub type Sequence = Vec<u8>;
/// A sequence slice.
pub type Seq<'a> = &'a [u8];
/// A path trough the edit graph.
pub type Path = Vec<Pos>;

/// Find the cost using exponential search based on `cost_assuming_bounded_dist`.
fn exponential_search<T>(
    s0: Cost,
    factor: f32,
    mut f: impl FnMut(Cost) -> Option<(Cost, T)>,
) -> (Cost, T) {
    let mut s = s0;
    // TODO: Fix the potential infinite loop here.
    loop {
        if let Some((cost,t)) = f(s) && cost <= s{
            return (cost, t);
        }
        s = max((factor * s as f32).ceil() as Cost, 1);
    }
}

/// An aligner is a type that supports aligning sequences using some algorithm.
/// It should implement the most general of the methods below.
/// The cost-only variant can sometimes be implemented using less memory.
///
/// There is one function for each cost model:
/// - LinearCost
/// - AffineCost
///
/// The output can be:
/// - cost only
/// - cost and alignment
/// - cost, alignment, and a visualization.
///
/// Note that insertions are when `b` has more characters than `a`, and deletions are when `b` has less characters than `a`.
pub trait Aligner {
    type CostModel: CostModel;

    type Fronts;

    /// Returns the cost model used by the aligner.
    fn cost_model(&self) -> &Self::CostModel;

    /// Returns the parent state of the given state, or none from the root.
    fn parent(&self, a: Seq, b: Seq, fronts: Self::Fronts, st: State) -> Option<State>;

    /// Finds the cost of aligning `a` and `b`.
    /// Uses the visualizer to record progress.
    fn cost(&mut self, a: Seq, b: Seq) -> Cost;

    /// Finds an alignments (path/Cigar) of sequences `a` and `b`.
    /// Uses the visualizer to record progress.
    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Path, Cigar);

    /// Finds the cost of aligning `a` and `b`, assuming the cost of the alignment is at most `s_bound`.
    /// The returned cost may be `None` in case aligning with cost at most `s` is not possible.
    /// The returned cost may be larger than `s` when a path was found, even
    /// though this may not be the optimal cost.
    ///
    /// When `_s_bound` is `None`, there is no upper bound, and this is the same as simply `cost`.
    fn cost_for_bounded_dist(&mut self, _a: Seq, _b: Seq, _s_bound: Option<Cost>) -> Option<Cost>;

    /// Finds an alignments (path/Cigar) of sequences `a` and `b`, assuming the
    /// cost of the alignment is at most s.
    /// The returned cost may be `None` in case aligning with cost at most `s` is not possible.
    /// The returned cost may be larger than `s` when a path was found, even
    /// though this may not be the optimal cost.
    ///
    /// When `_s_bound` is `None`, there is no upper bound, and this is the same as simply `align`.
    fn align_for_bounded_dist(
        &mut self,
        _a: Seq,
        _b: Seq,
        _s_bound: Option<Cost>,
    ) -> Option<(Cost, Path, Cigar)>;
}
