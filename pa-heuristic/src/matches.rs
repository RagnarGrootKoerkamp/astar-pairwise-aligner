mod local_pruning;
mod ordered;
mod qgrams;
pub mod suffix_array;

use crate::{matches::local_pruning::preserve_for_local_pruning, prelude::*, seeds::*};
use bio::{
    alphabets::{Alphabet, RankTransform},
    data_structures::qgram_index::QGramIndex,
};

pub use ordered::*;
pub use qgrams::fixed_seeds;
use qgrams::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatchStatus {
    /// Active
    Active,
    /// Pruned by match pruning because the start or end was expanded.
    Pruned,
    /// Filtered out by PatHeuristic
    Filtered,
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
    #[inline]
    pub fn score(&self) -> MatchCost {
        self.seed_potential - self.match_cost
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.pruned == MatchStatus::Active
    }

    #[inline]
    pub fn prune(&mut self) {
        debug_assert!(self.pruned == MatchStatus::Active);
        self.pruned = MatchStatus::Pruned;
    }

    #[inline]
    pub fn filter(&mut self) {
        debug_assert!(self.pruned == MatchStatus::Active);
        self.pruned = MatchStatus::Filtered;
    }
}

/// A vector that is centered around 0.
struct CenteredVec<T> {
    vec: Vec<T>,

    default: T,
}

impl<T: Copy> CenteredVec<T> {
    fn new(center: I, default: T) -> Self {
        Self {
            vec: vec![default; 2 * center.abs() as usize + 1],
            default,
        }
    }
    fn index(&self, index: I) -> T {
        self.vec
            .get((index + self.vec.len() as I / 2) as usize)
            .copied()
            .unwrap_or(self.default)
    }
    fn index_mut(&mut self, index: I) -> &mut T {
        if index.abs() > self.vec.len() as I / 2 {
            // Grow to contain the index and at least double in size.
            let old_mid = self.vec.len() / 2;
            let new_mid = max(index.abs() as usize, self.vec.len());
            let grow = new_mid - old_mid;
            self.vec
                .splice(0..0, std::iter::repeat(self.default).take(grow));
            self.vec.extend(std::iter::repeat(self.default).take(grow));
            assert_eq!(self.vec.len() / 2, new_mid);
        }
        let mid = self.vec.len() as I / 2;
        &mut self.vec[(index + mid) as usize]
    }
}

/// Helper for constructing and filtering matches.
///
/// Note that this requires the seeds to be already determined, since they are
/// required for the transform filter.
struct MatchBuilder<'a> {
    a: Seq<'a>,
    b: Seq<'a>,
    config: MatchConfig,
    seeds: Seeds,
    matches: Vec<Match>,

    transform_filter: bool,
    transform_target: Pos,

    local_pruning_cache: [Vec<I>; 3],

    /// The i of the next (left/topmost) match on each diagonal.
    next_match_per_diag: CenteredVec<I>,

    stats: MatchStats,
}

#[derive(Default)]
struct MatchStats {
    pushed: usize,
    after_transform: usize,
    after_local_pruning: usize,
}

impl<'a> MatchBuilder<'a> {
    fn new(
        a: Seq<'a>,
        b: Seq<'a>,
        config: MatchConfig,
        seeds: Vec<Seed>,
        transform_filter: bool,
    ) -> Self {
        let seeds = Seeds::new(a, seeds);
        let transform_target = seeds.transform(Pos::target(a, b));
        let d = transform_target.0 - transform_target.1;
        Self {
            a,
            b,
            config,
            seeds,
            matches: Vec::new(),
            transform_target,
            transform_filter,
            local_pruning_cache: Default::default(),
            stats: MatchStats::default(),
            // Make space for the 0 and target diagonal, and 10 padding on each side.
            next_match_per_diag: CenteredVec::new(d, I::MAX),
        }
    }

    /// Add a new match. If enabled, filters for m.start <=_T end and/or local pruning.
    /// Returns whether the match was added.
    fn push(&mut self, m: Match) -> bool {
        self.stats.pushed += 1;
        if self.transform_filter && !(self.seeds.transform(m.start) <= self.transform_target) {
            return false;
        }
        self.stats.after_transform += 1;
        if !preserve_for_local_pruning(
            self.a,
            self.b,
            &self.seeds,
            &m,
            self.config.local_pruning,
            &mut self.local_pruning_cache,
            &mut self.next_match_per_diag,
        ) {
            return false;
        }
        self.stats.after_local_pruning += 1;

        // Checks have passed; add the match.

        let sc = &mut self.seeds.seed_at_mut(m.start).unwrap().seed_cost;
        *sc = min(*sc, m.match_cost);

        let d = m.start.0 - m.start.1;
        let old = self.next_match_per_diag.index_mut(d);
        assert!(
            *old >= m.start.0,
            "Matches should be added in reverse order (right-to-left or bot-to-top) on each diagonal."
        );
        *old = m.start.0;

        self.matches.push(m);

        true
    }

    fn finish(mut self) -> Matches {
        // First sort by start, then by end, then by match cost.
        assert!(self
            .matches
            .is_sorted_by_key(|m| (LexPos(m.start), LexPos(m.end), m.match_cost)));
        // Dedup to only keep the lowest match cost between each start and end.
        self.matches.dedup_by_key(|m| (m.start, m.end));

        eprintln!(
            "Matches after:
  pushed        {:>8}
  transform     {:>8}
  local pruning {:>8}",
            self.stats.pushed, self.stats.after_transform, self.stats.after_local_pruning
        );

        if self.config.local_pruning > 0 {
            eprintln!("Local pruning up to");
            for (g, cnt) in self.local_pruning_cache[2].iter().enumerate() {
                eprint!("{g:>0$} ", format!("{cnt}").len());
            }
            eprintln!();
            for cnt in &self.local_pruning_cache[2] {
                eprint!("{cnt} ");
            }
            eprintln!();
        }

        Matches {
            seeds: self.seeds,
            matches: self.matches,
        }
    }
}

/// A wrapper to contain all seed and match information.
pub struct Matches {
    pub seeds: Seeds,
    /// Sorted by start (i, j).
    /// Empty for unordered matching.
    pub matches: Vec<Match>,
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
    /// The length of each seed, either a fixed `k`, or variable such that the
    /// number of matches per seed is limited.
    // TODO: Add settings for variable length matches in here.
    pub length: LengthConfig,
    /// The maximal cost per match, i.e. `r-1`.
    // TODO: Move r into MatchLength.
    pub r: MatchCost,
    /// The number of seeds to 'look ahead' in local pruning.
    pub local_pruning: usize,
}

impl MatchConfig {
    pub fn new(k: I, r: MatchCost) -> Self {
        Self {
            length: Fixed(k),
            r,
            local_pruning: 0,
        }
    }
    pub fn exact(k: I) -> Self {
        Self {
            length: Fixed(k),
            r: 1,
            local_pruning: 0,
        }
    }
    pub fn inexact(k: I) -> Self {
        Self {
            length: Fixed(k),
            r: 2,
            local_pruning: 0,
        }
    }
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            length: Fixed(0),
            r: 1,
            local_pruning: 0,
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

#[cfg(test)]
mod test {
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
}
