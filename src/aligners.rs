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
/// It must implement one of the methods below.
/// If possible, implement `visualize_params`, so that all of the other methods work by default.
trait Aligner {
    type Params: Default;
    fn align(a: &Sequence, b: &Sequence) {
        Self::align_params(a, b, Self::Params::default())
    }
    fn align_params(a: &Sequence, b: &Sequence, params: Self::Params) {
        Self::visualize_params(a, b, params, &mut NoVisualizer)
    }

    fn visualize(a: &Sequence, b: &Sequence, visualizer: &mut impl Visualizer) {
        Self::visualize_params(a, b, Self::Params::default(), visualizer)
    }
    fn visualize_params(
        _a: &Sequence,
        _b: &Sequence,
        _params: Self::Params,
        _visualizer: &mut impl Visualizer,
    ) {
        unimplemented!("This aligner does not support visualizations!");
    }
}
