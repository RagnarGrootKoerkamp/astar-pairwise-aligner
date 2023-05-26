use itertools::izip;

use super::*;
use crate::prelude::*;

pub struct QGrams<'a> {
    pub a: Seq<'a>,
    pub b: Seq<'a>,
    pub rt: RankTransform,
    pub width: usize,
}

impl<'a> QGrams<'a> {
    pub fn new(a: Seq<'a>, b: Seq<'a>) -> Self {
        let rt = RankTransform::new(&Alphabet::new(b"ACGT"));
        Self {
            a,
            b,
            width: rt.get_width(),
            rt,
        }
    }

    // NOTE: qgrams have their first character in the high-order bits.
    pub fn to_qgram(&self, seed: &[u8]) -> usize {
        let mut q = 0;
        for &c in seed {
            q <<= self.width;
            q |= self.rt.get(c) as usize;
        }
        q
    }

    pub fn a_qgrams(&self, k: I) -> impl '_ + Iterator<Item = (I, usize)> {
        // NOTE: Computing each k-mer separately is 3x faster than doing a rolling window with `step_by(k)`.
        (0..).step_by(k as _).zip(
            self.a
                .chunks_exact(k as _)
                .map(move |seed| self.to_qgram(seed)),
        )
    }

    pub fn a_qgrams_rev(&self, k: I) -> impl '_ + Iterator<Item = (I, usize)> {
        self.a
            .chunks_exact(k as _)
            .enumerate()
            .map(move |(i, seed)| (k * i as I, self.to_qgram(seed)))
            .rev()
    }

    pub fn b_qgrams(&self, k: I) -> impl '_ + Iterator<Item = (I, usize)> {
        (0..).zip(self.rt.qgrams(k as _, self.b))
    }

    pub fn b_qgrams_rev(&self, k: I) -> impl '_ + Iterator<Item = (I, usize)> {
        izip!(
            (0..self.b.len() as I - k + 1).rev(),
            self.rt.rev_qgrams(k as _, self.b)
        )
        .into_iter()
    }

    pub fn fixed_length_seeds(&self, k: I, r: MatchCost) -> Vec<Seed> {
        self.a_qgrams(k)
            .map(|(i, qgram)| Seed {
                start: i as I,
                end: i as I + k,
                seed_potential: r,
                qgram,
                seed_cost: r,
            })
            .collect()
    }
}
