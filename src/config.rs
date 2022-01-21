//! This module contains constants used throughout the code, that may eventually
//! be turned into configurable options.

use std::sync::atomic::AtomicBool;

#[cfg(debug_assertions)]
pub const DEBUG: bool = true;

#[cfg(not(debug_assertions))]
pub const DEBUG: bool = false;

/// If true: insert 'shadow' points in contours to make sure that contour v always contains contour v+1.
/// This makes contour[v].query(p) monotone in v, allowing simpler binary search/query operations.
pub const USE_SHADOW_POINTS: bool = true;

/// When true, indels are not considered when there is a match edge from the current position.
pub const GREEDY_EDGE_MATCHING: bool = true;

/// When true, do greedy matching inside the A* itself.
pub const GREEDY_EDGE_MATCHING_IN_ASTAR: bool = true;

/// Whenever A* pops a position, if the value of h and f is outdated, the point is pushed and not expanded.
pub const RETRY_OUDATED_HEURISTIC_VALUE: bool = true;

/// Whether printing is enabled.
pub static PRINT: AtomicBool = AtomicBool::new(false);

pub fn print() -> bool {
    PRINT.load(std::sync::atomic::Ordering::Relaxed)
}

/// When the priority queue moves to the next value, it can sort all points and
/// start with those closer to the target.
pub const SORT_QUEUE_ELEMENTS: bool = false;

/// Enables some assumptions that make code faster that should be fine, but are just safer to turn off by default.
pub const FAST_ASSUMPTIONS: bool = false;

/// Use a Trie instead of QGramIndex to find matches.
pub const USE_TRIE_TO_FIND_MATCHES: bool = true;
