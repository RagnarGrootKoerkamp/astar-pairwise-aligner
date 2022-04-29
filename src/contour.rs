pub mod bruteforce;
pub mod central;
pub mod hint_contours;

pub use bruteforce::*;
pub use central::*;
pub use hint_contours::*;

use std::fmt::{Debug, Display};

use crate::prelude::{Cost, HashMap, MatchCost, Pos, I};

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
}

/// An arrow implies f(start) >= f(end) + len.
/// This is only needed for Contours, since Contour already assumes the points all have the same value.
/// NOTE: It is assumed that |start - end|_1 <= 2 * len. This is needed to ensure the bounded width of each contour.
pub struct Arrow {
    pub start: Pos,
    pub end: Pos,
    // ~ discount
    pub len: MatchCost,
}

// Implementations for Arrow
impl Display for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{:?} => {:?} [{}]",
            self.start, self.end, self.len
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
    fn new(_arrows: impl IntoIterator<Item = Arrow>, max_len: I) -> Self;
    /// The value of the contour this point is on.
    /// Hint is guaranteed to be for the current position.
    fn value(&self, _q: Pos) -> Cost;

    type Hint: Copy + Debug + Default = ();
    fn value_with_hint(&self, _q: Pos, _hint: Self::Hint) -> (Cost, Self::Hint)
    where
        Self::Hint: Default,
    {
        (self.value(_q), Self::Hint::default())
    }
    /// Remove the point at the given position, and shift all contours.
    /// This removes all arrows starting at the given position.
    /// Returns true when at the point was removed.
    /// TODO: also prune all arrows ending in the given position.
    ///       or at least when this is the only outgoing arrow.
    /// If the additional Cost return is positive, this indicates that position
    /// `p` was the only arrow in its layer, and a total of Cost layers were
    /// removed.
    fn prune_with_hint(
        &mut self,
        p: Pos,
        hint: Self::Hint,
        arrows: &HashMap<Pos, Vec<Arrow>>,
    ) -> (bool, Cost);

    /// Returns some statistics.
    fn print_stats(&self) {}
}
