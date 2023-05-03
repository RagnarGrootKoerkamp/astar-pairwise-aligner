//! `ordered` matches return the positions of all matches.  There used to be
//! also a module `unordered` that only returns counts, but it has been moved to
//! the graveyard since it is not in use.
use smallvec::SmallVec;

use super::{suffix_array::minimal_unique_matches, *};
use crate::prelude::*;

pub fn find_matches_qgramindex<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    MatchConfig {
        length,
        max_match_cost,
        ..
    }: MatchConfig,
    gapcost: bool,
) -> Matches {
    assert!(max_match_cost == 0 || max_match_cost == 1);

    // Qgrams of B.
    // TODO: Profile this index and possibly use something more efficient for large k.
    let qgrams = &mut HashMap::<I, QGramIndex>::default();
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
        let mut cnt = get_matches(qgrams, b, k, qgram).len();
        if cnt >= max_count {
            return max_count;
        }
        if max_match_cost == 1 {
            let mutations = mutations(k, qgram, true, gapcost);
            for (v, k) in [
                (mutations.deletions, k - 1),
                (mutations.substitutions, k),
                (mutations.insertions, k + 1),
            ] {
                for qgram in v {
                    cnt += get_matches(qgrams, b, k, qgram).len();
                    if cnt >= max_count {
                        return max_count;
                    }
                }
            }
        }
        cnt
    };

    // Convert to a binary sequences.
    let rank_transform = RankTransform::new(&Alphabet::new(b"ACGT"));
    let width = rank_transform.get_width();

    let mut seeds = {
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
                            && count_matches(
                                k,
                                to_qgram(&rank_transform, width, &a[..k as usize]),
                                max_matches + 1,
                            ) > max_matches
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

            let (seed, tail) = a.split_at(seed_len as usize);
            a = tail;

            v.push(Seed {
                start: i,
                end: i + seed_len,
                seed_potential: max_match_cost + 1,
                qgram: to_qgram(&rank_transform, width, seed),
                seed_cost: max_match_cost + 1,
            });
            i += seed_len;
        }
        v
    };

    // Find matches of the seeds of a in b.
    // NOTE: This uses O(alphabet^k) memory.
    let mut matches = Vec::<Match>::new();

    for seed @ &mut Seed {
        start,
        end,
        seed_potential,
        qgram,
        ..
    } in &mut seeds
    {
        let len = end - start;

        // Exact matches
        for &j in get_matches(qgrams, b, len, qgram) {
            seed.seed_cost = 0;
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
            let mutations = mutations(len, qgram, true, gapcost);
            for mutation in mutations.deletions {
                for &j in get_matches(qgrams, b, len - 1, mutation) {
                    seed.seed_cost = min(seed.seed_cost, 1);
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
                for &j in get_matches(qgrams, b, len, mutation) {
                    seed.seed_cost = min(seed.seed_cost, 1);
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
                for &j in get_matches(qgrams, b, len + 1, mutation) {
                    seed.seed_cost = min(seed.seed_cost, 1);
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

    Matches::new(a, seeds, matches)
}

/// Build a hashset of the kmers in b, and query all mutations of seeds in a.
/// Returns the set of matches sorted by `(LexPos(start), LexPos(end), cost)`.
pub fn find_matches_qgram_hash_inexact<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    MatchConfig {
        length,
        max_match_cost,
        ..
    }: MatchConfig,
    gapcost: bool,
) -> Matches {
    let k: I = match length {
        Fixed(k) => k,
        _ => unimplemented!("QGram Hashing only works for fixed k for now."),
    };
    assert!(max_match_cost == 1);

    let rank_transform = RankTransform::new(&Alphabet::new(b"ACGT"));

    let mut seeds = fixed_seeds(&rank_transform, max_match_cost, a, k);

    // type of Qgrams
    type Q = usize;
    assert!(k <= 31);

    // TODO: See if we can get rid of the Vec alltogether.
    let mut m = HashMap::<Q, SmallVec<[Cost; 4]>>::default();
    m.reserve(3 * b.len());
    for k in k - 1..=k + 1 {
        for (j, w) in rank_transform.qgrams(k as _, b).enumerate() {
            m.entry(key_for_sized_qgram(k, w))
                .or_default()
                .push(j as Cost);
        }
    }
    let mut matches = Vec::<Match>::new();
    for seed @ &mut Seed { start, qgram, .. } in &mut seeds {
        let matches_before_seed = matches.len();
        if let Some(js) = m.get(&key_for_sized_qgram(k, qgram)) {
            for &j in js {
                seed.seed_cost = 0;
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
        let ms = mutations(k, qgram, false, gapcost);
        for w in ms.deletions {
            if let Some(js) = m.get(&key_for_sized_qgram(k - 1, w)) {
                for &j in js {
                    seed.seed_cost = min(seed.seed_cost, 1);
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
                    seed.seed_cost = min(seed.seed_cost, 1);
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
                    seed.seed_cost = min(seed.seed_cost, 1);
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
        let matches_after_seed = matches.len();
        matches[matches_before_seed..matches_after_seed]
            .sort_by_key(|m| (LexPos(m.start), LexPos(m.end), m.match_cost));
    }

    #[cfg(test)]
    assert!(matches.is_sorted_by_key(|m| (LexPos(m.start), LexPos(m.end), m.match_cost)));
    Matches::new(a, seeds, matches)
}

/// Build a hashset of the seeds in a, and query all kmers in b.
pub fn find_matches_qgram_hash_exact<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    MatchConfig {
        length,
        max_match_cost,
        ..
    }: MatchConfig,
) -> Matches {
    if length.kmin() != length.kmax() {
        unimplemented!("QGram Hashing only works for fixed k for now.");
    }
    let k = length.kmin();

    assert!(max_match_cost == 0);

    let rank_transform = RankTransform::new(&Alphabet::new(b"ACGT"));
    let width = rank_transform.get_width();

    let mut seeds = fixed_seeds(&rank_transform, max_match_cost, a, k);

    type Key = u64;

    // TODO: See if we can get rid of the Vec alltogether.
    let mut m = HashMap::<Key, SmallVec<[I; 4]>>::default();
    let mut matches = Vec::<Match>::new();

    if SLIDING_WINDOW_MATCHES {
        let capacity = a.len() / k as usize / (k - 1) as usize / 2;
        m.reserve(capacity);

        const CHECK_EACH_J_LAYERS: Cost = 6;

        // Target position.
        let p = Pos::target(a, b);
        // Target in transformed domain.
        let t = Pos(
            ((p.0 - 1) / k + p.0).saturating_sub(p.1),
            ((p.0 - 1) / k + p.1).saturating_sub(p.0),
        );
        // Given a j, the range of i values where we want to find matches.
        let i_range_for_j = |j: Cost| -> (Cost, Cost) {
            // Do computation as usize because Cost can overflow.
            let j = j as usize;
            let k = k as usize;
            let max_match_cost = max_match_cost as usize;
            (
                ((j.saturating_sub(t.1 as usize)) * (max_match_cost + 1) * k / (k - 1))
                    .saturating_sub(max_match_cost + 2) as Cost,
                ((t.0 as usize + j) * (max_match_cost + 1) * k / (k + 1) + max_match_cost + 2)
                    as Cost,
            )
        };

        // Iterators pointing to the next i to be inserted to/removed from the hashmap.
        let mut to_remove = (0..a.len() + 1 - k as usize)
            .step_by(k as usize)
            .rev()
            .peekable();
        let mut to_insert = (0..a.len() + 1 - k as usize)
            .step_by(k as usize)
            .rev()
            .peekable();
        let mut qb = 0usize;
        let prepend_qgram_b = |j: usize, qb: &mut usize| {
            *qb =
                (*qb >> width) | ((rank_transform.get(b[j]) as usize) << ((k - 1) as usize * width))
        };

        for j in (0..b.len()).rev() {
            if (b.len() - 1 - j) as Cost & ((1 << CHECK_EACH_J_LAYERS) - 1) == 0 {
                let (new_start, new_end) = i_range_for_j(j as Cost);
                // Remove elements after new_end.
                while let Some(&i) = to_remove.peek() {
                    if (i as Cost) > new_end {
                        let wi = to_qgram(&rank_transform, width, &a[i..i + k as usize]);
                        to_remove.next();
                        let v = m.get_mut(&(wi as Key)).unwrap();
                        assert!(!v.is_empty());
                        // If last element in the smallvec, remove entirely. Else only remove from vector.
                        if v.len() == 1 {
                            assert_eq!(v[0], i as Cost);
                            m.remove(&(wi as Key)).unwrap();
                        } else {
                            // NOTE: This removes in O(1), but changes the order of the elements.
                            v.swap_remove(v.iter().position(|x| *x == i as Cost).unwrap());
                            assert!(v.len() > 0);
                        }
                    } else {
                        break;
                    }
                }
                // Insert new elements after new_start
                while let Some(&i) = to_insert.peek() {
                    if (i as Cost) >= new_start.saturating_sub(2 * (1 << CHECK_EACH_J_LAYERS)) {
                        to_insert.next();
                        let wi = to_qgram(&rank_transform, width, &a[i..i + k as usize]);
                        m.entry(wi as Key).or_default().push(i as I);
                    } else {
                        break;
                    }
                }
            }
            prepend_qgram_b(j, &mut qb);
            if j + k as usize > b.len() {
                continue;
            }
            if let Some(is) = m.get(&(qb as Key)) {
                for &i in is {
                    seeds[(i / k) as usize].seed_cost = 0;
                    matches.push(Match {
                        start: Pos(i, j as I),
                        end: Pos(i + k, j as I + k),
                        match_cost: 0,
                        seed_potential: 1,
                        pruned: MatchStatus::Active,
                    });
                }
            }
        }
    } else {
        m.reserve(a.len() / k as usize + 1);
        for (i, w) in rank_transform
            .qgrams(k as _, a)
            .enumerate()
            .step_by(k as usize)
        {
            m.entry(w as Key).or_default().push(i as I);
        }

        for (j, w) in rank_transform.qgrams(k as _, b).enumerate() {
            if let Some(is) = m.get(&(w as Key)) {
                for &i in is {
                    seeds[(i / k) as usize].seed_cost = 0;
                    matches.push(Match {
                        start: Pos(i, j as I),
                        end: Pos(i + k, j as I + k),
                        match_cost: 0,
                        seed_potential: 1,
                        pruned: MatchStatus::Active,
                    });
                }
            }
        }
    }

    Matches::new(a, seeds, matches)
}

pub fn find_matches<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    match_config: MatchConfig,
    gapcost: bool,
) -> Matches {
    if let Some(max_matches) = match_config.length.max_matches() {
        return minimal_unique_matches(a, b, match_config.max_match_cost + 1, max_matches);
    }
    if FIND_MATCHES_HASH {
        return match match_config.max_match_cost {
            0 => find_matches_qgram_hash_exact(a, b, match_config),
            1 => find_matches_qgram_hash_inexact(a, b, match_config, gapcost),
            _ => unimplemented!("FIND_MATCHES with HashMap only works for max match cost 0 or 1"),
        };
    } else {
        return find_matches_qgramindex(a, b, match_config, gapcost);
    }
}

#[cfg(test)]
mod test {
    use pa_generate::uniform_fixed;

    use super::*;

    #[test]
    fn hash_matches_exact() {
        // TODO: Replace max match distance from 0 to 1 here once supported.
        for (k, max_match_cost) in [(4, 0), (5, 0), (6, 0), (7, 0)] {
            for n in [10, 20, 40, 100, 200, 500, 1000, 10000] {
                for e in [0.01, 0.1, 0.3, 1.0] {
                    let (a, b) = uniform_fixed(n, e);
                    let matchconfig = MatchConfig::new(k, max_match_cost);
                    let r = find_matches_qgramindex(&a, &b, matchconfig, false);
                    let h = find_matches_qgram_hash_exact(&a, &b, matchconfig);
                    if !SLIDING_WINDOW_MATCHES {
                        if r.matches == h.matches {
                            continue;
                        }
                        println!("{}\n{}", seq_to_string(&a), seq_to_string(&b));
                        println!("-----------------------");
                        println!("n={n} e={e} k={k} mmc={max_match_cost}");
                        println!("-----------------------");
                        for x in &r.matches {
                            println!("{x:?}");
                        }
                        println!("-----------------------");
                        for x in &h.matches {
                            println!("{x:?}");
                        }
                        assert_eq!(r.matches, h.matches);
                    }
                }
            }
        }
    }

    #[test]
    fn hash_matches_inexact() {
        // TODO: Replace max match distance from 0 to 1 here once supported.
        for (k, max_match_cost) in [(6, 1), (7, 1), (10, 1)] {
            for n in [40, 100, 200, 500, 1000, 10000] {
                for e in [0.01, 0.1, 0.3, 1.0] {
                    let (a, b) = uniform_fixed(n, e);
                    println!("{}\n{}", seq_to_string(&a), seq_to_string(&b));
                    let matchconfig = MatchConfig::new(k, max_match_cost);
                    println!("-----------------------");
                    println!("n={n} e={e} k={k} mmc={max_match_cost}");
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
                    if r.matches != k.matches {
                        println!("-----------------------");
                        for x in &r.matches {
                            println!("{x:?}");
                        }
                        println!("-----------------------");
                        for x in &k.matches {
                            println!("{x:?}");
                        }
                    }
                    assert_eq!(r.matches, k.matches);
                }
            }
        }
    }
}
