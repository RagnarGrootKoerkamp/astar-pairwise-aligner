#![feature(is_sorted, associated_type_defaults, drain_filter)]

#[cfg(feature = "cli")]
mod cli;
mod config;
mod contour;
pub mod heuristic;
pub mod matches;
mod split_vec;

#[cfg(feature = "cli")]
pub use cli::*;
pub use heuristic::*;
pub use matches::MatchConfig;

mod prelude {
    pub use crate::config::*;
    pub use pa_types::*;

    pub use rustc_hash::FxHashMap as HashMap;
    pub use std::cmp::{max, min};
}
