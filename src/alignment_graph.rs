//! Types related to the pairwise alignment graph.
use std::{
    cmp::max,
    fmt::{Debug, Display},
    ops::{Add, Sub},
};

use std::cmp::Ordering;

use crate::{
    aligners::Seq,
    prelude::{Cost, DtPos},
};

/// Type for positions in a sequence, and derived quantities.
pub type I = u32;
/// Type for the cost of a single match/mutation.
pub type MatchCost = u8;

/// A position in a pairwise matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Pos(pub I, pub I);

impl Add for Pos {
    type Output = Pos;

    fn add(self, Pos(x, y): Self) -> Self::Output {
        Pos(self.0 + x, self.1 + y)
    }
}

impl Sub for Pos {
    type Output = Pos;

    fn sub(self, Pos(x, y): Self) -> Self::Output {
        Pos(self.0 - x, self.1 - y)
    }
}

impl Pos {
    pub fn root() -> Self {
        Pos(0, 0)
    }

    pub fn from_lengths(a: Seq, b: Seq) -> Self {
        Pos(a.len() as I, b.len() as I)
    }

    pub fn mirror(&self) -> Pos {
        Pos(self.1, self.0)
    }

    pub fn from<T>(i: T, j: T) -> Self
    where
        T: TryInto<I>,
        <T as TryInto<u32>>::Error: Debug,
    {
        Pos(i.try_into().unwrap(), j.try_into().unwrap())
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

    pub fn forward(&self, &Pos(i, j): &Pos) -> Option<Pos> {
        Some(match self {
            Edge::None => None?,
            Edge::Match => Pos(i + 1, j + 1),
            Edge::Substitution => Pos(i + 1, j + 1),
            Edge::Right => Pos(i + 1, j),
            Edge::Down => Pos(i, j + 1),
        })
    }

    pub fn dt_forward(&self, &DtPos { diagonal, g }: &DtPos) -> Option<DtPos> {
        Some(match self {
            Edge::None => None?,
            Edge::Match => DtPos { diagonal, g },
            Edge::Substitution => DtPos { diagonal, g: g + 1 },
            Edge::Right => DtPos {
                diagonal: diagonal + 1,
                g: g + 1,
            },
            Edge::Down => DtPos {
                diagonal: diagonal - 1,
                g: g + 1,
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
    a: Seq<'a>,
    b: Seq<'a>,
    target: Pos,
    pub greedy_matching: bool,
}

impl<'a> EditGraph<'a> {
    pub fn new(a: Seq<'a>, b: Seq<'a>, greedy_matching: bool) -> EditGraph<'a> {
        EditGraph {
            a,
            b,
            target: Pos::from_lengths(a, b),
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
