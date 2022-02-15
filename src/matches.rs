use crate::prelude::*;
use itertools::Itertools;
use smallvec::SmallVec;
use std::iter::repeat;

#[derive(Clone, Debug)]
pub struct Seed {
    pub start: I,
    pub end: I,
    // The seed_potential is 1 more than the maximal number of errors allowed in this seed.
    pub seed_potential: Cost,
    pub qgram: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    pub start: Pos,
    pub end: Pos,
    pub match_cost: Cost,
    pub seed_potential: Cost,
}

#[derive(Default)]
pub struct SeedMatches {
    // Sorted by start.
    pub seeds: Vec<Seed>,
    // Sorted by start (i, j).
    pub matches: Vec<Match>,
    // Index of the start of the rightmost seed covering the given position.
    pub start_of_seed: Vec<I>,
    pub potential: Vec<Cost>,
}

fn to_qgram(rank_transform: &RankTransform, width: usize, seed: &[u8]) -> usize {
    let mut q = 0;
    for &c in seed {
        q <<= width;
        q |= rank_transform.get(c) as usize;
    }
    q
}

impl SeedMatches {
    pub fn iter(&self) -> std::slice::Iter<Match> {
        self.matches.iter()
    }

    // The potential at p is the cost of going from p to the end, without hitting any matches.
    pub fn potential(&self, Pos(i, _): Pos) -> Cost {
        self.potential[i as usize]
    }

    // TODO: Generalize this for overlapping seeds.
    pub fn is_start_of_seed(&self, Pos(i, _): Pos) -> bool {
        self.start_of_seed[i as usize] == i
    }
}

impl<'a> HeuristicInstance<'a> for SeedMatches {
    fn h(&self, _: Pos) -> Cost {
        unimplemented!("SeedMatches can only be used as a distance, not as a heuristic!");
    }
}
impl<'a> DistanceInstance<'a> for SeedMatches {
    /// The minimal distance is the potential of the seeds entirely within the `[from, to)` interval.
    /// NOTE: Assumes disjoint seeds.
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        assert!(from.0 <= to.0);
        self.potential[from.0 as usize]
            - (self.potential[self.start_of_seed[to.0 as usize] as usize])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MaxMatches {
    // The smallest k with at most this many matches within the band.
    pub max_matches: usize,
    // Return the band as a function of n.
    pub band: fn(I) -> I,
}

#[derive(Clone, Copy, Debug)]
pub struct MinMatches {
    // The largest k with at least this many matches within the band.
    pub min_matches: usize,
    // Return the band as a function of n.
    pub band: fn(I) -> I,
}

#[derive(Clone, Copy, Debug)]
pub enum LengthConfig {
    Fixed(I),
    Max(MaxMatches),
    Min(MinMatches),
}

impl LengthConfig {
    pub fn fixed(k: I) -> LengthConfig {
        LengthConfig::Fixed(k)
    }
    pub fn max(max_matches: usize, band: fn(I) -> I) -> LengthConfig {
        LengthConfig::Max(MaxMatches { max_matches, band })
    }
    pub fn min(min_matches: usize, band: fn(I) -> I) -> LengthConfig {
        assert!(min_matches > 0);
        LengthConfig::Min(MinMatches { min_matches, band })
    }
    pub fn k(&self) -> Option<I> {
        match *self {
            Fixed(k) => Some(k),
            _ => None,
        }
    }
}

impl Default for LengthConfig {
    fn default() -> Self {
        LengthConfig::Fixed(0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MatchConfig {
    // TODO: Add settings for variable length matches in here.
    pub length: LengthConfig,
    // TODO: Move the max_match_cost into MatchLength.
    pub max_match_cost: Cost,
}

pub fn find_matches_trie<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    alph: &Alphabet,
    MatchConfig {
        length,
        max_match_cost,
        ..
    }: MatchConfig,
) -> SeedMatches {
    let k: I = match length {
        Fixed(k) => k,
        _ => unimplemented!("Trie only works for fixed k for now."),
    };
    // Create a trie from all windows of b.
    let mut trie = Trie::new(
        b.windows((k + max_match_cost) as usize)
            .enumerate()
            .map(|(i, w)| (w, i as crate::datastructures::trie::Data)),
        alph,
    );
    // Push all remaining suffixes of b.
    for i in b.len() as I - k - max_match_cost + 1..b.len() as I {
        trie.push(&b[i as usize..], i);
    }

    let rank_transform = RankTransform::new(alph);
    let width = rank_transform.get_width();
    let seeds = a
        .chunks_exact(k as usize)
        .enumerate()
        .map(|(i, seed)| Seed {
            start: i as I * k,
            end: (i + 1) as I * k,
            seed_potential: max_match_cost + 1,
            qgram: to_qgram(&rank_transform, width, seed),
        })
        .collect_vec();

    let n = a.len();
    let mut potential = Vec::with_capacity(n + 1);
    let mut start_of_seed = Vec::with_capacity(n + 1);
    let last_seed = seeds.last();

    // Find matches of the seeds of a in b.
    let mut matches = Vec::<Match>::new();

    let mut cur_potential = seeds.iter().map(|seed| seed.seed_potential).sum();
    potential.push(cur_potential);
    for &Seed {
        start,
        end,
        seed_potential,
        ..
    } in &seeds
    {
        let seed_len = end - start;
        cur_potential -= seed_potential;
        potential.extend(repeat(cur_potential).take(seed_len as usize));
        start_of_seed.extend(repeat(start).take(seed_len as usize));

        trie.matches(
            &a[start as usize..end as usize],
            (seed_potential - 1) as MatchCost,
            |match_start, match_len, cost| {
                matches.push(Match {
                    start: Pos(start, match_start),
                    end: Pos(end, match_start + match_len as I),
                    match_cost: cost as Cost,
                    seed_potential,
                });
            },
        );
    }

    // Backfill a potential gap after the last seed.
    potential.extend(repeat(0).take(n + 1 - potential.len()));
    start_of_seed.extend(repeat(last_seed.unwrap().end).take(n + 1 - start_of_seed.len()));

    // First sort by start, then by end, then by match cost.
    matches.sort_unstable_by_key(
        |&Match {
             start,
             end,
             match_cost,
             ..
         }| (LexPos(start), LexPos(end), match_cost),
    );
    // Dedup to only keep the lowest match cost.
    matches.dedup_by_key(|m| (m.start, m.end));

    SeedMatches {
        seeds,
        matches,
        start_of_seed,
        potential,
    }
}

pub fn find_matches_qgramindex<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    alph: &Alphabet,
    MatchConfig {
        length,
        max_match_cost,
    }: MatchConfig,
) -> SeedMatches {
    assert!(max_match_cost == 0 || max_match_cost == 1);

    let Pos(n, _m) = Pos::from_length(a, b);

    // Qgrams of B.
    // TODO: Profile this index and possibly use something more efficient for large k.
    let qgrams = &mut HashMap::<I, QGramIndex>::default();
    // TODO: This should return &[I] instead.
    fn get_matches<'a, 'c>(
        qgrams: &'c mut HashMap<I, QGramIndex>,
        b: &'a Sequence,
        alph: &Alphabet,
        k: I,
        qgram: usize,
    ) -> &'c [usize] {
        qgrams
            .entry(k)
            .or_insert_with_key(|k| QGramIndex::new(*k as u32, b, alph))
            .qgram_matches(qgram)
    }

    // Stops counting when max_count is reached.
    let mut count_matches = |k: I, qgram, max_count: usize, i: I, band: I| -> usize {
        let count_in_band = |matches: &[usize]| -> usize {
            // println!(
            //     "{} {} {} {} for {:?}",
            //     k,
            //     max_count,
            //     i.saturating_sub(band),
            //     i + band,
            //     matches
            // );
            if matches.len() <= 32 {
                matches
                    .iter()
                    .copied()
                    .filter(|&j| i <= j as I + band && j as I <= i + band)
                    .count()
            } else {
                let start = matches
                    .binary_search(&(i.saturating_sub(band) as usize))
                    .map_or_else(|x| x, |x| x);
                let end = matches
                    .binary_search(&((i + band) as usize))
                    .map_or_else(|x| x + 1, |x| x);
                end - start
            }
        };

        // exact matches
        let mut cnt = 0;
        cnt += count_in_band(get_matches(qgrams, b, alph, k, qgram));
        if cnt >= max_count {
            return max_count;
        }
        if max_match_cost == 1 {
            let mutations = mutations(k, qgram, true);
            for (v, k) in [
                (mutations.deletions, k - 1),
                (mutations.substitutions, k),
                (mutations.insertions, k + 1),
            ] {
                for qgram in v {
                    cnt += count_in_band(get_matches(qgrams, b, alph, k, qgram));
                    if cnt >= max_count {
                        return max_count;
                    }
                }
            }
        }
        cnt
    };

    // Convert to a binary sequences.
    let rank_transform = RankTransform::new(alph);
    let width = rank_transform.get_width();

    let seeds = {
        let mut v: Vec<Seed> = Vec::default();
        let mut a = &a[..];
        let mut long = false;
        let mut i = 0 as I;
        loop {
            // TODO: Clever seed choice, using variable k and m.
            let seed_len = {
                match length {
                    Fixed(k) => k,
                    LengthConfig::Max(MaxMatches { max_matches, band }) => {
                        let mut k = 3 as I;
                        while k <= a.len() as I && k <= 10
                                // TODO: Use band(min(a.len(), n-a.len())) or something like it.
                                && count_matches(k, to_qgram(&rank_transform, width, &a[..k as usize]), max_matches + 1, i, band(n))
                                    > max_matches
                        {
                            k += 1;
                        }
                        k
                    }
                    LengthConfig::Min(MinMatches { min_matches, band }) => {
                        let mut k = 4 as I;
                        // TODO: Remove max length, which is only needed because of memory reasons.
                        while k <= a.len() as I && k <= 11
                                // TODO: Use band(min(a.len(), n-a.len())) or something like it.
                                && count_matches(k, to_qgram(&rank_transform, width, &a[..k as usize]), min_matches, i, band(n))
                                    >= min_matches
                        {
                            k += 1;
                        }
                        k - 1
                    }
                }
            };
            if seed_len > a.len() as I {
                break;
            }
            //print!("{} ", seed_len);

            let (seed, tail) = a.split_at(seed_len as usize);
            a = tail;

            v.push(Seed {
                start: i,
                end: i + seed_len,
                seed_potential: max_match_cost + 1,
                qgram: to_qgram(&rank_transform, width, seed),
            });
            i += seed_len;

            long = !long;
        }
        //println!();
        v
    };
    // println!(
    //     "k: {}",
    //     //length,
    //     //num_seeds,
    //     a.len() as f32 / num_seeds as f32
    // );

    let n = a.len();
    let mut potential = Vec::with_capacity(n + 1);
    let mut start_of_seed = Vec::with_capacity(n + 1);

    // Find matches of the seeds of a in b.
    // NOTE: This uses O(alphabet^k) memory.
    let mut matches = Vec::<Match>::new();

    let mut cur_potential = seeds
        .iter()
        .map(|Seed { seed_potential, .. }| seed_potential)
        .sum();
    potential.push(cur_potential);
    //println!("{:?}", seed_qgrams);
    for &Seed {
        start,
        end,
        seed_potential,
        qgram,
        ..
    } in &seeds
    {
        let len = end - start;
        cur_potential -= seed_potential;
        potential.extend(repeat(cur_potential).take(len as usize));
        start_of_seed.extend(repeat(start).take(len as usize));

        // Exact matches
        for &j in get_matches(qgrams, b, alph, len, qgram) {
            matches.push(Match {
                start: Pos(start, j as I),
                end: Pos(end, j as I + len),
                match_cost: 0,
                seed_potential,
            });
        }
        // Inexact matches.
        if seed_potential > 1 {
            let mutations = mutations(len, qgram, true);
            for mutation in mutations.deletions {
                for &j in get_matches(qgrams, b, alph, len - 1, mutation) {
                    matches.push(Match {
                        start: Pos(start, j as I),
                        end: Pos(end, j as I + len - 1),
                        match_cost: 1,
                        seed_potential,
                    });
                }
            }
            for mutation in mutations.substitutions {
                for &j in get_matches(qgrams, b, alph, len, mutation) {
                    matches.push(Match {
                        start: Pos(start, j as I),
                        end: Pos(end, j as I + len),
                        match_cost: 1,
                        seed_potential,
                    });
                }
            }
            for mutation in mutations.insertions {
                for &j in get_matches(qgrams, b, alph, len + 1, mutation) {
                    matches.push(Match {
                        start: Pos(start, j as I),
                        end: Pos(end, j as I + len + 1),
                        match_cost: 1,
                        seed_potential,
                    });
                }
            }
        }
    }
    // Backfill a potential gap after the last seed.
    potential.extend(repeat(0).take(n + 1 - potential.len()));
    start_of_seed.extend(repeat(seeds.last().unwrap().end).take(n + 1 - start_of_seed.len()));

    //println!("{:?}", potential);
    //println!("{:?}", start_of_seed);

    // TODO: This sorting could be a no-op if we generate matches in order.
    // First sort by start, then by end, then by match cost.
    matches.sort_unstable_by_key(
        |&Match {
             start,
             end,
             match_cost,
             ..
         }| (LexPos(start), LexPos(end), match_cost),
    );
    // Dedup to only keep the lowest match cost.
    matches.dedup_by_key(|m| (m.start, m.end));

    SeedMatches {
        seeds,
        matches,
        start_of_seed,
        potential,
    }
}

/// Build a hashset of the kmers in b, and query all mutations of seeds in a.
pub fn find_matches_qgram_hash_inexact<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    alph: &Alphabet,
    MatchConfig {
        length,
        max_match_cost,
        ..
    }: MatchConfig,
) -> SeedMatches {
    let k: I = match length {
        Fixed(k) => k,
        _ => unimplemented!("QGram Hashing only works for fixed k for now."),
    };
    assert!(max_match_cost == 1);

    let rank_transform = RankTransform::new(alph);
    let width = rank_transform.get_width();

    let seeds = a
        .chunks_exact(k as usize)
        .enumerate()
        .map(|(i, seed)| Seed {
            start: i as I * k,
            end: (i + 1) as I * k,
            seed_potential: max_match_cost + 1,
            qgram: to_qgram(&rank_transform, width, seed),
        })
        .collect_vec();

    // type of Qgrams
    type Q = u64;
    assert!(k <= 31);

    // TODO: See if we can get rid of the Vec alltogether.
    let key = |l: Cost, w: usize| -> Q { ((w as Q) << 2) + (l + 1 - k) as Q };
    let mut m = HashMap::<Q, SmallVec<[Cost; 4]>>::default();
    m.reserve(3 * b.len());
    for (j, w) in rank_transform.qgrams(k - 1, b).enumerate() {
        m.entry(key(k - 1, w)).or_default().push(j as Cost);
    }
    for (j, w) in rank_transform.qgrams(k, b).enumerate() {
        m.entry(key(k, w)).or_default().push(j as Cost);
    }
    for (j, w) in rank_transform.qgrams(k + 1, b).enumerate() {
        m.entry(key(k + 1, w)).or_default().push(j as Cost);
    }
    let mut matches = Vec::<Match>::new();
    for &Seed { start, qgram, .. } in &seeds {
        if let Some(js) = m.get(&key(k, qgram)) {
            for &j in js {
                matches.push(Match {
                    start: Pos(start, j),
                    end: Pos(start + k, j + k),
                    match_cost: 0,
                    seed_potential: 2,
                });
            }
        }
        // We don't dedup here, since we'll be sorting and deduplicating the list of all matches anyway.
        let ms = mutations(k, qgram, false);
        for w in ms.deletions {
            if let Some(js) = m.get(&key(k - 1, w)) {
                for &j in js {
                    matches.push(Match {
                        start: Pos(start, j),
                        end: Pos(start + k, j + k - 1),
                        match_cost: 1,
                        seed_potential: 2,
                    });
                }
            }
        }
        for w in ms.substitutions {
            if let Some(js) = m.get(&key(k, w)) {
                for &j in js {
                    matches.push(Match {
                        start: Pos(start, j),
                        end: Pos(start + k, j + k),
                        match_cost: 1,
                        seed_potential: 2,
                    });
                }
            }
        }
        for w in ms.insertions {
            if let Some(js) = m.get(&key(k + 1, w)) {
                for &j in js {
                    matches.push(Match {
                        start: Pos(start, j),
                        end: Pos(start + k, j + k + 1),
                        match_cost: 1,
                        seed_potential: 2,
                    });
                }
            }
        }
    }

    // First sort by start, then by end, then by match cost.
    matches.sort_unstable_by_key(
        |&Match {
             start,
             end,
             match_cost,
             ..
         }| (LexPos(start), LexPos(end), match_cost),
    );
    // Dedup to only keep the lowest match cost for each (start, end) pair.
    //println!("Size before: {}", matches.len());
    matches.dedup_by_key(|m| (m.start, m.end));
    //println!("Size after : {}", matches.len());

    // Sort better matches for a given start first.
    matches.sort_unstable_by_key(
        |&Match {
             start, match_cost, ..
         }| (LexPos(start), match_cost),
    );

    // Compute some remaining data.
    let num_seeds = a.len() as I / k;

    let n = a.len();
    let mut potential = Vec::with_capacity(n + 1);
    let mut start_of_seed = Vec::with_capacity(n + 1);
    {
        let mut cur_potential = 2 * num_seeds;

        potential.push(cur_potential);
        for i in (0..a.len() - (k as usize - 1)).step_by(k as usize) {
            cur_potential -= 2;
            potential.extend(repeat(cur_potential).take(k as usize));
            start_of_seed.extend(repeat(i as I).take(k as usize));
        }

        // Backfill a potential gap after the last seed.
        potential.extend(repeat(0).take(n + 1 - potential.len()));
        start_of_seed.extend(repeat(num_seeds * k).take(n + 1 - start_of_seed.len()));
    }

    SeedMatches {
        seeds,
        matches,
        start_of_seed,
        potential,
    }
}

/// Build a hashset of the seeds in a, and query all kmers in b.
pub fn find_matches_qgram_hash_exact<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    alph: &Alphabet,
    MatchConfig {
        length,
        max_match_cost,
        ..
    }: MatchConfig,
) -> SeedMatches {
    let k: I = match length {
        Fixed(k) => k,
        _ => unimplemented!("QGram Hashing only works for fixed k for now."),
    };
    assert!(max_match_cost == 0);

    let rank_transform = RankTransform::new(alph);
    let width = rank_transform.get_width();

    let seeds = a
        .chunks_exact(k as usize)
        .enumerate()
        .map(|(i, seed)| Seed {
            start: i as I * k,
            end: (i + 1) as I * k,
            seed_potential: max_match_cost + 1,
            qgram: to_qgram(&rank_transform, width, seed),
        })
        .collect_vec();

    type Key = u64;

    // TODO: See if we can get rid of the Vec alltogether.
    let mut m = HashMap::<Key, SmallVec<[I; 4]>>::default();
    let mut matches = Vec::<Match>::new();

    if SLIDING_WINDOW_MATCHES {
        let capacity = a.len() / k as usize / (k - 1) as usize / 2;
        m.reserve(capacity);

        const CHECK_EACH_J_LAYERS: Cost = 6;

        // Target position.
        let p = Pos::from_length(a, b);
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
                    matches.push(Match {
                        start: Pos(i, j as I),
                        end: Pos(i + k, j as I + k),
                        match_cost: 0,
                        seed_potential: 1,
                    });
                }
            }
        }
    } else {
        m.reserve(a.len() / k as usize + 1);
        for (i, w) in rank_transform.qgrams(k, a).enumerate().step_by(k as usize) {
            m.entry(w as Key).or_default().push(i as I);
        }

        for (j, w) in rank_transform.qgrams(k, b).enumerate() {
            if let Some(is) = m.get(&(w as Key)) {
                for &i in is {
                    matches.push(Match {
                        start: Pos(i, j as I),
                        end: Pos(i + k, j as I + k),
                        match_cost: 0,
                        seed_potential: 1,
                    });
                }
            }
        }
    }

    // First sort by start, then by end, then by match cost.
    matches.sort_unstable_by_key(
        |&Match {
             start,
             end,
             match_cost,
             ..
         }| (LexPos(start), LexPos(end), match_cost),
    );
    // Dedup to only keep the lowest match cost.
    matches.dedup_by_key(|m| (m.start, m.end));

    // Compute some remaining data.
    let num_seeds = a.len() as I / k;

    let n = a.len();
    let mut potential = Vec::with_capacity(n + 1);
    let mut start_of_seed = Vec::with_capacity(n + 1);
    {
        let mut cur_potential = num_seeds;

        potential.push(cur_potential);
        for i in (0..a.len() - (k as usize - 1)).step_by(k as usize) {
            cur_potential -= 1;
            potential.extend(repeat(cur_potential).take(k as usize));
            start_of_seed.extend(repeat(i as I).take(k as usize));
        }

        // Backfill a potential gap after the last seed.
        potential.extend(repeat(0).take(n + 1 - potential.len()));
        start_of_seed.extend(repeat(num_seeds * k).take(n + 1 - start_of_seed.len()));
    }

    SeedMatches {
        seeds,
        matches,
        start_of_seed,
        potential,
    }
}

pub fn find_matches<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    alph: &Alphabet,
    match_config: MatchConfig,
) -> SeedMatches {
    if FIND_MATCHES_HASH {
        return match match_config.max_match_cost {
            0 => find_matches_qgram_hash_exact(a, b, alph, match_config),
            1 => find_matches_qgram_hash_inexact(a, b, alph, match_config),
            _ => unimplemented!("FIND_MATCHES with HashMap only works for max match cost 0 or 1"),
        };
    } else if FIND_MATCHES_TRIE {
        return find_matches_trie(a, b, alph, match_config);
    } else {
        return find_matches_qgramindex(a, b, alph, match_config);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Mutations {
    pub deletions: Vec<usize>,
    pub substitutions: Vec<usize>,
    pub insertions: Vec<usize>,
}

// TODO: Do not generate insertions at the end. (Also do not generate similar
// sequences by inserting elsewhere.)
// TODO: Move to seeds.rs.
pub fn mutations(k: I, kmer: usize, dedup: bool) -> Mutations {
    // This assumes the alphabet size is 4.
    let mut deletions = Vec::with_capacity(k as usize);
    let mut substitutions = Vec::with_capacity(4 * k as usize);
    let mut insertions = Vec::with_capacity(4 * (k + 1) as usize);
    // Substitutions
    for i in 0..k {
        let mask = !(3 << (2 * i));
        for s in 0..4 {
            // TODO: Skip the identity substitution.
            substitutions.push((kmer & mask) | s << (2 * i));
        }
    }
    // Insertions
    // TODO: Test that excluding insertions at the start and end doesn't matter.
    // NOTE: Apparently skipping insertions at the start is fine, but skipping at the end is not.
    for i in 0..=k {
        let mask = (1 << (2 * i)) - 1;
        for s in 0..4 {
            insertions.push((kmer & mask) | (s << (2 * i)) | ((kmer & !mask) << 2));
        }
    }
    // Deletions
    for i in 0..=k - 1 {
        let mask = (1 << (2 * i)) - 1;
        deletions.push((kmer & mask) | ((kmer & (!mask << 2)) >> 2));
    }
    if dedup {
        for v in [&mut deletions, &mut substitutions, &mut insertions] {
            // TODO: This sorting is slow; maybe we can work around it.
            v.sort_unstable();
            v.dedup();
        }
        // Remove original
        substitutions.retain(|&x| x != kmer);
    }
    Mutations {
        deletions,
        substitutions,
        insertions,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::matches::{self, mutations};
    use crate::{
        matches::{
            find_matches_qgram_hash_exact, find_matches_qgram_hash_inexact,
            find_matches_qgramindex, find_matches_trie,
        },
        prelude::{setup, to_string, MatchConfig, SLIDING_WINDOW_MATCHES},
    };

    #[test]
    fn trie_matches() {
        for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
            for n in [10, 20, 40, 100, 200, 500, 1000, 10000] {
                for e in [0.01, 0.1, 0.3, 1.0] {
                    let (a, b, alph, _) = setup(n, e);
                    println!("{}\n{}", to_string(&a), to_string(&b));
                    let matchconfig = MatchConfig {
                        length: crate::prelude::LengthConfig::Fixed(k),
                        max_match_cost,
                        ..Default::default()
                    };
                    println!("-----------------------");
                    println!("n={n} e={e} k={k} mmc={max_match_cost}");
                    let k = find_matches_trie(&a, &b, &alph, matchconfig);
                    let r = find_matches_qgramindex(&a, &b, &alph, matchconfig);
                    println!("-----------------------");
                    for x in &k.matches {
                        println!("{x:?}");
                    }
                    println!("-----------------------");
                    for x in &r.matches {
                        println!("{x:?}");
                    }
                    assert_eq!(k.matches, r.matches);
                }
            }
        }
    }

    #[test]
    fn hash_matches_exact() {
        // TODO: Replace max match distance from 0 to 1 here once supported.
        for (k, max_match_cost) in [(4, 0), (5, 0), (6, 0), (7, 0)] {
            for n in [10, 20, 40, 100, 200, 500, 1000, 10000] {
                for e in [0.01, 0.1, 0.3, 1.0] {
                    let (a, b, alph, _) = setup(n, e);
                    println!("{}\n{}", to_string(&a), to_string(&b));
                    let matchconfig = MatchConfig {
                        length: crate::prelude::LengthConfig::Fixed(k),
                        max_match_cost,
                        ..Default::default()
                    };
                    println!("-----------------------");
                    println!("n={n} e={e} k={k} mmc={max_match_cost}");
                    let r = find_matches_qgramindex(&a, &b, &alph, matchconfig);
                    let k = find_matches_qgram_hash_exact(&a, &b, &alph, matchconfig);
                    if !SLIDING_WINDOW_MATCHES {
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

    #[test]
    fn hash_matches_inexact() {
        // TODO: Replace max match distance from 0 to 1 here once supported.
        for (k, max_match_cost) in [(6, 1), (7, 1), (10, 1)] {
            for n in [40, 100, 200, 500, 1000, 10000] {
                for e in [0.01, 0.1, 0.3, 1.0] {
                    let (a, b, alph, _) = setup(n, e);
                    println!("{}\n{}", to_string(&a), to_string(&b));
                    let matchconfig = MatchConfig {
                        length: crate::prelude::LengthConfig::Fixed(k),
                        max_match_cost,
                        ..Default::default()
                    };
                    println!("-----------------------");
                    println!("n={n} e={e} k={k} mmc={max_match_cost}");
                    let mut r = find_matches_qgramindex(&a, &b, &alph, matchconfig);
                    let mut k = find_matches_qgram_hash_inexact(&a, &b, &alph, matchconfig);
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
            matches::Mutations {
                deletions: [6, 7, 11, 27].to_vec(),
                substitutions: [11, 19, 23, 24, 25, 26, 31, 43, 59, 91, 155, 219].to_vec(),
                insertions: [
                    27, 75, 91, 99, 103, 107, 108, 109, 110, 111, 123, 155, 219, 283, 539, 795
                ]
                .to_vec(),
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
}
