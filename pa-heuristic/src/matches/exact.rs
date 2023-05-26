//! Methods to find all (filtered) exact k-mer matches between sequences a and b.
//!
//! Sequence `a` is *split* into disjoint k-mers, while for sequence `b` we consider *all* (sliding-window) k-mers.
//! Thus, `b` contains `k` times more `k-mers`.
//! Typically it's 2-3x faster to build a smaller hashmap over `a` and query
//! that more often with the k-mers of `b` than the reverse.
//!
//! We implement the algorithms in terms of two iterators that can be
//! swapped to reverse the roles of a (sparse chunks) and b (dense windows).
use super::*;
use crate::prelude::*;
use smallvec::SmallVec;

/// Build a hashset of the seeds in a, and query all kmers in b.
pub fn hash_a<'a>(a: Seq<'a>, b: Seq<'a>, config: MatchConfig, transform_filter: bool) -> Matches {
    assert!(config.r == 1);
    let k = config.length.k().unwrap();
    let q = QGrams::new(a, b);
    let mut matches = MatchBuilder::new(&q, config, transform_filter);
    hash_to_smallvec(q.a_qgrams(k), q.b_qgrams_rev(k), &mut matches, k, |i, j| {
        Pos(i, j)
    });
    matches.sort();
    matches.finish()
}

/// Build a hashset of the seeds in a, and query all kmers in b.
pub fn hash_b<'a>(a: Seq<'a>, b: Seq<'a>, config: MatchConfig, transform_filter: bool) -> Matches {
    assert!(config.r == 1);
    let k = config.length.k().unwrap();
    let q = QGrams::new(a, b);
    let mut matches = MatchBuilder::new(&q, config, transform_filter);
    hash_to_smallvec(q.b_qgrams(k), q.a_qgrams_rev(k), &mut matches, k, |j, i| {
        Pos(i, j)
    });
    matches.sort();
    matches.finish()
}

fn hash_to_smallvec(
    qgrams_hashed: impl Iterator<Item = (i32, usize)>,
    qgrams_lookup: impl Iterator<Item = (i32, usize)>,
    matches: &mut MatchBuilder,
    k: i32,
    to_pos: impl Fn(I, I) -> Pos,
) {
    type Key = u32;

    // TODO: See if we can get rid of the Vec alltogether.
    let mut h = HashMap::<Key, SmallVec<[I; 2]>>::default();
    h.reserve(qgrams_hashed.size_hint().0);
    for (i, q) in qgrams_hashed {
        h.entry(q as Key).or_default().push(i as I);
    }
    for (j, q) in qgrams_lookup {
        if let Some(is) = h.get(&(q as Key)) {
            for &i in is {
                let start = to_pos(i as I, j);
                matches.push(Match {
                    start,
                    end: start + Pos(k, k),
                    match_cost: 0,
                    seed_potential: 1,
                    pruned: MatchStatus::Active,
                });
            }
        }
    }
}

/// Build a hashset of the seeds in a, and query all kmers in b.
pub fn hash_a_single<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    config: MatchConfig,
    transform_filter: bool,
) -> Matches {
    assert!(config.r == 1);
    let k = config.length.k().unwrap();
    let q = QGrams::new(a, b);
    let mut matches = MatchBuilder::new(&q, config, transform_filter);
    hash_to_single_vec(q.a_qgrams(k), q.b_qgrams_rev(k), &mut matches, k, Pos);
    matches.sort();
    matches.finish()
}

/// Build a hashset of the seeds in a, and query all kmers in b.
pub fn hash_b_single<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    config: MatchConfig,
    transform_filter: bool,
) -> Matches {
    assert!(config.r == 1);
    let k = config.length.k().unwrap();
    let q = QGrams::new(a, b);
    let mut matches = MatchBuilder::new(&q, config, transform_filter);
    hash_to_single_vec(q.b_qgrams(k), q.a_qgrams_rev(k), &mut matches, k, |j, i| {
        Pos(i, j)
    });
    matches.sort();
    matches.finish()
}

fn hash_to_single_vec(
    qgrams_hashed: impl Iterator<Item = (i32, usize)> + Clone,
    qgrams_lookup: impl Iterator<Item = (i32, usize)>,
    matches: &mut MatchBuilder,
    k: i32,
    to_pos: impl Fn(I, I) -> Pos,
) {
    type Key = u32;

    // TODO: See if we can get rid of the Vec alltogether.
    // maps qgram `q` to (idx, cnt). `idx..idx+cnt` is the range of `pos` for `q`.
    let mut idx = HashMap::<Key, (u32, u32)>::default();

    // Count qgrams.
    idx.reserve(qgrams_hashed.size_hint().0);
    for (_i, q) in qgrams_hashed.clone() {
        idx.entry(q as Key).or_default().1 += 1;
    }

    // Accumulate
    let mut acc = 0;
    for cnt in idx.values_mut() {
        cnt.0 = acc;
        acc += cnt.1;
    }

    // Positions in qgrams_hashed where qgrams occur.
    let mut pos = vec![0; acc as usize];

    // Fill the pos vector.
    for (i, q) in qgrams_hashed {
        let (idx, _cnt) = idx.get_mut(&(q as Key)).unwrap();
        pos[*idx as usize] = i as I;
        *idx += 1;
    }
    // `idx` now points to the end of the range.

    // Do the lookups.
    for (j, q) in qgrams_lookup {
        if let Some(&(idx, cnt)) = idx.get(&(q as Key)) {
            for &i in &pos[(idx - cnt) as usize..idx as usize] {
                let start = to_pos(i, j);
                matches.push(Match {
                    start,
                    end: start + Pos(k, k),
                    match_cost: 0,
                    seed_potential: 1,
                    pruned: MatchStatus::Active,
                });
            }
        }
    }
}

/// Build a hashset of the seeds in a, and query all kmers in b.
pub fn hash_a_qgram_index<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    config: MatchConfig,
    transform_filter: bool,
) -> Matches {
    assert!(config.r == 1);
    let k = config.length.k().unwrap();
    let q = QGrams::new(a, b);
    let mut matches = MatchBuilder::new(&q, config, transform_filter);
    qgram_index(q.a_qgrams(k), q.b_qgrams_rev(k), &mut matches, k, Pos);
    matches.sort();
    matches.finish()
}

/// Build a hashset of the seeds in a, and query all kmers in b.
pub fn hash_b_qgram_index<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    config: MatchConfig,
    transform_filter: bool,
) -> Matches {
    assert!(config.r == 1);
    let k = config.length.k().unwrap();
    let q = QGrams::new(a, b);
    let mut matches = MatchBuilder::new(&q, config, transform_filter);
    qgram_index(q.b_qgrams(k), q.a_qgrams_rev(k), &mut matches, k, |j, i| {
        Pos(i, j)
    });
    matches.sort();
    matches.finish()
}

/// A qgram index first stores the count per kmer in a list of size 4^k.
/// If 4^k is more than the size of the input (typically the case for us), this is slower than hashmap-based algorithms.
fn qgram_index(
    qgrams_hashed: impl Iterator<Item = (i32, usize)> + Clone,
    qgrams_lookup: impl Iterator<Item = (i32, usize)>,
    matches: &mut MatchBuilder,
    k: i32,
    to_pos: impl Fn(I, I) -> Pos,
) {
    // TODO: See if we can get rid of the Vec alltogether.
    // maps qgram `q` to (idx, cnt). `idx..idx+cnt` is the range of `pos` for `q`.
    let mut idx = vec![(0u32, 0u32); 1 << (2 * k)];

    // Count qgrams.
    for (_i, q) in qgrams_hashed.clone() {
        idx[q].1 += 1;
    }

    // Accumulate
    let mut acc = 0;
    for cnt in &mut idx {
        cnt.0 = acc;
        acc += cnt.1;
    }

    // Positions in qgrams_hashed where qgrams occur.
    let mut pos = vec![0; acc as usize];

    // Fill the pos vector.
    for (i, q) in qgrams_hashed {
        let (idx, _cnt) = &mut idx[q];
        pos[*idx as usize] = i as I;
        *idx += 1;
    }
    // `idx` now points to the end of the range.

    // Do the lookups.
    for (j, q) in qgrams_lookup {
        let (idx, cnt) = idx[q];
        for &i in &pos[(idx - cnt) as usize..idx as usize] {
            let start = to_pos(i, j);
            matches.push(Match {
                start,
                end: start + Pos(k, k),
                match_cost: 0,
                seed_potential: 1,
                pruned: MatchStatus::Active,
            });
        }
    }
}

// =============================================================
// BELOW HERE ARE MORE COMPLEX METHODS.

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
                            && count_matches(k, QGrams::to_qgram(&a[..k as usize])) > max_matches
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

    for i in 0..matches.seeds.seeds.len() {
        let Seed {
            start,
            end,
            seed_potential,
            ..
        } = matches.seeds.seeds[i];
        let qgram = QGrams::to_qgram(&a[start as usize..end as usize]);
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
pub fn hash_a_sliding_window<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    config @ MatchConfig { length, r, .. }: MatchConfig,
    transform_filter: bool,
) -> Matches {
    assert!(transform_filter);
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
    let t = matches.seeds.transform(p);
    // Given a j, the range of i values where we want to find matches.
    // T: u(i,j) -> Pos(i - j - pot[u], j - i - pot[u])
    // P = pot[u]
    // j-i-P <= p.1 => i >= j+p.1-P
    // i-j-P <= p.0 => i <= j+p.0+P
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
                    let wi = QGrams::to_qgram(&a[i..i + k as usize]);
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
                    let wi = QGrams::to_qgram(&a[i..i + k as usize]);
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
    matches.sort();
    matches.finish()
}

#[cfg(test)]
mod test {
    use pa_generate::uniform_fixed;

    use super::*;

    #[test]
    fn hash_matches_exact() {
        // TODO: Replace max match distance from 0 to 1 here once supported.
        for n in [10, 20, 40, 100, 200, 500, 1000, 10000] {
            for e in [0.01, 0.1, 0.3, 1.0] {
                for k in [4, 5, 6, 7] {
                    let (a, b) = uniform_fixed(n, e);
                    let mut matchconfig = MatchConfig::new(k, 1);
                    matchconfig.local_pruning = 1;
                    // These are broken :/
                    // let m = find_matches_qgramindex(&a, &b, matchconfig, true);
                    // let a_sw = hash_a_sliding_window(&a, &b, matchconfig, true);
                    let a1 = hash_a(&a, &b, matchconfig, true);
                    let a2 = hash_a_single(&a, &b, matchconfig, true);
                    let a3 = hash_a_qgram_index(&a, &b, matchconfig, true);
                    let b1 = hash_b(&a, &b, matchconfig, true);
                    let b2 = hash_b_single(&a, &b, matchconfig, true);
                    let b3 = hash_b_qgram_index(&a, &b, matchconfig, true);
                    let m = &a1;
                    assert_eq!(
                        a1.matches,
                        m.matches,
                        "Unequal matches: n={}, e={}, k={}\n{}\n{}\n{:?}\n{:?}",
                        n,
                        e,
                        k,
                        seq_to_string(&a),
                        seq_to_string(&b),
                        a1.matches,
                        m.matches
                    );
                    assert_eq!(a2.matches, m.matches);
                    assert_eq!(a3.matches, m.matches);
                    assert_eq!(b1.matches, m.matches);
                    assert_eq!(b2.matches, m.matches);
                    assert_eq!(b3.matches, m.matches);
                }
            }
        }
    }
}
