//! The basic bitpacked algorithm from Myers'99.
use crate::{HEncoding, Profile, B, S, V, W};
use std::simd::{LaneCount, SupportedLaneCount};

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
pub fn compute_block<P: Profile, H: HEncoding>(h0: &mut H, v: &mut V, ca: &P::A, cb: &P::B) {
    let eq = P::eq(ca, cb);
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

    *h0 = H::from(phw as B, mhw);
    *v = V::from(mh | !(xv | ph), ph & xv);
}

/// Simd version of `compute_block`.
///
/// This assumes HEncoding of `(u64,u64)`.
#[inline(always)]
pub fn compute_block_simd<const L: usize>(
    ph0: &mut S<L>,
    mh0: &mut S<L>,
    pv: &mut S<L>,
    mv: &mut S<L>,
    eq: S<L>,
) where
    LaneCount<L>: SupportedLaneCount,
{
    let xv = eq | *mv;
    let eq = eq | *mh0;
    // The add here contains the 'folding' magic that makes this algorithm
    // 'non-local' and prevents simple SIMDification. See Myers'99 for details.
    let xh = (((eq & *pv) + *pv) ^ *pv) | eq;
    let ph = *mv | !(xh | *pv);
    let mh = *pv & xh;
    // Extract `hw` from `ph` and `mh`.
    let right_shift = S::splat(W as B - 1);
    let phw = ph >> right_shift;
    let mhw = mh >> right_shift;

    // Push `hw` out of `ph` and `mh` and shift in `h0`.
    let left_shift = S::splat(1);
    let ph = (ph << left_shift) | *ph0;
    let mh = (mh << left_shift) | *mh0;

    *pv = mh | !(xv | ph);
    *mv = ph & xv;
    *ph0 = phw;
    *mh0 = mhw;
}
