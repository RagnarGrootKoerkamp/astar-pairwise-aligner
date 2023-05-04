#![allow(incomplete_features)]
#![feature(
    let_chains,
    int_roundings,
    test,
    array_chunks,
    iter_array_chunks,
    array_windows,
    split_array,
    portable_simd,
    generic_const_exprs,
    concat_idents,
    bigint_helper_methods,
    core_intrinsics
)]

mod encoding;
pub mod simd;

use std::ops::Deref;

use bio::alphabets::{Alphabet, RankTransform};
pub use encoding::*;
use itertools::{izip, Itertools};
pub use pa_types::Seq;
use pa_types::Sequence;

/// The type used for all bitvectors.
#[cfg(feature = "small_blocks")]
pub type B = u8;
/// The type used for all bitvectors.
#[cfg(not(feature = "small_blocks"))]
pub type B = u64;
/// The length of each bitvector.
pub const W: usize = B::BITS as usize;
/// The type used for differences.
pub type D = i64;
/// Default encoding used for horizontal differences.
pub type H = (u8, u8);

pub fn num_words(seq: Seq) -> usize {
    seq.len().div_ceil(W)
}

/// Newtype for compressed sequences that have characters 0,1,2,3.
pub struct CompressedSequence(Sequence);
pub type CompressedSeq<'a> = &'a CompressedSequence;
impl Deref for CompressedSequence {
    type Target = Sequence;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Profile = Vec<[B; 4]>;
pub type ProfileSlice<'a> = &'a [[B; 4]];

/// RankTransform `a` and `b`
#[inline(always)]
pub fn compress(a: Seq, b: Seq) -> (CompressedSequence, CompressedSequence) {
    let r = RankTransform::new(&Alphabet::new(b"ACGT"));
    let a = a.iter().map(|ca| r.get(*ca)).collect_vec();
    let b = b.iter().map(|ca| r.get(*ca)).collect_vec();
    (CompressedSequence(a), CompressedSequence(b))
}

/// RankTransform `a` and give the profile for `b`.
#[inline(always)]
pub fn profile(a: Seq, b: Seq) -> (CompressedSequence, Profile) {
    let r = RankTransform::new(&Alphabet::new(b"ACGT"));
    let a = a.iter().map(|ca| r.get(*ca)).collect_vec();
    let words = num_words(b);
    let mut pb: Profile = vec![[0; 4]; words];
    for (j, cb) in b.iter().enumerate() {
        pb[j / W][r.get(*cb) as usize] |= 1 << (j % W);
    }
    (CompressedSequence(a), pb)
}

/// Implements Myers '99 bitpacking based algorithm. Terminology is as in the
/// paper. The code is a translation from the implementation in Edlib.
///
/// Modifies `h0` (horizontal difference at top) and `v` (vertical differences
/// along the left) in place.
///
/// Given the scores below:
///
/// A0 - B0
/// |    |
/// A1 - B1
///   ...
/// AW - BW
///
/// h0 = B0 - A0
/// v[i] = A(i+1) - Ai
///
/// H and V are wrapper types to encode the horizontal and vertical differences
/// using a + and - indicator bit.
///
/// 20 operations.
#[inline(always)]
pub fn compute_block<H: HEncoding>(h0: &mut H, v: &mut V, eq: B) {
    let (pv, mv) = v.pm();
    let xv = eq | mv;
    let eq = eq | h0.m();
    // The add here contains the 'folding' magic that makes this algorithm
    // 'non-local' and prevents simple SIMDification. See Myers'99 for details.
    let xh = (((eq & pv).wrapping_add(pv)) ^ pv) | eq;
    let ph = mv | !(xh | pv);
    let mh = pv & xh;
    // Extract `hw` from `ph` and `mh`.
    // TODO: Use carry-bit from shit-left operation.
    // TODO: Use 63-bit vectors to save some operations.

    // Push `hw` out of `ph` and `mh` and shift in `h0`.
    // NOTE: overflowing_add uses the carry bit, but is slow because reading the
    // carry bit right after this instruction interrupts pipelining.
    // NOTE: overflowing_shl returns whether the shift is too large, not the shifted out bit.
    let phw = ph >> (W - 1);
    let mhw = mh >> (W - 1);
    let ph = (ph << 1) | h0.p();
    let mh = (mh << 1) | h0.m();

    // Update the inputs.
    *h0 = H::from(phw as B, mhw);
    *v = V::from(mh | !(xv | ph), ph & xv);
}

/// Wrapper that takes an unpacked h
#[inline(always)]
pub fn compute_block_split_h(ph0: &mut B, mh0: &mut B, v: &mut V, eq: B) {
    let h0 = &mut (*ph0, *mh0);
    compute_block(h0, v, eq);
    *ph0 = h0.0;
    *mh0 = h0.1;
}

/// Convenience function around `compute_block` that computes a larger region at once.
/// Returns the score difference along the bottom row.
pub fn compute_rectangle(a: Seq, b: ProfileSlice, h: &mut [H], v: &mut [V]) -> D {
    assert_eq!(a.len(), h.len());
    assert_eq!(b.len(), v.len());
    for (ca, h) in izip!(a, h.iter_mut()) {
        for (cb, v) in izip!(b, v.iter_mut()) {
            compute_block(h, v, cb[*ca as usize]);
        }
    }
    h.iter().map(|h| h.value()).sum()
}

/// Same as `compute_rectangle`, but does not take or return horizontal differences.
pub fn compute_columns(a: Seq, b: ProfileSlice, v: &mut [V]) -> D {
    assert_eq!(b.len(), v.len());
    let mut bot_delta = 0;
    for ca in a {
        let h = &mut H::one();
        for (cb, v) in izip!(b, v.iter_mut()) {
            compute_block(h, v, cb[*ca as usize]);
        }
        bot_delta += h.value();
    }
    bot_delta
}

/// Same as `compute_columns`, but uses a SIMD-based implementation.
pub fn compute_columns_simd(a: CompressedSeq, b: ProfileSlice, v: &mut [V]) -> D {
    assert_eq!(b.len(), v.len());
    // NOTE: A quick experiment shows that 2 SIMD vecs in parallel works best.
    simd::nw_simd_striped_row::<2>(a, b, v)
}
