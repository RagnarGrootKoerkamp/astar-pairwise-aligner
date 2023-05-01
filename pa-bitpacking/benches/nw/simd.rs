use super::*;
use itertools::izip;
use pa_types::{Pos, I};
use pa_vis_types::{VisualizerInstance, VisualizerT};
use std::{array::from_fn, simd::Simd};

/// The number of lanes in a Simd vector.
pub const L: usize = 4;
/// The type for a Simd vector of `L` lanes of `B`.
pub type S = Simd<B, L>;

/// Pad the profile with `padding` words on each side.
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

#[inline(always)]
pub fn compute_block_simd_col(ph0: &mut B, mh0: &mut B, pv: &mut S, mv: &mut S, mut eq: S) {
    let xv = eq | *mv;
    // TODO: Can this be replaced by using *mh0 and initial carry?
    eq[0] |= *mh0;
    // The add here contains the 'folding' magic that makes this algorithm
    // 'non-local' and prevents simple SIMDification. See Myers'99 for details.
    let a = eq & *pv;
    let a = a.as_array();
    let b = pv.as_array();
    // TODO: Compute c with internal carry-over.
    let mut c = S::default();
    let ca = c.as_mut_array();
    let mut carry;
    (ca[0], carry) = a[0].overflowing_add(b[0]);
    (ca[1], carry) = a[1].carrying_add(b[1], carry);
    (ca[2], carry) = a[2].carrying_add(b[2], carry);
    (ca[3], _) = a[3].carrying_add(b[3], carry);
    let xh = (c ^ *pv) | eq;
    let mut ph = *mv | !(xh | *pv);
    let mut mh = *pv & xh;

    // Extract `hw` from `ph` and `mh`.
    let mut ph_carry = *ph0 != 0;
    (ph[0], ph_carry) = ph[0].carrying_add(ph[0], ph_carry);
    (ph[1], ph_carry) = ph[1].carrying_add(ph[1], ph_carry);
    (ph[2], ph_carry) = ph[2].carrying_add(ph[2], ph_carry);
    (ph[3], ph_carry) = ph[3].carrying_add(ph[3], ph_carry);
    let mut mh_carry = *mh0 != 0;
    (mh[0], mh_carry) = mh[0].carrying_add(mh[0], mh_carry);
    (mh[1], mh_carry) = mh[1].carrying_add(mh[1], mh_carry);
    (mh[2], mh_carry) = mh[2].carrying_add(mh[2], mh_carry);
    (mh[3], mh_carry) = mh[3].carrying_add(mh[3], mh_carry);

    *pv = mh | !(xv | ph);
    *mv = ph & xv;
    *ph0 = ph_carry as B;
    *mh0 = mh_carry as B;
}

/// TODO optimizations:
/// - Reverse a or b in memory, so that anti-diagonals align.
/// - Reverse ph and pm in memory?
/// - Reverse for-loop order.
/// - Skip vectors completely out-of-bounds.
pub fn nw_simd_striped_col<const N: usize>(a: Seq, b: Seq, viz: &impl VisualizerT) -> D
where
    [(); L * N]: Sized,
{
    let ref mut viz = viz.build(a, b);
    assert!(b.len() % W == 0);

    let mut bottom_row_score = b.len() as D;
    let padding = L * N - 1;
    let words = num_words(b);
    let b = padded_profile(b, padding);

    let mut pv = vec![B::MAX; b.len()];
    let mut mv = vec![0; b.len()];

    let chunks = a.array_chunks::<{ L * N }>();
    for (cas, i) in chunks.clone().zip((1..).step_by(L * N)) {
        // unsafe {
        //     prefetch_read_data((&chars[0] as *const u8).add(L * N), 3);
        // }
        let mut ph = [S::splat(1); N];
        let mut mh = [S::splat(0); N];

        for j in 0..words + padding {
            // unsafe {
            //     prefetch_read_data((&profile[i] as *const [B; 4]).add(N * L), 3);
            //     prefetch_write_data((&pcol[i] as *const B).add(N * L), 3);
            //     prefetch_write_data((&mcol[i] as *const B).add(N * L), 3);
            // }
            // NOTE: The rev is important for higher instructions/cycle.
            // This loop is unrolled by the compiler.
            unsafe {
                for k in (0..N).rev() {
                    let offset = k * L;
                    if j + offset + L <= padding || j + offset + L * N > b.len() {
                        continue;
                    }
                    // There is some annoying wrapping and unwrapping into Simd here, since we can't
                    // directly borrow unaligned array slices.
                    //S::from_slice(slice)

                    //let cbs = b[j + offset..].split_array_ref::<L>().0;
                    //let pv_mut = pv[j + offset..].split_array_mut::<L>().0;
                    //let mv_mut = mv[j + offset..].split_array_mut::<L>().0;
                    let cbs = &*(b[j + offset..].as_ptr() as *const [[B; 4]; L]);
                    let pv_mut = &mut *(pv[j + offset..].as_ptr() as *mut [B; L]);
                    let mv_mut = &mut *(mv[j + offset..].as_ptr() as *mut [B; L]);
                    let mut pv = (*pv_mut).into();
                    let mut mv = (*mv_mut).into();
                    let eqs =
                        from_fn(|l| *cbs[l].get_unchecked(cas[L * N - 1 - l - offset] as usize))
                            .into();
                    compute_block_simd(&mut ph[k], &mut mh[k], &mut pv, &mut mv, eqs);
                    *pv_mut = *pv.as_array();
                    *mv_mut = *mv.as_array();

                    viz.expand_blocks_simple(
                        from_fn(|l| {
                            Pos(
                                (i + L * N - 1 - offset - l) as I,
                                ((j + offset + l) as I - padding as I) * W as I + 1,
                            )
                        })
                        .into(),
                        [Pos(1, W as I); L],
                    );
                }
            }
        }

        bottom_row_score += ph
            .map(|ph| ph.to_array().into_iter().sum::<B>())
            .into_iter()
            .sum::<B>() as D
            - mh.map(|ph| ph.to_array().into_iter().sum::<B>())
                .into_iter()
                .sum::<B>() as D;
    }

    // Do simple per-column scan for the remaining cols.
    for c in chunks.remainder() {
        let h = &mut (1u8, 0u8);
        for (pv, mv, block_profile) in izip!(pv.iter_mut(), mv.iter_mut(), &b) {
            let v = &mut V::from(*pv, *mv);
            compute_block(h, v, block_profile[*c as usize]);
            (*pv, *mv) = v.pm();
        }
        bottom_row_score += h.value();
    }

    viz.last_frame_simple();
    bottom_row_score
}

/// TODO optimizations:
/// - Reverse a or b in memory, so that anti-diagonals align.
/// - Reverse ph and pm in memory?
/// - Reverse for-loop order.
/// - Skip vectors completely out-of-bounds.
pub fn nw_simd_col<const N: usize>(a: Seq, b: Seq, viz: &impl VisualizerT) -> D
where
    [(); L * N]: Sized,
{
    let ref mut viz = viz.build(a, b);
    assert!(b.len() % W == 0);

    let mut bottom_row_score = b.len() as D;
    let padding = L * N - 1;
    let words = num_words(b);
    let b = padded_profile(b, padding);

    let mut pv = vec![B::MAX; b.len()];
    let mut mv = vec![0; b.len()];

    let chunks = a.array_chunks::<N>();
    for (cas, i) in chunks.clone().zip((1..).step_by(N)) {
        // unsafe {
        //     prefetch_read_data((&chars[0] as *const u8).add(L * N), 3);
        // }
        let mut ph = [1; N];
        let mut mh = [0; N];

        for j in 0..words + padding {
            // unsafe {
            //     prefetch_read_data((&profile[i] as *const [B; 4]).add(N * L), 3);
            //     prefetch_write_data((&pcol[i] as *const B).add(N * L), 3);
            //     prefetch_write_data((&mcol[i] as *const B).add(N * L), 3);
            // }
            // NOTE: The rev is important for higher instructions/cycle.
            // This loop is unrolled by the compiler.
            unsafe {
                for k in (0..N).rev() {
                    let offset = k * L;
                    if j + offset + L <= padding || j + offset + L > b.len() {
                        continue;
                    }
                    // There is some annoying wrapping and unwrapping into Simd here, since we can't
                    // directly borrow unaligned array slices.
                    //S::from_slice(slice)

                    //let cbs = b[j + offset..].split_array_ref::<L>().0;
                    //let pv_mut = pv[j + offset..].split_array_mut::<L>().0;
                    //let mv_mut = mv[j + offset..].split_array_mut::<L>().0;
                    let cbs = &*(b[j + offset..].as_ptr() as *const [[B; 4]; L]);
                    let pv_mut = &mut *(pv[j + offset..].as_ptr() as *mut [B; L]);
                    let mv_mut = &mut *(mv[j + offset..].as_ptr() as *mut [B; L]);
                    let mut pv = (*pv_mut).into();
                    let mut mv = (*mv_mut).into();
                    let eqs =
                        from_fn(|l| *cbs[l].get_unchecked(cas[L * N - 1 - l - offset] as usize))
                            .into();
                    compute_block_simd_col(&mut ph[k], &mut mh[k], &mut pv, &mut mv, eqs);
                    *pv_mut = *pv.as_array();
                    *mv_mut = *mv.as_array();

                    viz.expand_blocks_simple(
                        from_fn(|l| {
                            Pos(
                                (i + L * N - 1 - offset - l) as I,
                                ((j + offset + l) as I - padding as I) * W as I + 1,
                            )
                        })
                        .into(),
                        [Pos(1, W as I); L],
                    );
                }
            }
        }

        bottom_row_score += ph.into_iter().sum::<B>() as D - mh.into_iter().sum::<B>() as D;
    }

    // Do simple per-column scan for the remaining cols.
    for c in chunks.remainder() {
        let h = &mut (1u8, 0u8);
        for (pv, mv, block_profile) in izip!(pv.iter_mut(), mv.iter_mut(), &b) {
            let v = &mut V::from(*pv, *mv);
            compute_block(h, v, block_profile[*c as usize]);
            (*pv, *mv) = v.pm();
        }
        bottom_row_score += h.value();
    }

    viz.last_frame_simple();
    bottom_row_score
}
