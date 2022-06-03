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

    pub fn mirror(&self) -> Pos {
        Pos(self.1, self.0)
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    // The root, or an unvisited state.
    #[default]
    None,
    Match,
    Substitution,
    /// Deletion
    Right,
    /// Insertion
    Down,
}

impl Edge {
    pub fn back(&self, &Pos(i, j): &Pos) -> Option<Pos> {
        Some(match self {
            Edge::None => None?,
            Edge::Match => Pos(i.checked_sub(1)?, j.checked_sub(1)?),
            Edge::Substitution => Pos(i.checked_sub(1)?, j.checked_sub(1)?),
            Edge::Right => Pos(i.checked_sub(1)?, j),
            Edge::Down => Pos(i, j.checked_sub(1)?),
        })
    }

    pub fn forward(&self, &Pos(i, j): &Pos) -> Option<Pos> {
        Some(match self {
            Edge::None => None?,
            Edge::Match => Pos(i + 1, j + 1),
            Edge::Substitution => Pos(i + 1, j + 1),
            Edge::Right => Pos(i + 1, j),
            Edge::Down => Pos(i, j + 1),
        })
    }

    pub fn cost(&self) -> Cost {
        match self {
            Edge::Match => 0,
            Edge::None => panic!("Cost of None!"),
            _ => 1,
        }
    }

    pub fn match_cost(&self) -> MatchCost {
        self.cost() as MatchCost
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
pub struct EditGraph<'a> {
    a: &'a Sequence,
    b: &'a Sequence,
    target: Pos,
    pub greedy_matching: bool,
}

impl<'a> EditGraph<'a> {
    pub fn new(a: &'a Sequence, b: &'a Sequence, greedy_matching: bool) -> EditGraph<'a> {
        EditGraph {
            a,
            b,
            target: Pos::from_length(a, b),
            greedy_matching,
        }
    }
}

impl<'a> EditGraph<'a> {
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
        if self.a.get(i as usize)? == self.b.get(j as usize)? {
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
        F: FnMut(Pos, Edge),
    {
        // Take any of the 3 edges, and then walk as much diagonally as possible.
        let is_match = self.is_match(n);
        if self.greedy_matching {
            if let Some(n) = is_match {
                f(n, Edge::Match);
                return;
            }
        }
        for (di, dj, parent) in [
            (1, 0, Edge::Right),
            (0, 1, Edge::Down),
            // This edge is last, so that the LIFO behaviour of the priority
            // queue picks up diagonal edges first.
            if is_match.is_some() {
                (1, 1, Edge::Match)
            } else {
                (1, 1, Edge::Substitution)
            },
        ] {
            let pos = Pos(i + di, j + dj);
            if pos <= self.target {
                f(pos, parent)
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
