use std::{
    cell::RefCell,
    fmt::{Debug, Display},
};

use super::{implicit_graph, implicit_graph::Node, ImplicitGraph, NodeG};
use crate::{diagonal_map::DiagonalMap, heuristic::HeuristicInstance};
use bio_types::sequence::Sequence;
use serde::Serialize;
use std::cmp::Ordering;

/// A position in a pairwise matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Default)]
pub struct Pos(pub usize, pub usize);

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(&self, f)
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
}

/// Pos, but with a total lexicographic order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexPos(pub Pos);

impl PartialOrd for LexPos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

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
    greedy_matching: bool,
}

impl<'a> AlignmentGraph<'a> {
    pub fn new(a: &'a Sequence, b: &'a Sequence, greedy_matching: bool) -> AlignmentGraph<'a> {
        AlignmentGraph {
            a,
            b,
            target: Pos(a.len(), b.len()),
            greedy_matching,
        }
    }
}

impl<'a> ImplicitGraph for AlignmentGraph<'a> {
    type Pos = Pos;
    type Data = ();
    type DiagonalMap<T> = DiagonalMap<T>;

    #[inline]
    fn root(&self) -> Self::Pos {
        Pos(0, 0)
    }

    #[inline]
    fn target(&self) -> Self::Pos {
        self.target
    }

    #[inline]
    fn is_match(&self, Node(Pos(i, j), _): NodeG<Self>) -> Option<NodeG<Self>> {
        if i < self.target.0 && j < self.target.1 && self.a[i] == self.b[j] {
            Some(Node(Pos(i + 1, j + 1), ()))
        } else {
            None
        }
    }

    /// Internal iterator to get the edges from a position.
    #[inline]
    fn iterate_outgoing_edges<F>(&self, n @ Node(Pos(i, j), _): NodeG<Self>, mut f: F)
    where
        F: FnMut(NodeG<Self>, usize),
    {
        // Take any of the 3 edges, and then walk as much diagonally as possible.
        let is_match = self.is_match(n);
        if self.greedy_matching {
            if let Some(n) = is_match {
                f(n, 0);
                return;
            }
        }
        for (di, dj) in [(1, 0), (0, 1), (1, 1)] {
            let pos = Pos(i + di, j + dj);
            if pos <= self.target {
                f(
                    Node(pos, ()),
                    if is_match.is_some() && (di, dj) == (1, 1) {
                        0
                    } else {
                        1
                    },
                )
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
        greedy_matching: bool,
    ) -> IncrementalAlignmentGraph<'a, 'b, H> {
        IncrementalAlignmentGraph {
            graph: AlignmentGraph::new(a, b, greedy_matching),
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

    #[inline]
    fn root(&self) -> Self::Pos {
        self.graph.root()
    }

    #[inline]
    fn target(&self) -> Self::Pos {
        self.graph.target()
    }

    #[inline]
    fn is_match(&self, u: NodeG<Self>) -> Option<NodeG<Self>> {
        self.graph.is_match(Node(u.to_pos(), ())).map(|v| {
            Node(
                v.to_pos(),
                self.heuristic.borrow().incremental_h(u, v.to_pos(), 0),
            )
        })
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
