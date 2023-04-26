use crate::{B, D, S, W};

/// Implements Myers '99 bitpacking based algorithm. Terminology is as in the paper.
/// The code is a direct translation from the implementation in Edlib.
/// The `compute_block` function below is faster.
///
/// Given the scores below:
///
/// A0 - B0
/// |    |
/// A1 - B1
///   ...
/// AW - BW
///
/// h0/hw are the horizontal difference at the top and bottom:
/// h0 = B0 - A0
///
/// pv and mv bit-encode whether *v*ertical differences are *p*lus 1 or *m*inus 1:
/// pv[i] = [A[i+1] - A[i] ==  1]
/// mv[i] = [A[i+1] - A[i] == -1]
///
/// Returns (pvout, mvout, hw):
/// pvout[i] = [B[i+1] - B[i] ==  1]
/// mvout[i] = [B[i+1] - B[i] == -1]
/// hw = Bw - Aw
///
/// 24 operations.
#[inline(always)]
pub fn compute_block_edlib(pv: B, mv: B, h0: D, eq: B) -> (B, B, D) {
    // Indicator for h==-1:
    // 00..01 if h==-1, 00..00 otherwise.
    let mh0 = (h0 as B >> 1) & 1;
    let xv = eq | mv;
    let eq = eq | mh0;
    // The add here contains the 'folding' magic that makes this algorithm
    // 'non-local' and prevents simple SIMDification. See Myers'99 for details.
    let xh = (((eq & pv).wrapping_add(pv)) ^ pv) | eq;
    let ph = mv | !(xh | pv);
    let mh = pv & xh;
    // Extract `hw` from `ph` and `mh`.
    let hw = (ph >> (W - 1)) as D - (mh >> (W - 1)) as D;

    // Push `hw` out of `ph` and `mh` and shift in `h0`.
    // The | is equivalent to `if hin>0 { ph |= 1; }`.
    let ph = (ph << 1) | ((h0 + 1) >> 1) as B;
    // The | is equivalent to `if hin<0 { mh |= 1; }`.
    let mh = (mh << 1) | mh0;

    let pv_out = mh | !(xv | ph);
    let mv_out = ph & xv;
    (pv_out, mv_out, hw)
}

/// Same as above, but h0 and hw are encoded using p and m indicators.
/// h=-1:
/// p = 00..00
/// m = 00..01
/// h=0:
/// p = 00..00
/// m = 00..00
/// h=1:
/// p = 00..01
/// m = 00..00
///
/// Returns (pv, mv, phw, mhw).
///
/// 20 operations.
#[inline(always)]
pub fn compute_block(pv: &mut B, mv: &mut B, ph0: &mut B, mh0: &mut B, eq: B) {
    let xv = eq | *mv;
    let eq = eq | *mh0;
    // The add here contains the 'folding' magic that makes this algorithm
    // 'non-local' and prevents simple SIMDification. See Myers'99 for details.
    let xh = (((eq & *pv).wrapping_add(*pv)) ^ *pv) | eq;
    let ph = *mv | !(xh | *pv);
    let mh = *pv & xh;
    // Extract `hw` from `ph` and `mh`.
    let phw = ph >> (W - 1);
    let mhw = mh >> (W - 1);

    // Push `hw` out of `ph` and `mh` and shift in `h0`.
    let ph = (ph << 1) | *ph0;
    let mh = (mh << 1) | *mh0;

    *pv = mh | !(xv | ph);
    *mv = ph & xv;
    *ph0 = phw;
    *mh0 = mhw;
}

#[inline(always)]
pub fn compute_block_bool(pv: &mut B, mv: &mut B, ph0: &mut bool, mh0: &mut bool, eq: B) {
    let xv = eq | *mv;
    let eq = eq | *mh0 as B;
    // The add here contains the 'folding' magic that makes this algorithm
    // 'non-local' and prevents simple SIMDification. See Myers'99 for details.
    let xh = (((eq & *pv).wrapping_add(*pv)) ^ *pv) | eq;
    let ph = *mv | !(xh | *pv);
    let mh = *pv & xh;
    // Extract `hw` from `ph` and `mh`.
    let phw = ph >> (W - 1);
    let mhw = mh >> (W - 1);

    // Push `hw` out of `ph` and `mh` and shift in `h0`.
    let ph = (ph << 1) | *ph0 as B;
    let mh = (mh << 1) | *mh0 as B;

    *pv = mh | !(xv | ph);
    *mv = ph & xv;
    *ph0 = phw != 0;
    *mh0 = mhw != 0;
}

#[inline(always)]
pub fn compute_block_simd(pv: &mut S, mv: &mut S, ph0: &mut S, mh0: &mut S, eq: S) {
    let xv = eq | *mv;
    let eq = eq | *mh0;
    // The add here contains the 'folding' magic that makes this algorithm
    // 'non-local' and prevents simple SIMDification. See Myers'99 for details.
    let xh = (((eq & *pv) + *pv) ^ *pv) | eq;
    let ph = *mv | !(xh | *pv);
    let mh = *pv & xh;
    // Extract `hw` from `ph` and `mh`.
    let right_shift = S::splat(W as u64 - 1);
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
