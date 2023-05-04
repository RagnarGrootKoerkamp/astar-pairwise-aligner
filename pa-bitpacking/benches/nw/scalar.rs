use std::cmp::min;

use super::*;
use itertools::izip;
use pa_types::{Pos, I};
use pa_vis_types::{VisualizerInstance, VisualizerT};

/// Compute row-by-row or column-by-column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumIter)]
pub enum Order {
    Row,
    Col,
}

/// Compute diagonals in the up-right or down-left direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumIter)]
pub enum Direction {
    RightUp,
    LeftDown,
}

/// Local memory per row/column.
pub fn nw<H: HEncoding>(
    a: CompressedSeq,
    b: ProfileSlice,
    order: Order,
    viz: &impl VisualizerT,
) -> D {
    let ref mut viz = viz.build(&a, &a);
    let mut bottom_row_score = b.len() as D * W as D;

    match order {
        Order::Row => {
            let mut h = vec![H::one(); a.len()];
            for (cb, j) in b.iter().zip((1..).step_by(W)) {
                let v = &mut V::one();
                for ((ca, h), i) in izip!(a.iter(), h.iter_mut()).zip(1..) {
                    compute_block(h, v, cb[*ca as usize]);
                    viz.expand_block_simple(Pos(i, j), Pos(1, W as I));
                }
            }
            bottom_row_score += h.iter().map(|h| h.value()).sum::<D>();
        }
        Order::Col => {
            let mut v = vec![V::one(); b.len()];
            for (ca, i) in a.iter().zip(1..) {
                let h = &mut H::one();
                for ((cb, v), j) in izip!(b, v.iter_mut()).zip((1..).step_by(W)) {
                    compute_block(h, v, unsafe { *cb.get_unchecked(*ca as usize) });
                    viz.expand_block_simple(Pos(i, j), Pos(1, W as I));
                }
                bottom_row_score += h.value();
            }
        }
    }

    viz.last_frame_simple();
    bottom_row_score
}

pub fn nw_diag<H: HEncoding>(
    a: CompressedSeq,
    b: ProfileSlice,
    direction: Direction,
    viz: &impl VisualizerT,
) -> D {
    let ref mut viz = viz.build(&a, &a);

    let mut bottom_row_score = b.len() as D * W as D;
    let n = a.len();
    let m = b.len();

    // diagonal number d:
    // 1 2 3 .. n
    // 2 3 4 .. n+1
    // 3 4 5 .. n+2
    // ...
    // m m+1 m+2 .. m+n-1
    let mut h = vec![H::one(); a.len()];
    let mut v = vec![V::one(); m];
    match direction {
        // Duplicate outer loop to ensure good optimization.
        Direction::RightUp => {
            for d in 1..n + m {
                let i = d.saturating_sub(m);
                let ie = min(d, n);
                let j = d.saturating_sub(n);
                let je = min(d, m);
                let a = &a[i..ie];
                let b = &b[j..je];
                let h = &mut h[i..ie];
                let v = &mut v[j..je];
                assert!(a.len() == b.len());
                for ((ca, cb, h, v), k) in izip!(a, b.iter().rev(), h, v.iter_mut().rev()).zip(0..)
                {
                    compute_block(h, v, cb[*ca as usize]);
                    viz.expand_block_simple(
                        Pos(i as I + k + 1, (je as I - k - 1) * W as I + 1),
                        Pos(1, W as I),
                    );
                }
            }
        }
        Direction::LeftDown => {
            for d in 1..n + m {
                let i = d.saturating_sub(m);
                let ie = min(d, n);
                let j = d.saturating_sub(n);
                let je = min(d, m);
                let a = &a[i..ie];
                let b = &b[j..je];
                let h = &mut h[i..ie];
                let v = &mut v[j..je];
                assert!(a.len() == b.len());
                for ((ca, cb, h, v), k) in izip!(a.iter().rev(), b, h.iter_mut().rev(), v).zip(0..)
                {
                    compute_block(h, v, cb[*ca as usize]);
                    viz.expand_block_simple(
                        Pos(ie as I - k, (j as I + k) * W as I + 1),
                        Pos(1, W as I),
                    );
                }
            }
        }
    }
    viz.last_frame_simple();
    bottom_row_score += h.iter().map(|h| h.value()).sum::<D>();
    bottom_row_score
}

/// Do N columns in parallel at a time.
pub fn nw_striped_col<const N: usize, H: HEncoding>(
    a: CompressedSeq,
    b: ProfileSlice,
    direction: Direction,
    viz: &impl VisualizerT,
) -> D {
    let ref mut viz = viz.build(&a, &a);

    let mut bottom_row_score = b.len() as D * W as D;
    let padding = N - 1;

    let mut v = vec![V::one(); b.len()];

    let chunks = a.array_chunks::<N>();
    match direction {
        Direction::RightUp => {
            for (cas, i) in chunks.clone().zip((1..).step_by(N)) {
                let mut h = [H::one(); N];
                // NOTE: array_windows_mut would be nice.
                for (j, cbs) in b.array_windows::<N>().enumerate() {
                    for k in 0..N {
                        compute_block(
                            &mut h[k],
                            &mut v[j + N - 1 - k],
                            cbs[N - 1 - k][cas[k] as usize],
                        );
                        viz.expand_block_simple(
                            Pos(
                                (i + k) as I,
                                ((j + N - 1 - k) as I - padding as I) * W as I + 1,
                            ),
                            Pos(1, W as I),
                        );
                    }
                }

                bottom_row_score += h.into_iter().map(|h| h.value()).sum::<D>();
            }
        }
        Direction::LeftDown => {
            for (cas, i) in chunks.clone().zip((1..).step_by(N)) {
                let mut h = [H::one(); N];
                // NOTE: array_windows_mut would be nice.
                for (j, cbs) in b.array_windows::<N>().enumerate() {
                    for k in 0..N {
                        compute_block(&mut h[k], &mut v[j + k], cbs[k][cas[N - 1 - k] as usize]);
                        viz.expand_block_simple(
                            Pos(
                                (i + N - 1 - k) as I,
                                ((j + k) as I - padding as I) * W as I + 1,
                            ),
                            Pos(1, W as I),
                        );
                    }
                }

                bottom_row_score += h.into_iter().map(|h| h.value()).sum::<D>();
            }
        }
    }

    // Do simple per-column scan for the remaining cols.
    for c in chunks.remainder() {
        let h = &mut H::one();
        for (v, block_profile) in izip!(v.iter_mut(), b) {
            compute_block(h, v, block_profile[*c as usize]);
        }
        bottom_row_score += h.value();
    }

    viz.last_frame_simple();
    bottom_row_score
}

/// Do N rows in parallel at a time.
#[allow(unused)]
pub fn nw_striped_row<const N: usize, H: HEncoding>(
    _a: CompressedSeq,
    _b: ProfileSlice,
    _direction: Direction,
) -> D {
    todo!();
}
