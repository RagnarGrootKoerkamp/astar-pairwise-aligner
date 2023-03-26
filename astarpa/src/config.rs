//! This module contains constants used throughout the code, that may eventually
//! be turned into configurable options.

// ========= FLAGS IN THE PAPER (default true) =========

/// Whether to use shifting of the priority queue to reduce reordering.
pub const REDUCE_REORDERING: bool = true;

// ========= FLAGS NOT IN THE PAPER (default false) =========

/// When true, states close to the tip (after the last prune) are stored
/// separately for shifting purposes.
/// This seems helpful for CSH with high error rate, but causes significant slowdown for SH.
pub const USE_TIP_BUFFER: bool = false;
