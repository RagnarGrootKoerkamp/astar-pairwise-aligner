use std::{cell::RefCell, fmt::Debug, hash, iter::once};

use crate::{heuristic::HeuristicInstance, util::*};
use arrayvec::ArrayVec;
use bio_types::sequence::Sequence;

use crate::implicit_graph::{Edge, ImplicitGraph, ImplicitGraphBase};

/// AlignmentGraph that computes the heuristic on the fly.
pub struct AlignmentGraphBase<'a> {
    a: &'a Sequence,
    b: &'a Sequence,
}

impl<'a> Clone for AlignmentGraphBase<'a> {
    fn clone(&self) -> Self {
        Self {
            a: self.a,
            b: self.b,
        }
    }
}

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

        // TODO: Compare between:
        // - always walk 1 step any direction.
        // - in case of match, only walk 1 step diagonal.
        // - in case of match, only walk as far on diagonal as possible.

        let nbs: ArrayVec<Edge<Self::Node>, 3> =
            if false && i + 1 <= self.a.len() && j + 1 <= self.b.len() && self.a[i] == self.b[j] {
                // Walk multiple steps at once.
                let mut x = i + 1;
                let mut y = j + 1;
                while x + 1 <= self.a.len() && y + 1 <= self.b.len() && self.a[x] == self.b[y] {
                    x += 1;
                    y += 1;
                }
                let pos = Pos(x, y);

                // TODO: Update for reverse edges.
                todo!();
                once(Edge(u, pos, 0)).collect()
            } else {
                DELTAS
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
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0 .0, self.0 .1).cmp(&(other.0 .0, other.0 .1))
    }
}
impl<T: Debug> PartialOrd for Node<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// AlignmentGraph that computes the heuristic on the fly.
pub struct IncrementalAlignmentGraphBase<'a, 'b, 'c, H: HeuristicInstance<'a>> {
    graph: &'c AlignmentGraph<'a>,
    heuristic: &'b RefCell<H>,
}

pub type IncrementalAlignmentGraph<'a, 'b, 'c, H> =
    ImplicitGraph<IncrementalAlignmentGraphBase<'a, 'b, 'c, H>>;

impl<'a, 'b, 'c, H: HeuristicInstance<'a>> ImplicitGraphBase
    for IncrementalAlignmentGraphBase<'a, 'b, 'c, H>
{
    // A node directly contains the estimated distance to the end.
    type Node = Node<H::IncrementalState>;

    type Edges = arrayvec::IntoIter<Edge<Self::Node>, 3>;

    fn edges_directed(
        &self,
        u @ Node(Pos(i, j), _): Self::Node,
        dir: petgraph::EdgeDirection,
    ) -> arrayvec::IntoIter<Edge<Self::Node>, 3> {
        const DELTAS: [(usize, usize); 3] = [(1, 1), (1, 0), (0, 1)];

        // TODO: Compare between:
        // - always walk 1 step any direction.
        // - in case of match, only walk 1 step diagonal.
        // - in case of match, only walk as far on diagonal as possible.

        let nbs: ArrayVec<Edge<Self::Node>, 3> = if false
            && i + 1 <= self.graph.a.len()
            && j + 1 <= self.graph.b.len()
            && self.graph.a[i] == self.graph.b[j]
        {
            // Walk multiple steps at once.
            let mut x = i + 1;
            let mut y = j + 1;
            while x + 1 <= self.graph.a.len()
                && y + 1 <= self.graph.b.len()
                && self.graph.a[x] == self.graph.b[y]
            {
                x += 1;
                y += 1;
            }
            let pos = Pos(x, y);

            // TODO: Update for reverse edges.
            once(Edge(
                u,
                Node(pos, self.heuristic.borrow().incremental_h(u, pos)),
                0,
            ))
            .collect()
        } else {
            DELTAS
                .iter()
                .filter_map(|&(di, dj)| match dir {
                    petgraph::EdgeDirection::Outgoing => {
                        if i + di <= self.graph.a.len() && j + dj <= self.graph.b.len() {
                            let pos = Pos(i + di, j + dj);
                            Some(Edge(
                                u,
                                Node(pos, self.heuristic.borrow().incremental_h(u, pos)),
                                if (di, dj) == (1, 1) && self.graph.a[i] == self.graph.b[j] {
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
                                Node(pos, self.heuristic.borrow().incremental_h(u, pos)),
                                u,
                                if (di, dj) == (1, 1)
                                    && self.graph.a[i - di] == self.graph.b[j - dj]
                                {
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

pub fn new_alignment_graph<'a>(a: &'a Sequence, b: &'a Sequence) -> AlignmentGraph<'a> {
    ImplicitGraph::new(AlignmentGraphBase { a, b })
}

pub fn new_incremental_alignment_graph<'a, 'b, 'c, H: HeuristicInstance<'a>>(
    graph: &'c AlignmentGraph<'a>,
    heuristic: &'b RefCell<H>,
) -> IncrementalAlignmentGraph<'a, 'b, 'c, H> {
    ImplicitGraph::new(IncrementalAlignmentGraphBase { graph, heuristic })
}

impl From<(Pos, ())> for Pos {
    fn from((a, _): (Pos, ())) -> Self {
        a
    }
}
