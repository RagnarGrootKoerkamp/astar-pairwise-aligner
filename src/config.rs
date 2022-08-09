//! This module contains constants used throughout the code, that may eventually
//! be turned into configurable options.

use std::sync::atomic::AtomicBool;

/// Whether printing is enabled.
pub static PRINT: AtomicBool = AtomicBool::new(false);

pub fn print() -> bool {
    PRINT.load(std::sync::atomic::Ordering::Relaxed)
}

/// Use a Trie to find matches.
/// NOTE: Not in paper
pub const FIND_MATCHES_TRIE: bool = false;

/// Use a HashMap to find matches. Only works with exact matches and fixed k.
/// Default: Use QGramIndex to find matches.
pub const FIND_MATCHES_HASH: bool = true;

/// Whether to use shifting of the priority queue to reduce retries.
pub const REDUCE_RETRIES: bool = true;

/// Whether to use an offset array in the DiagonalMap.
/// NOTE: DiagonalMap is not in paper.
pub const DIAGONAL_MAP_OFFSET: bool = false;

/// Whether to use a sliding window approach for finding exact matches for fixed k.
/// This reduces the size of the hashmap by a factor k.
/// Only for CSH with gap-cost.
/// NOTE: Not in paper
pub const SLIDING_WINDOW_MATCHES: bool = false;

/// Whether to prune matches by start.
pub const PRUNE_MATCHES_BY_START: bool = true;

/// Whether to prune matches by end, in addition to pruning by start.
/// NOTE: Not in paper.
pub const PRUNE_MATCHES_BY_END: bool = false;

/// Given an exact match, when this is true any other inexact matches ending in the same position are also pruned.
/// NOTE: Not in paper.
pub const PRUNE_NEIGHBOURING_INEXACT_MATCHES_BY_END: bool = false;

/// Whether to check for consistency before pruning for SH and CSH.
/// NOTE: For CSH+gaps consistency is always checked.
pub const CHECK_MATCH_CONSISTENCY: bool = false;

/// When true, inexact matches with an insertion at the start/end are skipped.
/// NOTE: Not in paper.
pub const SKIP_INEXACT_INSERT_START_END: bool = false;

/// When true, states close to the tip (after the last prune) are stored
/// separately for shifting purposes.
/// This seems helpful for CSH with high error rate, but causes significant slowdown for SH.
/// NOTE: Not in paper.
pub const USE_TIP_BUFFER: bool = false;
