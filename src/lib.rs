#![feature(
    test,
    iter_intersperse,
    derive_default_enum,
    min_specialization,
    is_sorted,
    exclusive_range_pattern,
    associated_type_defaults,
    generic_associated_types,
    hash_drain_filter,
    drain_filter
)]

pub mod algorithms;
pub mod align;
pub mod alignment_graph;
pub mod astar;
pub mod config;
pub mod contour;
pub mod datastructures;
pub mod generate;
pub mod heuristic;
pub mod matches;

pub mod prelude {
    pub use bio_types::sequence::Sequence;
    pub use std::marker::PhantomData;

    pub use rustc_hash::FxHashMap as HashMap;
    pub use rustc_hash::FxHashSet as HashSet;

    pub use config::*;

    pub use super::*;
    pub use crate::algorithms::*;
    pub use crate::align::*;
    pub use crate::alignment_graph::*;
    pub use crate::contour::*;
    pub use crate::datastructures::*;
    pub use crate::generate::*;
    pub use crate::heuristic::*;
    pub use crate::matches::{LengthConfig, LengthConfig::Fixed, Match, MatchConfig};
    pub use bio::alphabets::{Alphabet, RankTransform};
    pub use bio::data_structures::qgram_index::QGramIndex;
    pub use std::cmp::{max, min};

    pub fn to_string(seq: &[u8]) -> String {
        String::from_utf8(seq.to_vec()).unwrap()
    }
}
