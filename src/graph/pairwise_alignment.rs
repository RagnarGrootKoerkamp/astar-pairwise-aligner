use std::{cell::RefCell, fmt::Debug};

use super::{implicit_graph, implicit_graph::Node, ImplicitGraph, NodeG};
use crate::{diagonal_map::DiagonalMap, heuristic::HeuristicInstance};
use bio_types::sequence::Sequence;
use serde::Serialize;
use std::cmp::Ordering;

/// A position in a pairwise matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Pos(pub usize, pub usize);

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
}

/// Pos, but with a total lexicographic order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct LexPos(Pos);

impl Ord for LexPos {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.0 .0, self.0 .1).cmp(&(other.0 .0, other.0 .1))
    }
}

impl implicit_graph::PosOrder for Pos {
    type Output = LexPos;

    fn key(pos: Self) -> Self::Output {
        LexPos(pos)
    }
}

/// AlignmentGraph, modelling the position and transitions in a pairwise matching graph.
#[derive(Clone)]
pub struct AlignmentGraph<'a> {
    a: &'a Sequence,
    b: &'a Sequence,
    target: Pos,
}

impl<'a> AlignmentGraph<'a> {
    pub fn new(a: &'a Sequence, b: &'a Sequence) -> AlignmentGraph<'a> {
        AlignmentGraph {
            a,
            b,
            target: Pos(a.len(), b.len()),
        }
    }
}

impl<'a> ImplicitGraph for AlignmentGraph<'a> {
    type Pos = Pos;
    type Data = ();
    type DiagonalMap<T> = DiagonalMap<T>;

    fn root(&self) -> Self::Pos {
        Pos(0, 0)
    }

    fn target(&self) -> Self::Pos {
        self.target
    }

    /// Internal iterator to get the edges from a position.
    #[inline]
    fn iterate_outgoing_edges<F>(&self, Node(u @ Pos(i, j), _): NodeG<Self>, mut f: F)
    where
        F: FnMut(NodeG<Self>, usize),
    {
        const DELTAS: [(usize, usize); 3] = [(1, 1), (1, 0), (0, 1)];
        const DIAGONAL_DELTAS: [(usize, usize); 1] = [(1, 1)];
        // Take any of the 3 edges, and then walk as much diagonally as possible.
        const GREEDY_AT_END: bool = false;
        let is_match = |pos @ Pos(i, j): Pos| {
            pos.0 < self.target.0 && pos.1 < self.target.1 && self.a[i] == self.b[j]
        };
        let extend_diagonally = |mut pos: Pos| -> Pos {
            if GREEDY_AT_END {
                while is_match(pos) {
                    pos = Pos(pos.0 + 1, pos.1 + 1)
                }
            }
            pos
        };
        let is_match = is_match(u);
        for &(di, dj) in if is_match {
            &DIAGONAL_DELTAS[..]
        } else {
            &DELTAS[..]
        } {
            let pos = Pos(i + di, j + dj);
            if pos <= self.target {
                let cost = if is_match { 0 } else { 1 };
                f(Node(extend_diagonally(pos), ()), cost)
            }
        }
    }
}

/// Incremental AlignmentGraph, modelling the position and transitions in a pairwise matching graph.
/// This computes h incrementally along edges.
pub struct IncrementalAlignmentGraph<'a, 'b, H: HeuristicInstance<'a>> {
    graph: AlignmentGraph<'a>,
    heuristic: &'b RefCell<H>,
}

impl<'a, 'b, H: HeuristicInstance<'a>> IncrementalAlignmentGraph<'a, 'b, H> {
    pub fn new(
        a: &'a Sequence,
        b: &'a Sequence,
        heuristic: &'b RefCell<H>,
    ) -> IncrementalAlignmentGraph<'a, 'b, H> {
        IncrementalAlignmentGraph {
            graph: AlignmentGraph::new(a, b),
            heuristic,
        }
    }
}

impl<'a, 'b, H> ImplicitGraph for IncrementalAlignmentGraph<'a, 'b, H>
where
    H: HeuristicInstance<'a, Pos = Pos>,
{
    type Pos = Pos;
    type Data = H::IncrementalState;
    type DiagonalMap<T> = DiagonalMap<T>;

    fn root(&self) -> Self::Pos {
        Pos(0, 0)
    }

    fn target(&self) -> Self::Pos {
        self.graph.target
    }

    #[inline]
    fn iterate_outgoing_edges<F>(&self, u: NodeG<Self>, mut f: F)
    where
        F: FnMut(NodeG<Self>, usize),
    {
        let h = &*self.heuristic.borrow();
        self.graph
            .iterate_outgoing_edges(Node(u.to_pos(), ()), move |v, cost| {
                f(Node(v.to_pos(), h.incremental_h(u, v.to_pos(), cost)), cost)
            });
    }
}
