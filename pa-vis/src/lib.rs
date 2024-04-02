#![feature(let_chains, int_roundings, never_type)]

pub mod cli;
#[cfg(feature = "sdl")]
mod sdl;
pub mod visualizer;

pub mod canvas;

use canvas::Canvas;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use pa_affine_types::*;
use pa_heuristic::*;
use pa_types::*;

pub type ParentFn<'a> = Option<&'a dyn Fn(State) -> Option<(State, [Option<AffineCigarOp>; 2])>>;

pub trait CanvasFactory {
    fn new(w: usize, h: usize, title: &str) -> Box<dyn Canvas>;
}

/// A `Visualizer` can be used to track progress of the A* search using callbacks.
/// The `Visualizer` configuration is `build` into a corresponding `VisualizerInstance` for each input pair.
pub trait VisualizerT: Clone + Default + Debug + PartialEq {
    type Instance: VisualizerInstance;
    // Build using an sdl2 canvas.
    fn build(&self, a: Seq, b: Seq) -> Self::Instance;
    fn build_from_factory<CF: CanvasFactory>(&self, a: Seq, b: Seq) -> Self::Instance;
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
    fn expand_preprune(&mut self, _pos: Pos) {}
    fn extend_preprune(&mut self, _pos: Pos) {}
    fn expand_trace(&mut self, _pos: Pos) {}
    fn extend_trace(&mut self, _pos: Pos) {}
    fn expand_block<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        _pos: Pos,
        _size: Pos,
        _g: Cost,
        _f: Cost,
        _h: Option<&HI>,
    ) {
    }
    fn expand_block_trace(&mut self, _pos: Pos, _size: Pos) {}
    fn expand_blocks<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        _poss: [Pos; 4],
        _sizes: [Pos; 4],
        _g: Cost,
        _f: Cost,
        _h: Option<&HI>,
    ) {
    }

    fn h_call(&mut self, _pos: Pos) {}
    fn f_call(&mut self, _pos: Pos, _in_bounds: bool, _fixed: bool) {}
    fn j_range(&mut self, _start: Pos, _end: Pos) {}
    fn fixed_j_range(&mut self, _start: Pos, _end: Pos) {}
    fn fixed_h(&mut self, _start: Pos, _end: Pos) {}
    fn next_fixed_h(&mut self, _start: Pos, _end: Pos) {}

    /// This function should be called after completing each layer
    fn new_layer<'a, HI: HeuristicInstance<'a>>(&mut self, _h: Option<&HI>) {}

    /// Add the given position to the optimal path for divide-and-conquer
    /// methods, and clear existing explored/expanded states.
    fn add_meeting_point<'a, HI: HeuristicInstance<'a>>(&mut self, _pos: Pos) {}

    /// This function may be called after the main loop to display final image.
    fn last_frame<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        _cigar: Option<&AffineCigar>,
        _parent: ParentFn<'_>,
        _h: Option<&HI>,
    ) {
    }

    fn expand_block_simple<'a>(&mut self, pos: Pos, size: Pos) {
        self.expand_block::<!>(pos, size, 0, 0, None)
    }
    fn expand_blocks_simple<'a>(&mut self, poss: [Pos; 4], sizes: [Pos; 4]) {
        self.expand_blocks::<!>(poss, sizes, 0, 0, None)
    }
    fn last_frame_simple<'a>(&mut self) {
        self.last_frame::<!>(None, None, None)
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoVis;

impl VisualizerT for NoVis {
    type Instance = Self;
    fn build(&self, _a: Seq, _b: Seq) -> Self::Instance {
        Self
    }

    fn build_from_factory<CF: CanvasFactory>(&self, _a: Seq, _b: Seq) -> Self::Instance {
        Self
    }
}
impl VisualizerInstance for NoVis {}
