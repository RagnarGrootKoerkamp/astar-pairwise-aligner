//! Bitpacking methods.
//!
//! Given a width of 256, the fastest methods per height are:
//!  64: bit/u8/row (2.4), bit/u64/row (2.6)
//! 128: bit/u64/simd2row/1 (3.50)
//! 256: bit/u64/simdrow/1 (3.8), bit/u8/simdrow/1 (3.82) (~2x speed of others)
//! 512: bit/u64/simdrow/2 (5.9), bit/u8/simdrow/2 (6.1) (~3x speed of others)
//!     => 2 SIMDs in parallel has higher IPC
//!
//! conclusion:
//!                              u8   u64
//! 1       => row              1.28  1.41
//! 2       => simd2            1.93  1.86
//! 3/4     => simd4/1 (padded) 2.04  2.07
//! 5/6/7/8 => simd4/2 (padded) 3.25  3.19
#![allow(incomplete_features)]
#![feature(
    array_chunks,
    array_windows,
    bigint_helper_methods,
    concat_idents,
    exclusive_range_pattern,
    generic_const_exprs,
    int_roundings,
    iter_array_chunks,
    let_chains,
    portable_simd,
    test
)]

mod encoding;
pub mod myers;
pub mod profile;
pub mod scalar;
pub mod simd;

pub use encoding::*;
pub use profile::*;

/// The type used for all bitvectors.
/// Small blocks are nicer for visualizations.
#[cfg(feature = "small_blocks")]
pub type B = u8;

/// The type used for all bitvectors.
#[cfg(not(feature = "small_blocks"))]
pub type B = u64;

/// The length of each bitvector.
pub const W: usize = B::BITS as usize;

/// Default encoding used for horizontal differences.
pub type H = (u8, u8);

/// The number of lanes in a Simd vector.
pub const L: usize = 4;

/// The type for a Simd vector of `L` lanes of `B`.
pub type S<const L: usize> = std::simd::Simd<B, L>;
