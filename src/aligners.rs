//! This module contains implementations of other alignment algorithms.

use crate::prelude::{Cost, Pos, Sequence};

use self::{cigar::Cigar, nw::PATH};

pub mod cigar;
pub mod diagonal_transition;
pub mod exp_band;
pub mod front;
pub mod layer;
pub mod nw;
pub mod nw_lib;

#[cfg(test)]
mod tests;

/// A visualizer can be used to visualize progress of an implementation.
pub trait Visualizer {
    fn explore(&mut self, _pos: Pos) {}
    fn expand(&mut self, _pos: Pos) {}
}

/// A trivial visualizer that does not do anything.
struct NoVisualizer;
impl Visualizer for NoVisualizer {}

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
    fn cost(&self, a: &Sequence, b: &Sequence) -> Cost {
        self.align(a, b).0
    }

    /// TODO: Make this return a path as well.
    fn align(&self, a: &Sequence, b: &Sequence) -> (Cost, PATH, Cigar) {
        self.visualize(a, b, &mut NoVisualizer)
    }

    fn visualize(
        &self,
        _a: &Sequence,
        _b: &Sequence,
        _visualizer: &mut impl Visualizer,
    ) -> (Cost, PATH, Cigar) {
        unimplemented!("This aligner does not support visualizations!");
    }
}
