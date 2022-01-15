//! Pos: Position identifier.
//! Node: contains a position, and possible additional data.
//! Edge: (Pos, Node, Cost)

use std::hash;

use crate::diagonal_map::DiagonalMapTrait;

use super::Cost;

pub trait PosOrder {
    type Output: Ord;
    fn key(&self) -> Self::Output;
}

pub trait ParentTrait<Pos>: Default + Clone + Copy {
    fn parent(&self, _pos: &Pos) -> Option<Pos>;
}

pub trait ImplicitGraph {
    type Pos: Copy + Eq + hash::Hash + PosOrder;
    type Parent: ParentTrait<Self::Pos>;
    type DiagonalMap<T: Default>: DiagonalMapTrait<Self::Pos, T>;

    fn root(&self) -> Self::Pos;
    fn target(&self) -> Self::Pos;

    fn is_match(&self, _u: Self::Pos) -> Option<Self::Pos> {
        None
    }

    fn iterate_outgoing_edges<F>(&self, u: Self::Pos, f: F)
    where
        F: FnMut(Self::Pos, Cost, Self::Parent),
        Self: Sized;
}
