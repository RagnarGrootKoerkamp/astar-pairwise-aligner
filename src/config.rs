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

/// When true, do greedy matching inside the A* itself, to avoid redundantly pushing and popping consecutive states.
pub const GREEDY_EDGE_MATCHING_IN_ASTAR: bool = true;

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

/// Whether to use a sliding window approach for finding exact matches for fixed k.
/// This reduces the size of the hashmap by a factor k.
pub const SLIDING_WINDOW_MATCHES: bool = false;

/// Given an exact match, when this is true any other inexact matches ending in the same position are also pruned.
pub const PRUNE_NEIGHBOURING_INEXACT_MATCHES_BY_END: bool = false;

/// Whether to prune matches by end, in addition to pruning by start.
pub const PRUNE_MATCHES_BY_END: bool = false;

/// Whether to prune matches by start.
pub const PRUNE_MATCHES_BY_START: bool = true;

/// Whether to check for consistency before pruning for SH and CSH.
/// NOTE: For CSH+gaps consistency is always checked.
pub const CHECK_MATCH_CONSISTENCY: bool = false;

/// When true, inexact matches with an insertion at the start/end are skipped.
/// TODO: This is not yet in the paper.
pub const SKIP_INEXACT_INSERT_START_END: bool = false;
