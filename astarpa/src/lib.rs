#![feature(
    test,
    array_methods,
    duration_constants,
    step_trait,
    int_roundings,
    iter_intersperse,
    slice_as_chunks,
    let_chains,
    is_sorted,
    exclusive_range_pattern,
    associated_type_defaults,
    hash_drain_filter,
    drain_filter
)]

mod align;
mod alignment_graph;
mod astar;
mod astar_dt;
pub mod cli;
mod config;
pub mod contour;
pub mod heuristic;
pub mod matches;
pub mod stats;
pub mod visualizer;

mod datastructures {
    pub mod bucket_queue;
    pub mod split_vec;

    pub use bucket_queue::*;
    pub use split_vec::*;
}

pub use align::*;
pub use astar::astar;
pub use astar_dt::astar_dt;
pub use heuristic::*;
pub use visualizer::NoVis;

pub mod prelude {
    pub use pa_types::*;
    pub use rustc_hash::FxHashMap as HashMap;
    pub use rustc_hash::FxHashSet as HashSet;
    pub use std::cmp::{max, min};

    pub use crate::config::*;
}

#[cfg(test)]
mod tests;
