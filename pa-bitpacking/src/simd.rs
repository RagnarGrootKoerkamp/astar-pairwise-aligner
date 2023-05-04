use std::{array::from_fn, mem::transmute, simd::Simd};

use super::*;

/// The number of lanes in a Simd vector.
pub const L: usize = 4;
/// The type for a Simd vector of `L` lanes of `B`.
pub type S = Simd<B, L>;

#[inline(always)]
pub fn compute_block_simd(ph0: &mut S, mh0: &mut S, pv: &mut S, mv: &mut S, eq: S) {
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

/// NOTE: This creates a new array with the right alignment.
#[inline]
fn slice_to_simd<const N: usize>(slice: &[B; 4 * N]) -> [S; N] {
    slice
        .array_chunks::<4>()
        .map(|&b| b.into())
        .array_chunks::<N>()
        .next()
        .unwrap()
}
/// NOTE: This is simply a cast.
#[inline]
fn simd_to_slice<const N: usize>(simd: &[S; N]) -> &[B; 4 * N] {
    unsafe { transmute(simd) }
}

/// Compute 4*N rows of 64-bit blocks at a time.
///
/// - Top/middle rows are with SIMD.
///   - Top-left and bot-right triangle of the 4N row block are done with scalars.
///     (4N(4N-1) blocks in total.)
/// - Last <4*N rows are done with scalars.
pub fn nw_simd_striped_row<const N: usize>(a: CompressedSeq, b: ProfileSlice, v: &mut [V]) -> D
where
    [(); L * N]: Sized,
{
    let mut ph = vec![1; a.len()];
    let mut mh = vec![0; a.len()];

    let b_chunks = b.array_chunks::<{ L * N }>();
    let v_chunks = v.array_chunks_mut::<{ L * N }>();
    let bv_chunks = izip!(b_chunks, v_chunks);

    for (cb, v) in bv_chunks {
        // Top-left triangle of block of rows.
        for j in 0..4 * N - 1 {
            for i in 0..4 * N - 1 - j {
                compute_block_split_h(&mut ph[i], &mut mh[i], &mut v[j], cb[j][a[i] as usize]);
            }
        }

        // Middle with SIMD.
        // Use a temp local SIMD `pv` and `mv` for vertical difference.
        let mut pv: [S; N] = slice_to_simd(&from_fn(|i| v[i].p()));
        let mut mv: [S; N] = slice_to_simd(&from_fn(|i| v[i].m()));
        for (i, ca) in a.array_windows::<{ 4 * N }>().enumerate() {
            let eqs: [S; N] = unsafe {
                from_fn(|k| {
                    from_fn(|l| *cb[l].get_unchecked(ca[L * N - 1 - l - k * 4] as usize)).into()
                })
            };
            let ph = ph[i..].split_array_mut().0;
            let mh = mh[i..].split_array_mut().0;
            let mut ph_simd = slice_to_simd(ph);
            let mut mh_simd = slice_to_simd(mh);
            for k in 0..N {
                compute_block_simd(
                    &mut ph_simd[k],
                    &mut mh_simd[k],
                    &mut pv[k],
                    &mut mv[k],
                    eqs[k],
                );
            }
            *ph = *simd_to_slice(&ph_simd);
            *mh = *simd_to_slice(&mh_simd);
        }
        // Write back the local `pv` and `pv`.
        let pv = simd_to_slice(&pv);
        let mv = simd_to_slice(&mv);
        *v = from_fn(|j| V::from(pv[j], mv[j]));

        // Bottom-right triangle of block of rows.
        for j in 1..4 * N {
            for i in a.len() - j..a.len() {
                compute_block_split_h(&mut ph[i], &mut mh[i], &mut v[j], cb[j][a[i] as usize]);
            }
        }
    }

    // TODO: Figure out which order is better for these 2 loops.
    let b_chunks = b.array_chunks::<{ L * N }>();
    let v_chunks = v.array_chunks_mut::<{ L * N }>();
    for (cb, v) in izip!(b_chunks.remainder(), v_chunks.into_remainder()) {
        for (ca, ph, mh) in izip!(a.iter(), ph.iter_mut(), mh.iter_mut()) {
            compute_block_split_h(ph, mh, v, cb[*ca as usize]);
        }
    }

    ph.iter().sum::<B>() as D - mh.iter().sum::<B>() as D
}
