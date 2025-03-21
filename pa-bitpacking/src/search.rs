use super::*;
use pa_types::{Cigar, CigarOp, Cost, Pos, I};

type P = ScatterProfile;

pub struct SearchResult<'s> {
    pub out: Vec<Cost>,
    text: &'s [u8],
    pattern: &'s [u8],
    // sketches
    t: Vec<<P as Profile>::A>,
    p: Vec<<P as Profile>::B>,
    _padding: usize,
    v0: Vec<V>,
}

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
pub fn search<'s>(pattern: &'s [u8], text: &'s [u8], unmatched_cost: f32) -> SearchResult<'s> {
    let bot_left;
    let mut h;
    let mut v;
    type P = ScatterProfile;
    let (t, p) = P::build(text, pattern);
    h = vec![<(u8, u8)>::zero(); t.len()];
    let mut v0 = vec![V::zero(); p.len()];
    let padding = pattern.len().next_multiple_of(64) - pattern.len();

    assert!(unmatched_cost >= 0.0 && unmatched_cost <= 1.0);
    if unmatched_cost > 0.0 {
        for i in 0.. {
            let idx = (i as f32 / unmatched_cost).ceil() as usize;
            if idx >= pattern.len() {
                break;
            }
            *v0[idx / 64].one_mut() |= 1 << (idx % 64);
        }
    }
    v = v0.clone();

    bot_left = v.iter().map(|x| x.value()).sum::<i32>();

    let mut out = vec![];
    crate::simd::scatter_profile::compute::<2, _, 4, false>(&t, &p, &mut h, &mut v, true, &mut out);

    let mut b = bot_left;
    let mut out_vec = vec![b];
    let mut skipped = 0;
    for x in h {
        b += x.value();
        if skipped < padding {
            skipped += 1;
        } else {
            out_vec.push(b);
        }
    }

    // Fix since we round up to multiple of 64 chars.
    for (v, vu) in std::iter::zip(v, &v0).rev() {
        for j in 1..=64 {
            let delta = v.value_of_suffix(j as _);
            let unmatched = vu.value_of_suffix(j as _);
            let val = b - delta + unmatched;
            if skipped < padding {
                skipped += 1;
            } else {
                out_vec.push(val);
            }
        }
        b -= v.value();
        b += vu.value();
    }
    assert_eq!(out_vec.len(), pattern.len() + text.len() + 1);
    SearchResult {
        out: out_vec,
        text,
        pattern,
        t,
        p,
        _padding: padding,
        v0,
    }
}

impl<'s> SearchResult<'s> {
    pub fn idx_to_pos(&self, idx: usize) -> Pos {
        assert!(idx < self.out.len());
        if idx <= self.text.len() {
            Pos(idx as _, self.pattern.len() as _)
        } else {
            Pos(
                self.text.len() as _,
                (self.pattern.len() - (idx - self.text.len())) as _,
            )
        }
    }

    pub fn trace(&self, idx: usize) -> (Cigar, Vec<Pos>) {
        let mut pos = self.idx_to_pos(idx);
        let mut target_cost = self.out[idx];
        if pos.0 as usize == self.text.len() {
            target_cost -= V::value_from(&self.v0, pos.1);
        }

        let mut width = 2 * self.pattern.len();

        let end = pos.0 as usize;
        let mut start;
        let mut h;
        let mut v;
        let mut fill;

        loop {
            start = end.saturating_sub(width);

            h = vec![<(u8, u8)>::zero(); end - start + 1];
            v = if start == 0 {
                self.v0.clone()
            } else {
                vec![V::one(); self.v0.len()]
            };

            fill = vec![vec![]; h.len()];
            fill[0] = v.clone();
            crate::simd::scatter_profile::compute::<2, _, 4, true>(
                &self.t[start..end],
                &self.p,
                &mut h[1..],
                &mut v,
                true,
                &mut fill[1..],
            );

            debug_assert_eq!(&v, fill.last().unwrap());
            let cost = V::value_to(&v, pos.1);

            assert!(
                cost >= target_cost,
                "Target cost {target_cost}, but found trace path of cost {cost}"
            );
            if cost == target_cost {
                break;
            }

            width *= 2;
        }

        let cost = |Pos(i, j)| -> Cost { V::value_to(&fill[i as usize - start], j) };

        let mut cigar = Cigar { ops: vec![] };
        let mut poss = vec![pos];
        let mut g = target_cost;

        while pos.0 > start as I && pos.1 > 0 {
            let mut cnt = 0;
            while pos.0 > start as I
                && pos.1 > 0
                && P::is_match(&self.t, &self.p, pos.0 - 1, pos.1 - 1)
            {
                cnt += 1;
                pos.0 -= 1;
                pos.1 -= 1;
                poss.push(pos);
            }
            if cnt > 0 {
                cigar.push_matches(cnt);
                continue;
            }
            if cost(Pos(pos.0 - 1, pos.1)) == g - 1 {
                g -= 1;
                pos.0 -= 1;
                poss.push(pos);
                cigar.push(CigarOp::Del);
                continue;
            }
            if cost(Pos(pos.0, pos.1 - 1)) == g - 1 {
                g -= 1;
                pos.1 -= 1;
                poss.push(pos);
                cigar.push(CigarOp::Ins);
                continue;
            }
            if cost(Pos(pos.0 - 1, pos.1 - 1)) == g - 1 {
                g -= 1;
                pos.0 -= 1;
                pos.1 -= 1;
                poss.push(pos);
                cigar.push(CigarOp::Sub);
                continue;
            }
            panic!("Bad trace! Got stuck at {pos:?}.");
        }

        assert!(pos.0 == 0 || g == 0);
        cigar.reverse();
        poss.reverse();
        (cigar, poss)
    }
}
