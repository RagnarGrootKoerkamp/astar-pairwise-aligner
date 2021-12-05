use std::{collections::HashSet, iter};

use arrayvec::ArrayVec;
use bio_types::sequence::Sequence;
use petgraph::visit::{
    Data, EdgeRef, GraphBase, GraphRef, IntoEdgeReferences, IntoEdges, IntoNeighbors, Visitable,
};

// Implit alignmentgraph implementation.
pub(crate) struct AlignmentGraph<'a> {
    pattern: &'a Sequence,
    text: &'a Sequence,
}

// Types representing a node (match position) and edge (joining two neighbouring
// nodes) in the implicit alignment graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos<T = usize>(pub T, pub T);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge(pub Pos, pub Pos);

impl EdgeRef for Edge {
    type NodeId = Pos;
    type EdgeId = (Pos, Pos);
    type Weight = ();
    fn source(&self) -> Self::NodeId {
        self.0
    }
    fn target(&self) -> Self::NodeId {
        self.1
    }
    fn weight(&self) -> &Self::Weight {
        &()
    }
    fn id(&self) -> Self::EdgeId {
        unimplemented!("Implicit edges do not have an ID");
    }
}
impl GraphBase for AlignmentGraph<'_> {
    type NodeId = Pos;
    type EdgeId = (Pos, Pos);
}
impl Clone for AlignmentGraph<'_> {
    fn clone(&self) -> Self {
        Self {
            pattern: &self.pattern,
            text: &self.text,
        }
    }
}
impl Copy for AlignmentGraph<'_> {}
impl Data for AlignmentGraph<'_> {
    type NodeWeight = ();

    type EdgeWeight = ();
}
impl GraphRef for AlignmentGraph<'_> {}
impl IntoEdgeReferences for AlignmentGraph<'_> {
    type EdgeRef = Edge;
    type EdgeReferences = std::vec::IntoIter<Edge>;
    fn edge_references(self) -> Self::EdgeReferences {
        unimplemented!("We do not list all edges for an implicit graph");
    }
}
impl IntoNeighbors for AlignmentGraph<'_> {
    type Neighbors = std::vec::IntoIter<Pos>;
    fn neighbors(self: Self, _: Self::NodeId) -> Self::Neighbors {
        unimplemented!("Calls should be made to edges(node) instead.");
    }
}
impl IntoEdges for AlignmentGraph<'_> {
    type Edges = arrayvec::IntoIter<Edge, 3>;

    fn edges(self, u @ Pos(i, j): Self::NodeId) -> arrayvec::IntoIter<Edge, 3> {
        const DELTAS: [(usize, usize); 3] = [(0, 1), (1, 0), (1, 1)];
        let nbs: ArrayVec<Edge, 3> = if false
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
            iter::once(Edge(u, Pos(x, y))).collect()
        } else {
            DELTAS
                .iter()
                .filter_map(|(di, dj)| {
                    if i + di <= self.pattern.len() && j + dj <= self.text.len() {
                        Some(Edge(u, Pos(i + di, j + dj)))
                    } else {
                        None
                    }
                })
                .collect()
        };
        nbs.into_iter()
    }
}
impl AlignmentGraph<'_> {
    // Two binary sequences.
    pub fn new<'a>(pattern: &'a Sequence, text: &'a Sequence) -> AlignmentGraph<'a> {
        AlignmentGraph { pattern, text }
    }
}
impl Visitable for AlignmentGraph<'_> {
    type Map = HashSet<Self::NodeId>;
    fn visit_map(&self) -> Self::Map {
        HashSet::new()
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
    }
}
