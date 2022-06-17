//! This module contains implementations of other alignment algorithms.

use self::{cigar::Cigar, nw::Path};
use crate::prelude::{Cost, CostModel, Pos};
use crate::prelude::{Cost, Pos, Sequence};
use sdl2::pixels::Color;
use std::cell::Cell;
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

#[derive(Clone)]
enum Gradient {
    NoGradient(Color, Color), //(expanded_color,explored color)
    Gradient(Color, Color),   //(start color, end color)
    TurboGradient(f32, f32), //(start value, end value); start < end; start > 0 && end > 0; start < 1 && end <= 1
}

#[derive(Clone)]
struct ColorScheme {
    gradient: Gradient,
    bg_color: Color,
}

// let default_colors: ColorScheme = ColorScheme{};

#[derive(Clone)]
pub struct Config {
    cell_size: usize,
    prescaler: usize, //for scaling image
    filepath: String, //maybe &str instead
    drawing: bool,
    delay: Cell<f32>,
    saving: bool,
    colors: ColorScheme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cell_size: 8,
            prescaler: 1,
            saving: false,
            filepath: String::from(""),
            drawing: false,
            delay: Cell::new(0.2),
            colors: ColorScheme {
                gradient: Gradient::NoGradient(Color::BLUE, Color::RGB(128, 0, 128)),
                bg_color: Color::BLACK,
            },
        }
    }
}

/// A visualizer can be used to visualize progress of an implementation.
pub trait VisualizerT {
    #[inline]
    fn init(&mut self, config: &Config, len1: u32, len2: u32) {}
    #[inline]
    fn explore(&mut self, _pos: Pos) {}
    #[inline]
    fn expand(&mut self, _pos: Pos) {}
    #[inline]
    fn draw(&mut self) {}
}

/// A trivial visualizer that does not do anything.
pub struct NoVisualizer;
impl VisualizerT for NoVisualizer {}

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

    /// Returns the cost model used by the aligner.
    fn cost_model(&self) -> &Self::CostModel;

    /// Finds the cost of aligning `a` and `b`.
    /// Uses the visualizer to record progress.
    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        self.cost_for_bounded_dist(a, b, None).unwrap()
    }

    /// Finds an alignments (path/Cigar) of sequences `a` and `b`.
    /// Uses the visualizer to record progress.
    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Path, Cigar) {
        self.align_for_bounded_dist(a, b, None).unwrap()
    }

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

    /// Find the cost using exponential search based on `cost_assuming_bounded_dist`.
    /// TODO: Allow customizing the growth factor.
    fn cost_exponential_search(&mut self, a: Seq, b: Seq) -> Cost {
        let mut s: Cost = self
            .cost_model()
            .gap_cost(Pos(0, 0), Pos::from_lengths(a, b));
        // TODO: Fix the potential infinite loop here.
        loop {
            let cost = self.cost_for_bounded_dist(a, b, Some(s));
            if let Some(cost) = cost && cost <= s{
                return cost;
            }
            s = max(2 * s, 1);
        }
    }

    /// Find the alignment using exponential search based on `align_assuming_bounded_dist`.
    /// TODO: Allow customizing the growth factor.
    fn align_exponential_search(&mut self, a: Seq, b: Seq) -> (Cost, Path, Cigar) {
        let mut s: Cost = self
            .cost_model()
            .gap_cost(Pos(0, 0), Pos::from_lengths(a, b));
        // TODO: Fix the potential infinite loop here.
        loop {
            if let Some(tuple@(cost, _, _)) = self.align_for_bounded_dist(a, b, Some(s)) && cost <= s{
                return tuple;
            }
            s = max(2 * s, 1);
        }
    }
}
