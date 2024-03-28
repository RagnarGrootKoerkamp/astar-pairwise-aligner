#![feature(
    is_sorted,
    associated_type_defaults,
    int_roundings,
    let_chains,
    portable_simd,
    never_type
)]

mod cli;
mod config;
mod contour;
// FIXME: MAKE MOST MODULES PRIVATE
// SEEDS AND MATCHES DO NOT NEED TO BE EXPOSED.
pub mod heuristic;
pub mod matches;
pub mod prune;
pub mod seeds;
mod split_vec;
pub mod util;

pub use cli::*;
pub use heuristic::*;
pub use matches::{LengthConfig, MatchConfig};
pub use prune::{Prune, Pruning};
pub use seeds::MatchCost;

mod prelude {
    pub use crate::config::*;
    pub use pa_types::*;

    pub use rustc_hash::FxHashMap as HashMap;
    pub use std::cmp::{max, min};
}

const PRINT: bool = false;
