mod ordered;
mod qgrams;
pub mod suffix_array;

use crate::{config::SKIP_INEXACT_INSERT_START_END, prelude::*, seeds::*};
use bio::{
    alphabets::{Alphabet, RankTransform},
    data_structures::qgram_index::QGramIndex,
};

pub use ordered::*;
pub use qgrams::fixed_seeds;
use qgrams::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatchStatus {
    Active,
    Pruned,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    pub start: Pos,
    pub end: Pos,
    pub match_cost: MatchCost,
    pub seed_potential: MatchCost,
    pub pruned: MatchStatus,
}

impl Match {
    pub fn score(&self) -> MatchCost {
        self.seed_potential - self.match_cost
    }

    pub fn is_active(&self) -> bool {
        self.pruned == MatchStatus::Active
    }

    pub fn prune(&mut self) {
        assert!(self.pruned == MatchStatus::Active);
        self.pruned = MatchStatus::Pruned;
    }
}

#[derive(Default)]
pub struct Matches {
    pub seeds: Seeds,
    /// Sorted by start (i, j).
    /// Empty for unordered matching.
    pub matches: Vec<Match>,
}

impl Matches {
    /// Seeds must be sorted by start.
    /// Matches will be sorted and deduplicated in this function.
    pub fn new(a: Seq, seeds: Vec<Seed>, mut matches: Vec<Match>) -> Self {
        // First sort by start, then by end, then by match cost.
        matches.sort_unstable_by_key(|m| (LexPos(m.start), LexPos(m.end), m.match_cost));
        // Dedup to only keep the lowest match cost between each start and end.
        matches.dedup_by_key(|m| (m.start, m.end));

        Matches {
            seeds: Seeds::new(a, seeds),
            matches,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MaxMatches {
    /// The smallest k with at most this many matches.
    pub max_matches: usize,
    /// Range of k to consider.
    pub k_min: I,
    pub k_max: I,
}

#[derive(Clone, Copy, Debug)]
pub enum LengthConfig {
    Fixed(I),
    Max(MaxMatches),
}
use LengthConfig::*;

impl LengthConfig {
    pub fn k(&self) -> Option<I> {
        match *self {
            Fixed(k) => Some(k),
            _ => None,
        }
    }
    pub fn kmax(&self) -> I {
        match *self {
            Fixed(k) => k,
            LengthConfig::Max(MaxMatches { k_max, .. }) => k_max,
        }
    }
    pub fn kmin(&self) -> I {
        match *self {
            Fixed(k) => k,
            LengthConfig::Max(MaxMatches { k_min, .. }) => k_min,
        }
    }
    pub fn max_matches(&self) -> Option<usize> {
        match *self {
            Fixed(_) => None,
            LengthConfig::Max(MaxMatches { max_matches, .. }) => Some(max_matches),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MatchConfig {
    // TODO: Add settings for variable length matches in here.
    pub length: LengthConfig,
    // TODO: Move the max_match_cost into MatchLength.
    pub max_match_cost: MatchCost,
}

impl MatchConfig {
    pub fn new(k: I, max_match_cost: MatchCost) -> Self {
        Self {
            length: Fixed(k),
            max_match_cost,
        }
    }
    pub fn exact(k: I) -> Self {
        Self {
            length: Fixed(k),
            max_match_cost: 0,
        }
    }
    pub fn inexact(k: I) -> Self {
        Self {
            length: Fixed(k),
            max_match_cost: 1,
        }
    }
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            length: Fixed(0),
            max_match_cost: 0,
        }
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
fn mutations(k: I, qgram: usize, dedup: bool, insert_at_start: bool) -> Mutations {
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
    // TODO: Test that excluding insertions at the start and end doesn't matter.
    // NOTE: Apparently skipping insertions at the end is fine, but with gapcost, skipping at the start is not.
    // 1: skip insert at end (vs 0)
    // ..k: skip insert at start (vs ..=k)
    // NOTE: start (low order bits) correspond to the end of the kmer.
    let mut forbidden_duplicate_head = usize::MAX;
    let mut forbidden_duplicate_tail = usize::MAX;
    let start = if SKIP_INEXACT_INSERT_START_END {
        forbidden_duplicate_tail = (qgram << 2) | (qgram & 3);
        1
    } else {
        0
    };
    // NOTE: end (high order bits) correspond to the start of the kmer.
    let end = if insert_at_start || !SKIP_INEXACT_INSERT_START_END {
        k + 1
    } else {
        forbidden_duplicate_head = qgram | ((qgram >> (2 * k - 2)) << 2 * k);
        k
    };
    for i in start..end {
        let mask = (1 << (2 * i)) - 1;
        for s in 0..4 {
            let candidate = (qgram & mask) | (s << (2 * i)) | ((qgram & !mask) << 2);
            if candidate != forbidden_duplicate_head && candidate != forbidden_duplicate_tail {
                insertions.push(candidate);
            }
        }
    }
    // Deletions
    for i in 0..=k - 1 {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mutations() {
        let kmer = 0b00011011usize;
        let k = 4;
        let ms = mutations(k, kmer, true, true);
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
                insertions: if SKIP_INEXACT_INSERT_START_END {
                    [27, 75, 91, 99, 103, 107, 123, 155, 219, 283, 539, 795].to_vec()
                } else {
                    [
                        27, 75, 91, 99, 103, 107, 108, 109, 110, 111, 123, 155, 219, 283, 539, 795,
                    ]
                    .to_vec()
                }
            }
        );
    }

    #[test]
    fn kmer_removal() {
        let kmer = 0b00011011usize;
        let k = 4;
        let ms = mutations(k, kmer, true, true);
        assert!(!ms.substitutions.contains(&kmer));
        assert!(ms.deletions.contains(&kmer));
        assert!(ms.insertions.contains(&kmer));
    }
}
