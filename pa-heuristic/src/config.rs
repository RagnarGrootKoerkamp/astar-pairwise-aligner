//! This module contains constants used throughout the code, that may eventually
//! be turned into configurable options.

// ========= FLAGS IN THE PAPER (default true) =========

/// Use a HashMap to find matches. Only works with exact matches and fixed k.
/// Default: true, use a hashmap.
/// When false, use a qgramindex.
pub const FIND_MATCHES_HASH: bool = true;

// ========= FLAGS NOT IN THE PAPER (default false) =========

/// Disable to disable timers.
pub const TIME: bool = true;
