//! This module contains implementations of other alignment algorithms.

use self::cigar::Cigar;
use crate::prelude::{Cost, Pos};
pub use pa_types::*;

pub mod astar;

#[cfg(test)]
mod tests;

pub trait StateT: std::fmt::Debug {
    fn is_root(&self) -> bool;
    fn pos(&self) -> Pos;
}

/// An aligner is a type that supports aligning sequences using some algorithm.
///
/// The output can be:
/// - cost only
/// - cost and alignment
///
/// Note that insertions are when `b` has more characters than `a`, and deletions are when `b` has less characters than `a`.
pub trait Aligner: std::fmt::Debug {
    /// Return the cost of aligning `a` and `b`.
    /// This may use less memory than `align` for some aligners.
    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        self.align(a, b).0
    }

    /// Return the cost and an alignment of `a` and `b`.
    fn align(&mut self, _a: Seq, _b: Seq) -> (Cost, Cigar) {
        unimplemented!("This aligner does not support returning an alignment.");
    }

    /// Finds the cost of aligning `a` and `b`, assuming the cost of the alignment is at most `f_max`.
    /// The returned cost may be `None` in case aligning with cost at most `s` is not possible.
    /// The returned cost may be larger than `s` when a path was found, even
    /// though this may not be the optimal cost.
    ///
    /// When `_f_max` is `None`, there is no upper bound, and this is the same as simply `cost`.
    fn cost_for_bounded_dist(&mut self, _a: Seq, _b: Seq, _f_max: Cost) -> Option<Cost> {
        unimplemented!("This aligner does not support aligning with a bounded distance.");
    }

    /// Finds an alignments (path/Cigar) of sequences `a` and `b`, assuming the
    /// cost of the alignment is at most s.
    /// The returned cost may be `None` in case aligning with cost at most `s` is not possible.
    /// The returned cost may be larger than `s` when a path was found, even
    /// though this may not be the optimal cost.
    ///
    /// When `_f_max` is `None`, there is no upper bound, and this is the same as simply `align`.
    fn align_for_bounded_dist(&mut self, _a: Seq, _b: Seq, _f_max: Cost) -> Option<(Cost, Cigar)> {
        unimplemented!("This aligner does not support aligning with a bounded distance.");
    }
}
