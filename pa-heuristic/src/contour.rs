pub mod bruteforce;
pub mod hint_contours;
pub mod rotate_to_front;
pub mod sh_contours;

pub use bruteforce::*;
pub use hint_contours::*;

use std::fmt::{Debug, Display};

use crate::{prelude::*, seeds::MatchCost};

pub type Layer = u32;

/// A datastructure that contains the contours of non-dominant points.
/// The 'main' contour is the set of dominant points: {P: P >= S for all S}.
/// It returns whether a query point Q is inside the contour: {is there an S s.t. Q <= S}.
/// This is an online/dynamic datastructure that allows addition and removal of points:
/// - Addition of P. Usually on the top left: {not (S < P) for all S}, but not always (see NaiveContours).
/// - Removal of P.
// TODO: An implementation that does lookup in O(lg(n))
// TODO: An implementation that does lookup, and push (and pop) in O(lg(n))
// TODO: An implementation that does lookup in O(1), using a hint.
pub trait Contour: Default + Debug + Clone {
    fn with_max_len(_max_len: I) -> Self {
        Default::default()
    }
    /// Add a new point to the graph.
    /// This point must be 'smaller' (actually: not larger) than every existing point.
    fn push(&mut self, _p: Pos);
    fn contains_equal(&self, _q: Pos) -> bool;
    /// Is point `q` above/top-left of the contour.
    fn contains(&self, _q: Pos) -> bool;
    /// Assuming that q is contained in the contour, find a witness of this.
    fn parent(&self, q: Pos) -> Pos;

    /// Is this point dominant?
    fn is_dominant(&self, _q: Pos) -> bool;
    /// Remove the point at the given position, and shift all contours.
    /// Returns whether p was dominant.
    fn prune(&mut self, p: Pos) -> bool {
        self.prune_filter(&mut |s| s == p)
    }
    /// Prune all points for which f returns true.
    /// NOTE: Implementations only have to make sure that the dominant points are correct.
    /// It is allowed to skip pruning of non-dominant points.
    fn prune_filter<F: FnMut(Pos) -> bool>(&mut self, f: &mut F) -> bool;

    fn len(&self) -> usize;
    fn num_dominant(&self) -> usize;

    fn print_points(&self) {}

    // Arguments:
    // - Point in contour
    fn iterate_points<F: FnMut(Pos)>(&self, _f: F) {}
}

/// An arrow implies f(start) >= f(end) + score.
/// This is only needed for Contours, since Contour already assumes the points all have the same value.
/// NOTE: It is assumed that |start - end|_1 <= 2 * score. This is needed to ensure the bounded width of each contour.
#[derive(Clone, PartialEq)]
pub struct Arrow {
    pub start: Pos,
    pub end: Pos,
    pub score: MatchCost,
}

// Implementations for Arrow
impl Display for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{:?} => {:?} [{}]",
            self.start, self.end, self.score
        ))
    }
}

impl Debug for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Arrow as Display>::fmt(self, f)
    }
}

/// A datastructure that contains multiple contours.
/// Supports incremental building from matches, querying, and pruning.
/// The structure is built by pushing matches in decreasing order.
// TODO: Make Pos and Cost template arguments instead?
// Pos could be either transformed or non-transformed domain.
// After transformation, it lives in the Cost domain.
pub trait Contours: Default + Debug {
    /// Build the contours from a set of arrows.
    /// NOTE: Arrows must be reverse sorted by start.
    fn new(arrows: impl IntoIterator<Item = Arrow>, max_len: Cost) -> Self {
        Self::new_with_filter(arrows, max_len, |_, _| true)
    }

    /// A secondary constructor used in PathHeuristic that filters arrows while constructing the heuristic.
    /// Only arrows for which [filter] returns true are kept.
    fn new_with_filter(
        _arrows: impl IntoIterator<Item = Arrow>,
        _max_len: Cost,
        _filter: impl FnMut(&Arrow, Cost) -> bool,
    ) -> Self {
        unimplemented!();
    }

    /// The value of the contour this point is on.
    /// Hint is guaranteed to be for the current position.
    fn score(&self, q: Pos) -> Cost;

    /// Find the value of the contour, and return a witness on that contour.
    fn parent(&self, q: Pos) -> (Cost, Pos);

    type Hint: Copy + Debug + Default = ();
    fn score_with_hint(&self, q: Pos, _hint: Self::Hint) -> (Cost, Self::Hint)
    where
        Self::Hint: Default,
    {
        (self.score(q), Self::Hint::default())
    }
    /// Remove the point at the given position, and shift all contours.
    /// This removes all arrows starting at the given position.
    /// Returns true when at the point was removed.
    /// TODO: also prune all arrows ending in the given position.
    ///       or at least when this is the only outgoing arrow.
    /// If the additional Cost return is positive, this indicates that position
    /// `p` was the only arrow in its layer, and a total of Cost layers were
    /// removed.
    fn prune_with_hint<R: Iterator<Item = Arrow>, F: Fn(&Pos) -> Option<R>>(
        &mut self,
        p: Pos,
        hint: Self::Hint,
        // TODO: Consider giving ownership to Contours, and add a getter to access it from the heuristic.
        arrows: F,
    ) -> (bool, Cost);

    /// Update layers starting at layer `v`, continuing at least to layer `last_change`.
    /// Stop when contours are fully left of `right_of`.
    fn update_layers<R: Iterator<Item = Arrow>, F: Fn(&Pos) -> Option<R>>(
        &mut self,
        _v: u32,
        _last_change: u32,
        _arrows: &F,
        _right_of: Option<(I, impl Fn(Pos) -> Pos)>,
    ) {
        unimplemented!();
    }

    /// Returns some statistics.
    fn print_stats(&mut self) {}
}
