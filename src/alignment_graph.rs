use std::iter::once;

use crate::{heuristic::Heuristic, util::*};
use arrayvec::ArrayVec;
use bio_types::sequence::Sequence;

use crate::implicit_graph::{Edge, ImplicitGraph, ImplicitGraphBase};

/// AlignmentGraph that computes the heuristic
#[derive(Clone, Copy)]
pub struct AlignmentGraphBase<'a, H: Heuristic> {
    pattern: &'a Sequence,
    text: &'a Sequence,
    heuristic: &'a H,
}

pub type AlignmentGraph<'a, H> = ImplicitGraph<AlignmentGraphBase<'a, H>>;

impl<'a, H: Heuristic> ImplicitGraphBase for AlignmentGraphBase<'a, H> {
    // A node directly contains the estimated distance to the end.
    type Node = (Pos, H::IncrementalState);

    type Edges = arrayvec::IntoIter<Edge<Self::Node>, 3>;

    fn edges(&self, u @ (Pos(i, j), _): Self::Node) -> arrayvec::IntoIter<Edge<Self::Node>, 3> {
        const DELTAS: [(usize, usize); 3] = [(0, 1), (1, 0), (1, 1)];
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
            once(Edge(u, (pos, self.heuristic.incremental_h(u, pos)))).collect()
        } else {
            DELTAS
                .iter()
                .filter_map(|(di, dj)| {
                    if i + di <= self.pattern.len() && j + dj <= self.text.len() {
                        let pos = Pos(i + di, j + dj);
                        Some(Edge(u, (pos, self.heuristic.incremental_h(u, pos))))
                    } else {
                        None
                    }
                })
                .collect()
        };
        nbs.into_iter()
    }
}

pub fn new_alignment_graph<'a, H: Heuristic>(
    pattern: &'a Sequence,
    text: &'a Sequence,
    heuristic: &'a H,
) -> AlignmentGraph<'a, H> {
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
