#![feature(
    test,
    iter_intersperse,
    let_else,
    label_break_value,
    min_specialization,
    is_sorted,
    exclusive_range_pattern,
    associated_type_defaults,
    generic_associated_types,
    hash_drain_filter,
    drain_filter
)]

pub mod align;
pub mod aligners;
pub mod alignment_graph;
pub mod astar;
pub mod astar_dt;
pub mod config;
pub mod contour;
pub mod cost_model;
pub mod datastructures;
pub mod generate;
pub mod heuristic;
pub mod matches;
pub mod runner;

pub mod prelude {
    pub use super::*;
    pub use crate::align::*;
    pub use crate::alignment_graph::*;
    pub use crate::config::*;
    pub use crate::contour::*;
    pub use crate::cost_model::*;
    pub use crate::datastructures::*;
    pub use crate::generate::*;
    pub use crate::heuristic::*;
    pub use crate::matches::{LengthConfig::Fixed, *};
    pub use crate::runner::*;
    pub use bio::alphabets::{Alphabet, RankTransform};
    pub use bio::data_structures::qgram_index::QGramIndex;
    pub use bio_types::sequence::Sequence;
    pub use rustc_hash::FxHashMap as HashMap;
    pub use rustc_hash::FxHashSet as HashSet;
    pub use std::cmp::{max, min};
    pub use std::marker::PhantomData;

    pub fn to_string(seq: &[u8]) -> String {
        String::from_utf8(seq.to_vec()).unwrap()
    }
}
