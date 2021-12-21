use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct Match {
    pub start: Pos,
    pub end: Pos,
    pub match_cost: usize,
}

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
    fn h(&self, pos: Node<Self::IncrementalState>) -> usize {
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
    a_text: &'a Sequence,
    b_text: &'a Sequence,
    text_alphabet: &Alphabet,
    l: usize,
    max_match_cost: usize,
) -> SeedMatches {
    assert!(max_match_cost == 0 || max_match_cost == 1);
    // Convert to a binary sequences.
    let rank_transform = RankTransform::new(text_alphabet);

    // Split a into seeds of size l, which are encoded as `usize`.
    // (pos, ngram value)
    let seed_qgrams: Vec<(usize, usize)> = a_text
        .chunks_exact(l)
        .enumerate()
        .map(|(i, s)| (l * i, s))
        // A chunk of size l has exactly one qgram of length l.
        .map(|(i, seed)| (i, rank_transform.qgrams(l as u32, seed).next().unwrap()))
        .collect::<Vec<_>>();

    let num_seeds = seed_qgrams.len();

    let n = a_text.len();
    let mut potential = Vec::with_capacity(n + 1);
    let mut start_of_seed = Vec::with_capacity(n + 1);
    for i in 0..=n {
        potential.push((max_match_cost + 1) * (n / l - min(i + l - 1, n) / l));
        start_of_seed.push(i / l * l);
    }
    let potential = potential;
    let start_of_seed = start_of_seed;

    // Find matches of the seeds of a in b.
    // NOTE: This uses O(alphabet^l) memory.
    let mut matches = Vec::<Match>::new();

    let qgram_index = QGramIndex::new(l as u32, b_text, &text_alphabet);
    let qgram_index_deletions = QGramIndex::new(l as u32 - 1, b_text, &text_alphabet);
    let qgram_index_insertions = QGramIndex::new(l as u32 + 1, b_text, &text_alphabet);

    for (i, seed) in seed_qgrams {
        // Exact matches
        for &j in qgram_index.qgram_matches(seed) {
            matches.push(Match {
                start: Pos(i, j),
                end: Pos(i + l, j + l),
                match_cost: 0,
            });
        }
        // Inexact matches.
        if max_match_cost == 1 {
            let mutations = mutations(l, seed);
            for mutation in mutations.deletions {
                for &j in qgram_index_deletions.qgram_matches(mutation) {
                    matches.push(Match {
                        start: Pos(i, j),
                        end: Pos(i + l, j + l - 1),
                        match_cost: 1,
                    });
                }
            }
            for mutation in mutations.substitutions {
                for &j in qgram_index.qgram_matches(mutation) {
                    matches.push(Match {
                        start: Pos(i, j),
                        end: Pos(i + l, j + l),
                        match_cost: 1,
                    });
                }
            }
            for mutation in mutations.insertions {
                for &j in qgram_index_insertions.qgram_matches(mutation) {
                    matches.push(Match {
                        start: Pos(i, j),
                        end: Pos(i + l, j + l + 1),
                        match_cost: 1,
                    });
                }
            }
        }
    }

    matches.sort_by_key(|&Match { start, .. }| (start.0, start.1));

    SeedMatches {
        num_seeds,
        matches,
        start_of_seed,
        potential,
    }
}
