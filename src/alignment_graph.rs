//! Types related to the pairwise alignment graph.
use std::{
    cmp::max,
    fmt::{Debug, Display},
};

use bio_types::sequence::Sequence;
use serde::Serialize;
use std::cmp::Ordering;

/// Type for positions in a sequence, and derived quantities.
pub type I = u32;
/// Type for costs.
/// TODO: Make this a strong type.
pub type Cost = u32;
/// Type for the cost of a single match.
pub type MatchCost = u8;

/// A position in a pairwise matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Default)]
pub struct Pos(pub I, pub I);

impl Pos {
    pub fn from_length(a: &Sequence, b: &Sequence) -> Self {
        Pos(a.len() as I, b.len() as I)
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum Parent {
    // The root, or an unvisited state.
    #[default]
    None,
    Match,
    Substitution,
    Left,
    Up,
}

impl Parent {
    pub fn parent(&self, &Pos(i, j): &Pos) -> Option<Pos> {
        match self {
            Parent::None => None,
            Parent::Match => Some(Pos(i - 1, j - 1)),
            Parent::Substitution => Some(Pos(i - 1, j - 1)),
            Parent::Left => Some(Pos(i - 1, j)),
            Parent::Up => Some(Pos(i, j - 1)),
        }
    }

    pub fn match_value() -> Self {
        Parent::Match
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

/// Partial ordering by
/// (a,b) <= (c,d) when a<=c and b<=d.
/// (a,b) < (c,d) when a<=c and b<=d and a<c or b<d.
impl PartialOrd for Pos {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.0.cmp(&other.0);
        let b = self.1.cmp(&other.1);
        if a == b {
            return Some(a);
        }
        if a == Ordering::Equal {
            return Some(b);
        }
        if b == Ordering::Equal {
            return Some(a);
        }
        None
    }

    #[inline]
    fn le(&self, other: &Self) -> bool {
        self.0 <= other.0 && self.1 <= other.1
    }
}

/// Pos, but with a total lexicographic order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexPos(pub Pos);

impl PartialOrd for LexPos {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    #[inline]
    fn lt(&self, other: &Self) -> bool {
        (self.0 .0, self.0 .1) < (other.0 .0, other.0 .1)
    }
}

impl Ord for LexPos {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        (self.0 .0, self.0 .1).cmp(&(other.0 .0, other.0 .1))
    }
}

impl Pos {
    #[inline]
    pub fn key(&self) -> LexPos {
        LexPos(*self)
    }

    #[inline]
    pub fn add_diagonal(&self, step: I) -> Self {
        Pos(self.0 + step, self.1 + step)
    }

    #[inline]
    pub fn max_with(&mut self, other: &Self) {
        self.0 = max(self.0, other.0);
        self.1 = max(self.1, other.1);
    }
}

/// AlignmentGraph, modelling the position and transitions in a pairwise matching graph.
#[derive(Clone)]
pub struct AlignmentGraph<'a> {
    a: &'a Sequence,
    b: &'a Sequence,
    target: Pos,
    greedy_matching: bool,
}

impl<'a> AlignmentGraph<'a> {
    pub fn new(a: &'a Sequence, b: &'a Sequence, greedy_matching: bool) -> AlignmentGraph<'a> {
        AlignmentGraph {
            a,
            b,
            target: Pos::from_length(a, b),
            greedy_matching,
        }
    }
}

impl<'a> AlignmentGraph<'a> {
    #[inline]
    pub fn root(&self) -> Pos {
        Pos(0, 0)
    }

    #[inline]
    pub fn target(&self) -> Pos {
        self.target
    }

    #[inline]
    pub fn is_match(&self, Pos(i, j): Pos) -> Option<Pos> {
        if i < self.target.0 && j < self.target.1 && self.a[i as usize] == self.b[j as usize] {
            Some(Pos(i + 1, j + 1))
        } else {
            None
        }
    }

    #[inline]
    pub fn count_match(&self, Pos(i, j): Pos) -> usize {
        let max = std::cmp::min(self.target.0 - i, self.target.1 - j) as usize;
        let mut cnt = 0;
        while cnt < max && self.a[i as usize + cnt] == self.b[j as usize + cnt] {
            cnt += 1;
        }
        cnt
    }

    /// Internal iterator to get the edges from a position.
    #[inline]
    pub fn iterate_outgoing_edges<F>(&self, n @ Pos(i, j): Pos, mut f: F)
    where
        F: FnMut(Pos, Cost, Parent),
    {
        // Take any of the 3 edges, and then walk as much diagonally as possible.
        let is_match = self.is_match(n);
        if self.greedy_matching {
            if let Some(n) = is_match {
                f(n, 0, Parent::Match);
                return;
            }
        }
        for (di, dj, cost, parent) in [
            (1, 0, 1, Parent::Left),
            (0, 1, 1, Parent::Up),
            // This edge is last, so that the LIFO behaviour of the priority
            // queue picks up diagonal edges first.
            if is_match.is_some() {
                (1, 1, 0, Parent::Match)
            } else {
                (1, 1, 1, Parent::Substitution)
            },
        ] {
            let pos = Pos(i + di, j + dj);
            if pos <= self.target {
                f(pos, cost, parent)
            }
        }
    }
}

/// The costs to use for the distance computation.
/// TODO: Gap-Affine costs.
pub struct CostModel {
    pub mismatch: MatchCost,
    pub insertion: MatchCost,
    pub deletion: MatchCost,
}

/// Default costs for EditDistance:
/// mismatch, insertion, and deletion all cost 1.
pub const EDIT_DISTANCE_COSTS: CostModel = CostModel {
    mismatch: 1,
    insertion: 1,
    deletion: 1,
};

/// LCS corresponds to disallowing mismatches.
pub const LCS_COSTS: CostModel = CostModel {
    mismatch: MatchCost::MAX,
    insertion: 1,
    deletion: 1,
};
