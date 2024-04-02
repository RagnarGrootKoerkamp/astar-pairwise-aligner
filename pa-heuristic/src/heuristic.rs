pub mod bruteforce_gcsh;
pub mod csh;
pub mod distances;
pub mod sh;
pub mod wrappers;

use std::ops::Range;

use crate::matches::*;
use crate::prelude::*;
use crate::seeds::Seeds;
use derive_more::AddAssign;

pub use bruteforce_gcsh::*;
pub use csh::*;
pub use distances::*;
pub use sh::*;

#[derive(Clone, AddAssign, Default, Copy, Debug)]
pub struct HeuristicStats {
    pub num_seeds: I,
    pub num_matches: usize,
    pub num_filtered_matches: usize,
    pub num_pruned: usize,
    pub h0: Cost,
    pub h0_end: Cost,

    // Timers
    pub prune_duration: f64,
    pub prune_calls: usize,

    pub contours_duration: f64,
    pub contours_calls: usize,

    pub h_duration: f64,
    pub h_calls: usize,
}

/// An object containing the settings for a heuristic.
pub trait Heuristic: std::fmt::Debug + Copy {
    type Instance<'a>: HeuristicInstance<'a>;
    const IS_DEFAULT: bool = false;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        self.build_with_filter(a, b, None::<fn(&Match, Cost) -> bool>)
    }

    /// Matches can be filtered during construction of the contours.
    /// Used in PathHeuristic.
    fn build_with_filter<'a>(
        &self,
        _a: Seq<'a>,
        _b: Seq<'a>,
        _f: Option<impl FnMut(&Match, Cost) -> bool>,
    ) -> Self::Instance<'a> {
        unimplemented!();
    }

    // Heuristic properties.
    fn name(&self) -> String;
}

pub trait PosOrderT: PartialOrd + Default + Copy + std::fmt::Debug {
    fn from_pos(p: Pos) -> Self;
    fn max(p: Self, q: Self) -> Self;
    type D: std::fmt::Debug;
    fn tip_start(s: usize, p: Self) -> Self;
}

impl PosOrderT for () {
    fn from_pos(_: Pos) -> Self {}
    fn max(_: Self, _: Self) -> Self {}
    type D = ();
    fn tip_start(_: usize, _: Self) -> Self {}
}

/// The order for CSH
impl PosOrderT for Pos {
    fn from_pos(p: Pos) -> Self {
        p
    }
    fn max(p: Self, q: Self) -> Self {
        Pos(max(p.0, q.0), max(p.1, q.1))
    }
    type D = (i32, i32);
    fn tip_start(s: usize, p: Self) -> Self {
        Pos(p.0.saturating_sub(s as I), p.1.saturating_sub(s as I))
    }
}

/// The order of SH.
impl PosOrderT for I {
    fn from_pos(p: Pos) -> Self {
        p.0
    }
    fn max(p: Self, q: Self) -> Self {
        max(p, q)
    }
    type D = i32;
    fn tip_start(s: usize, p: Self) -> Self {
        p.saturating_sub(s as I)
    }
}

/// An instantiation of a heuristic for a specific pair of sequences.
pub trait HeuristicInstance<'a> {
    fn h(&self, pos: Pos) -> Cost;

    /// The internal contour value at the given position, if available.
    fn layer(&self, _pos: Pos) -> Option<Cost> {
        None
    }

    /// The internal contour value at the given position, if available.
    fn layer_with_hint(&self, pos: Pos, _hint: Self::Hint) -> Option<(Cost, Self::Hint)> {
        self.layer(pos).map(|c| (c, Default::default()))
    }

    fn h_with_parent(&self, pos: Pos) -> (Cost, Pos) {
        (self.h(pos), Pos::default())
    }

    type Hint: Copy + Default + std::fmt::Debug = ();
    fn h_with_hint(&self, pos: Pos, _hint: Self::Hint) -> (Cost, Self::Hint) {
        (self.h(pos), Default::default())
    }

    fn h_with_hint_timed(&mut self, pos: Pos, hint: Self::Hint) -> ((Cost, Self::Hint), f64) {
        (self.h_with_hint(pos, hint), 0.)
    }

    fn root_potential(&self) -> Cost {
        0
    }

    /// A* will checked for consistency whenever this returns true.
    fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        self.seeds()
            .map_or(false, |sm| sm.is_seed_start_or_end(pos))
    }

    type Order: PosOrderT = ();

    /// Returns the offset by which all expanded states in the priority queue can be shifted.
    ///
    /// `seed_cost`: The cost made in the seed ending at pos.
    fn prune(&mut self, _pos: Pos, _hint: Self::Hint) -> (Cost, Self::Order) {
        (0, Default::default())
    }
    fn prune_block(&mut self, _i_range: Range<I>, _j_range: Range<I>) {
        //unimplemented!();
    }

    /// Update contours from the current minimum changed layer up to the given `_pos`.
    fn update_contours(&mut self, _pos: Pos) {
        //unimplemented!();
    }

    /// Tells the heuristic that the position was explored, so it knows which
    /// positions need to be updated when propagating the pruning to the
    /// priority queue.
    fn explore(&mut self, _pos: Pos) {}

    fn stats(&mut self) -> HeuristicStats {
        Default::default()
    }

    fn matches(&self) -> Option<Vec<Match>> {
        None
    }

    fn seeds(&self) -> Option<&Seeds> {
        None
    }

    /// A descriptive string of the heuristic settings, used for failing assertions.
    fn params_string(&self) -> String {
        "".into()
    }
}

impl<'a> HeuristicInstance<'a> for ! {
    type Hint = ();
    type Order = ();
    fn h(&self, _pos: Pos) -> Cost {
        unreachable!()
    }
}
