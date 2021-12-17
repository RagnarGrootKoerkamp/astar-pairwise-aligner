use std::{cell::RefCell, hash, iter::once};

use crate::{heuristic::HeuristicInstance, util::*};
use arrayvec::ArrayVec;
use bio_types::sequence::Sequence;

use crate::implicit_graph::{Edge, ImplicitGraph, ImplicitGraphBase};

/// AlignmentGraph that computes the heuristic
//#[derive(Clone)]
pub struct AlignmentGraphBase<'a, 'b, H: HeuristicInstance<'a>> {
    pattern: &'a Sequence,
    text: &'a Sequence,
    heuristic: &'b RefCell<H>,
}

pub type AlignmentGraph<'a, 'b, H> = ImplicitGraph<AlignmentGraphBase<'a, 'b, H>>;

// Nodes can carry extra data T, which is ignored for their identity.
#[derive(Copy, Clone)]
pub struct Node<T>(pub Pos, pub T);

impl<T> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for Node<T> {}

impl<T> hash::Hash for Node<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

// Order nodes by position on increasing x.
impl<T> Ord for Node<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0 .0, self.0 .1).cmp(&(other.0 .0, other.0 .1))
    }
}
impl<T> PartialOrd for Node<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, 'b, H: HeuristicInstance<'a>> ImplicitGraphBase for AlignmentGraphBase<'a, 'b, H> {
    // A node directly contains the estimated distance to the end.
    type Node = Node<H::IncrementalState>;

    type Edges = arrayvec::IntoIter<Edge<Self::Node>, 3>;

    fn edges(&self, u @ Node(Pos(i, j), _): Self::Node) -> arrayvec::IntoIter<Edge<Self::Node>, 3> {
        const DELTAS: [(usize, usize); 3] = [(1, 1), (1, 0), (0, 1)];
        let nbs: ArrayVec<Edge<Self::Node>, 3> = if false
            && i + 1 <= self.pattern.len()
            && j + 1 <= self.text.len()
            && self.pattern[i] == self.text[j]
        {
            let mut x = i + 1;
            let mut y = j + 1;
            while x + 1 <= self.pattern.len()
                && y + 1 <= self.text.len()
                && self.pattern[x] == self.text[y]
            {
                x += 1;
                y += 1;
            }
            let pos = Pos(x, y);
            once(Edge(
                u,
                Node(pos, self.heuristic.borrow().incremental_h(u, pos)),
            ))
            .collect()
        } else {
            DELTAS
                .iter()
                .filter_map(|(di, dj)| {
                    if i + di <= self.pattern.len() && j + dj <= self.text.len() {
                        let pos = Pos(i + di, j + dj);
                        Some(Edge(
                            u,
                            Node(pos, self.heuristic.borrow().incremental_h(u, pos)),
                        ))
                    } else {
                        None
                    }
                })
                .collect()
        };
        nbs.into_iter()
    }
}

pub fn new_alignment_graph<'a, 'b, H: HeuristicInstance<'a>>(
    pattern: &'a Sequence,
    text: &'a Sequence,
    heuristic: &'b RefCell<H>,
) -> AlignmentGraph<'a, 'b, H> {
    ImplicitGraph::new(AlignmentGraphBase {
        pattern,
        text,
        heuristic,
    })
}

impl From<(Pos, ())> for Pos {
    fn from((a, _): (Pos, ())) -> Self {
        a
    }
}
