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
    fn edges_directed_map<F, T>(&'a self, u @ Pos(i, j): Pos, f: F) -> ArrayVec<T, 3>
    where
        F: Fn((Pos, usize)) -> T,
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
        .map(f)
        .collect()
    }
}

pub type AlignmentGraph<'a> = ImplicitGraph<AlignmentGraphBase<'a>>;

impl<'a> ImplicitGraphBase for AlignmentGraphBase<'a> {
    // A node directly contains the estimated distance to the end.
    type Node = Pos;

    type Edges = arrayvec::IntoIter<Edge<Self::Node>, 3>;

    fn edges_directed(&self, u: Self::Node, dir: petgraph::EdgeDirection) -> Self::Edges {
        // We don't need incoming edges.
        // This should help the compiler.
        assert_eq!(dir, petgraph::EdgeDirection::Outgoing);

        self.edges_directed_map(u, |(v, len)| Edge(u, v, len))
            .into_iter()
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

    #[inline]
    fn edges_directed(
        &self,
        u @ Node(pos, _): Self::Node,
        dir: petgraph::EdgeDirection,
    ) -> arrayvec::IntoIter<Edge<Self::Node>, 3> {
        assert_eq!(dir, petgraph::EdgeDirection::Outgoing);
        let heuristic = &*self.heuristic.borrow();
        new_alignment_graph(self.graph.a, self.graph.b)
            .edges_directed_map(pos, |(v, cost)| {
                Edge(u, Node(v, heuristic.incremental_h(u, v)), cost)
            })
            .into_iter()
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
