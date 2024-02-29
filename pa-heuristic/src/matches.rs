// Modules are pub for benchmarking.
pub mod exact;
pub mod inexact;
mod local_pruning;
pub mod qgrams;
mod suffix_array;

use crate::{prelude::*, seeds::*, PRINT};
use bio::{
    alphabets::{Alphabet, RankTransform},
    data_structures::qgram_index::QGramIndex,
};
use local_pruning::preserve_for_local_pruning;

/// Find all matches between `a` and `b` with the given match configuration.
/// If `transform_filter` is true, then only matches with T(m.start) <= target are kept.
pub fn find_matches<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    match_config: MatchConfig,
    transform_filter: bool,
) -> Matches {
    if let LengthConfig::Max(_) = match_config.length {
        return suffix_array::minimal_unique_matches(a, b, match_config);
    }
    if FIND_MATCHES_HASH {
        return match match_config.r {
            1 => exact::hash_a(a, b, match_config, transform_filter),
            2 => inexact::find_matches_qgram_hash_inexact(a, b, match_config, transform_filter),
            _ => unimplemented!("FIND_MATCHES with HashMap only works for r = 1 or r = 2"),
        };
    } else {
        return match match_config.r {
            1 => exact::find_matches_qgramindex(a, b, match_config, transform_filter),
            2 => inexact::find_matches_qgramindex(a, b, match_config, transform_filter),
            _ => unimplemented!(),
        };
    }
}

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
    qgrams: &'a QGrams<'a>,
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
    /// New MatchBuilder with fixed length seeds.
    fn new(qgrams: &'a QGrams<'a>, config: MatchConfig, transform_filter: bool) -> Self {
        let seeds = Seeds::new(
            qgrams.a,
            qgrams.fixed_length_seeds(config.length.k().unwrap(), config.r),
        );
        let transform_target = seeds.transform(Pos::target(qgrams.a, qgrams.b));
        let d = transform_target.0 - transform_target.1;
        Self {
            qgrams,
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

    fn new_with_seeds(
        qgrams: &'a QGrams<'a>,
        config: MatchConfig,
        transform_filter: bool,
        seeds: Vec<Seed>,
    ) -> Self {
        let seeds = Seeds::new(qgrams.a, seeds);
        let transform_target = seeds.transform(Pos::target(qgrams.a, qgrams.b));
        let d = transform_target.0 - transform_target.1;
        Self {
            qgrams,
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
        if self.config.local_pruning != 0
            && !preserve_for_local_pruning(
                self.qgrams.a,
                self.qgrams.b,
                &self.seeds,
                &m,
                self.config.local_pruning,
                &mut self.local_pruning_cache,
                &mut self.next_match_per_diag,
            )
        {
            return false;
        }
        self.stats.after_local_pruning += 1;

        // Checks have passed; add the match.

        let sc = &mut self.seeds.seed_at_mut(m.start).unwrap().seed_cost;
        *sc = min(*sc, m.match_cost);

        if self.config.local_pruning != 0 {
            let d = m.start.0 - m.start.1;
            let old = self.next_match_per_diag.index_mut(d);
            assert!(
                *old >= m.start.0,
                "Matches should be added in reverse order (right-to-left or bot-to-top) on each diagonal."
            );
            *old = m.start.0;
        }

        self.matches.push(m);

        true
    }

    fn sort(&mut self) {
        self.matches
            .sort_by_key(|m| (LexPos(m.start), LexPos(m.end), m.match_cost));
    }

    fn finish(mut self) -> Matches {
        // First sort by start, then by end, then by match cost.
        self.sort();
        // Dedup to only keep the lowest match cost between each start and end.
        self.matches.dedup_by_key(|m| (m.start, m.end));

        if PRINT && self.config.local_pruning > 0 {
            eprintln!(
                "Matches after:
        pushed        {:>8}
        transform     {:>8}
        local pruning {:>8}",
                self.stats.pushed, self.stats.after_transform, self.stats.after_local_pruning
            );

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

use self::qgrams::QGrams;

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
