//! `ordered` matches return the positions of all matches.  There used to be
//! also a module `unordered` that only returns counts, but it has been moved to
//! the graveyard since it is not in use.
use super::*;
use crate::prelude::*;
use smallvec::SmallVec;

#[derive(Debug, PartialEq, Eq)]
pub struct Mutations {
    pub deletions: Vec<usize>,
    pub substitutions: Vec<usize>,
    pub insertions: Vec<usize>,
}

// TODO: Do not generate insertions at the end. (Also do not generate similar
// sequences by inserting elsewhere.)
// TODO: Move to seeds.rs.
fn mutations(k: I, qgram: usize, dedup: bool) -> Mutations {
    // This assumes the alphabet size is 4.
    let mut deletions = Vec::with_capacity(k as usize);
    let mut substitutions = Vec::with_capacity(4 * k as usize);
    let mut insertions = Vec::with_capacity(4 * (k + 1) as usize);
    // Substitutions
    for i in 0..k {
        let mask = !(3 << (2 * i));
        for s in 0..4 {
            let q = (qgram & mask) | s << (2 * i);
            if q != qgram {
                substitutions.push(q);
            }
        }
    }
    // Insertions
    for i in 0..=k {
        let mask = (1 << (2 * i)) - 1;
        for s in 0..4 {
            let candidate = (qgram & mask) | (s << (2 * i)) | ((qgram & !mask) << 2);
            insertions.push(candidate);
        }
    }
    // Deletions
    for i in 0..k {
        let mask = (1 << (2 * i)) - 1;
        deletions.push((qgram & mask) | ((qgram & (!mask << 2)) >> 2));
    }
    if dedup {
        for v in [&mut deletions, &mut substitutions, &mut insertions] {
            // TODO: This sorting is slow; maybe we can work around it.
            v.sort_unstable();
            v.dedup();
        }
    }
    Mutations {
        deletions,
        substitutions,
        insertions,
    }
}

// FIXME: Just hardcode T to u64 here.
// For T=u32, k can be at most 15 (or 14 with r=2).
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

pub fn find_matches_qgramindex<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    config @ MatchConfig { length, r, .. }: MatchConfig,
    transform_filter: bool,
) -> Matches {
    assert!(r == 2);

    // Qgrams of B.
    // TODO: Profile this index and possibly use something more efficient for large k.
    let qgram_map = &mut HashMap::<I, QGramIndex>::default();
    // TODO: This should return &[I] instead.
    fn get_matches<'a, 'c>(
        qgrams: &'c mut HashMap<I, QGramIndex>,
        b: Seq<'a>,
        k: I,
        qgram: usize,
    ) -> &'c [usize] {
        qgrams
            .entry(k)
            .or_insert_with_key(|k| QGramIndex::new(*k as u32, b, &Alphabet::new(b"ACGT")))
            .qgram_matches(qgram)
    }

    // Stops counting when max_count is reached.
    let mut count_matches = |k: I, qgram, max_count: usize| -> usize {
        // exact matches
        let mut cnt = get_matches(qgram_map, b, k, qgram).len();
        if cnt >= max_count {
            return max_count;
        }
        if r == 2 {
            let mutations = mutations(k, qgram, true);
            for (v, k) in [
                (mutations.deletions, k - 1),
                (mutations.substitutions, k),
                (mutations.insertions, k + 1),
            ] {
                for qgram in v {
                    cnt += get_matches(qgram_map, b, k, qgram).len();
                    if cnt >= max_count {
                        return max_count;
                    }
                }
            }
        }
        cnt
    };

    // Convert to a binary sequences.
    let qgrams = QGrams::new(a, b);

    let seeds = {
        let mut v: Vec<Seed> = Vec::default();
        let mut a = &a[..];
        let mut i = 0 as I;
        loop {
            // TODO: Clever seed choice, using variable k and m.
            let seed_len = {
                match length {
                    Fixed(k) => Some(k),
                    LengthConfig::Max(MaxMatches {
                        max_matches,
                        k_min,
                        k_max,
                    }) => {
                        let mut k = k_min as I;
                        while k <= a.len() as I
                            && k <= k_max
                            && count_matches(k, QGrams::to_qgram(&a[..k as usize]), max_matches + 1)
                                > max_matches
                        {
                            k += 1;
                        }
                        if k <= k_max {
                            Some(k)
                        } else {
                            None
                        }
                    }
                }
            };
            let Some(seed_len) = seed_len else {
                a = &a[1..];
                i += 1;
                continue;
            };

            if seed_len > a.len() as I {
                break;
            }

            a = &a[seed_len as usize..];

            v.push(Seed {
                start: i,
                end: i + seed_len,
                seed_potential: r,
                seed_cost: r,
            });
            i += seed_len;
        }
        v
    };

    let mut matches = MatchBuilder::new_with_seeds(&qgrams, config, transform_filter, seeds);

    for i in (0..matches.seeds.seeds.len()).rev() {
        let Seed {
            start,
            end,
            seed_potential,
            ..
        } = matches.seeds.seeds[i];
        let len = end - start;
        let qgram = QGrams::to_qgram(&qgrams.a[start as usize..end as usize]);

        // Exact matches
        for &j in get_matches(qgram_map, b, len, qgram) {
            matches.push(Match {
                start: Pos(start, j as I),
                end: Pos(end, j as I + len),
                match_cost: 0,
                seed_potential,
                pruned: MatchStatus::Active,
            });
        }
        // Inexact matches.
        if seed_potential > 1 {
            let mutations = mutations(len, qgram, true);
            for mutation in mutations.deletions {
                for &j in get_matches(qgram_map, b, len - 1, mutation) {
                    matches.push(Match {
                        start: Pos(start, j as I),
                        end: Pos(end, j as I + len - 1),
                        match_cost: 1,
                        seed_potential,
                        pruned: MatchStatus::Active,
                    });
                }
            }
            for mutation in mutations.substitutions {
                for &j in get_matches(qgram_map, b, len, mutation) {
                    matches.push(Match {
                        start: Pos(start, j as I),
                        end: Pos(end, j as I + len),
                        match_cost: 1,
                        seed_potential,
                        pruned: MatchStatus::Active,
                    });
                }
            }
            for mutation in mutations.insertions {
                for &j in get_matches(qgram_map, b, len + 1, mutation) {
                    matches.push(Match {
                        start: Pos(start, j as I),
                        end: Pos(end, j as I + len + 1),
                        match_cost: 1,
                        seed_potential,
                        pruned: MatchStatus::Active,
                    });
                }
            }
        }
    }

    matches.finish()
}

/// Build a hashset of the kmers in b, and query all mutations of seeds in a.
/// Returns the set of matches sorted by `(LexPos(start), LexPos(end), cost)`.
pub fn find_matches_qgram_hash_inexact<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    config @ MatchConfig { length, r, .. }: MatchConfig,
    transform_filter: bool,
) -> Matches {
    let k: I = match length {
        Fixed(k) => k,
        _ => unimplemented!("QGram Hashing only works for fixed k for now."),
    };
    assert!(r == 2);

    let qgrams = QGrams::new(a, b);
    let mut matches = MatchBuilder::new(&qgrams, config, transform_filter);

    // type of Qgrams
    type Q = usize;
    assert!(k <= 31);

    // TODO: See if we can get rid of the Vec alltogether.
    let mut m = HashMap::<Q, SmallVec<[Cost; 4]>>::default();
    m.reserve(3 * b.len());
    for k in k - 1..=k + 1 {
        for (j, w) in qgrams.b_qgrams(k) {
            m.entry(key_for_sized_qgram(k, w))
                .or_default()
                .push(j as Cost);
        }
    }
    for i in (0..matches.seeds.seeds.len()).rev() {
        let Seed { start, end, .. } = matches.seeds.seeds[i];
        let qgram = QGrams::to_qgram(&qgrams.a[start as usize..end as usize]);
        let matches_before_seed = matches.matches.len();
        if let Some(js) = m.get(&key_for_sized_qgram(k, qgram)) {
            for &j in js {
                matches.push(Match {
                    start: Pos(start, j),
                    end: Pos(start + k, j + k),
                    match_cost: 0,
                    seed_potential: 2,
                    pruned: MatchStatus::Active,
                });
            }
        }
        // We don't dedup here, since we'll be sorting and deduplicating the list of all matches anyway.
        let ms = mutations(k, qgram, false);
        for w in ms.deletions {
            if let Some(js) = m.get(&key_for_sized_qgram(k - 1, w)) {
                for &j in js {
                    matches.push(Match {
                        start: Pos(start, j),
                        end: Pos(start + k, j + k - 1),
                        match_cost: 1,
                        seed_potential: 2,
                        pruned: MatchStatus::Active,
                    });
                }
            }
        }
        for w in ms.substitutions {
            if let Some(js) = m.get(&key_for_sized_qgram(k, w)) {
                for &j in js {
                    matches.push(Match {
                        start: Pos(start, j),
                        end: Pos(start + k, j + k),
                        match_cost: 1,
                        seed_potential: 2,
                        pruned: MatchStatus::Active,
                    });
                }
            }
        }
        for w in ms.insertions {
            if let Some(js) = m.get(&key_for_sized_qgram(k + 1, w)) {
                for &j in js {
                    matches.push(Match {
                        start: Pos(start, j),
                        end: Pos(start + k, j + k + 1),
                        match_cost: 1,
                        seed_potential: 2,
                        pruned: MatchStatus::Active,
                    });
                }
            }
        }
        // NOTE: `sort_unstable_by_key` (quicksort) is slower than `sort_by_key` (mergesort) here.
        matches.matches[matches_before_seed..]
            .sort_by_key(|m| (LexPos(m.start), LexPos(m.end), m.match_cost));
    }

    matches.finish()
}

#[cfg(test)]
mod test {
    use pa_generate::uniform_fixed;

    use super::*;

    #[test]
    fn test_mutations() {
        let kmer = 0b00011011usize;
        let k = 4;
        let ms = mutations(k, kmer, true);
        // substitution
        assert!(ms.substitutions.contains(&0b11011011));
        // insertion
        assert!(ms.insertions.contains(&0b0011011011));
        // deletion
        assert!(ms.deletions.contains(&0b000111));
        assert_eq!(
            ms,
            Mutations {
                deletions: [6, 7, 11, 27].to_vec(),
                substitutions: [11, 19, 23, 24, 25, 26, 31, 43, 59, 91, 155, 219].to_vec(),
                insertions: [
                    27, 75, 91, 99, 103, 107, 108, 109, 110, 111, 123, 155, 219, 283, 539, 795,
                ]
                .to_vec()
            }
        );
    }

    #[test]
    fn kmer_removal() {
        let kmer = 0b00011011usize;
        let k = 4;
        let ms = mutations(k, kmer, true);
        assert!(!ms.substitutions.contains(&kmer));
        assert!(ms.deletions.contains(&kmer));
        assert!(ms.insertions.contains(&kmer));
    }

    #[test]
    fn hash_matches_inexact() {
        // TODO: Replace max match distance from 0 to 1 here once supported.
        for (k, r) in [(6, 2), (7, 2), (10, 2)] {
            for n in [40, 100, 200, 500, 1000, 10000] {
                for e in [0.01, 0.1, 0.3, 1.0] {
                    let (a, b) = uniform_fixed(n, e);
                    let matchconfig = MatchConfig::new(k, r);
                    let mut r = find_matches_qgramindex(&a, &b, matchconfig, false);
                    let mut k = find_matches_qgram_hash_inexact(&a, &b, matchconfig, false);
                    assert!(r
                        .matches
                        .is_sorted_by_key(|Match { start, .. }| LexPos(*start)));
                    assert!(k
                        .matches
                        .is_sorted_by_key(|Match { start, .. }| LexPos(*start)));
                    r.matches.sort_by_key(
                        |&Match {
                             start,
                             end,
                             match_cost,
                             ..
                         }| { (LexPos(start), LexPos(end), match_cost) },
                    );
                    k.matches.sort_by_key(
                        |&Match {
                             start,
                             end,
                             match_cost,
                             ..
                         }| { (LexPos(start), LexPos(end), match_cost) },
                    );
                    assert_eq!(r.matches, k.matches);
                }
            }
        }
    }
}
