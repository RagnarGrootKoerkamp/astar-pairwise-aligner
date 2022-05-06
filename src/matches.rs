pub mod ordered;
pub mod qgrams;
pub mod unordered;

use itertools::Itertools;
use strum_macros::EnumString;

pub use ordered::*;
pub use qgrams::*;
pub use unordered::*;

#[derive(Clone, Debug)]
pub struct Seed {
    pub start: I,
    pub end: I,
    /// The seed_potential is 1 more than the maximal number of errors allowed in this seed.
    pub seed_potential: MatchCost,
    /// A lower bound on the cost of crossing this seed.
    /// For unordered matches, if this is < seed_potential there must be exactly one such seed.
    pub seed_cost: MatchCost,
    pub qgram: usize,
}

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

#[derive(Default)]
pub struct SeedMatches {
    /// Sorted by start.
    pub seeds: Vec<Seed>,
    /// Sorted by start (i, j).
    /// Empty for unordered matching.
    pub matches: Vec<Match>,
    /// The index of the seed covering position I.
    /// Seeds cover [start, end) here.
    pub seed_at: Vec<Option<I>>,
    /// The sum of seed potentials of all seeds not starting before each position.
    pub potential: Vec<Cost>,
}

impl SeedMatches {
    /// Seeds must be sorted by start.
    /// Matches will be sorted and deduplicated in this function.
    pub fn new(a: &Sequence, seeds: Vec<Seed>, mut matches: Vec<Match>) -> Self {
        // Check that seeds are sorted and non-overlapping.
        assert!(seeds.is_sorted_by_key(|seed| seed.start));
        assert!(seeds
            .iter()
            .tuple_windows()
            .all(|(seed1, seed2)| seed1.end <= seed2.start));

        // First sort by start, then by end, then by match cost.
        matches.sort_unstable_by_key(|m| (LexPos(m.start), LexPos(m.end), m.match_cost));
        // Dedup to only keep the lowest match cost.
        matches.dedup_by_key(|m| (m.start, m.end));

        let n = a.len();
        let mut potential = vec![0; n + 1];
        let mut seed_at = vec![None; n + 1];
        let mut cur_potential = 0;
        let mut next_seed = seeds.iter().enumerate().rev().peekable();
        for i in (0..=n).rev() {
            if let Some((seed_idx, ns)) = next_seed.peek() {
                if i < ns.end as usize {
                    seed_at[i] = Some(*seed_idx as I);
                }

                if i == ns.start as usize {
                    cur_potential += ns.seed_potential as Cost;
                    next_seed.next();
                }
            }
            potential[i] = cur_potential;
        }
        SeedMatches {
            seeds,
            matches,
            seed_at,
            potential,
        }
    }

    /// The potential at p is the cost of going from p to the end, without hitting any matches.
    #[inline]
    pub fn potential(&self, Pos(i, _): Pos) -> Cost {
        self.potential[i as usize]
    }

    #[inline]
    pub fn potential_distance(&self, from: Pos, to: Pos) -> Cost {
        assert!(from.0 <= to.0);
        let end_i = self.seed_at(to).map_or(to.0, |s| s.start);
        self.potential[from.0 as usize] - self.potential[end_i as usize]
    }

    /// The seed covering a given position.
    #[inline]
    pub fn seed_at(&self, Pos(i, _): Pos) -> Option<&Seed> {
        match self.seed_at[i as usize] {
            Some(idx) => Some(&self.seeds[idx as usize]),
            None => None,
        }
    }

    /// The seed ending in the given position.
    #[inline]
    pub fn seed_ending_at(&self, Pos(i, _): Pos) -> Option<&Seed> {
        if i == 0 {
            None
        } else {
            match self.seed_at[i as usize - 1] {
                Some(idx) => Some(&self.seeds[idx as usize]),
                None => None,
            }
        }
    }

    #[inline]
    pub fn is_seed_start(&self, pos: Pos) -> bool {
        self.seed_at(pos).map_or(false, |s| pos.0 == s.start)
    }

    #[inline]
    pub fn is_seed_end(&self, pos: Pos) -> bool {
        self.seed_ending_at(pos).map_or(false, |s| pos.0 == s.end)
    }

    #[inline]
    pub fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        self.is_seed_start(pos) || self.is_seed_end(pos)
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
}

#[derive(Clone, Copy, Debug, Default, EnumString, strum_macros::Display)]
#[strum(ascii_case_insensitive)]
pub enum MatchAlgorithm {
    #[default]
    Hash,
    HashSet,
    Bloom,
    Cuckoo,
}

#[derive(Clone, Copy, Debug)]
pub struct MatchConfig {
    // TODO: Add settings for variable length matches in here.
    pub length: LengthConfig,
    // TODO: Move the max_match_cost into MatchLength.
    pub max_match_cost: MatchCost,
    pub algorithm: MatchAlgorithm,
    pub window_filter: bool,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            length: Fixed(0),
            max_match_cost: 0,
            algorithm: MatchAlgorithm::default(),
            window_filter: false,
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
pub fn mutations(k: I, qgram: usize, dedup: bool) -> Mutations {
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
    // NOTE: Apparently skipping insertions at the start is fine, but skipping at the end is not.
    for i in 0..=k {
        let mask = (1 << (2 * i)) - 1;
        for s in 0..4 {
            insertions.push((qgram & mask) | (s << (2 * i)) | ((qgram & !mask) << 2));
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
