use super::*;

// If `exact_end` is false, padding rows may be added at the end to speed things
// up. This means `h` will have a meaningless value at the end that does not
// correspond to the bottom row of the input range.
pub fn compute<const N: usize, H: HEncoding, const L: usize, const FILL: bool>(
    a: &[CC],
    b: &[[B; 4]],
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
    assert_eq!(b.len(), v.len());

    if FILL {
        assert_eq!(values.len(), h.len());

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
    }

    if a.len() < 2 * L * N {
        // TODO: This could be optimized a bit more.
        if N > 1 {
            return compute::<1, H, L, FILL>(a, b, h, v, exact_end, values);
        }
        if L > 2 {
            return compute::<1, H, 2, FILL>(a, b, h, v, exact_end, values);
        }
        for i in 0..a.len() {
            for j in 0..b.len() {
                myers::compute_block::<ScatterProfile, H>(&mut h[i], &mut v[j], &a[i], &b[j]);
            }
            if FILL {
                values[i].copy_from_slice(v);
            }
        }
        return h.iter().map(|h| h.value()).sum::<Cost>();
    }

    // Prevent allocation of unzipped `a` in this case.
    if b.len() == 1 {
        for i in 0..a.len() {
            myers::compute_block::<ScatterProfile, H>(&mut h[i], &mut v[0], &a[i], &b[0]);
            if FILL {
                values[i].copy_from_slice(v);
            }
        }
        return h.iter().map(|h| h.value()).sum::<Cost>();
    }

    // Iterate over blocks of L*N rows at a time.
    let b_chunks = b.array_chunks::<{ L * N }>();
    let v_chunks = v.array_chunks_mut::<{ L * N }>();
    let mut offset = 0;
    for (cbs, v) in izip!(b_chunks, v_chunks) {
        compute_block_of_rows::<N, H, L, FILL>(a, cbs, h, v, values, offset);
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
            compute_block_of_rows::<1, H, 4, FILL>(a, cbs, h, v2, values, offset);
            offset += 4;
        }
        if b.len() >= 2 {
            let (cbs, b_rem) = b.split_first_chunk::<2>().unwrap();
            b = b_rem;
            let (v2, v_rem) = v.split_first_chunk_mut::<2>().unwrap();
            v = v_rem;
            compute_block_of_rows::<1, H, 2, FILL>(a, cbs, h, v2, values, offset);
            offset += 2;
        }
        if b.len() >= 1 {
            let (cbs, b_rem) = b.split_first_chunk::<1>().unwrap();
            b = b_rem;
            let (v2, v_rem) = v.split_first_chunk_mut::<1>().unwrap();
            v = v_rem;
            for i in 0..a.len() {
                myers::compute_block::<ScatterProfile, H>(&mut h[i], &mut v2[0], &a[i], &cbs[0]);
                if FILL {
                    values[i][offset] = v2[0];
                }
            }
            // offset += 1;
        }
        assert!(b.len() == 0);
        assert!(v.len() == 0);
        h.iter().map(|h| h.value()).sum()
    } else {
        if FILL {
            panic!("Use exact mode for filling.");
        }

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
                    myers::compute_block::<ScatterProfile, H>(&mut h[i], &mut v[0], &a[i], &b[0]);
                }
            }
            2 => {
                compute_block_of_rows::<1, H, 2, FILL>(
                    a,
                    b.first_chunk().unwrap(),
                    h,
                    v.first_chunk_mut().unwrap(),
                    values,
                    offset,
                );
            }
            l @ (3 | 4) => {
                let b_tmp = from_fn(|i| if i < l { b[i] } else { Default::default() });
                let mut v_tmp = from_fn(|i| if i < l { v[i] } else { V::default() });
                compute_block_of_rows::<1, H, 4, FILL>(a, &b_tmp, h, &mut v_tmp, values, offset);
                v[0..l].copy_from_slice(&v_tmp[0..l]);
                correction = v_tmp[l..].iter().map(|v| v.value()).sum::<Cost>();
            }
            l @ (5 | 6 | 7) => {
                let b_tmp = from_fn(|i| if i < l { b[i] } else { Default::default() });
                let mut v_tmp = from_fn(|i| if i < l { v[i] } else { V::default() });
                compute_block_of_rows::<2, H, 4, FILL>(a, &b_tmp, h, &mut v_tmp, values, offset);
                v[0..l].copy_from_slice(&v_tmp[0..l]);
                correction = v_tmp[l..].iter().map(|v| v.value()).sum::<Cost>();
            }
            _ => panic!(),
        }
        h.iter().map(|h| h.value()).sum::<Cost>() - correction
    }
}

#[inline(always)]
fn compute_block_of_rows<const N: usize, H: HEncoding, const L: usize, const FILL: bool>(
    a: &[CC],
    cbs: &[[B; 4]; L * N],
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
            myers::compute_block::<ScatterProfile, H>(&mut h[i], &mut v[j], &a[i], &cbs[j]);
            if FILL {
                values[i][offset + j] = v[j];
            }
        }
    }

    // Align b.
    // let b0: [S<L>; N] = slice_to_simd(&from_fn(|k| cbs[rev(k)]));

    // Align h.
    let mut ph_simd: [S<L>; N] = slice_to_simd(&from_fn(|k| h[k].p()));
    let mut mh_simd: [S<L>; N] = slice_to_simd(&from_fn(|k| h[k].m()));

    // Align v.
    let mut pv_simd: [S<L>; N] = slice_to_simd(&from_fn(|k| v[rev(k)].p()));
    let mut mv_simd: [S<L>; N] = slice_to_simd(&from_fn(|k| v[rev(k)].m()));

    // Loop over horizontal windows of a.
    // The h windows are updated manually by rotating simd lanes of the
    // local variable h_simd.
    for (i, a) in a.array_windows::<{ L * N }>().skip(1).enumerate() {
        let eq: [S<L>; N] = from_fn(|k| {
            from_fn(|l| ScatterProfile::eq(&a[k * L + l], &cbs[rev(k * L + l)])).into()
        });

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

        if FILL {
            // This instruction is probably HOT during traceback. Could be faster by
            // first just writing out the diagonal SIMD vectors sequentially and
            // shuffling in a separate step.
            for (k, (pv, mv)) in izip!(simd_to_slice(&pv_simd), simd_to_slice(&mv_simd)).enumerate()
            {
                values[i + 1 + k][offset + rev(k)] = V::from(*pv, *mv);
            }
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
            myers::compute_block::<ScatterProfile, H>(&mut h[i], &mut v[j], &a[i], &cbs[j]);
            if FILL {
                values[i][offset + j] = v[j];
            }
        }
    }
}
