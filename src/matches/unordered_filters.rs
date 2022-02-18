use rustc_hash::FxHasher;

use crate::prelude::*;

fn unordered_matches_exact_fixed_hashset<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    alph: &Alphabet,
    k: I,
) -> Seeds {
    let rank_transform = RankTransform::new(alph);

    assert!((2 * k as usize) < 8 * std::mem::size_of::<Key>());

    let mut bf = HashSet::<Key>::default();
    bf.reserve(2 * a.len() / k as usize);
    // println!(
    //     "size: {}, k: {}",
    //     bf.number_of_bits(),
    //     bf.number_of_hash_functions()
    // );

    let bit = (1 as Key) << (Key::BITS - 1);

    // First set all elements of a that will be considered.
    // This makes for a factor k smaller datastructure.
    for (_, w) in iterate_fixed_qgrams(&rank_transform, a, k) {
        bf.insert(w);
    }

    for mut w in rank_transform.qgrams(k, b) {
        if bf.contains(&w) {
            w ^= bit;
            // We insert qgrams of b. If a qgram is already present, we also insert the
            // negated qgram to indicate a count of at least 2.
            if !bf.insert(w) {
                bf.insert(!w);
            }
        }
    }

    // NOTE: We don't iterate the hashmap, since future iterations may not store
    // seeds in the hashmap at all.
    let mut seeds = Vec::<Seed>::new();
    for (i, mut w) in iterate_fixed_qgrams(&rank_transform, a, k) {
        w ^= bit;
        let num_matches = if !bf.contains(&w) {
            0
        } else if !bf.contains(&!w) {
            1
        } else {
            2
        };

        if num_matches <= 1 {
            seeds.push(Seed {
                start: i as I,
                end: i as I + k,
                seed_potential: 1,
                seed_cost: 1 - num_matches,
                qgram: 0,
            })
        }
    }

    Seeds::new(a, seeds, Vec::default())
}

pub fn unordered_matches_exact_fixed_cuckoofilter<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    alph: &Alphabet,
    k: I,
) -> Seeds {
    type Key = usize;

    let rank_transform = RankTransform::new(alph);

    assert!((2 * k as usize) < 8 * std::mem::size_of::<Key>());

    let mut bf: cuckoofilter::CuckooFilter<FxHasher> =
        cuckoofilter::CuckooFilter::with_capacity(2 * a.len() / k as usize);
    // println!(
    //     "size: {}, k: {}",
    //     bf.number_of_bits(),
    //     bf.number_of_hash_functions()
    // );

    let bit = (1 as Key) << (Key::BITS - 1);

    // First set all elements of a that will be considered.
    // This makes for a factor k smaller datastructure.
    for (_, w) in iterate_fixed_qgrams(&rank_transform, a, k) {
        bf.add(&w).unwrap();
    }

    for mut w in rank_transform.qgrams(k, b) {
        if bf.contains(&w) {
            w ^= bit;
            // We insert qgrams of b. If a qgram is already present, we also insert the
            // negated qgram to indicate a count of at least 2.
            if !bf.test_and_add(&w).unwrap() {
                bf.add(&!w).unwrap();
            }
        }
    }

    // NOTE: We don't iterate the hashmap, since future iterations may not store
    // seeds in the hashmap at all.
    let mut seeds = Vec::<Seed>::new();
    for (i, mut w) in iterate_fixed_qgrams(&rank_transform, a, k) {
        w ^= bit;
        let num_matches = if !bf.contains(&w) {
            0
        } else if !bf.contains(&!w) {
            1
        } else {
            2
        };

        if num_matches <= 1 {
            seeds.push(Seed {
                start: i as I,
                end: i as I + k,
                seed_potential: 1,
                seed_cost: 1 - num_matches,
                qgram: 0,
            })
        }
    }

    Seeds::new(a, seeds, Vec::default())
}

pub fn unordered_matches_exact_fixed_bloomfilter<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    alph: &Alphabet,
    k: I,
) -> Seeds {
    type Key = usize;

    let rank_transform = RankTransform::new(alph);

    assert!((2 * k as usize) < 8 * std::mem::size_of::<Key>());

    let mut bf = bloomfilter::Bloom::new_for_fp_rate(2 * a.len() / k as usize, 0.01);
    // println!(
    //     "size: {}, k: {}",
    //     bf.number_of_bits(),
    //     bf.number_of_hash_functions()
    // );

    let bit = (1 as Key) << (Key::BITS - 1);

    // First set all elements of a that will be considered.
    // This makes for a factor k smaller datastructure.
    for (_, w) in iterate_fixed_qgrams(&rank_transform, a, k) {
        bf.set(&w);
    }

    for mut w in rank_transform.qgrams(k, b) {
        if bf.check(&w) {
            w ^= bit;
            // We insert qgrams of b. If a qgram is already present, we also insert the
            // negated qgram to indicate a count of at least 2.
            if bf.check_and_set(&w) {
                bf.set(&!w);
            }
        }
    }

    // NOTE: We don't iterate the hashmap, since future iterations may not store
    // seeds in the hashmap at all.
    let mut seeds = Vec::<Seed>::new();
    for (i, mut w) in iterate_fixed_qgrams(&rank_transform, a, k) {
        w ^= bit;
        let num_matches = if !bf.check(&w) {
            0
        } else if !bf.check(&!w) {
            1
        } else {
            2
        };

        if num_matches <= 1 {
            seeds.push(Seed {
                start: i as I,
                end: i as I + k,
                seed_potential: 1,
                seed_cost: 1 - num_matches,
                qgram: 0,
            })
        }
    }

    Seeds::new(a, seeds, Vec::default())
}
