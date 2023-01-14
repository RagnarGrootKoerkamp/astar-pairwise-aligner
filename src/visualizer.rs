use pa_types::{Cigar, CigarOp, Cost};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{
    heuristic::HeuristicInstance,
    prelude::{Pos, Seq},
};

pub type ParentFn<'a> = Option<&'a dyn Fn(Pos) -> Option<(Pos, [Option<CigarOp>; 2])>>;

/// A `Visualizer` can be used to track progress of the A* search using callbacks.
/// The `Visualizer` configuration is `build` into a corresponding `VisualizerInstance` for each input pair.
pub trait Visualizer: Clone + Copy + Default + Debug + PartialEq {
    type Instance: VisualizerInstance;
    fn build(&self, a: Seq, b: Seq) -> Self::Instance;
}

pub trait VisualizerInstance {
    fn explore<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        _pos: Pos,
        _g: Cost,
        _f: Cost,
        _h: Option<&HI>,
    ) {
    }
    fn expand<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        _pos: Pos,
        _g: Cost,
        _f: Cost,
        _h: Option<&HI>,
    ) {
    }
    fn extend<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        _pos: Pos,
        _g: Cost,
        _f: Cost,
        _h: Option<&HI>,
    ) {
    }

    /// This function should be called after completing each layer
    fn new_layer<'a, HI: HeuristicInstance<'a>>(&mut self, _h: Option<&HI>) {}

    /// This function may be called after the main loop to display final image.
    fn last_frame<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        _cigar: Option<&Cigar>,
        _parent: ParentFn<'_>,
        _h: Option<&HI>,
    ) {
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoVis;
impl Visualizer for NoVis {
    type Instance = Self;
    fn build(&self, _a: Seq, _b: Seq) -> Self::Instance {
        Self
    }
}
impl VisualizerInstance for NoVis {}
