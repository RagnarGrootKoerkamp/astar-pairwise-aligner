#![feature(
    let_chains,
    int_roundings,
    test,
    array_chunks,
    array_windows,
    split_array,
    portable_simd,
    generic_const_exprs
)]

use std::{
    array::from_fn,
    simd::{LaneCount, Simd, SupportedLaneCount},
};

use itertools::izip;
use pa_types::Seq;

/// The type used for all bitvectors.
type B = u64;
/// The type for a Simd vector of `L` lanes of `B`.
type S<const L: usize> = Simd<B, L>;
/// The length of each bitvector.
const W: usize = B::BITS as usize;
/// The type used for differences.
type D = i64;

fn num_words(seq: Seq) -> usize {
    seq.len().div_ceil(W)
}

/// NOTE: This assumes an alphabet of {0,1,2,3} encoded as `u8`.
#[inline(always)]
pub fn profile(seq: Seq) -> Vec<[B; 4]> {
    let words = num_words(seq);
    let mut p: Vec<[B; 4]> = vec![[0; 4]; words];
    // TODO: Vectorize this, or ensure auto-vectorization.
    for (i, c) in seq.iter().enumerate() {
        p[i / W][*c as usize] |= 1 << (i % W);
    }
    p
}

#[inline(always)]
pub fn padded_profile(seq: Seq, padding: usize) -> Vec<[B; 4]> {
    let words = num_words(seq);
    let mut p: Vec<[B; 4]> = vec![[0; 4]; words + 2 * padding];
    // TODO: Vectorize this, or ensure auto-vectorization.
    for (i, c) in seq.iter().enumerate() {
        p[i / W + padding][*c as usize] |= 1 << (i % W);
    }
    p
}

/// Implements Myers '99 bitpacking based algorithm. Terminology is as in the paper.
/// The code is a direct translation from the implementation in Edlib.
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
pub fn compute_block(pv: B, mv: B, h0: D, eq: B) -> (B, B, D) {
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
pub fn compute_block_pmh(pv: B, mv: B, ph0: B, mh0: B, eq: B) -> (B, B, B, B) {
    let xv = eq | mv;
    let eq = eq | mh0;
    // The add here contains the 'folding' magic that makes this algorithm
    // 'non-local' and prevents simple SIMDification. See Myers'99 for details.
    let xh = (((eq & pv).wrapping_add(pv)) ^ pv) | eq;
    let ph = mv | !(xh | pv);
    let mh = pv & xh;
    // Extract `hw` from `ph` and `mh`.
    let phw = ph >> (W - 1);
    let mhw = mh >> (W - 1);

    // Push `hw` out of `ph` and `mh` and shift in `h0`.
    let ph = (ph << 1) | ph0;
    let mh = (mh << 1) | mh0;

    let pv_out = mh | !(xv | ph);
    let mv_out = ph & xv;
    (pv_out, mv_out, phw, mhw)
}

#[inline(always)]
pub fn compute_block_simd<const L: usize>(
    pv: S<L>,
    mv: S<L>,
    ph0: S<L>,
    mh0: S<L>,
    eq: S<L>,
) -> (S<L>, S<L>, S<L>, S<L>)
where
    LaneCount<L>: SupportedLaneCount,
{
    let xv = eq | mv;
    let eq = eq | mh0;
    // The add here contains the 'folding' magic that makes this algorithm
    // 'non-local' and prevents simple SIMDification. See Myers'99 for details.
    let xh = (((eq & pv) + pv) ^ pv) | eq;
    let ph = mv | !(xh | pv);
    let mh = pv & xh;
    // Extract `hw` from `ph` and `mh`.
    let right_shift: S<L> = Simd::<u64, L>::splat(W as u64 - 1);
    let phw = ph >> right_shift;
    let mhw = mh >> right_shift;

    // Push `hw` out of `ph` and `mh` and shift in `h0`.
    let left_shift: S<L> = Simd::<u64, L>::splat(1);
    let ph = (ph << left_shift) | ph0;
    let mh = (mh << left_shift) | mh0;

    let pv_out = mh | !(xv | ph);
    let mv_out = ph & xv;
    (pv_out, mv_out, phw, mhw)
}

/// NOTE: This assumes an alphabet of {0,1,2,3} encoded as `u8`.
pub fn nw_1(a: Seq, b: Seq) -> i64 {
    // For simplicity.
    assert!(b.len() % W == 0);

    let profile = profile(b);
    let words = num_words(b);
    // (pv, mv) for each block.
    // In the first column, vertical deltas are all +1.
    let mut col = vec![(B::MAX, 0); words];
    let mut bottom_row_score = b.len() as _;

    for c in a {
        // In the first row, horizontal deltas are all +1.
        let mut h = 1;
        for ((pv, mv), block_profile) in col.iter_mut().zip(&profile) {
            (*pv, *mv, h) = compute_block(*pv, *mv, h, block_profile[*c as usize]);
            assert!(*pv & *mv == 0);
        }
        bottom_row_score += h;
    }
    bottom_row_score
}

/// Same as `nw_1`, but stores horizontal deltas bitencoded as `ph` and `mh`.
pub fn nw_2(a: Seq, b: Seq) -> i64 {
    // For simplicity.
    assert!(b.len() % W == 0);

    let profile = profile(b);
    let words = num_words(b);
    // (pv, mv) for each block.
    // In the first column, vertical deltas are all +1.
    let mut col = vec![(B::MAX, 0); words];
    let mut bottom_row_score = b.len() as _;

    for c in a {
        // In the first row, horizontal deltas are all +1.
        let mut ph = 1;
        let mut mh = 0;
        for ((pv, mv), block_profile) in col.iter_mut().zip(&profile) {
            (*pv, *mv, ph, mh) = compute_block_pmh(*pv, *mv, ph, mh, block_profile[*c as usize]);
            assert!(*pv & *mv == 0);
        }
        bottom_row_score += ph as i64 - mh as i64;
    }
    bottom_row_score
}

/// Same as `nw_2`, but strides across four columns at a time.
///
/// Each | is one block/word.
/// ||||
/// ||||
/// ||||
/// ||||
///
/// They are computed in order:
///
///  col
///  v
/// 0123
/// 1234 <- row
/// 2345 <- base_row
/// 3456
///
/// when base_row == 2 and col == 1, then row == 1.
/// Within one diagonal stripe, computation is from bot-left to top-right.
pub fn nw_3(a: Seq, b: Seq) -> i64 {
    // For simplicity.
    assert!(b.len() % W == 0);

    let profile = profile(b);
    let words = num_words(b);
    // (pv, mv) for each block.
    // In the first column, vertical deltas are all +1.
    let mut col = vec![(B::MAX, 0); words];
    let mut bottom_row_score = b.len() as i64;

    let chunks = a.array_chunks::<4>();
    for chars in chunks.clone() {
        let mut ph = [1, 1, 1, 1];
        let mut mh = [0, 0, 0, 0];

        // TODO: Extract first and last 3 iterations that are special.
        for base_row in 0..words + 3 {
            for c in 0..4 {
                let r = base_row.wrapping_sub(c);
                // r < 0 is captured by wrapping.
                if r >= words {
                    continue;
                }
                (col[r].0, col[r].1, ph[c], mh[c]) = compute_block_pmh(
                    col[r].0,
                    col[r].1,
                    ph[c],
                    mh[c],
                    profile[r][chars[c] as usize],
                );
            }
        }

        bottom_row_score += ph.into_iter().sum::<u64>() as i64 - mh.into_iter().sum::<u64>() as i64;
    }

    // Do simple per-column scan for the remaining cols.
    for c in chunks.remainder() {
        let mut ph = 1;
        let mut mh = 0;
        for ((pv, mv), block_profile) in col.iter_mut().zip(&profile) {
            (*pv, *mv, ph, mh) = compute_block_pmh(*pv, *mv, ph, mh, block_profile[*c as usize]);
            assert!(*pv & *mv == 0);
        }
        bottom_row_score += ph as i64 - mh as i64;
    }

    bottom_row_score
}

/// Same as `nw_3`, but the first and last 3 base_rows are done separately.
pub fn nw_4(a: Seq, b: Seq) -> i64 {
    // For simplicity.
    assert!(b.len() % W == 0);

    let profile = profile(b);
    let words = num_words(b);
    // (pv, mv) for each block.
    // In the first column, vertical deltas are all +1.
    let mut col = vec![(B::MAX, 0); words];
    let mut bottom_row_score = b.len() as i64;

    let chunks = a.array_chunks::<4>();
    for chars in chunks.clone() {
        let mut ph = [1, 1, 1, 1];
        let mut mh = [0, 0, 0, 0];

        // TODO: Extract first and last 3 iterations that are special.
        for base_row in 0..3usize {
            for c in 0..4 {
                let r = base_row.wrapping_sub(c);
                // r < 0 is captured by wrapping.
                if r >= words {
                    continue;
                }
                (col[r].0, col[r].1, ph[c], mh[c]) = compute_block_pmh(
                    col[r].0,
                    col[r].1,
                    ph[c],
                    mh[c],
                    profile[r][chars[c] as usize],
                );
            }
        }

        // TODO: Extract first and last 3 iterations that are special.
        for base_row in 3..words {
            for c in 0..4 {
                let r = base_row.wrapping_sub(c);
                // r is always in range here.
                (col[r].0, col[r].1, ph[c], mh[c]) = compute_block_pmh(
                    col[r].0,
                    col[r].1,
                    ph[c],
                    mh[c],
                    profile[r][chars[c] as usize],
                );
            }
        }

        // TODO: Extract first and last 3 iterations that are special.
        for base_row in words..words + 3 {
            for c in 0..4 {
                let r = base_row.wrapping_sub(c);
                // r < 0 is captured by wrapping.
                if r >= words {
                    continue;
                }
                (col[r].0, col[r].1, ph[c], mh[c]) = compute_block_pmh(
                    col[r].0,
                    col[r].1,
                    ph[c],
                    mh[c],
                    profile[r][chars[c] as usize],
                );
            }
        }

        bottom_row_score += ph.into_iter().sum::<u64>() as i64 - mh.into_iter().sum::<u64>() as i64;
    }

    // Do simple per-column scan for the remaining cols.
    for c in chunks.remainder() {
        let mut ph = 1;
        let mut mh = 0;
        for ((pv, mv), block_profile) in col.iter_mut().zip(&profile) {
            (*pv, *mv, ph, mh) = compute_block_pmh(*pv, *mv, ph, mh, block_profile[*c as usize]);
            assert!(*pv & *mv == 0);
        }
        bottom_row_score += ph as i64 - mh as i64;
    }

    bottom_row_score
}

/// nw_3, but with padding and zipping and iterators
pub fn nw_5(a: Seq, b: Seq) -> i64 {
    // For simplicity.
    assert!(b.len() % W == 0);

    let profile = padded_profile(b, 3);

    let words = num_words(b);
    // (pv, mv) for each block.
    // In the first column, vertical deltas are all +1.
    let mut col = vec![(B::MAX, 0); words + 6];
    assert!(profile.len() == col.len());
    let mut bottom_row_score = b.len() as i64;

    let chunks = a.array_chunks::<4>();
    for chars in chunks.clone() {
        let mut ph = [1, 1, 1, 1];
        let mut mh = [0, 0, 0, 0];

        // TODO: Extract first and last 3 iterations that are special.
        for base_row in 3..words + 6 {
            for (ch, ph, mh, col, profile) in izip!(
                chars,
                ph.iter_mut(),
                mh.iter_mut(),
                col[base_row - 3..=base_row].iter_mut().rev(),
                profile[base_row - 3..=base_row].iter().rev(),
            ) {
                //let r = base_row - c;
                (col.0, col.1, *ph, *mh) =
                    compute_block_pmh(col.0, col.1, *ph, *mh, profile[*ch as usize]);
            }
        }

        bottom_row_score += ph.into_iter().sum::<u64>() as i64 - mh.into_iter().sum::<u64>() as i64;
    }

    // Do simple per-column scan for the remaining cols.
    for c in chunks.remainder() {
        let mut ph = 1;
        let mut mh = 0;
        for ((pv, mv), block_profile) in col.iter_mut().zip(&profile) {
            (*pv, *mv, ph, mh) = compute_block_pmh(*pv, *mv, ph, mh, block_profile[*c as usize]);
            assert!(*pv & *mv == 0);
        }
        bottom_row_score += ph as i64 - mh as i64;
    }

    bottom_row_score
}

/// nw_5, with even more iterators
pub fn nw_6(a: Seq, b: Seq) -> i64 {
    // For simplicity.
    assert!(b.len() % W == 0);

    let profile = padded_profile(b, 3);

    let words = num_words(b);
    // (pv, mv) for each block.
    // In the first column, vertical deltas are all +1.
    let mut col = vec![(B::MAX, 0); words + 6];
    assert!(profile.len() == col.len());
    let mut bottom_row_score = b.len() as i64;

    let chunks = a.array_chunks::<4>();
    for chars in chunks.clone() {
        let mut ph = [1, 1, 1, 1];
        let mut mh = [0, 0, 0, 0];

        // NOTE: `array_windows_mut` would be cool here but sadly doesn't exist.
        for (i, profiles) in profile.array_windows::<4>().enumerate() {
            let cols = col[i..i + 4].split_array_mut::<4>().0;
            for (ch, ph, mh, col, profile) in izip!(
                chars,
                ph.iter_mut(),
                mh.iter_mut(),
                cols.iter_mut().rev(),
                profiles.iter().rev(),
            ) {
                (col.0, col.1, *ph, *mh) =
                    compute_block_pmh(col.0, col.1, *ph, *mh, profile[*ch as usize]);
            }
        }

        bottom_row_score += ph.into_iter().sum::<u64>() as i64 - mh.into_iter().sum::<u64>() as i64;
    }

    // Do simple per-column scan for the remaining cols.
    for c in chunks.remainder() {
        let mut ph = 1;
        let mut mh = 0;
        for ((pv, mv), block_profile) in col.iter_mut().zip(&profile) {
            (*pv, *mv, ph, mh) = compute_block_pmh(*pv, *mv, ph, mh, block_profile[*c as usize]);
            assert!(*pv & *mv == 0);
        }
        bottom_row_score += ph as i64 - mh as i64;
    }

    bottom_row_score
}

/// nw_6, with unrolled inner loop
pub fn nw_7(a: Seq, b: Seq) -> i64 {
    // For simplicity.
    assert!(b.len() % W == 0);

    let profile = padded_profile(b, 3);

    let words = num_words(b);
    // (pv, mv) for each block.
    // In the first column, vertical deltas are all +1.
    let mut col = vec![(B::MAX, 0); words + 6];
    assert!(profile.len() == col.len());
    let mut bottom_row_score = b.len() as i64;

    let chunks = a.array_chunks::<4>();
    for chars in chunks.clone() {
        let mut ph = [1, 1, 1, 1];
        let mut mh = [0, 0, 0, 0];

        // NOTE: `array_windows_mut` would be cool here but sadly doesn't exist.
        for (i, profiles) in profile.array_windows::<4>().enumerate() {
            let cols = col[i..i + 4].split_array_mut::<4>().0;
            let eqs = [
                profiles[0][chars[3] as usize],
                profiles[1][chars[2] as usize],
                profiles[2][chars[1] as usize],
                profiles[3][chars[0] as usize],
            ];
            #[rustfmt::skip]
            {
            // (cols[3].0, cols[3].1, ph[0], mh[0]) = compute_block_pmh(cols[3].0, cols[3].1, ph[0], mh[0], profiles[3][chars[0] as usize],);
            // (cols[2].0, cols[2].1, ph[1], mh[1]) = compute_block_pmh(cols[2].0, cols[2].1, ph[1], mh[1], profiles[2][chars[1] as usize],);
            // (cols[1].0, cols[1].1, ph[2], mh[2]) = compute_block_pmh(cols[1].0, cols[1].1, ph[2], mh[2], profiles[1][chars[2] as usize],);
            // (cols[0].0, cols[0].1, ph[3], mh[3]) = compute_block_pmh(cols[0].0, cols[0].1, ph[3], mh[3], profiles[0][chars[3] as usize],);
            (cols[0].0, cols[0].1, ph[0], mh[0]) = compute_block_pmh(cols[0].0, cols[0].1, ph[0], mh[0], eqs[0]);
            (cols[1].0, cols[1].1, ph[1], mh[1]) = compute_block_pmh(cols[1].0, cols[1].1, ph[1], mh[1], eqs[1]);
            (cols[2].0, cols[2].1, ph[2], mh[2]) = compute_block_pmh(cols[2].0, cols[2].1, ph[2], mh[2], eqs[2]);
            (cols[3].0, cols[3].1, ph[3], mh[3]) = compute_block_pmh(cols[3].0, cols[3].1, ph[3], mh[3], eqs[3]);
            };
        }

        bottom_row_score += ph.into_iter().sum::<u64>() as i64 - mh.into_iter().sum::<u64>() as i64;
    }

    // Do simple per-column scan for the remaining cols.
    for c in chunks.remainder() {
        let mut ph = 1;
        let mut mh = 0;
        for ((pv, mv), block_profile) in col.iter_mut().zip(&profile) {
            (*pv, *mv, ph, mh) = compute_block_pmh(*pv, *mv, ph, mh, block_profile[*c as usize]);
            assert!(*pv & *mv == 0);
        }
        bottom_row_score += ph as i64 - mh as i64;
    }

    bottom_row_score
}

/// N: Number of parallel columns
pub fn nw_scalar_copies<const N: usize>(a: Seq, b: Seq) -> i64
where
    [(); 1 * N]: Sized,
{
    const L: usize = 1;
    // For simplicity.
    assert!(b.len() % W == 0);

    let padding = L * N - 1;
    let profile = padded_profile(b, padding);

    let words = num_words(b);
    // (pv, mv) for each block.
    // In the first column, vertical deltas are all +1.
    let mut col = vec![(B::MAX, 0); words + 2 * padding];
    assert!(profile.len() == col.len());
    let mut bottom_row_score = b.len() as i64;

    let chunks = a.array_chunks::<{ L * N }>();
    for chars in chunks.clone() {
        let mut ph = [1; N];
        let mut mh = [0; N];

        // NOTE: `array_windows_mut` would be cool here but sadly doesn't exist.
        for (i, profiles) in profile.array_windows::<{ L * N }>().enumerate() {
            // NOTE: The rev is important for higher instructions/cycle.
            for j in (0..N).rev() {
                let offset = j * L;
                let (pcols, mcols) = &mut col[i + offset];
                let eqs: B = profiles[offset][chars[L * N - 1 - offset] as usize];
                (*pcols, *mcols, ph[j], mh[j]) =
                    compute_block_pmh(*pcols, *mcols, ph[j], mh[j], eqs);
            }
        }

        bottom_row_score += ph.into_iter().sum::<u64>() as i64 - mh.into_iter().sum::<u64>() as i64;
    }

    // Do simple per-column scan for the remaining cols.
    for c in chunks.remainder() {
        let mut ph = 1;
        let mut mh = 0;
        for ((pv, mv), block_profile) in izip!(col.iter_mut(), &profile) {
            (*pv, *mv, ph, mh) = compute_block_pmh(*pv, *mv, ph, mh, block_profile[*c as usize]);
            assert!(*pv & *mv == 0);
        }
        bottom_row_score += ph as i64 - mh as i64;
    }

    bottom_row_score
}

/// nw_9, but inner loop is a for-loop instead of manually unrolled.
/// L: Number of simd lanes to use
/// N: Number of parallel simd units to use
pub fn nw_simd_copies<const L: usize, const N: usize>(a: Seq, b: Seq) -> i64
where
    LaneCount<L>: SupportedLaneCount,
    [(); L * N]: Sized,
{
    // For simplicity.
    assert!(b.len() % W == 0);

    let padding = L * N - 1;
    let profile = padded_profile(b, padding);

    let words = num_words(b);
    // (pv, mv) for each block.
    // In the first column, vertical deltas are all +1.
    let mut pcol = vec![B::MAX; words + 2 * padding];
    let mut mcol = vec![0; words + 2 * padding];
    assert!(profile.len() == pcol.len());
    assert!(profile.len() == mcol.len());
    let mut bottom_row_score = b.len() as i64;

    let chunks = a.array_chunks::<{ L * N }>();
    for chars in chunks.clone() {
        // unsafe {
        //     prefetch_read_data((&chars[0] as *const u8).add(L * N), 3);
        // }
        let mut ph = [S::<L>::splat(1); N];
        let mut mh = [S::<L>::splat(0); N];

        for i in 0..words + padding {
            // unsafe {
            //     prefetch_read_data((&profile[i] as *const [B; 4]).add(N * L), 3);
            //     prefetch_write_data((&pcol[i] as *const B).add(N * L), 3);
            //     prefetch_write_data((&mcol[i] as *const B).add(N * L), 3);
            // }
            // NOTE: The rev is important for higher instructions/cycle.
            // This loop is unrolled by the compiler.
            for j in (0..N).rev() {
                let offset = j * L;
                let profiles = profile[i + offset..i + offset + L].split_array_ref::<L>().0;
                let pcols_mut = pcol[i + offset..i + offset + L].split_array_mut::<L>().0;
                let mcols_mut = mcol[i + offset..i + offset + L].split_array_mut::<L>().0;
                let mut pcols: S<L> = (*pcols_mut).into();
                let mut mcols: S<L> = (*mcols_mut).into();
                let eqs: S<L> =
                    from_fn(|k| profiles[k][chars[L * N - 1 - k - offset] as usize]).into();
                (pcols, mcols, ph[j], mh[j]) = compute_block_simd(pcols, mcols, ph[j], mh[j], eqs);
                *pcols_mut = *pcols.as_array();
                *mcols_mut = *mcols.as_array();
            }
        }

        bottom_row_score += ph
            .map(|ph| ph.as_array().into_iter().sum::<u64>())
            .iter()
            .sum::<u64>() as i64
            - mh.map(|ph| ph.as_array().into_iter().sum::<u64>())
                .iter()
                .sum::<u64>() as i64;
    }

    // Do simple per-column scan for the remaining cols.
    for c in chunks.remainder() {
        let mut ph = 1;
        let mut mh = 0;
        for (pv, mv, block_profile) in izip!(pcol.iter_mut(), mcol.iter_mut(), &profile) {
            (*pv, *mv, ph, mh) = compute_block_pmh(*pv, *mv, ph, mh, block_profile[*c as usize]);
            assert!(*pv & *mv == 0);
        }
        bottom_row_score += ph as i64 - mh as i64;
    }

    bottom_row_score
}

#[cfg(test)]
mod bench {
    extern crate test;
    use bio::alphabets::{Alphabet, RankTransform};
    use pa_types::Seq;
    use test::Bencher;

    #[bench]
    fn profile(bench: &mut Bencher) {
        let (b, _) = pa_generate::uniform_fixed(1024, 0.1);
        let b = &RankTransform::new(&Alphabet::new(b"ACGT")).transform(b);
        bench.iter(|| super::profile(b));
    }
    fn bench_aligner(f: fn(Seq, Seq) -> i64, bench: &mut Bencher) {
        let (b, a) = pa_generate::uniform_fixed(1024, 0.1);
        let a = &RankTransform::new(&Alphabet::new(b"ACGT")).transform(a);
        let b = &RankTransform::new(&Alphabet::new(b"ACGT")).transform(b);
        let d = bio::alignment::distance::simd::levenshtein(a, b) as _;
        assert_eq!(f(a, b), d);
        bench.iter(|| f(a, b));
    }

    #[bench]
    fn triple_accel(bench: &mut Bencher) {
        let f = |a: Seq, b: Seq| bio::alignment::distance::simd::levenshtein(a, b) as i64;
        bench_aligner(f, bench);
    }

    #[bench]
    fn nw_1(bench: &mut Bencher) {
        bench_aligner(super::nw_1, bench);
    }
    #[bench]
    fn nw_2(bench: &mut Bencher) {
        bench_aligner(super::nw_2, bench);
    }
    #[bench]
    fn nw_3(bench: &mut Bencher) {
        bench_aligner(super::nw_3, bench);
    }
    #[bench]
    fn nw_4(bench: &mut Bencher) {
        bench_aligner(super::nw_4, bench);
    }
    #[bench]
    fn nw_5(bench: &mut Bencher) {
        bench_aligner(super::nw_5, bench);
    }
    #[bench]
    fn nw_6(bench: &mut Bencher) {
        bench_aligner(super::nw_6, bench);
    }
    #[bench]
    fn nw_7(bench: &mut Bencher) {
        bench_aligner(super::nw_7, bench);
    }
    macro_rules! scalar_test {
        // h is a function (exact: bool, pruning: bool) -> Heuristic.
        ($name:ident, $N:expr) => {
            #[bench]
            fn $name(bench: &mut Bencher) {
                bench_aligner(super::nw_scalar_copies::<$N>, bench);
            }
        };
    }
    scalar_test!(scalar_1, 1);
    scalar_test!(scalar_2, 2);
    scalar_test!(scalar_3, 3);
    scalar_test!(scalar_4, 4);

    macro_rules! simd_test {
        // h is a function (exact: bool, pruning: bool) -> Heuristic.
        ($name:ident, $L:expr, $N:expr) => {
            #[bench]
            fn $name(bench: &mut Bencher) {
                bench_aligner(super::nw_simd_copies::<$L, $N>, bench);
            }
        };
    }
    simd_test!(simd_1_1, 1, 1);
    simd_test!(simd_1_2, 1, 2);
    simd_test!(simd_1_3, 1, 3);
    simd_test!(simd_1_4, 1, 4);
    simd_test!(simd_2_1, 2, 1);
    simd_test!(simd_2_2, 2, 2);
    simd_test!(simd_2_3, 2, 3);
    simd_test!(simd_2_4, 2, 4);
    simd_test!(simd_4_1, 4, 1);
    simd_test!(simd_4_2, 4, 2);
    simd_test!(simd_4_3, 4, 3);
    simd_test!(simd_4_4, 4, 4);
}
