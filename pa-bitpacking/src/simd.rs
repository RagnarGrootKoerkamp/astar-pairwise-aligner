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
#[inline(always)]
fn slice_to_simd<const N: usize>(slice: &[B; 4 * N]) -> [S; N] {
    // SAFETY:
    unsafe {
        slice
            .array_chunks::<4>()
            .map(|&b| b.into())
            .array_chunks::<N>()
            .next()
            .unwrap_unchecked()
    }
}
/// NOTE: This is simply a cast.
#[inline(always)]
fn simd_to_slice<const N: usize>(simd: &[S; N]) -> &[B; 4 * N] {
    unsafe { transmute(simd) }
}

/// Compute 4*N rows of 64-bit blocks at a time.
///
/// - Top/middle rows are with SIMD.
///   - Top-left and bot-right triangle of the 4N row block are done with scalars.
///     (4N(4N-1) blocks in total.)
/// - Last <4*N rows are done with scalars.
/// Returns the difference along the bottom row.
pub fn compute_columns_simd<const N: usize>(a: CompressedSeq, b: ProfileSlice, v: &mut [V]) -> D
where
    [(); L * N]: Sized,
{
    assert_eq!(b.len(), v.len());
    if a.len() < 2 * 4 * N || b.len() < 4 * N {
        return compute_columns(a, b, v);
    }

    let mut ph = vec![1; a.len()];
    let mut mh = vec![0; a.len()];

    let b_chunks = b.array_chunks::<{ L * N }>();
    let v_chunks = v.array_chunks_mut::<{ L * N }>();
    let bv_chunks = izip!(b_chunks, v_chunks);

    let rev = |i| 4 * N - 1 - i;

    for (cb, v) in bv_chunks {
        // Top-left triangle of block of rows.
        for j in 0..4 * N - 1 {
            for i in 0..4 * N - 1 - j {
                compute_block_split_h(&mut ph[i], &mut mh[i], &mut v[j], cb[j][a[i] as usize]);
            }
        }

        // Middle with SIMD.
        // Use a temp local SIMD `pv` and `mv` for vertical difference.
        // NOTE: This 'unzipping' and 'zipping' of `pv` and `mv` is a bit ugly,
        // but given that the loop goes over many columns, it doesn't matter for
        // performance.
        let mut pv_simd: [S; N] = slice_to_simd(&from_fn(|i| v[rev(i)].p()));
        let mut mv_simd: [S; N] = slice_to_simd(&from_fn(|i| v[rev(i)].m()));
        for (i, ca) in a.array_windows::<{ 4 * N }>().enumerate() {
            // NOTE: The 'gather' operation resulting from this is slow!
            let eqs: [S; N] = slice_to_simd(&from_fn(|i| unsafe {
                *cb.get_unchecked(rev(i)).get_unchecked(ca[i] as usize)
            }));

            // SAFETY: By construction, a has the same length as ph and mh, and
            // i iterates over windows of size L*N of a, so we can take equal
            // windows of ph and mh.  Would be replaced by `array_windows_mut`
            // if it existed.
            let ph: &mut [B; L * N] = unsafe {
                (ph.get_unchecked_mut(i..i + L * N))
                    .try_into()
                    .unwrap_unchecked()
            };
            let mh: &mut [B; L * N] = unsafe {
                (mh.get_unchecked_mut(i..i + L * N))
                    .try_into()
                    .unwrap_unchecked()
            };
            let mut ph_simd = slice_to_simd(ph);
            let mut mh_simd = slice_to_simd(mh);
            for k in 0..N {
                compute_block_simd(
                    &mut ph_simd[k],
                    &mut mh_simd[k],
                    &mut pv_simd[k],
                    &mut mv_simd[k],
                    eqs[k],
                );
            }
            *ph = *simd_to_slice(&ph_simd);
            *mh = *simd_to_slice(&mh_simd);
        }
        // Write back the local `pv` and `pv`.
        let pv = simd_to_slice(&pv_simd);
        let mv = simd_to_slice(&mv_simd);
        *v = from_fn(|i| V::from(pv[rev(i)], mv[rev(i)]));

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
