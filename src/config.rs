//! This module contains constants used throughout the code, that may eventually
//! be turned into configurable options.

#[cfg(debug_assertions)]
pub const DEBUG: bool = false;

#[cfg(not(debug_assertions))]
const DEBUG: bool = false;

/// If true: insert 'shadow' points in contours to make sure that contour v always contains contour v+1.
/// This makes contour[v].query(p) monotone in v, allowing simpler binary search/query operations.
pub const USE_SHADOW_POINTS: bool = false;

/// Whenever A* pops a position, if the value of h and f is outdated, the point is pushed and not expanded.
pub const RETRY_OUDATED_HEURISTIC_VALUE: bool = true;
