use std::cmp::min;

use itertools::izip;
use pa_types::Cost;

use crate::{myers, profile::Profile, HEncoding, V};

/// Compute a rectangle column by column.
pub fn col<P: Profile, H: HEncoding>(a: &[P::A], b: &[P::B], h: &mut [H], v: &mut [V]) -> Cost {
    assert_eq!(a.len(), h.len());
    assert_eq!(b.len(), v.len());
    for (ca, h) in izip!(a.iter(), h.iter_mut()) {
        for (cb, v) in izip!(b, v.iter_mut()) {
            myers::compute_block::<P, H>(h, v, ca, cb);
        }
    }
    h.iter().map(|h| h.value()).sum::<Cost>()
}

/// Compute a rectangle column by column, assuming horizontal input deltas of 1.
///
/// Slightly faster than `col_with_h` below because `h` can be local.
pub fn col_local_h<P: Profile, H: HEncoding>(a: &[P::A], b: &[P::B], v: &mut [V]) -> Cost {
    assert_eq!(b.len(), v.len());
    let mut bot_delta = 0;
    for ca in a.iter() {
        let h = &mut H::one();
        for (cb, v) in izip!(b, v.iter_mut()) {
            myers::compute_block::<P, H>(h, v, ca, cb);
        }
        bot_delta += h.value();
    }
    bot_delta
}

/// Compute a rectangle row by row.
pub fn row<P: Profile, H: HEncoding>(a: &[P::A], b: &[P::B], h: &mut [H], v: &mut [V]) -> Cost {
    assert_eq!(a.len(), h.len());
    assert_eq!(b.len(), v.len());
    for (cb, v) in izip!(b, v.iter_mut()) {
        for (ca, h) in izip!(a.iter(), h.iter_mut()) {
            myers::compute_block::<P, H>(h, v, ca, cb);
        }
    }
    h.iter().map(|h| h.value()).sum::<Cost>()
}

/// Diagonal by diagonal, where each diag goes right-up.
// diagonal number d:
// 1 2 3 .. n
// 2 3 4 .. n+1
// 3 4 5 .. n+2
// ...
// m m+1 m+2 .. m+n-1
pub fn diag_ru<P: Profile, H: HEncoding>(a: &[P::A], b: &[P::B], h: &mut [H], v: &mut [V]) -> Cost {
    assert_eq!(a.len(), h.len());
    assert_eq!(b.len(), v.len());
    let n = a.len();
    let m = b.len();

    for d in 1..n + m {
        let i = d.saturating_sub(m)..min(d, n);
        let j = d.saturating_sub(n)..min(d, m);
        assert!(i.len() == j.len());
        let a = &a[i.clone()];
        let b = &b[j.clone()];
        let h = &mut h[i];
        let v = &mut v[j];
        for (ca, cb, h, v) in izip!(a, b.iter().rev(), h, v.iter_mut().rev()) {
            myers::compute_block::<P, H>(h, v, ca, cb);
        }
    }

    h.iter().map(|h| h.value()).sum::<Cost>()
}

/// Diagonal by diagonal, where each diag goes left-down.
// Same as previous, but inner loop is reversed.
pub fn diag_ld<P: Profile, H: HEncoding>(a: &[P::A], b: &[P::B], h: &mut [H], v: &mut [V]) -> Cost {
    assert_eq!(a.len(), h.len());
    assert_eq!(b.len(), v.len());
    let n = a.len();
    let m = b.len();

    for d in 1..n + m {
        let i = d.saturating_sub(m)..min(d, n);
        let j = d.saturating_sub(n)..min(d, m);
        assert!(i.len() == j.len());
        let a = &a[i.clone()];
        let b = &b[j.clone()];
        let h = &mut h[i];
        let v = &mut v[j];
        for (ca, cb, h, v) in izip!(a, b.iter().rev(), h, v.iter_mut().rev()).rev() {
            myers::compute_block::<P, H>(h, v, ca, cb);
        }
    }

    h.iter().map(|h| h.value()).sum::<Cost>()
}

/// Do N columns at a time.
pub fn cols_ru<const N: usize, P: Profile, H: HEncoding>(
    a: &[P::A],
    b: &[P::B],
    h: &mut [H],
    v: &mut [V],
) -> Cost {
    assert_eq!(a.len(), h.len());
    assert_eq!(b.len(), v.len());

    if b.len() < N {
        return 0;
    }

    let rev = |k| N - 1 - k;

    let a_chunks = a.array_chunks::<N>();
    let h_chunks = h.array_chunks_mut::<N>();
    for (cas, hs) in izip!(a_chunks, h_chunks) {
        // Do the top-left triangle.
        for i in 0..N {
            for j in 0..N - i - 1 {
                myers::compute_block::<P, H>(&mut hs[i], &mut v[j], &cas[i], &b[j]);
            }
        }
        // NOTE: array_windows_mut would be nice to avoid the manual index.
        for (j, cbs) in b.array_windows::<N>().enumerate() {
            for k in 0..N {
                myers::compute_block::<P, H>(&mut hs[k], &mut v[j + rev(k)], &cas[k], &cbs[rev(k)]);
            }
        }
        // Do the top-left triangle.
        for i in 0..N {
            for j in b.len() - i..b.len() {
                myers::compute_block::<P, H>(&mut hs[i], &mut v[j], &cas[i], &b[j]);
            }
        }
    }

    // Do simple per-column scan for the remaining cols.
    let a_chunks = a.array_chunks::<N>();
    let h_chunks = h.array_chunks_mut::<N>();
    for (ca, h) in izip!(a_chunks.remainder(), h_chunks.into_remainder()) {
        for (cb, v) in izip!(b, v.iter_mut()) {
            myers::compute_block::<P, H>(h, v, ca, cb);
        }
    }

    h.iter().map(|h| h.value()).sum::<Cost>()
}

/// Do N columns at a time.
pub fn cols_ld<const N: usize, P: Profile, H: HEncoding>(
    a: &[P::A],
    b: &[P::B],
    h: &mut [H],
    v: &mut [V],
) -> Cost {
    assert_eq!(a.len(), h.len());
    assert_eq!(b.len(), v.len());

    if b.len() < N {
        return 0;
    }

    let rev = |k| N - 1 - k;

    let a_chunks = a.array_chunks::<N>();
    let h_chunks = h.array_chunks_mut::<N>();
    for (cas, hs) in izip!(a_chunks, h_chunks) {
        // Do the top-left triangle.
        for i in 0..N {
            for j in 0..N - i - 1 {
                myers::compute_block::<P, H>(&mut hs[i], &mut v[j], &cas[i], &b[j]);
            }
        }
        // NOTE: array_windows_mut would be nice to avoid the manual index.
        for (j, cbs) in b.array_windows::<N>().enumerate() {
            for k in (0..N).rev() {
                myers::compute_block::<P, H>(&mut hs[k], &mut v[j + rev(k)], &cas[k], &cbs[rev(k)]);
            }
        }
        // Do the top-left triangle.
        for i in 0..N {
            for j in b.len() - i..b.len() {
                myers::compute_block::<P, H>(&mut hs[i], &mut v[j], &cas[i], &b[j]);
            }
        }
    }

    // Do simple per-column scan for the remaining cols.
    let a_chunks = a.array_chunks::<N>();
    let h_chunks = h.array_chunks_mut::<N>();
    for (ca, h) in izip!(a_chunks.remainder(), h_chunks.into_remainder()) {
        for (cb, v) in izip!(b, v.iter_mut()) {
            myers::compute_block::<P, H>(h, v, ca, cb);
        }
    }

    h.iter().map(|h| h.value()).sum::<Cost>()
}

/// Do N columns at a time.
pub fn cols_ru_local_h<const N: usize, P: Profile, H: HEncoding>(
    a: &[P::A],
    b: &[P::B],
    v: &mut [V],
) -> Cost {
    assert_eq!(b.len(), v.len());

    if b.len() < N {
        return 0;
    }

    let mut bot_delta = 0;

    let rev = |k| N - 1 - k;

    for cas in a.array_chunks::<N>() {
        let hs = &mut [H::one(); N];
        // Do the top-left triangle.
        for i in 0..N {
            for j in 0..N - i - 1 {
                myers::compute_block::<P, H>(&mut hs[i], &mut v[j], &cas[i], &b[j]);
            }
        }
        // NOTE: array_windows_mut would be nice to avoid the manual index.
        for (j, cbs) in b.array_windows::<N>().enumerate() {
            for k in 0..N {
                myers::compute_block::<P, H>(&mut hs[k], &mut v[j + rev(k)], &cas[k], &cbs[rev(k)]);
            }
        }
        // Do the top-left triangle.
        for i in 0..N {
            for j in b.len() - i..b.len() {
                myers::compute_block::<P, H>(&mut hs[i], &mut v[j], &cas[i], &b[j]);
            }
        }
        bot_delta += hs.iter().map(|h| h.value()).sum::<Cost>()
    }

    // Do simple per-column scan for the remaining cols.
    for ca in a.array_chunks::<N>().remainder() {
        let h = &mut H::one();
        for (cb, v) in izip!(b, v.iter_mut()) {
            myers::compute_block::<P, H>(h, v, ca, cb);
        }
        bot_delta += h.value();
    }

    bot_delta
}

/// Do N columns at a time.
pub fn cols_ld_local_h<const N: usize, P: Profile, H: HEncoding>(
    a: &[P::A],
    b: &[P::B],
    v: &mut [V],
) -> Cost {
    assert_eq!(b.len(), v.len());

    if b.len() < N {
        return 0;
    }

    let mut bot_delta = 0;

    let rev = |k| N - 1 - k;

    for cas in a.array_chunks::<N>() {
        let hs = &mut [H::one(); N];
        // Do the top-left triangle.
        for i in 0..N {
            for j in 0..N - i - 1 {
                myers::compute_block::<P, H>(&mut hs[i], &mut v[j], &cas[i], &b[j]);
            }
        }
        // NOTE: array_windows_mut would be nice to avoid the manual index.
        for (j, cbs) in b.array_windows::<N>().enumerate() {
            for k in (0..N).rev() {
                myers::compute_block::<P, H>(&mut hs[k], &mut v[j + rev(k)], &cas[k], &cbs[rev(k)]);
            }
        }
        // Do the top-left triangle.
        for i in 0..N {
            for j in b.len() - i..b.len() {
                myers::compute_block::<P, H>(&mut hs[i], &mut v[j], &cas[i], &b[j]);
            }
        }
        bot_delta += hs.iter().map(|h| h.value()).sum::<Cost>()
    }

    // Do simple per-column scan for the remaining cols.
    for ca in a.array_chunks::<N>().remainder() {
        let h = &mut H::one();
        for (cb, v) in izip!(b, v.iter_mut()) {
            myers::compute_block::<P, H>(h, v, ca, cb);
        }
        bot_delta += h.value();
    }

    bot_delta
}
/// Do N rows at a time.
pub fn rows_ru<const N: usize, P: Profile, H: HEncoding>(
    a: &[P::A],
    b: &[P::B],
    h: &mut [H],
    v: &mut [V],
) -> Cost {
    assert_eq!(a.len(), h.len());
    assert_eq!(b.len(), v.len());

    if b.len() < N {
        return 0;
    }

    let rev = |k| N - 1 - k;

    let b_chunks = b.array_chunks::<N>();
    let v_chunks = v.array_chunks_mut::<N>();
    for (cbs, vs) in izip!(b_chunks, v_chunks) {
        // Do the top-left triangle.
        for j in 0..N {
            for i in 0..N - j - 1 {
                myers::compute_block::<P, H>(&mut h[i], &mut vs[j], &a[i], &cbs[j]);
            }
        }
        // NOTE: array_windows_mut would be nice to avoid the manual index.
        for (i, cas) in a.array_windows::<N>().enumerate() {
            for k in 0..N {
                myers::compute_block::<P, H>(&mut h[i + k], &mut vs[rev(k)], &cas[k], &cbs[rev(k)]);
            }
        }
        // Do the bot-right triangle.
        for j in 0..N {
            for i in a.len() - j..a.len() {
                myers::compute_block::<P, H>(&mut h[i], &mut vs[j], &a[i], &cbs[j]);
            }
        }
    }

    // Do simple per-column scan for the remaining cols.
    let b_chunks = b.array_chunks::<N>();
    let v_chunks = v.array_chunks_mut::<N>();
    for (cb, v) in izip!(b_chunks.remainder(), v_chunks.into_remainder()) {
        for (ca, h) in izip!(a, h.iter_mut()) {
            myers::compute_block::<P, H>(h, v, ca, cb);
        }
    }

    h.iter().map(|h| h.value()).sum::<Cost>()
}

/// Do N rows at a time.
pub fn rows_ld<const N: usize, P: Profile, H: HEncoding>(
    a: &[P::A],
    b: &[P::B],
    h: &mut [H],
    v: &mut [V],
) -> Cost {
    assert_eq!(a.len(), h.len());
    assert_eq!(b.len(), v.len());

    if b.len() < N {
        return 0;
    }

    let rev = |k| N - 1 - k;

    let b_chunks = b.array_chunks::<N>();
    let v_chunks = v.array_chunks_mut::<N>();
    for (cbs, vs) in izip!(b_chunks, v_chunks) {
        // Do the top-left triangle.
        for j in 0..N {
            for i in 0..N - j - 1 {
                myers::compute_block::<P, H>(&mut h[i], &mut vs[j], &a[i], &cbs[j]);
            }
        }
        // NOTE: array_windows_mut would be nice to avoid the manual index.
        for (i, cas) in a.array_windows::<N>().enumerate() {
            for k in (0..N).rev() {
                myers::compute_block::<P, H>(&mut h[i + k], &mut vs[rev(k)], &cas[k], &cbs[rev(k)]);
            }
        }
        // Do the top-left triangle.
        for j in 0..N {
            for i in a.len() - j..a.len() {
                myers::compute_block::<P, H>(&mut h[i], &mut vs[j], &a[i], &cbs[j]);
            }
        }
    }

    // Do simple per-column scan for the remaining cols.
    let b_chunks = b.array_chunks::<N>();
    let v_chunks = v.array_chunks_mut::<N>();
    for (cb, v) in izip!(b_chunks.remainder(), v_chunks.into_remainder()) {
        for (ca, h) in izip!(a, h.iter_mut()) {
            myers::compute_block::<P, H>(h, v, ca, cb);
        }
    }

    h.iter().map(|h| h.value()).sum::<Cost>()
}

/// Same as `compute`, but returns all computed value.
pub fn fill<P: Profile, H: HEncoding>(
    a: &[P::A],
    b: &[P::B],
    h: &mut [H],
    v: &mut [V],
    values: &mut [Vec<V>],
) -> Cost {
    assert_eq!(a.len(), h.len());
    assert_eq!(values.len(), h.len());
    assert_eq!(b.len(), v.len());
    for vv in values.iter_mut() {
        vv.resize(v.len(), V::default());
    }
    for i in 0..a.len() {
        for j in 0..b.len() {
            myers::compute_block::<P, H>(&mut h[i], &mut v[j], &a[i], &b[j]);
        }
        values[i].copy_from_slice(v);
    }
    return h.iter().map(|h| h.value()).sum::<Cost>();
}
