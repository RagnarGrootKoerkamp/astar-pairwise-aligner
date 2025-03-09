use super::*;
use pa_types::Cost;

/// Search a short pattern in a long text.
/// Text must be `actgACTG` only.
/// Pattern may contain `nN` or `*` as wildcard, and `yY` to match `C` or `T`.
///
/// Partial matches of the pattern are allowed:
/// the alignment can start anywhere along the top or left of the matrix.
/// The cost of unmatched characters is `unmatched_cost`, which must be between `0.0` and `1.0`.
/// When set to e.g. `0.5`, half the bits on the left column are set to `1`.
///
/// Output is the vector of costs along the bottom and right of the matrix.
///
/// Example:
/// ```
/// let out = pa_bitpacking::search::search(b"AC", b"CTTACTTA", 0.0);
/// assert_eq!(out, vec![0, 0, 1, 2, 1, 0, 1, 2, 1, 0, 0]);
/// ```
///
/// ```text
///    C T T A C T T A
///   0 0 0 0 0 0 0 0 0 < zeros (start anywhere in text)
/// A
///   0 1 1 1 0 1 1 1 0
/// C
///   0 0 1 2 1 0 1 2 1 < output
///   ^               ^ output
///   |
///   zeros (start anywhere in pattern)
/// ```
/// The bottom row and right column (in reverse) are the output, of total length `|pattern| + |text| + 1`.
///
/// TODO: Optionally disallow partial matches of the pattern, and only output the bottom row.
pub fn search(pattern: &[u8], text: &[u8], unmatched_cost: f32) -> Vec<Cost> {
    let bot_left;
    let mut h;
    let mut v;
    type P = ScatterProfile;
    let (t, p) = P::build(text, pattern);
    h = vec![<(u8, u8)>::zero(); t.len()];
    let mut v_unmatched = vec![V::zero(); p.len()];

    assert!(unmatched_cost >= 0.0 && unmatched_cost <= 1.0);
    if unmatched_cost > 0.0 {
        for i in 0.. {
            let idx = (i as f32 / unmatched_cost).ceil() as usize;
            if idx >= pattern.len() {
                break;
            }
            *v_unmatched[idx / 64].one_mut() |= 1 << (idx % 64);
        }
    }
    v = v_unmatched.clone();

    bot_left = v.iter().map(|x| x.value()).sum::<i32>();

    crate::simd::scatter_profile::compute::<2, _, 4>(&t, &p, &mut h, &mut v, true);

    let mut b = bot_left;
    let mut out_vec = vec![b];
    let extra = pattern.len().next_multiple_of(64) - pattern.len();
    let mut skipped = 0;
    for x in h {
        b += x.value();
        if skipped < extra {
            skipped += 1;
        } else {
            out_vec.push(b);
        }
    }

    // Fix since we round up to multiple of 64 chars.
    for (v, vu) in std::iter::zip(v, v_unmatched).rev() {
        for j in 1..=64 {
            let delta = v.value_of_suffix(j as _);
            let unmatched = vu.value_of_suffix(j as _);
            let val = b - delta + unmatched;
            if skipped < extra {
                skipped += 1;
            } else {
                out_vec.push(val);
            }
        }
        b -= v.value();
        b += vu.value();
    }
    assert_eq!(out_vec.len(), pattern.len() + text.len() + 1);
    out_vec
}
