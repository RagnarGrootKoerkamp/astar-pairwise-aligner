use std::{cell::RefCell, fmt::Debug, hash};

use crate::{diagonal_map::ToPos, heuristic::HeuristicInstance, util::*};
use arrayvec::ArrayVec;
use bio_types::sequence::Sequence;

use crate::implicit_graph::{Edge, ImplicitGraph, ImplicitGraphBase};

/// AlignmentGraph that computes the heuristic on the fly.
#[derive(Clone)]
pub struct AlignmentGraphBase<'a> {
    a: &'a Sequence,
    b: &'a Sequence,
    target: Pos,
}

impl<'a> AlignmentGraphBase<'a> {
    /// Internal iterator to get the edges from a position.
    #[inline]
    fn iterate_edges_directed<F>(&'a self, u @ Pos(i, j): Pos, f: F)
    where
        F: FnMut((Pos, usize)),
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
        (if is_match(u) {
            &DIAGONAL_DELTAS[..]
        } else {
            &DELTAS[..]
        })
        .iter()
        .filter_map(|&(di, dj)| {
            let pos = Pos(i + di, j + dj);
            if pos <= self.target {
                Some((
                    extend_diagonally(pos),
                    if (di, dj) == (1, 1) && self.a[i] == self.b[j] {
                        0
                    } else {
                        1
                    },
                ))
            } else {
                None
            }
        })
        .for_each(f)
    }
}

pub type AlignmentGraph<'a> = ImplicitGraph<AlignmentGraphBase<'a>>;

impl<'a> ImplicitGraphBase for AlignmentGraphBase<'a> {
    // A node directly contains the estimated distance to the end.
    type Node = Pos;

    type Edges = arrayvec::IntoIter<Edge<Self::Node>, 3>;

    fn iterate_edges_directed<F>(&self, u: Self::Node, dir: petgraph::EdgeDirection, mut f: F)
    where
        F: FnMut(Edge<Self::Node>),
    {
        assert!(dir == petgraph::EdgeDirection::Outgoing);
        self.iterate_edges_directed(u, |(v, cost)| f(Edge(u, v, cost)))
    }

    fn edges_directed(&self, u: Self::Node, dir: petgraph::EdgeDirection) -> Self::Edges {
        // We don't need incoming edges.
        // This should help the compiler.
        assert_eq!(dir, petgraph::EdgeDirection::Outgoing);

        let mut edges = ArrayVec::default();
        self.iterate_edges_directed(u, |(v, cost)| edges.push(Edge(u, v, cost)));
        edges.into_iter()
    }
}

// * INCREMENTAL ALIGNMENT GRAPH

/// A Node in a graph.
/// Nodes can carry extra data T for incremental heuristic computation, which is ignored for their identity.
#[derive(Copy, Clone, Debug)]
pub struct Node<T: Debug>(pub Pos, pub T);

impl<T: Debug> ToPos for Node<T> {
    #[inline]
    fn to_pos(&self) -> Pos {
        self.0
    }
}

impl<T: Debug> PartialEq for Node<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: Debug> Eq for Node<T> {}

impl<T: Debug> hash::Hash for Node<T> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

// Order nodes by position on increasing x.
impl<T: Debug> Ord for Node<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0 .0, self.0 .1).cmp(&(other.0 .0, other.0 .1))
    }
}
impl<T: Debug> PartialOrd for Node<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// AlignmentGraph that computes the heuristic on the fly.
pub struct IncrementalAlignmentGraphBase<'a, 'b, H: HeuristicInstance<'a>> {
    graph: AlignmentGraph<'a>,
    heuristic: &'b RefCell<H>,
}

pub type IncrementalAlignmentGraph<'a, 'b, H> =
    ImplicitGraph<IncrementalAlignmentGraphBase<'a, 'b, H>>;

impl<'a, 'b, H: HeuristicInstance<'a>> ImplicitGraphBase
    for IncrementalAlignmentGraphBase<'a, 'b, H>
{
    // A node directly contains the estimated distance to the end.
    type Node = Node<H::IncrementalState>;

    type Edges = arrayvec::IntoIter<Edge<Self::Node>, 3>;

    fn iterate_edges_directed<F>(&self, u: Self::Node, dir: petgraph::EdgeDirection, mut f: F)
    where
        F: FnMut(Edge<Self::Node>),
    {
        assert_eq!(dir, petgraph::EdgeDirection::Outgoing);

        let h = &*self.heuristic.borrow();
        self.graph
            .iterate_edges_directed(u.to_pos(), move |(v, cost)| {
                f(Edge(u, Node(v, h.incremental_h(u, v)), cost))
            });
    }

    fn edges_directed(&self, u: Self::Node, dir: petgraph::EdgeDirection) -> Self::Edges {
        assert_eq!(dir, petgraph::EdgeDirection::Outgoing);

        let mut edges = ArrayVec::default();
        self.iterate_edges_directed(u, dir, |edge| edges.push(edge));
        edges.into_iter()
    }
}

pub fn new_alignment_graph<'a>(a: &'a Sequence, b: &'a Sequence) -> AlignmentGraph<'a> {
    ImplicitGraph::new(AlignmentGraphBase {
        a,
        b,
        target: Pos(a.len(), b.len()),
    })
}

pub fn new_incremental_alignment_graph<'a, 'b, H: HeuristicInstance<'a>>(
    a: &'a Sequence,
    b: &'a Sequence,
    heuristic: &'b RefCell<H>,
) -> IncrementalAlignmentGraph<'a, 'b, H> {
    ImplicitGraph::new(IncrementalAlignmentGraphBase {
        graph: new_alignment_graph(a, b),
        heuristic,
    })
}
