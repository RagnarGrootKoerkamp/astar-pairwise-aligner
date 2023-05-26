//! `ordered` matches return the positions of all matches.  There used to be
//! also a module `unordered` that only returns counts, but it has been moved to
//! the graveyard since it is not in use.
use smallvec::SmallVec;

use crate::prelude::*;

use super::*;

pub fn find_matches_qgramindex<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    config @ MatchConfig { length, r, .. }: MatchConfig,
    transform_filter: bool,
) -> Matches {
    assert!(r == 1);

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
    let mut count_matches = |k: I, qgram| -> usize {
        // exact matches
        get_matches(qgram_map, b, k, qgram).len()
    };

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
                            && count_matches(k, qgrams.to_qgram(&a[..k as usize])) > max_matches
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
                seed_potential: r,
                qgram: qgrams.to_qgram(seed),
                seed_cost: r,
            });
            i += seed_len;
        }
        v
    };

    let mut matches = MatchBuilder::new_with_seeds(&qgrams, config, transform_filter, seeds);

    for i in 0..matches.seeds.seeds.len() {
        let Seed {
            start,
            end,
            seed_potential,
            qgram,
            ..
        } = matches.seeds.seeds[i];
        let len = end - start;

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
    }

    matches.finish()
}

/// Build a hashset of the seeds in a, and query all kmers in b.
pub fn exact_matches_hashmap<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    config @ MatchConfig { length, r, .. }: MatchConfig,
    transform_filter: bool,
) -> Matches {
    assert!(r == 1);
    let k = length.k().unwrap();

    let qgrams = QGrams::new(a, b);
    let mut matches = MatchBuilder::new(&qgrams, config, transform_filter);

    type Key = u32;

    // TODO: See if we can get rid of the Vec alltogether.
    let mut m = HashMap::<Key, SmallVec<[I; 4]>>::default();

    m.reserve(a.len() / k as usize + 1);
    for (i, q) in qgrams.a_qgrams(k) {
        m.entry(q as Key).or_default().push(i as I);
    }
    for (j, q) in qgrams.b_qgrams_rev(k) {
        if let Some(is) = m.get(&(q as Key)) {
            for &i in is {
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

    matches.sort();
    matches.finish()
}

/// Build a hashset of the seeds in a, and query all kmers in b.
pub fn find_matches_qgram_hash_exact_sliding_window<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    config @ MatchConfig { length, r, .. }: MatchConfig,
    transform_filter: bool,
) -> Matches {
    if length.kmin() != length.kmax() {
        unimplemented!("QGram Hashing only works for fixed k for now.");
    }
    let k = length.kmin();

    assert!(r == 1);

    let rank_transform = RankTransform::new(&Alphabet::new(b"ACGT"));
    let width = rank_transform.get_width();

    let qgrams = QGrams::new(a, b);
    let mut matches = MatchBuilder::new(&qgrams, config, transform_filter);

    type Key = u64;

    // TODO: See if we can get rid of the Vec alltogether.
    let mut m = HashMap::<Key, SmallVec<[I; 4]>>::default();

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
        let r = r as usize;
        (
            ((j.saturating_sub(t.1 as usize)) * r * k / (k - 1)).saturating_sub(r + 1) as Cost,
            ((t.0 as usize + j) * r * k / (k + 1) + r + 1) as Cost,
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
        *qb = (*qb >> width) | ((rank_transform.get(b[j]) as usize) << ((k - 1) as usize * width))
    };

    for j in (0..b.len()).rev() {
        if (b.len() - 1 - j) as Cost & ((1 << CHECK_EACH_J_LAYERS) - 1) == 0 {
            let (new_start, new_end) = i_range_for_j(j as Cost);
            // Remove elements after new_end.
            while let Some(&i) = to_remove.peek() {
                if (i as Cost) > new_end {
                    let wi = qgrams.to_qgram(&a[i..i + k as usize]);
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
                    let wi = qgrams.to_qgram(&a[i..i + k as usize]);
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
    matches.finish()
}

#[cfg(test)]
mod test {
    use pa_generate::uniform_fixed;

    use super::*;

    #[test]
    fn hash_matches_exact() {
        // TODO: Replace max match distance from 0 to 1 here once supported.
        for (k, r) in [(4, 1), (5, 1), (6, 1), (7, 1)] {
            for n in [10, 20, 40, 100, 200, 500, 1000, 10000] {
                for e in [0.01, 0.1, 0.3, 1.0] {
                    let (a, b) = uniform_fixed(n, e);
                    let matchconfig = MatchConfig::new(k, r);
                    let qi = find_matches_qgramindex(&a, &b, matchconfig, false);
                    let h = exact_matches_hashmap(&a, &b, matchconfig, false);
                    assert_eq!(qi.matches, h.matches);
                }
            }
        }
    }
}