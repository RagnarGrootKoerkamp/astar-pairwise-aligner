//! TODO:
//! - col-first instead of row-first
//! - col-first with local-h
//! - padding instead of manual filling in edges
//! - N=1 simd for edges before scalar
//! - try 2-lane simd for bottom edge
//!
//! Reading and writing directly into unaligned sliding windows of h and v is inefficient!
//! We solve this by keeping a local SIMD vector that's rotated one lane at a time.
//!
//! Conclusions:
//! - Inside the inner loop, everything must be unpacked.
//! - Need to convert a from Vec<(a0, a1)> into (Vec<a0>, Vec<a1>).
//! - For 256 wide rows, edges are <1% of time and no need to optimize.
//! - The row version is SIMD instructions only apart from a single loop increment; looks very efficient.
//! - row::<1> is good
//! - row::<2> seems best; slightly better IPC
//! - row::<3> and row::<L> use too many registers probably; almost 2x slower
//! - local_h for simd doesn't help, since effectively we already create a local h in each iteration anyway.
//! - row-first is always as good as col-first.
//!
//! Timings for 256 wide block:
//! - 1       row  (scalar) : 1.4
//! - 2       rows (simd2)  : 1.8
//! - 3/4     rows (simd4)  : 2.0
//! - 5/6/7/8 rows (2xsimd4): 3.2
//! => Doing a 4 high SIMD block is better than 2 individual rows.
//!
use super::*;
use crate::bit_profile::Bits;
use itertools::{izip, Itertools};
use pa_types::Cost;
use std::{
    array::from_fn,
    mem::transmute,
    simd::{LaneCount, SupportedLaneCount},
};

/// NOTE: This is simply a cast.
#[inline(always)]
fn simd_to_slice<const N: usize, const L: usize>(simd: &[S<L>; N]) -> &[B; L * N]
where
    LaneCount<L>: SupportedLaneCount,
{
    unsafe { transmute(simd) }
}

/// NOTE: This creates a new array with the right alignment.
#[inline(always)]
fn slice_to_simd<const N: usize, const L: usize>(slice: &[B; L * N]) -> [S<L>; N]
where
    LaneCount<L>: SupportedLaneCount,
{
    unsafe {
        slice
            .array_chunks::<L>()
            .map(|&b| b.into())
            .array_chunks::<N>()
            .next()
            .unwrap_unchecked()
    }
}

#[inline(always)]
fn rotate_left<const N: usize, const L: usize>(ph_simd: &mut [S<L>; N], mut carry: B) -> B
where
    LaneCount<L>: SupportedLaneCount,
{
    for k in (0..N).rev() {
        ph_simd[k] = ph_simd[k].rotate_elements_left::<1>();
        let new_carry = ph_simd[k].as_array()[L - 1];
        ph_simd[k].as_mut_array()[L - 1] = carry;
        carry = new_carry;
    }
    carry
}

// If `exact_end` is false, padding rows may be added at the end to speed things
// up. This means `h` will have a meaningless value at the end that does not
// correspond to the bottom row of the input range.
pub fn compute<const N: usize, H: HEncoding, const L: usize>(
    a: &[Bits],
    b: &[Bits],
    h: &mut [H],
    v: &mut [V],
    exact_end: bool,
) -> Cost
where
    LaneCount<L>: SupportedLaneCount,
    [(); L * N]: Sized,
    [(); L * 1]: Sized,
{
    assert_eq!(a.len(), h.len());
    assert_eq!(b.len(), v.len());
    if a.len() < 2 * L * N {
        // TODO: This could be optimized a bit more.
        if N > 1 {
            return compute::<1, H, L>(a, b, h, v, exact_end);
        }
        if L > 2 {
            return compute::<1, H, 2>(a, b, h, v, exact_end);
        }
        for i in 0..a.len() {
            for j in 0..b.len() {
                myers::compute_block::<BitProfile, H>(&mut h[i], &mut v[j], &a[i], &b[j]);
            }
        }
        return h.iter().map(|h| h.value()).sum::<Cost>();
    }

    // Prevent allocation of unzipped `a` in this case.
    if b.len() == 1 {
        for i in 0..a.len() {
            myers::compute_block::<BitProfile, H>(&mut h[i], &mut v[0], &a[i], &b[0]);
        }
        return h.iter().map(|h| h.value()).sum::<Cost>();
    }

    // Unzip bits of a so we can directly use unaligned reads later.
    let ap0 = a.iter().map(|ca| ca.0).collect_vec();
    let ap1 = a.iter().map(|ca| ca.1).collect_vec();

    // Iterate over blocks of L*N rows at a time.
    let b_chunks = b.array_chunks::<{ L * N }>();
    let v_chunks = v.array_chunks_mut::<{ L * N }>();
    for (cbs, v) in izip!(b_chunks, v_chunks) {
        compute_block_of_rows(a, &ap0, &ap1, cbs, h, v);
    }

    // Handle the remaining rows.
    // - With exponential decay in exact mode.
    // - With padding an a single extra call otherwise.
    let mut b = b.array_chunks::<{ L * N }>().remainder();
    let mut v = v.array_chunks_mut::<{ L * N }>().into_remainder();
    assert_eq!(b.len(), v.len());
    if exact_end {
        // b.len() < 8 for N=2, L=4.
        // - if >=4: N=1, L=4 simd row
        // - if >=2: N=1, L=2 half-simd row
        // - if >=1: scalar row
        while b.len() >= 4 {
            let (cbs, b_rem) = b.split_first_chunk::<4>().unwrap();
            b = b_rem;
            let (v2, v_rem) = v.split_first_chunk_mut::<4>().unwrap();
            v = v_rem;
            compute_block_of_rows::<1, H, 4>(a, &ap0, &ap1, cbs, h, v2);
        }
        if b.len() >= 2 {
            let (cbs, b_rem) = b.split_first_chunk::<2>().unwrap();
            b = b_rem;
            let (v2, v_rem) = v.split_first_chunk_mut::<2>().unwrap();
            v = v_rem;
            compute_block_of_rows::<1, H, 2>(a, &ap0, &ap1, cbs, h, v2);
        }
        if b.len() >= 1 {
            let (cbs, b_rem) = b.split_first_chunk::<1>().unwrap();
            b = b_rem;
            let (v2, v_rem) = v.split_first_chunk_mut::<1>().unwrap();
            v = v_rem;
            for i in 0..a.len() {
                myers::compute_block::<BitProfile, H>(&mut h[i], &mut v2[0], &a[i], &cbs[0]);
            }
        }
        assert!(b.len() == 0);
        assert!(v.len() == 0);
        h.iter().map(|h| h.value()).sum()
    } else {
        // Do a 1, 2, 4, or 8 row block.
        // If needed, add padding: Add some extra v=0 elements to v and random
        // chars to b and compute a larger block. Then, compute the horizontal
        // delta, and remove the vertical delta at the end. Lastly, overwrite
        // vertical deltas with the temporary variable.
        let mut correction = 0;
        match b.len() {
            0 => {}
            1 => {
                for i in 0..a.len() {
                    myers::compute_block::<BitProfile, H>(&mut h[i], &mut v[0], &a[i], &b[0]);
                }
            }
            2 => {
                compute_block_of_rows::<1, H, 2>(
                    a,
                    &ap0,
                    &ap1,
                    b.first_chunk().unwrap(),
                    h,
                    v.first_chunk_mut().unwrap(),
                );
            }
            l @ (3 | 4) => {
                let b_tmp = from_fn(|i| if i < l { b[i] } else { Bits(0, 0) });
                let mut v_tmp = from_fn(|i| if i < l { v[i] } else { V::default() });
                compute_block_of_rows::<1, H, 4>(a, &ap0, &ap1, &b_tmp, h, &mut v_tmp);
                v[0..l].copy_from_slice(&v_tmp[0..l]);
                correction = v_tmp[l..].iter().map(|v| v.value()).sum::<Cost>();
            }
            l @ (5 | 6 | 7) => {
                let b_tmp = from_fn(|i| if i < l { b[i] } else { Bits(0, 0) });
                let mut v_tmp = from_fn(|i| if i < l { v[i] } else { V::default() });
                compute_block_of_rows::<2, H, 4>(a, &ap0, &ap1, &b_tmp, h, &mut v_tmp);
                v[0..l].copy_from_slice(&v_tmp[0..l]);
                correction = v_tmp[l..].iter().map(|v| v.value()).sum::<Cost>();
            }
            _ => panic!(),
        }
        h.iter().map(|h| h.value()).sum::<Cost>() - correction
    }
}

#[inline(always)]
fn compute_block_of_rows<const N: usize, H: HEncoding, const L: usize>(
    a: &[Bits],
    ap0: &[B],
    ap1: &[B],
    cbs: &[Bits; L * N],
    h: &mut [H],
    v: &mut [V; L * N],
) where
    LaneCount<L>: SupportedLaneCount,
    [(); L * N]: Sized,
{
    let rev = |k| L * N - 1 - k;

    // Top-left triangle of block of rows.
    for j in 0..L * N {
        for i in 0..L * N - j {
            myers::compute_block::<BitProfile, H>(&mut h[i], &mut v[j], &a[i], &cbs[j]);
        }
    }

    // Align b.
    let b0: [S<L>; N] = slice_to_simd(&from_fn(|k| cbs[rev(k)].0));
    let b1: [S<L>; N] = slice_to_simd(&from_fn(|k| cbs[rev(k)].1));

    // Align h.
    let mut ph_simd: [S<L>; N] = slice_to_simd(&from_fn(|k| h[k].p()));
    let mut mh_simd: [S<L>; N] = slice_to_simd(&from_fn(|k| h[k].m()));

    // Align v.
    let mut pv_simd: [S<L>; N] = slice_to_simd(&from_fn(|k| v[rev(k)].p()));
    let mut mv_simd: [S<L>; N] = slice_to_simd(&from_fn(|k| v[rev(k)].m()));

    // Loop over horizontal windows of a.
    // The h windows are updated manually by rotating simd lanes of the
    // local variable h_simd.
    assert_eq!(ap0.len(), ap1.len());
    for (i, (a0, a1)) in izip!(
        ap0.array_windows::<{ L * N }>().skip(1),
        ap1.array_windows::<{ L * N }>().skip(1)
    )
    .enumerate()
    {
        // Read the unaligned lanes of a.
        let a0 = slice_to_simd(a0);
        let a1 = slice_to_simd(a1);

        let eq: [S<L>; N] = from_fn(|k| BitProfile::eq_simd((&a0[k], &a1[k]), (&b0[k], &b1[k])));

        // Rotate the lanes of h.
        unsafe {
            let (p, m) = h.get_unchecked(i + L * N).pm();
            let pcarry = rotate_left(&mut ph_simd, p);
            let mcarry = rotate_left(&mut mh_simd, m);
            *h.get_unchecked_mut(i) = H::from(pcarry, mcarry);
        }
        for k in 0..N {
            myers::compute_block_simd(
                &mut ph_simd[k],
                &mut mh_simd[k],
                &mut pv_simd[k],
                &mut mv_simd[k],
                eq[k],
            );
        }
    }

    // Write back h to unaligned slice.
    unsafe {
        let ph = simd_to_slice(&ph_simd);
        let mh = simd_to_slice(&mh_simd);
        for i in 0..L * N {
            *h.get_unchecked_mut(h.len() - L * N + i) = H::from(ph[i], mh[i]);
        }
    }

    // Write back v to unaligned slice.
    let pv = simd_to_slice(&pv_simd);
    let mv = simd_to_slice(&mv_simd);
    *v = from_fn(|k| V::from(pv[rev(k)], mv[rev(k)]));

    // Bottom-right triangle of block of rows.
    for j in 0..L * N {
        for i in a.len() - j..a.len() {
            myers::compute_block::<BitProfile, H>(&mut h[i], &mut v[j], &a[i], &cbs[j]);
        }
    }
}

/// Same as `compute`, but returns all computed value.
pub fn fill<const N: usize, H: HEncoding, const L: usize>(
    a: &[Bits],
    b: &[Bits],
    h: &mut [H],
    v: &mut [V],
    exact_end: bool,
    values: &mut [Vec<V>],
) -> Cost
where
    LaneCount<L>: SupportedLaneCount,
    [(); L * N]: Sized,
    [(); L * 1]: Sized,
{
    assert_eq!(a.len(), h.len());
    assert_eq!(values.len(), h.len());
    assert_eq!(b.len(), v.len());
    for vv in values.iter_mut() {
        // Grow `vv`, but do not initialize its elements since they will be overwritten anyway.
        if vv.capacity() < v.len() {
            vv.resize(v.len(), V::default());
        } else {
            // SAFETY: We check above that the capacity is at least `v.len()`.
            // No initialization is needed for (tuples of) ints.
            unsafe {
                vv.set_len(v.len());
            }
        }
    }
    if a.len() < 2 * L * N {
        // TODO: This could be optimized a bit more.
        if N > 1 {
            return fill::<1, H, L>(a, b, h, v, exact_end, values);
        }
        if L > 2 {
            return fill::<1, H, 2>(a, b, h, v, exact_end, values);
        }
        for i in 0..a.len() {
            for j in 0..b.len() {
                myers::compute_block::<BitProfile, H>(&mut h[i], &mut v[j], &a[i], &b[j]);
            }
            values[i].copy_from_slice(v);
        }
        return h.iter().map(|h| h.value()).sum::<Cost>();
    }

    // Prevent allocation of unzipped `a` in this case.
    if b.len() == 1 {
        for i in 0..a.len() {
            myers::compute_block::<BitProfile, H>(&mut h[i], &mut v[0], &a[i], &b[0]);
            values[i].copy_from_slice(v);
        }
        return h.iter().map(|h| h.value()).sum::<Cost>();
    }

    // Unzip bits of a so we can directly use unaligned reads later.
    let ap0 = a.iter().map(|ca| ca.0).collect_vec();
    let ap1 = a.iter().map(|ca| ca.1).collect_vec();

    // Iterate over blocks of L*N rows at a time.
    let b_chunks = b.array_chunks::<{ L * N }>();
    let v_chunks = v.array_chunks_mut::<{ L * N }>();
    let mut offset = 0;
    for (cbs, v) in izip!(b_chunks, v_chunks) {
        fill_block_of_rows(a, &ap0, &ap1, cbs, h, v, values, offset);
        offset += L * N;
    }

    // Handle the remaining rows.
    // - With exponential decay in exact mode.
    // - With padding an a single extra call otherwise.
    let mut b = b.array_chunks::<{ L * N }>().remainder();
    let mut v = v.array_chunks_mut::<{ L * N }>().into_remainder();
    assert_eq!(b.len(), v.len());
    if exact_end {
        // b.len() < 8 for N=2, L=4.
        // - if >=4: N=1, L=4 simd row
        // - if >=2: N=1, L=2 half-simd row
        // - if >=1: scalar row
        while b.len() >= 4 {
            let (cbs, b_rem) = b.split_first_chunk::<4>().unwrap();
            b = b_rem;
            let (v2, v_rem) = v.split_first_chunk_mut::<4>().unwrap();
            v = v_rem;
            fill_block_of_rows::<1, H, 4>(a, &ap0, &ap1, cbs, h, v2, values, offset);
            offset += 4;
        }
        if b.len() >= 2 {
            let (cbs, b_rem) = b.split_first_chunk::<2>().unwrap();
            b = b_rem;
            let (v2, v_rem) = v.split_first_chunk_mut::<2>().unwrap();
            v = v_rem;
            fill_block_of_rows::<1, H, 2>(a, &ap0, &ap1, cbs, h, v2, values, offset);
            offset += 2;
        }
        if b.len() >= 1 {
            let (cbs, b_rem) = b.split_first_chunk::<1>().unwrap();
            b = b_rem;
            let (v2, v_rem) = v.split_first_chunk_mut::<1>().unwrap();
            v = v_rem;
            for i in 0..a.len() {
                myers::compute_block::<BitProfile, H>(&mut h[i], &mut v2[0], &a[i], &cbs[0]);
                values[i][offset] = v2[0];
            }
            //offset += 1;
        }
        assert!(b.len() == 0);
        assert!(v.len() == 0);
        h.iter().map(|h| h.value()).sum()
    } else {
        panic!("Use exact mode for filling");
    }
}

#[inline(always)]
fn fill_block_of_rows<const N: usize, H: HEncoding, const L: usize>(
    a: &[Bits],
    ap0: &[B],
    ap1: &[B],
    cbs: &[Bits; L * N],
    h: &mut [H],
    v: &mut [V; L * N],
    values: &mut [Vec<V>],
    offset: usize,
) where
    LaneCount<L>: SupportedLaneCount,
    [(); L * N]: Sized,
{
    let rev = |k| L * N - 1 - k;

    // Top-left triangle of block of rows.
    for j in 0..L * N {
        for i in 0..L * N - j {
            myers::compute_block::<BitProfile, H>(&mut h[i], &mut v[j], &a[i], &cbs[j]);
            values[i][offset + j] = v[j];
        }
    }

    // Align b.
    let b0: [S<L>; N] = slice_to_simd(&from_fn(|k| cbs[rev(k)].0));
    let b1: [S<L>; N] = slice_to_simd(&from_fn(|k| cbs[rev(k)].1));

    // Align h.
    let mut ph_simd: [S<L>; N] = slice_to_simd(&from_fn(|k| h[k].p()));
    let mut mh_simd: [S<L>; N] = slice_to_simd(&from_fn(|k| h[k].m()));

    // Align v.
    let mut pv_simd: [S<L>; N] = slice_to_simd(&from_fn(|k| v[rev(k)].p()));
    let mut mv_simd: [S<L>; N] = slice_to_simd(&from_fn(|k| v[rev(k)].m()));

    // Loop over horizontal windows of a.
    // The h windows are updated manually by rotating simd lanes of the
    // local variable h_simd.
    assert_eq!(ap0.len(), ap1.len());
    for (i, (a0, a1)) in izip!(
        ap0.array_windows::<{ L * N }>().skip(1),
        ap1.array_windows::<{ L * N }>().skip(1)
    )
    .enumerate()
    {
        // Read the unaligned lanes of a.
        let a0 = slice_to_simd(a0);
        let a1 = slice_to_simd(a1);

        let eq: [S<L>; N] = from_fn(|k| BitProfile::eq_simd((&a0[k], &a1[k]), (&b0[k], &b1[k])));

        // Rotate the lanes of h.
        unsafe {
            let (p, m) = h.get_unchecked(i + L * N).pm();
            let pcarry = rotate_left(&mut ph_simd, p);
            let mcarry = rotate_left(&mut mh_simd, m);
            *h.get_unchecked_mut(i) = H::from(pcarry, mcarry);
        }
        for k in 0..N {
            myers::compute_block_simd(
                &mut ph_simd[k],
                &mut mh_simd[k],
                &mut pv_simd[k],
                &mut mv_simd[k],
                eq[k],
            );
        }

        // This instruction is probably HOT during traceback. Could be faster by
        // first just writing out the diagonal SIMD vectors sequentially and
        // shuffling in a separate step.
        for (k, (pv, mv)) in izip!(simd_to_slice(&pv_simd), simd_to_slice(&mv_simd)).enumerate() {
            values[i + 1 + k][offset + rev(k)] = V::from(*pv, *mv);
        }
    }

    // Write back h to unaligned slice.
    unsafe {
        let ph = simd_to_slice(&ph_simd);
        let mh = simd_to_slice(&mh_simd);
        for i in 0..L * N {
            *h.get_unchecked_mut(h.len() - L * N + i) = H::from(ph[i], mh[i]);
        }
    }

    // Write back v to unaligned slice.
    let pv = simd_to_slice(&pv_simd);
    let mv = simd_to_slice(&mv_simd);
    *v = from_fn(|k| V::from(pv[rev(k)], mv[rev(k)]));

    // Bottom-right triangle of block of rows.
    for j in 0..L * N {
        for i in a.len() - j..a.len() {
            myers::compute_block::<BitProfile, H>(&mut h[i], &mut v[j], &a[i], &cbs[j]);
            values[i][offset + j] = v[j];
        }
    }
}

#[cfg(feature = "example")]
pub fn vis_block_of_rows<const N: usize, const B: usize>(
    n: usize,
    m: usize,
    vis: &mut impl pa_vis::VisualizerInstance,
) where
    [(); 4 * N]: Sized,
{
    use pa_types::{Pos, I};

    let m = m as I;
    const L: usize = 4;
    let rev = |k| L * N - 1 - k;

    // Top-left triangle of block of rows.
    for j in 0..L * N {
        for i in 0..L * N - j {
            vis.expand_block_simple(Pos(i as I - 1, m + B as I * j as I), Pos(1, B as I));
        }
    }
    vis.new_layer::<pa_heuristic::NoCostI>(None);

    for i in 1..=n - L * N {
        for k in 0..N {
            let pos = [L * k + 0, L * k + 1, L * k + 2, L * k + 3]
                .map(|k| Pos(i as I + rev(k) as I - 1, m + B as I * k as I));
            let sizes = [Pos(1, B as I); L];
            vis.expand_blocks_simple(pos, sizes);
            vis.new_layer::<pa_heuristic::NoCostI>(None);
        }
    }

    // Bottom-right triangle of block of rows.
    for j in 0..L * N {
        for i in n - j..n {
            vis.expand_block_simple(Pos(i as I - 1, m + B as I * j as I), Pos(1, B as I));
        }
    }
}
