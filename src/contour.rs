use std::cmp::Ordering;

use crate::graph::Pos;

/// A datastructure that contains the contours of non-dominant points.
/// The 'main' contour is the set of dominant points: {P: P >= S for all S}.
/// It returns whether a query point Q is inside the contour: {is there an S s.t. Q <= S}.
/// This is an online/dynamic datastructure that allows addition and removal of points:
/// - Addition of P must always be done on the top left: {not (S <= P) for all S}.
/// - Removal of P.
// TODO: An implementation that does lookup in O(lg(n))
// TODO: An implementation that does lookup, and push (and pop) in O(lg(n))
// TODO: An implementation that does lookup in O(1), using a hint.
trait Contour {
    fn contains(&self, _q: Pos) -> bool;
    /// Add a new point to the graph.
    /// This point must be 'smaller' (actually: not larger) than every existing point.
    fn push(&mut self, _p: Pos);
    /// Remove the point at the given position, and shift all contours.
    fn prune(&mut self, _p: Pos);
}

/// An arrow implies f(start) >= f(end) + len.
/// This is only needed for Contours, since Contour already assumes the
struct Arrow {
    start: Pos,
    end: Pos,
    len: usize,
}

/// A datastructure that contains multiple contours.
/// Supports incremental building from matches, querying, and pruning.
/// The structure is built by pushing matches in decreasing order.
trait Contours {
    /// The value of the contour this point is on.
    fn value(&self, _q: Pos) -> usize;
    /// Build the contours from a set of arrows.
    /// Arrows must be reverse sorted by start.
    fn build(&mut self, _arrow: impl IntoIterator<Item = Arrow>);
    /// Remove the point at the given position, and shift all contours.
    /// TODO: also prune all arrows ending in the given position.
    ///       or at least when this is the only outgoing arrow.
    fn prune(&mut self, _p: Pos);
}

/// A contour implementation that does all operations in O(n).
struct NaiveContour {
    points: Vec<Pos>,
}

impl Contour for NaiveContour {
    fn contains(&self, q: Pos) -> bool {
        for &s in &self.points {
            if q <= s {
                return true;
            }
        }
        return false;
    }

    fn push(&mut self, p: Pos) {
        for &s in &self.points {
            assert!(!(s <= p));
        }
        self.points.push(p);
    }

    fn prune(&mut self, p: Pos) {
        self.points.drain_filter(|&mut s| s == p);
    }
}

/// A Contours implementation with
struct NaiveContours<C: Contour> {
    contours: Vec<C>,
}

impl<C: Contour> Contours for NaiveContours<C> {
    fn value(&self, q: Pos) -> usize {
        self.contours
            .binary_search_by(|c: &C| {
                if c.contains(q) {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .unwrap_err()
            - 1
    }

    fn build(&mut self, arrows: impl IntoIterator<Item = Arrow>) {
        for a in arrows {
            let v = self.value(a.end) + a.len;
            self.contours[v].push(a.start);
        }
    }

    fn prune(&mut self, _p: Pos) {
        todo!();
    }
}
