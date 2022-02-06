//! Pos: Position identifier.
//! Node: contains a position, and possible additional data.
//! Edge: (Pos, Node, Cost)

use std::hash;

use crate::diagonal_map::DiagonalMapTrait;

use super::{Cost, I};

pub trait PosTrait:
    Copy + Eq + hash::Hash + std::fmt::Debug + PartialOrd + std::fmt::Display
{
    type Output: Ord;
    fn key(&self) -> Self::Output;
    fn add_diagonal(&self, step: I) -> Self;
    fn max_with(&mut self, other: &Self);
}

pub trait ParentTrait<Pos>: Default + Clone + Copy {
    fn parent(&self, _pos: &Pos) -> Option<Pos>;
    fn match_value() -> Self;
}

/// An implicit graph.
pub trait ImplicitGraph {
    type Pos: PosTrait;
    type Parent: ParentTrait<Self::Pos> + std::fmt::Debug;
    type DiagonalMap<T: Default + Clone + Copy>: DiagonalMapTrait<Self::Pos, T>;

    fn root(&self) -> Self::Pos;
    fn target(&self) -> Self::Pos;

    fn is_match(&self, _u: Self::Pos) -> Option<Self::Pos> {
        None
    }

    /// Count the number of matching characters starting at the given position.
    fn count_match(&self, mut u: Self::Pos) -> usize {
        let mut cnt = 0;
        while let Some(v) = self.is_match(u) {
            cnt += 1;
            u = v;
        }
        cnt
    }

    fn iterate_outgoing_edges<F>(&self, u: Self::Pos, f: F)
    where
        F: FnMut(Self::Pos, Cost, Self::Parent),
        Self: Sized;
}
