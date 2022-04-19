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
/// This must be true for correctness.
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

/// Use a Trie to find matches.
pub const FIND_MATCHES_TRIE: bool = false;
/// Use a HashMap to find matches. Only works with exact matches and fixed k.
/// Default: Use QGramIndex to find matches.
pub const FIND_MATCHES_HASH: bool = true;

/// Whether to use shifting of the priority queue to reduce retries.
pub const REDUCE_RETRIES: bool = true;

/// Whether to use an offset array in the DiagonalMap.
pub const DIAGONAL_MAP_OFFSET: bool = false;

/// Do not save states after greedy matching.
/// Instead, quickly jump over them and redo work later if needed.
pub const DO_NOT_SAVE_GREEDY_MATCHES: bool = true;

/// Whether to use a sliding window approach for finding exact matches for fixed k.
/// This reduces the size of the hashmap by a factor k.
pub const SLIDING_WINDOW_MATCHES: bool = true;
