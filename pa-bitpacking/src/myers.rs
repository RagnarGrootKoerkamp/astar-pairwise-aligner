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
/// 20 operations, excluding `eq`.
#[inline(always)]
pub fn compute_block<P: Profile, H: HEncoding>(h0: &mut H, v: &mut V, ca: &P::A, cb: &P::B) {
    let eq = P::eq(ca, cb); // this one is not counted as an operation
    let (vp, vm) = v.pm();
    let vx = eq | vm;
    // NOTE: This is not in Myers' original code because he assumes the input delta can never be -1.
    let eq = eq | h0.m();
    // The add here contains the 'folding' magic that makes this algorithm
    // 'non-local' and prevents simple SIMDification. See Myers'99 for details.
    let hx = (((eq & vp).wrapping_add(vp)) ^ vp) | eq;
    let hp = vm | !(hx | vp);
    let hm = vp & hx;
    // Extract `hw` from `ph` and `mh`.
    // TODO: Use carry-bit from shit-left operation.
    // - The problem with carry bits is that they block pipelining, hence
    //   incurring a bit performance hit.
    // TODO: Could we save ops with 63-bit vectors?

    // Push `hw` out of `ph` and `mh` and shift in `h0`.
    // NOTE: overflowing_add uses the carry bit, but is slow because reading the
    // carry bit right after this instruction interrupts pipelining.
    // NOTE: overflowing_shl returns whether the shift is too large, not the shifted out bit.
    let hpw = hp >> (W - 1);
    let hmw = hm >> (W - 1);
    let hp = (hp << 1) | h0.p();
    let hm = (hm << 1) | h0.m();

    *h0 = H::from(hpw as B, hmw);
    *v = V::from(hm | !(vx | hp), hp & vx);
}

/// Simd version of `compute_block`.
///
/// This assumes HEncoding of `(u64,u64)`.
#[inline(always)]
pub fn compute_block_simd<const L: usize>(
    hp0: &mut S<L>,
    hm0: &mut S<L>,
    vp: &mut S<L>,
    vm: &mut S<L>,
    eq: S<L>,
) where
    LaneCount<L>: SupportedLaneCount,
{
    let vx = eq | *vm;
    let eq = eq | *hm0;
    // The add here contains the 'folding' magic that makes this algorithm
    // 'non-local' and prevents simple SIMDification. See Myers'99 for details.
    let hx = (((eq & *vp) + *vp) ^ *vp) | eq;
    let hp = *vm | !(hx | *vp);
    let hm = *vp & hx;
    // Extract `hw` from `ph` and `mh`.
    let right_shift = S::splat(W as B - 1);
    let hpw = hp >> right_shift;
    let hmw = hm >> right_shift;

    // Push `hw` out of `ph` and `mh` and shift in `h0`.
    let left_shift = S::splat(1);
    let hp = (hp << left_shift) | *hp0;
    let hm = (hm << left_shift) | *hm0;

    *hp0 = hpw;
    *hm0 = hmw;
    *vp = hm | !(vx | hp);
    *vm = hp & vx;
}
