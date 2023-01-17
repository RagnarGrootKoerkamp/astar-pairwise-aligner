#![feature(is_sorted, associated_type_defaults, drain_filter)]

mod config;
mod contour;
pub mod heuristic;
mod matches;
mod split_vec;

pub use heuristic::*;

mod prelude {
    pub use crate::config::*;
    pub use pa_types::*;

    pub use rustc_hash::FxHashMap as HashMap;
    pub use std::cmp::{max, min};
}
