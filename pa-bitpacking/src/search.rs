use super::*;
use pa_types::Cost;

/// Search a short pattern in a long text.
/// Text must be `actgACTG` only.
/// Pattern may contain `nN` or `*` as wildcard, and `yY` to match `C` or `T`.
///
/// Partial matches of the pattern are allowed:
/// the alignment can start anywhere along the top or left of the matrix.
///
/// Output is the vector of costs along the bottom and right of the matrix.
///
/// Example:
/// ```
/// let out = pa_bitpacking::search::search(b"AC", b"CTTACTTA");
/// assert_eq!(out, vec![0, 0, 1, 2, 1, 0, 1, 2, 1, 0, 0]);
/// ```
///
/// ```text
///    C T T A C T T A
///   0 0 0 0 0 0 0 0 0
/// A
///   0 1 1 1 0 1 1 1 0
/// C
///   0 0 1 2 1 0 1 2 1 < output
///                   ^ output
/// ```
/// The bottom row and right column (in reverse) are the output, of total length `|pattern| + |text| + 1`.
///
/// TODO: Optionally disallow partial matches of the pattern, and only output the bottom row.
pub fn search(pattern: &[u8], text: &[u8]) -> Vec<Cost> {
    let bot_left;
    let mut h;
    let mut v;
    type P = ScatterProfile;
    let (t, p) = P::build(text, pattern);
    h = vec![<(u8, u8)>::zero(); t.len()];
    v = vec![V::zero(); p.len()];

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
            assert_eq!(b, 0);
        } else {
            out_vec.push(b);
        }
    }

    // Fix since we round up to multiple of 64 chars.
    for v in v.iter().rev() {
        for j in 1..=64 {
            let delta = v.value_of_suffix(j as _);
            let val = b - delta;
            if skipped < extra {
                skipped += 1;
                assert_eq!(b, 0);
            } else {
                out_vec.push(val);
            }
        }
        b -= v.value();
    }
    assert_eq!(b, 0);
    assert_eq!(out_vec.len(), pattern.len() + text.len() + 1);
    out_vec
}
