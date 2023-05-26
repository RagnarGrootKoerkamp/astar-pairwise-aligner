use itertools::izip;

use super::*;
use crate::prelude::*;

// NOTE: This assumes an alphabet of 'ACGT'.
pub struct QGrams<'a> {
    pub a: Seq<'a>,
    pub b: Seq<'a>,
}

// Bit-widths have type u32.
const W: u32 = 2;

impl<'a> QGrams<'a> {
    pub fn new(a: Seq<'a>, b: Seq<'a>) -> Self {
        #[cfg(test)]
        {
            for c in a.iter() {
                assert!(b"ACGT".contains(c));
            }
            for c in b.iter() {
                assert!(b"ACGT".contains(c));
            }
        }
        Self { a, b }
    }

    #[inline]
    pub fn char_to_bits(c: u8) -> usize {
        ((c >> 1) & 0b11) as usize
    }

    // NOTE: qgrams have their first character in the high-order bits.
    #[inline]
    pub fn to_qgram(seed: &[u8]) -> usize {
        let mut q = 0;
        for &c in seed {
            q <<= W;
            q |= Self::char_to_bits(c);
        }
        q
    }

    pub fn a_qgrams(&self, k: I) -> impl '_ + Iterator<Item = (I, usize)> + Clone {
        // NOTE: Computing each k-mer separately is 3x faster than doing a rolling window with `step_by(k)`.
        (0..).step_by(k as _).zip(
            self.a
                .chunks_exact(k as _)
                .map(move |seed| Self::to_qgram(seed)),
        )
    }

    pub fn a_qgrams_rev(&self, k: I) -> impl '_ + Iterator<Item = (I, usize)> + Clone {
        self.a
            .chunks_exact(k as _)
            .enumerate()
            .map(move |(i, seed)| (k * i as I, Self::to_qgram(seed)))
            .rev()
    }

    pub fn b_qgrams(&self, k: I) -> impl '_ + Iterator<Item = (I, usize)> + Clone {
        let mut q = 0;
        let mask = 1usize
            .checked_shl(k as u32 * W)
            .unwrap_or(0)
            .wrapping_sub(1);
        (0..).zip(
            self.b
                .iter()
                .map(move |&c| {
                    q <<= W;
                    q |= Self::char_to_bits(c);
                    q &= mask;
                    q
                })
                .skip(k as usize - 1),
        )
    }

    pub fn b_qgrams_rev(&self, k: I) -> impl '_ + Iterator<Item = (I, usize)> + Clone {
        let mut q = 0;
        let leftshift = W * (k as u32 - 1);
        izip!(
            (0..self.b.len() as I - k + 1).rev(),
            self.b
                .iter()
                .rev()
                .map(move |&c| {
                    q >>= W;
                    q |= Self::char_to_bits(c) << leftshift;
                    q
                })
                .skip(k as usize - 1),
        )
        .into_iter()
    }

    pub fn fixed_length_seeds(&self, k: I, r: MatchCost) -> Vec<Seed> {
        (0..self.a.len() as I - k + 1)
            .step_by(k as _)
            .map(|i| Seed {
                start: i as I,
                end: i as I + k,
                seed_potential: r,
                seed_cost: r,
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use super::*;
    #[test]
    fn to_qgram() {
        assert_eq!(QGrams::char_to_bits(b'A'), 0b00);
        assert_eq!(QGrams::char_to_bits(b'C'), 0b01);
        assert_eq!(QGrams::char_to_bits(b'G'), 0b11);
        assert_eq!(QGrams::char_to_bits(b'T'), 0b10);
        assert_eq!(QGrams::to_qgram(b"ACGT"), 0b00_01_11_10);
        assert_eq!(QGrams::to_qgram(b"TGCA"), 0b10_11_01_00);
    }
    #[test]
    fn iterators() {
        let a = b"ACGT";
        let b = b"ACGT";
        let qgrams = QGrams::new(a, b);
        assert_eq!(qgrams.a_qgrams(2).collect_vec(), [(0, 0b0001), (2, 0b1110)]);
        assert_eq!(
            qgrams.a_qgrams_rev(2).collect_vec(),
            [(2, 0b1110), (0, 0b0001)]
        );
        assert_eq!(
            qgrams.b_qgrams(2).collect_vec(),
            [(0, 0b0001), (1, 0b0111), (2, 0b1110)]
        );
        assert_eq!(
            qgrams.b_qgrams_rev(2).collect_vec(),
            [(2, 0b1110), (1, 0b0111), (0, 0b0001)]
        );
    }
}
