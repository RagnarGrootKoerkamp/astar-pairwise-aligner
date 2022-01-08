use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    mem,
};

use crate::graph::Pos;

/// A datastructure that contains the contours of non-dominant points.
/// The 'main' contour is the set of dominant points: {P: P >= S for all S}.
/// It returns whether a query point Q is inside the contour: {is there an S s.t. Q <= S}.
/// This is an online/dynamic datastructure that allows addition and removal of points:
/// - Addition of P must always be done on the top left: {not (S < P) for all S}.
/// - Removal of P.
// TODO: An implementation that does lookup in O(lg(n))
// TODO: An implementation that does lookup, and push (and pop) in O(lg(n))
// TODO: An implementation that does lookup in O(1), using a hint.
pub trait Contour: Default + Debug {
    /// Add a new point to the graph.
    /// This point must be 'smaller' (actually: not larger) than every existing point.
    fn push(&mut self, _p: Pos);
    /// Is point `q` above/top-left of the contour.
    fn contains(&self, _q: Pos) -> bool;
    /// Remove the point at the given position, and shift all contours.
    fn prune(&mut self, _p: Pos);
}

/// An arrow implies f(start) >= f(end) + len.
/// This is only needed for Contours, since Contour already assumes the
pub struct Arrow {
    pub start: Pos,
    pub end: Pos,
    pub len: usize,
}

impl std::fmt::Display for Arrow {
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
pub trait Contours: Default + Debug {
    /// Build the contours from a set of arrows.
    /// NOTE: Arrows must be reverse sorted by start.
    fn new(_arrows: impl IntoIterator<Item = Arrow>) -> Self;
    /// The value of the contour this point is on.
    /// Hint is guaranteed to be for the current position.
    fn value(&self, _q: Pos) -> usize;
    /// Remove the point at the given position, and shift all contours.
    /// TODO: also prune all arrows ending in the given position.
    ///       or at least when this is the only outgoing arrow.
    fn prune(&mut self, _p: Pos);
}

/// A contour implementation that does all operations in O(n).
#[derive(Default, Debug)]
pub struct NaiveContour {
    points: Vec<Pos>,
}

impl Contour for NaiveContour {
    fn push(&mut self, p: Pos) {
        for &s in &self.points {
            assert!(!(s < p));
        }
        self.points.push(p);
    }

    fn contains(&self, q: Pos) -> bool {
        for &s in &self.points {
            if q <= s {
                return true;
            }
        }
        return false;
    }

    fn prune(&mut self, p: Pos) {
        self.points.drain_filter(|&mut s| s == p);
    }
}

/// A Contours implementation based on Contour layers with value queries in O(log(r)^2).
#[derive(Default, Debug)]
pub struct NaiveContours<C: Contour> {
    contours: Vec<C>,
}

impl<C: Contour> Contours for NaiveContours<C> {
    fn new(arrows: impl IntoIterator<Item = Arrow>) -> Self {
        let mut this = NaiveContours {
            contours: vec![C::default()],
        };
        this.contours[0].push(Pos(usize::MAX, usize::MAX));
        for a in arrows {
            let mut v = this.value(a.end) + a.len;
            if this.contours.len() <= v {
                this.contours.resize_with(v + 1, || C::default());
            }
            loop {
                this.contours[v].push(a.start);
                v -= 1;
                // Make sure this position is also contained in all lower
                // contours, to preserve the binary search.
                if this.contours[v].contains(a.start) {
                    break;
                }
            }
        }
        this
    }

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

    fn prune(&mut self, _p: Pos) {
        todo!();
    }
}

/// A bruteforce Contours implementation answering queries in O(r), and pruning
/// in O(r^2) by rebuilding the entire datastructure.
#[derive(Default, Debug)]
pub struct BruteforceContours {
    valued_arrows: Vec<(Arrow, usize)>,
}

impl Contours for BruteforceContours {
    fn new(arrows: impl IntoIterator<Item = Arrow>) -> Self {
        let mut this = BruteforceContours {
            valued_arrows: Vec::default(),
        };
        for arrow in arrows {
            let val = this.value(arrow.end) + arrow.len;
            this.valued_arrows.push((arrow, val));
        }
        this
    }

    fn value(&self, q: Pos) -> usize {
        self.valued_arrows
            .iter()
            .filter(|(arrow, _)| q <= arrow.start)
            .map(|(_arrow, value)| *value)
            .max()
            .unwrap_or(0)
    }

    fn prune(&mut self, pos: Pos) {
        self.valued_arrows
            .drain_filter(|(a, _)| a.start != pos && a.end != pos);
        self.valued_arrows = Self::new(mem::take(&mut self.valued_arrows).into_iter().filter_map(
            |(a, _)| {
                if a.start != pos && a.end != pos {
                    Some(a)
                } else {
                    None
                }
            },
        ))
        .valued_arrows;
    }
}
