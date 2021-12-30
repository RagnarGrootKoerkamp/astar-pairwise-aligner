//! Pos: Position identifier.
//! Node: contains a position, and possible additional data.
//! Edge: (Pos, Node, Cost)

use std::hash;

use crate::diagonal_map::DiagonalMapTrait;

pub trait PosOrder {
    type Output: Ord;
    fn key(pos: Self) -> Self::Output;
}

/// A Node in a graph.
/// Nodes can carry extra data T for incremental heuristic computation, which is ignored for their identity.
//TODO: Make these members private.
#[derive(Copy, Clone, Debug)]
pub struct Node<Pos: Copy, Data: Copy>(pub Pos, pub Data);
pub type NodeG<G> = crate::graph::Node<<G as ImplicitGraph>::Pos, <G as ImplicitGraph>::Data>;

impl<Pos: Copy, Data: Copy> Node<Pos, Data> {
    pub fn to_pos(&self) -> Pos {
        self.0
    }
    pub fn data(&self) -> &Data {
        &self.1
    }
}

impl<Pos: Copy + Eq, Data: Copy> PartialEq for Node<Pos, Data> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<Pos: Copy + Eq, Data: Copy> Eq for Node<Pos, Data> {}
impl<Pos: Copy + hash::Hash, Data: Copy> hash::Hash for Node<Pos, Data> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl<Pos: PosOrder + Eq + Copy, Data: Copy> PartialOrd for Node<Pos, Data> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
// Order nodes by position on increasing x.
impl<Pos: PosOrder + Eq + Copy, Data: Copy> Ord for Node<Pos, Data> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        PosOrder::key(self.0).cmp(&PosOrder::key(other.0))
    }
}

pub trait ImplicitGraph {
    type Pos: Copy + Eq + hash::Hash + PosOrder;
    type Data: Copy;
    type DiagonalMap<T>: DiagonalMapTrait<Self::Pos, T>;

    fn root(&self) -> Self::Pos;
    fn target(&self) -> Self::Pos;

    fn is_match(&self, _u: NodeG<Self>) -> Option<NodeG<Self>> {
        None
    }

    fn iterate_outgoing_edges<F>(&self, u: NodeG<Self>, f: F)
    where
        F: FnMut(NodeG<Self>, usize),
        Self: Sized;
}
