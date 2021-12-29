use std::iter::repeat;

use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct Match {
    pub start: Pos,
    pub end: Pos,
    pub match_cost: usize,
    pub max_match_cost: usize,
}

#[derive(Default)]
pub struct SeedMatches {
    // Sorted by (i, j)
    pub num_seeds: usize,
    pub matches: Vec<Match>,
    // Index of the start of the rightmost seed covering the given position.
    pub start_of_seed: Vec<usize>,
    potential: Vec<usize>,
}

impl SeedMatches {
    pub fn iter(&self) -> std::slice::Iter<Match> {
        self.matches.iter()
    }

    // The potential at p is the cost of going from p to the end, without hitting any matches.
    pub fn potential(&self, Pos(i, _): Pos) -> usize {
        self.potential[i]
    }

    // TODO: Generalize this for overlapping seeds.
    pub fn is_start_of_seed(&self, Pos(i, _): Pos) -> bool {
        self.start_of_seed[i] == i
    }
}

impl<'a> HeuristicInstance<'a> for SeedMatches {
    fn h(&self, _: Node<Self::IncrementalState>) -> usize {
        unimplemented!("SeedMatches can only be used as a distance, not as a heuristic!");
    }
}
impl<'a> DistanceHeuristicInstance<'a> for SeedMatches {
    /// The minimal distance is the potential of the seeds entirely within the `[from, to)` interval.
    /// NOTE: Assumes disjoint seeds.
    fn distance(&self, from: Pos, to: Pos) -> usize {
        assert!(from.0 <= to.0);
        self.potential[from.0] - (self.potential[self.start_of_seed[to.0]])
    }
}

pub fn find_matches<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    text_alphabet: &Alphabet,
    l: usize,
    max_match_cost: usize,
) -> SeedMatches {
    assert!(max_match_cost == 0 || max_match_cost == 1);
    // Convert to a binary sequences.
    let rank_transform = RankTransform::new(text_alphabet);

    // Split a into seeds of size l and l+1 alternating, which are bit-encoded.
    // (start, end, max_match_cost, ngram value)
    let seed_qgrams = {
        // TODO: Make a dedicated struct for seeds, apart from Matches.
        let mut v: Vec<(usize, usize, usize, usize)> = Vec::default();
        let mut a = &a[..];
        let mut long = false;
        let mut pos = 0;
        loop {
            // TODO: Clever seed choice, using variable l and m.
            let seed_len = l;
            if seed_len > a.len() {
                break;
            }

            let (seed, tail) = a.split_at(seed_len);
            a = tail;

            v.push((
                pos,
                pos + seed_len,
                max_match_cost,
                rank_transform.qgrams(seed_len as u32, seed).next().unwrap(),
            ));
            pos += seed_len;

            long = !long;
        }
        v
    };
    let num_seeds = seed_qgrams.len();

    let n = a.len();
    let mut potential = Vec::with_capacity(n + 1);
    let mut start_of_seed = Vec::with_capacity(n + 1);

    // Find matches of the seeds of a in b.
    // NOTE: This uses O(alphabet^l) memory.
    let mut matches = Vec::<Match>::new();

    let mut qgrams = HashMap::<usize, QGramIndex>::default();
    for l in [l - 1, l, l + 1] {
        // TODO: Profile this index and possibly use something more efficient for large l.
        qgrams.insert(l, QGramIndex::new(l as u32, b, text_alphabet));
    }

    let mut cur_potential = seed_qgrams.iter().map(|(_, _, cost, _)| cost + 1).sum();
    potential.push(cur_potential);
    //println!("{:?}", seed_qgrams);
    for &(start, end, max_match_cost, seed) in &seed_qgrams {
        let len = end - start;
        cur_potential -= max_match_cost + 1;
        potential.extend(repeat(cur_potential).take(len));
        start_of_seed.extend(repeat(start).take(len));

        // Exact matches
        for &j in qgrams[&len].qgram_matches(seed) {
            matches.push(Match {
                start: Pos(start, j),
                end: Pos(end, j + len),
                match_cost: 0,
                max_match_cost,
            });
        }
        // Inexact matches.
        if max_match_cost == 1 {
            let mutations = mutations(len, seed);
            for mutation in mutations.deletions {
                for &j in qgrams[&(len - 1)].qgram_matches(mutation) {
                    matches.push(Match {
                        start: Pos(start, j),
                        end: Pos(end, j + len - 1),
                        match_cost: 1,
                        max_match_cost,
                    });
                }
            }
            for mutation in mutations.substitutions {
                for &j in qgrams[&len].qgram_matches(mutation) {
                    matches.push(Match {
                        start: Pos(start, j),
                        end: Pos(end, j + len),
                        match_cost: 1,
                        max_match_cost,
                    });
                }
            }
            for mutation in mutations.insertions {
                for &j in qgrams[&(len + 1)].qgram_matches(mutation) {
                    matches.push(Match {
                        start: Pos(start, j),
                        end: Pos(end, j + len + 1),
                        match_cost: 1,
                        max_match_cost,
                    });
                }
            }
        }
    }
    // Backfill a potential gap after the last seed.
    potential.extend(repeat(0).take(n + 1 - potential.len()));
    start_of_seed.extend(repeat(seed_qgrams.last().unwrap().1).take(n + 1 - start_of_seed.len()));

    //println!("{:?}", potential);
    //println!("{:?}", start_of_seed);

    // TODO: This sorting could be a no-op if we generate matches in order.
    matches.sort_unstable_by_key(|&Match { start, .. }| (start.0, start.1));
    //for m in &matches {
    //println!("{:?}", m);
    //}

    SeedMatches {
        num_seeds,
        matches,
        start_of_seed,
        potential,
    }
}
