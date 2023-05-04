use super::*;
use itertools::izip;
use pa_bitpacking::simd::*;
use pa_types::{Pos, I};
use pa_vis_types::{VisualizerInstance, VisualizerT};
use std::array::from_fn;

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

/// TODO optimizations:
/// - Reverse a or b in memory, so that anti-diagonals align.
/// - Reverse ph and pm in memory?
/// - Reverse for-loop order.
/// - Skip vectors completely out-of-bounds.
/// NOTE: This requires padded profiles, because SIMD vecs can go out-of-bounds.
pub fn nw_simd_striped_col<const N: usize>(
    a: CompressedSeq,
    b: CompressedSeq,
    viz: &impl VisualizerT,
) -> D
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

pub fn nw_simd_striped_row_wrapper<const N: usize>(a: CompressedSeq, b: CompressedSeq) -> D
where
    [(); L * N]: Sized,
{
    let b = padded_profile(b, 0);

    let mut v = vec![V::one(); b.len()];
    nw_simd_striped_row::<N>(&a, &b, &mut v)
}
