//! Local Pruning
//!
//! When a match (length k, cost < r) is followed by a noisy area, we can effectively remove the match.
//! The 'local pruning parameter' `p` gives the number of seeds of 'look-ahead'.
//! p=0 disables local pruning.
//! For p>0, we consider the upcoming `p` seeds, and for each prefix of `i` of
//! those seeds, the cost to traverse those `i` seeds must be less than `(i+1)*r`.
//!
//! Implementation:
//! We use diagonal transition (DT) since distances are small.
//! TODO: Test NW-DP as well
//! TODO: If extending a match runs into another preserved match, we can stop extending and keep the match.

use std::{
    cmp::{max, min},
    mem::swap,
    simd::{Simd, SimdPartialEq, ToBitMask},
};

use super::{CenteredVec, Match};
use crate::seeds::Seeds;
use pa_types::{Cost, Seq, I};

/// Returns true when `end_i` is reached.
fn extend(a: Seq, b: Seq, i: &mut I, mut j: I, end_i: I) -> bool {
    while *i < end_i && j < b.len() as I && a[*i as usize] == b[j as usize] {
        *i += 1;
        j += 1;
    }
    *i >= end_i
}

/// Same as `extend` but uses SIMD.
/// TODO: We can also try a version that does 8 chars at a time using `u64`s.
fn extend_simd(a: Seq, b: Seq, i: &mut I, mut j: I, end_i: I) -> bool {
    // Do the first char manually to throw away some easy bad cases before going into SIMD.
    if *i < a.len() as I && j < b.len() as I {
        if a[*i as usize] == b[j as usize] {
            *i += 1;
            j += 1;
        } else {
            return *i >= end_i;
        }
    } else {
        return *i >= end_i;
    }
    while *i < a.len() as I - 32 && j < b.len() as I - 32 {
        let simd_a: Simd<u8, 32> = Simd::from_array(*a[*i as usize..].split_array_ref().0);
        let simd_b: Simd<u8, 32> = Simd::from_array(*b[j as usize..].split_array_ref().0);
        let eq = simd_a.simd_eq(simd_b).to_bitmask();
        let cnt = if cfg!(target_endian = "little") {
            eq.trailing_ones() as I
        } else {
            eq.leading_ones() as I
        };
        *i += cnt;
        j += cnt;
        if cnt < 32 {
            return *i >= end_i;
        }
        if *i >= end_i {
            return true;
        }
    }
    extend(a, b, i, j, end_i)
}

/// Returns `false` for matches that should be removed by local pruning.
/// After covering `i <= p` additional seeds, the cost should be less than `(i+1)*r`.
/// For an optimal path from `s = start(match)` to `t=end of path after covering i seeds`, we should have
/// g(t) + p(t) <= p(s) to keep the match.
///
/// The last argument is a reusable buffer for the DT fronts that can simply be `&mut Default::default()`.
pub(super) fn preserve_for_local_pruning(
    a: Seq,
    b: Seq,
    seeds: &Seeds,
    m: &Match,
    p: usize,
    [fr, next_fr, stats]: &mut [Vec<I>; 3],
    next_match_per_diag: &mut CenteredVec<I>,
) -> bool {
    let s = m.start;
    let start_pot = seeds.potential(s);
    let seed_idx = seeds.seed_at[s.0 as usize].unwrap();
    // Near the end, fewer than `p` seeds are considered.
    let last_seed = &seeds.seeds[min(seed_idx as usize + p, seeds.seeds.len() - 1)];
    let end_i = last_seed.end;
    let end_pot = seeds.potential[end_i as usize];

    let pd = (start_pot - end_pot) as usize;

    // Reinitialize the fronts.
    // They are reused between calls to save allocations.
    fr.resize(2 * pd + 1, I::MIN);
    next_fr.resize(2 * pd + 1, I::MIN);
    if pd > stats.len() {
        stats.resize(pd, 0);
    }

    // d: the diagonal relative to s.
    // d=1: the diagonal above s.
    let mut d_range = pd..pd + 1;
    // Initialize the first front.
    fr[pd] = s.0;
    next_fr[pd] = I::MIN;

    if extend_simd(a, b, &mut fr[pd], s.1, end_i) {
        stats[0] += 1;
        return true;
    }
    if next_match_per_diag.index(s.0 - s.1) <= fr[pd] {
        stats[0] += 1;
        return true;
    }

    for g in 1..pd as Cost {
        fr[d_range.start - 1] = I::MIN;
        fr[d_range.end] = I::MIN;
        next_fr[d_range.start - 1] = I::MIN;
        next_fr[d_range.end] = I::MIN;
        // expand
        for d in d_range.clone() {
            next_fr[d - 1] = max(next_fr[d - 1], fr[d]);
            next_fr[d] = max(next_fr[d], fr[d] + 1);
            next_fr[d + 1] = max(next_fr[d + 1], fr[d] + 1);
        }
        swap(fr, next_fr);

        d_range = (d_range.start - 1)..(d_range.end + 1);

        // for d in d_range.clone() {
        //     assert!(fr[d] <= end_i);
        // }

        // check & shrink
        while !d_range.is_empty()
            && g + seeds.potential[fr[d_range.start as usize] as usize] >= start_pot
        {
            d_range.start += 1;
        }
        while !d_range.is_empty()
            && g + seeds.potential[fr[d_range.end as usize - 1] as usize] >= start_pot
        {
            d_range.end -= 1;
        }
        if d_range.is_empty() {
            stats[g as usize] += 1;
            return false;
        }

        // extend
        for d in d_range.clone() {
            let i = &mut fr[d];
            let dd = s.0 - s.1 + (d as I - pd as I);
            let j = *i - dd;
            let old_i = *i;

            // If reached end of range => KEEP MATCH.
            if extend_simd(a, b, i, j, end_i) {
                stats[g as usize] += 1;
                return true;
            }

            // If reached *the start* of an existing match => KEEP MATCH.
            // We check that the start is covered by the current extend.
            if old_i <= next_match_per_diag.index(dd) && next_match_per_diag.index(dd) <= *i {
                stats[g as usize] += 1;
                return true;
            }
        }
    }
    // Did not find a path with cost < pd to `end_i`.
    false
}
