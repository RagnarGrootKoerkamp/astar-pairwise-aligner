//! This module contains implementations of other alignment algorithms.

use crate::prelude::*;

/// A visualizer can be used to visualize progress of an implementation.
trait Visualizer {
    fn explore(&mut self, _pos: Pos) {}
    fn expand(&mut self, _pos: Pos) {}
}

/// A trivial visualizer that does not do anything.
struct NoVisualizer;
impl Visualizer for NoVisualizer {}

/// An aligner is a type that supports aligning sequences using some algorithm.
/// It should implement the most general of the methods below, and never override variants with default parameters.
/// The cost-only variant can sometimes be implemented using less memory.
///
/// The output can be:
/// - cost only
/// - cost and alignment
/// - cost, alignment, and a visualization.
///
/// Note that insertions are when `b` has more characters than `a`, and deletions are when `b` has less characters than `a`.
trait Aligner {
    type Params;

    /// Can be either LinearCost or AffineCost, as needed by the algorithm.
    type CostModel: CostModel;

    fn cost(cm: Self::CostModel, a: &Sequence, b: &Sequence, params: Self::Params) -> Cost {
        Self::align(cm, a, b, params)
    }

    /// TODO: Make this return a path as well.
    fn align(cm: Self::CostModel, a: &Sequence, b: &Sequence, params: Self::Params) -> Cost {
        Self::visualize(cm, a, b, params, &mut NoVisualizer)
    }

    fn visualize(
        _cm: Self::CostModel,
        _a: &Sequence,
        _b: &Sequence,
        _params: Self::Params,
        _visualizer: &mut impl Visualizer,
    ) -> Cost {
        unimplemented!("This aligner does not support visualizations!");
    }
}
