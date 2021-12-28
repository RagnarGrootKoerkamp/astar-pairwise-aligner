use std::{hash, iter::Empty, ops::Deref};

use petgraph::visit::{
    Data, EdgeRef, GraphBase, IntoEdgeReferences, IntoEdges, IntoEdgesDirected, IntoNeighbors,
    IntoNeighborsDirected,
};

/// Source, Target, Cost
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge<Node>(pub Node, pub Node, pub usize);

pub trait ImplicitGraphBase {
    type Node: Copy + Eq + hash::Hash;

    type Edges: Iterator<Item = Edge<Self::Node>>;

    fn iterate_edges_directed<F>(&self, u: Self::Node, dir: petgraph::EdgeDirection, f: F)
    where
        F: FnMut(Edge<Self::Node>);

    fn edges_directed(&self, a: Self::Node, dir: petgraph::EdgeDirection) -> Self::Edges;
}
pub struct ImplicitGraph<G: ImplicitGraphBase>(G);

impl<G: ImplicitGraphBase> ImplicitGraph<G> {
    pub fn new(g: G) -> ImplicitGraph<G> {
        ImplicitGraph(g)
    }
}

impl<G: ImplicitGraphBase> Deref for ImplicitGraph<G> {
    type Target = G;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO: inline?
impl<Node: Copy> EdgeRef for Edge<Node> {
    type NodeId = Node;
    type EdgeId = ();
    type Weight = ();
    #[inline]
    fn source(&self) -> Self::NodeId {
        self.0
    }
    #[inline]
    fn target(&self) -> Self::NodeId {
        self.1
    }
    #[inline]
    fn weight(&self) -> &Self::Weight {
        &()
    }
    fn id(&self) -> Self::EdgeId {
        unimplemented!("Implicit edges do not have an ID");
    }
}

impl<G: ImplicitGraphBase> GraphBase for ImplicitGraph<G> {
    type NodeId = G::Node;
    type EdgeId = ();
}
impl<G: ImplicitGraphBase> Data for ImplicitGraph<G> {
    type NodeWeight = ();
    type EdgeWeight = ();
}
impl<'a, G: ImplicitGraphBase> IntoEdgeReferences for &'a ImplicitGraph<G> {
    type EdgeRef = Edge<G::Node>;
    type EdgeReferences = Empty<Self::EdgeRef>;
    fn edge_references(self) -> Self::EdgeReferences {
        unimplemented!("We do not list all edges for an implicit graph");
    }
}
impl<'a, G: ImplicitGraphBase> IntoNeighbors for &'a ImplicitGraph<G> {
    type Neighbors = Empty<G::Node>;
    fn neighbors(self: Self, _: Self::NodeId) -> Self::Neighbors {
        unimplemented!("Calls should be made to edges(node) instead.");
    }
}
impl<'a, G: ImplicitGraphBase> IntoNeighborsDirected for &'a ImplicitGraph<G> {
    type NeighborsDirected = Empty<G::Node>;

    fn neighbors_directed(
        self,
        _n: Self::NodeId,
        _d: petgraph::EdgeDirection,
    ) -> Self::NeighborsDirected {
        unimplemented!("Calls should be made to edges_directed(node) instead.");
    }
}

impl<'a, G: ImplicitGraphBase> IntoEdges for &'a ImplicitGraph<G> {
    type Edges = G::Edges;

    #[inline]
    fn edges(self, a: Self::NodeId) -> Self::Edges {
        self.0.edges_directed(a, petgraph::EdgeDirection::Outgoing)
    }
}

impl<'a, G: ImplicitGraphBase> IntoEdgesDirected for &'a ImplicitGraph<G> {
    type EdgesDirected = G::Edges;

    #[inline]
    fn edges_directed(self, a: Self::NodeId, dir: petgraph::EdgeDirection) -> Self::EdgesDirected {
        self.0.edges_directed(a, dir)
    }
}

pub trait IterateEdgesDirected: GraphBase + IntoEdgeReferences {
    fn iterate_edges_directed<F>(&self, u: Self::NodeId, dir: petgraph::EdgeDirection, f: F)
    where
        F: FnMut(Self::EdgeRef);
}

impl<'a, G: ImplicitGraphBase> IterateEdgesDirected for &'a ImplicitGraph<G> {
    #[inline]
    fn iterate_edges_directed<F>(&self, u: Self::NodeId, dir: petgraph::EdgeDirection, f: F)
    where
        F: FnMut(Self::EdgeRef),
    {
        self.0.iterate_edges_directed(u, dir, f);
    }
}
