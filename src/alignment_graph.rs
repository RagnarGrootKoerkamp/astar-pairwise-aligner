//! Types related to the pairwise alignment graph.
use std::fmt::{Debug, Display};

use std::cmp::Ordering;

use pa_types::*;

/// Type for the cost of a single match/mutation.
pub type MatchCost = u8;

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

    pub fn dt_back(&self, &DtPos { diagonal, g }: &DtPos) -> Option<DtPos> {
        Some(match self {
            Edge::None => None?,
            Edge::Match => DtPos { diagonal, g },
            Edge::Substitution => DtPos {
                diagonal,
                g: g.checked_sub(1)?,
            },
            Edge::Right => DtPos {
                diagonal: diagonal - 1,
                g: g.checked_sub(1)?,
            },
            Edge::Down => DtPos {
                diagonal: diagonal + 1,
                g: g.checked_sub(1)?,
            },
        })
    }

    pub fn cost(&self) -> Cost {
        match self {
            Edge::Match => 0,
            Edge::None => panic!("Cost of None!"),
            _ => 1,
        }
    }

    pub fn to_f(&self) -> Cost {
        match self {
            Edge::None => 0,
            Edge::Down => 0,
            _ => 1,
        }
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

/// AlignmentGraph, modelling the position and transitions in a pairwise matching graph.
#[derive(Clone)]
pub struct EditGraph<'a> {
    pub a: Seq<'a>,
    pub b: Seq<'a>,
    pub target: Pos,
    pub greedy_matching: bool,
}

impl<'a> EditGraph<'a> {
    pub fn new(a: Seq<'a>, b: Seq<'a>, greedy_matching: bool) -> EditGraph<'a> {
        EditGraph {
            a,
            b,
            target: Pos::target(a, b),
            greedy_matching,
        }
    }
}

impl<'a> EditGraph<'a> {
    #[allow(unused)]
    #[inline]
    pub fn start(&self) -> Pos {
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

    /// Counts the number of matching characters starting at the given position.
    #[allow(unused)]
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

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct DtPos {
    pub diagonal: i32,
    pub g: Cost,
}

impl Display for DtPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Debug>::fmt(self, f)
    }
}

impl DtPos {
    pub fn from_pos(Pos(i, j): Pos, g: Cost) -> Self {
        Self {
            diagonal: i as i32 - j as i32,
            g,
        }
    }
    pub fn to_pos(self, fr: I) -> Pos {
        Pos(
            (fr as i32 + self.diagonal) as I / 2,
            (fr as i32 - self.diagonal) as I / 2,
        )
    }

    pub fn fr(Pos(i, j): Pos) -> I {
        i + j
    }
}
