use std::{cell::RefCell, fmt::Debug, hash, ops::Deref};

use crate::{diagonal_map::ToPos, heuristic::HeuristicInstance, util::*};
use arrayvec::ArrayVec;
use bio_types::sequence::Sequence;

use crate::implicit_graph::{Edge, ImplicitGraph, ImplicitGraphBase};

/// AlignmentGraph that computes the heuristic on the fly.
#[derive(Clone)]
pub struct AlignmentGraphBase<'a> {
    a: &'a Sequence,
    b: &'a Sequence,
}

// impl<'a> Clone for AlignmentGraphBase<'a> {
//     fn clone(&self) -> Self {
//         Self {
//             a: self.a,
//             b: self.b,
//         }
//     }
// }

pub type AlignmentGraph<'a> = ImplicitGraph<AlignmentGraphBase<'a>>;

impl<'a> ImplicitGraphBase for AlignmentGraphBase<'a> {
    // A node directly contains the estimated distance to the end.
    type Node = Pos;

    type Edges = arrayvec::IntoIter<Edge<Self::Node>, 3>;

    fn edges_directed(
        &self,
        u @ Pos(i, j): Self::Node,
        dir: petgraph::EdgeDirection,
    ) -> arrayvec::IntoIter<Edge<Self::Node>, 3> {
        const DELTAS: [(usize, usize); 3] = [(1, 1), (1, 0), (0, 1)];
        const DIAGONAL_DELTAS: [(usize, usize); 1] = [(1, 1)];
        const LONG_DIAGONALS: bool = false;

        // TODO: Compare edge strategies:
        // - always walk 1 step any direction.
        // - in case of match, only walk 1 step diagonal. [current choice]
        // - in case of match, only walk as far on diagonal as possible.

        let diagonal_match = match dir {
            petgraph::EdgeDirection::Outgoing => {
                i < self.a.len() && j < self.b.len() && self.a[i] == self.b[j]
            }
            petgraph::EdgeDirection::Incoming => i > 0 && j > 0 && self.a[i - 1] == self.b[j - 1],
        };

        let nbs: ArrayVec<Edge<Self::Node>, 3> = if diagonal_match && LONG_DIAGONALS {
            // Only walk diagonally when there is

            // Walk multiple steps at once.
            let mut x = i + 1;
            let mut y = j + 1;
            while x + 1 <= self.a.len() && y + 1 <= self.b.len() && self.a[x] == self.b[y] {
                x += 1;
                y += 1;
            }
            //let pos = Pos(x, y);

            // TODO: Update for reverse edges.
            //once(Edge(u, pos, 0)).collect();
            todo!();
        } else {
            (if diagonal_match {
                &DIAGONAL_DELTAS[..]
            } else {
                &DELTAS[..]
            })
            .iter()
            .filter_map(|&(di, dj)| match dir {
                petgraph::EdgeDirection::Outgoing => {
                    if i + di <= self.a.len() && j + dj <= self.b.len() {
                        let pos = Pos(i + di, j + dj);
                        Some(Edge(
                            u,
                            pos,
                            if (di, dj) == (1, 1) && self.a[i] == self.b[j] {
                                0
                            } else {
                                1
                            },
                        ))
                    } else {
                        None
                    }
                }
                petgraph::EdgeDirection::Incoming => {
                    if di <= i && dj <= j {
                        let pos = Pos(i - di, j - dj);
                        Some(Edge(
                            pos,
                            u,
                            if (di, dj) == (1, 1) && self.a[i - di] == self.b[j - dj] {
                                0
                            } else {
                                1
                            },
                        ))
                    } else {
                        None
                    }
                }
            })
            .collect()
        };
        nbs.into_iter()
    }
}

// * INCREMENTAL ALIGNMENT GRAPH

/// A Node in a graph.
/// Nodes can carry extra data T for incremental heuristic computation, which is ignored for their identity.
#[derive(Copy, Clone, Debug)]
pub struct Node<T: Debug>(pub Pos, pub T);

impl<T: Debug> ToPos for Node<T> {
    fn to_pos(&self) -> Pos {
        self.0
    }
}

impl<T: Debug> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T: Debug> Eq for Node<T> {}

impl<T: Debug> hash::Hash for Node<T> {
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

pub fn incremental_edges<'a, R: Deref<Target = H>, H: HeuristicInstance<'a>>(
    a: &'a Sequence,
    b: &'a Sequence,
    heuristic: R,
    u @ Node(cur_pos, _): Node<H::IncrementalState>,
    dir: petgraph::EdgeDirection,
) -> arrayvec::IntoIter<Edge<Node<H::IncrementalState>>, 3> {
    let edges = new_alignment_graph(a, b).edges_directed(cur_pos, dir);
    let nbs: ArrayVec<Edge<Node<H::IncrementalState>>, 3> = match dir {
        petgraph::EdgeDirection::Outgoing => edges
            .map(|Edge(.., end, cost)| Edge(u, Node(end, (*heuristic).incremental_h(u, end)), cost))
            .collect(),
        petgraph::EdgeDirection::Incoming => edges
            .map(|Edge(start, .., cost)| {
                Edge(Node(start, (*heuristic).incremental_h(u, start)), u, cost)
            })
            .collect(),
    };
    nbs.into_iter()
}

impl<'a, 'b, H: HeuristicInstance<'a>> ImplicitGraphBase
    for IncrementalAlignmentGraphBase<'a, 'b, H>
{
    // A node directly contains the estimated distance to the end.
    type Node = Node<H::IncrementalState>;

    type Edges = arrayvec::IntoIter<Edge<Self::Node>, 3>;

    fn edges_directed(
        &self,
        u: Self::Node,
        dir: petgraph::EdgeDirection,
    ) -> arrayvec::IntoIter<Edge<Self::Node>, 3> {
        incremental_edges(self.graph.a, self.graph.b, self.heuristic.borrow(), u, dir)
    }
}

pub fn new_alignment_graph<'a>(a: &'a Sequence, b: &'a Sequence) -> AlignmentGraph<'a> {
    ImplicitGraph::new(AlignmentGraphBase { a, b })
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

impl From<(Pos, ())> for Pos {
    fn from((a, _): (Pos, ())) -> Self {
        a
    }
}
