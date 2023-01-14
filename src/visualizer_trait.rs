// FIXME
#![allow(unused_variables, dead_code)]
//! To turn images into a video, use this:
//!
//! ```sh
//! ffmpeg -framerate 20 -i %d.bmp output.mp4
//! ```
//! or when that gives errors:
//! ```sh
//! ffmpeg -framerate 20 -i %d.bmp -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" output.mp4
//! ```

use std::fmt::Debug;

use pa_types::{Cigar, CigarOp, Cost, I};

use crate::{
    heuristic::{HeuristicInstance, NoCostI},
    prelude::{Pos, Seq},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub i: I,
    pub j: I,
    pub layer: Option<usize>,
}
type ParentFn<'a> = Option<&'a dyn Fn(State) -> Option<(State, [Option<CigarOp>; 2])>>;

pub trait Visualizer: Clone + Default + Debug {
    type Instance: VisualizerInstance;
    fn build(&self, a: Seq, b: Seq) -> Self::Instance;
}

#[derive(Clone, Default, Debug)]
pub struct NoVis;
impl Visualizer for NoVis {
    type Instance = Self;
    fn build(&self, _a: Seq, _b: Seq) -> Self::Instance {
        Self
    }
}
impl VisualizerInstance for NoVis {}

/// A visualizer can be used to visualize progress of an implementation.
pub trait VisualizerInstance {
    fn explore(&mut self, pos: Pos, g: Cost, f: Cost) {
        self.explore_with_h::<NoCostI>(pos, g, f, None);
    }
    fn expand(&mut self, pos: Pos, g: Cost, f: Cost) {
        self.expand_with_h::<NoCostI>(pos, g, f, None);
    }
    fn extend(&mut self, pos: Pos, g: Cost, f: Cost) {
        self.extend_with_h::<NoCostI>(pos, g, f, None);
    }
    fn explore_with_h<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        _pos: Pos,
        _g: Cost,
        _f: Cost,
        _h: Option<&HI>,
    ) {
    }
    fn expand_with_h<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        _pos: Pos,
        _g: Cost,
        _f: Cost,
        _h: Option<&HI>,
    ) {
    }
    fn extend_with_h<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        _pos: Pos,
        _g: Cost,
        _f: Cost,
        _h: Option<&HI>,
    ) {
    }

    /// This function should be called after completing each layer
    fn new_layer(&mut self) {
        self.new_layer_with_h::<NoCostI>(None);
    }
    fn new_layer_with_h<'a, HI: HeuristicInstance<'a>>(&mut self, _h: Option<&HI>) {}

    /// This function may be called after the main loop to display final image.
    fn last_frame(&mut self, cigar: Option<&Cigar>) {
        self.last_frame_with_h::<NoCostI>(cigar, None, None);
    }
    fn last_frame_with_tree(&mut self, cigar: Option<&Cigar>, parent: ParentFn) {
        self.last_frame_with_h::<NoCostI>(cigar, parent, None);
    }
    fn last_frame_with_h<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        _cigar: Option<&Cigar>,
        _parent: ParentFn<'_>,
        _h: Option<&HI>,
    ) {
    }
}
