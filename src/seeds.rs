use crate::util::*;

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
        i + 1 < self.potential.len() && self.potential[i] > self.potential[i + 1]
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
    for i in 0..=n {
        potential.push((max_match_cost + 1) * (n / l - min(i + l - 1, n) / l));
    }

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
        potential,
    }
}
