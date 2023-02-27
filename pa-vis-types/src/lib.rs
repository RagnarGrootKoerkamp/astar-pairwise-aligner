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
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoVis;

impl Canvas for NoVis {
    fn fill_background(&mut self, _color: canvas::Color) {}
    fn fill_rect(&mut self, _p: canvas::CPos, _w: I, _h: I, _color: canvas::Color) {}
    fn draw_rect(&mut self, _p: canvas::CPos, _w: I, _h: I, _color: canvas::Color) {}
    fn draw_line(&mut self, _p: canvas::CPos, _q: canvas::CPos, _color: canvas::Color) {}
    fn write_text(
        &mut self,
        _p: canvas::CPos,
        _ha: canvas::HAlign,
        _va: canvas::VAlign,
        _text: &str,
        _color: canvas::Color,
    ) {
    }
    fn wait(&mut self, _timeout: std::time::Duration) -> canvas::KeyboardAction {
        canvas::KeyboardAction::Exit
    }
}

impl CanvasFactory for NoVis {
    fn new(_w: usize, _h: usize, _title: &str) -> Box<dyn Canvas> {
        Box::new(Self)
    }
}

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
