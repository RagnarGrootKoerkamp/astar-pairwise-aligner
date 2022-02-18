use itertools::Itertools;

use crate::prelude::*;

pub fn to_qgram(rank_transform: &RankTransform, width: usize, seed: &[u8]) -> usize {
    let mut q = 0;
    for &c in seed {
        q <<= width;
        q |= rank_transform.get(c) as usize;
    }
    q
}

pub fn qgrams_overlap(mut k: I, mut q: usize, mut k2: I, mut q2: usize) -> bool {
    if k > k2 {
        std::mem::swap(&mut k, &mut k2);
        std::mem::swap(&mut q, &mut q2);
    }

    let mut has_match = false;
    for i in 0..=k2 - k {
        if ((q2 >> (2 * i)) ^ q) & ((1 << (2 * k)) - 1) == 0 {
            has_match = true;
        }
    }
    has_match
}

pub fn iterate_fixed_qgrams<'a>(
    rank_transform: &'a RankTransform,
    a: &'a Vec<u8>,
    k: u32,
) -> impl Iterator<Item = (usize, usize)> + 'a {
    let width = rank_transform.get_width();
    a.chunks_exact(k as usize)
        .enumerate()
        .map(move |(i, seed)| (k as usize * i, to_qgram(&rank_transform, width, seed)))
}

pub fn fixed_seeds(
    rank_transform: &RankTransform,
    max_match_cost: MatchCost,
    a: &Vec<u8>,
    k: u32,
) -> Vec<Seed> {
    iterate_fixed_qgrams(rank_transform, a, k)
        .map(|(i, qgram)| Seed {
            start: i as I,
            end: i as I + k,
            seed_potential: max_match_cost + 1,
            qgram,
            seed_cost: max_match_cost + 1,
        })
        .collect_vec()
}

pub fn key_for_sized_qgram<
    T: num_traits::Bounded
        + num_traits::Zero
        + num_traits::AsPrimitive<usize>
        + std::ops::Shl<usize, Output = T>
        + std::ops::BitOr<Output = T>,
>(
    k: I,
    qgram: T,
) -> T {
    let size = 8 * std::mem::size_of::<T>();
    assert!(
        (2 * k as usize) < 8 * size,
        "kmer size {k} does not leave spare bits in base type of size {size}"
    );
    let shift = 2 * k as usize + 2;
    let mask = if shift == size {
        T::zero()
    } else {
        T::max_value() << shift
    };
    qgram | mask
}
