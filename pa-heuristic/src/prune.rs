use std::ops::Range;

use crate::{matches::Match, prelude::*, seeds::MatchCost};
use clap::ValueEnum;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::seeds::Seeds;

#[derive(Debug, ValueEnum, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Prune {
    None,
    Start,
    End,
    Both,
}
impl Prune {
    pub fn is_enabled(&self) -> bool {
        match self {
            Prune::None => false,
            _ => true,
        }
    }
    pub fn start(&self) -> bool {
        match self {
            Prune::None | Prune::End => false,
            Prune::Start | Prune::Both => true,
        }
    }
    pub fn end(&self) -> bool {
        match self {
            Prune::None | Prune::Start => false,
            Prune::End | Prune::Both => true,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Pruning {
    pub enabled: Prune,
    /// Skip pruning one in N.
    pub skip_prune: Option<usize>,
}

impl Default for Pruning {
    fn default() -> Self {
        Self::start()
    }
}

impl Pruning {
    pub fn new(enabled: Prune) -> Self {
        Self {
            enabled,
            skip_prune: None,
        }
    }
    pub fn disabled() -> Self {
        Pruning {
            enabled: Prune::None,
            skip_prune: None,
        }
    }
    pub fn start() -> Self {
        Pruning {
            enabled: Prune::Start,
            skip_prune: None,
        }
    }
    pub fn both() -> Self {
        Pruning {
            enabled: Prune::Both,
            skip_prune: None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self.enabled {
            Prune::None => false,
            _ => true,
        }
    }
    pub fn prune_start(&self) -> bool {
        match self.enabled {
            Prune::None | Prune::End => false,
            Prune::Start | Prune::Both => true,
        }
    }
    pub fn prune_end(&self) -> bool {
        match self.enabled {
            Prune::None | Prune::Start => false,
            Prune::End | Prune::Both => true,
        }
    }
}

/// Datastructure that holds all matches and allows for efficient lookup of
/// matches by start, end (if needed), and range.
///
/// TODO: Memory could be saved by using `Range<u32>` or only the start `u32`.
/// TODO: More memory could be saved by reusing the sorting by start also to find matches by end.
pub struct MatchPruner {
    pruning: Pruning,
    check_consistency: bool,
    /// Skip a prune when this reaches 0 and `skip_prune` is set.
    skip: usize,

    /// Matches, sorted by `(LexPos(start), match_cost)`.
    by_start: Vec<Match>,
    /// For each match start, the index `matches_by_start` where matches start.
    start_index: HashMap<Pos, Range<usize>>,

    /// Matches, sorted by `(LexPos(end), match_cost)`.
    by_end: Vec<Match>,
    /// For each match end, the index `matches_by_end` where matches end.
    end_index: HashMap<Pos, Range<usize>>,
}

impl MatchPruner {
    pub fn new(
        pruning: Pruning,
        check_consistency: bool,
        mut matches_by_start: Vec<Match>,
    ) -> MatchPruner {
        // Sort by start, then by  match cost.
        // This ensures that matches are pruned from low cost to high cost.
        let positions = |matches: &mut Vec<Match>, f: fn(&Match) -> Pos| {
            matches.sort_by_key(|m| (LexPos(f(m)), m.match_cost));
            matches
                .iter()
                .enumerate()
                .group_by(|(_, m)| f(m))
                .into_iter()
                .map(|(pos, mut ms)| {
                    (pos, {
                        let start = ms.next().unwrap().0;
                        let end = ms.last().map_or(start, |x| x.0) + 1;
                        start..end
                    })
                })
                .collect()
        };
        let by_start = positions(&mut matches_by_start, |m| m.start);

        let (matches_by_end, by_end) = if pruning.prune_end() {
            let mut matches_by_end = matches_by_start.clone();
            let by_end = positions(&mut matches_by_end, |m| m.end);
            (matches_by_end, by_end)
        } else {
            Default::default()
        };

        MatchPruner {
            pruning,
            check_consistency,
            skip: 1,
            by_start: matches_by_start,
            start_index: by_start,
            by_end: matches_by_end,
            end_index: by_end,
        }
    }

    /// Iterates over all matches starting in the given `pos`.
    pub fn matches_for_start(&self, pos: Pos) -> Option<&[Match]> {
        Some(&self.by_start[self.start_index.get(&pos)?.clone()])
    }

    /// Iterates over all matches sorted by `LexPos(start)`.
    pub fn iter(&self) -> impl '_ + DoubleEndedIterator<Item = &Match> {
        self.by_start.iter()
    }

    /// Returns number of matches pruned by start (succeeding this pos) and by end (preceding this pos).
    pub fn prune(&mut self, seeds: &Seeds, pos: Pos, mut f: impl FnMut(&Match)) -> (usize, usize) {
        let mut cnt = (0, 0);
        if self.pruning.prune_start() && seeds.is_seed_start(pos) {
            if let Some(ms) = self.start_index.get(&pos).cloned() {
                for i in ms {
                    let m = &self.by_start[i].clone();
                    if m.is_active() && self.check_consistency(m) && self.skip_prune_filter() {
                        self.prune_match(m);
                        cnt.0 += 1;
                        f(m);
                    }
                }
            }
        };
        if self.pruning.prune_end() && seeds.is_seed_end(pos) {
            if let Some(ms) = self.end_index.get(&pos).cloned() {
                for i in ms {
                    let m = &self.by_end[i].clone();
                    if m.is_active() && self.check_consistency(m) && self.skip_prune_filter() {
                        self.prune_match(m);
                        cnt.0 += 1;
                        f(m);
                    }
                }
            }
        };
        cnt
    }

    fn prune_match(&mut self, m: &Match) {
        self.mut_match_start(m).unwrap().prune();
        self.mut_match_end(m).unwrap().prune();
    }

    pub fn mut_match_start(&mut self, m: &Match) -> Option<&mut Match> {
        self.by_start[self.start_index.get(&m.start)?.clone()]
            .iter_mut()
            .find(|m2| m2 == &m)
    }

    pub fn mut_match_end(&mut self, m: &Match) -> Option<&mut Match> {
        self.by_end[self.end_index.get(&m.end)?.clone()]
            .iter_mut()
            .find(|m2| m2 == &m)
    }

    fn max_score_for_match(&self, start: Pos, end: Pos) -> MatchCost {
        let Some(ms) = self.start_index.get(&start) else { return 0; };
        self.by_start[ms.clone()]
            .iter()
            .filter(|m| m.is_active() && m.end == end)
            .map(|m| m.score())
            .max()
            .unwrap_or(0)
    }

    /// Returns true when the match can be pruned without causing consistency problems.
    fn check_consistency(&self, m: &Match) -> bool {
        if !self.check_consistency {
            return true;
        }
        if m.match_cost == 0 {
            return true;
        }

        // Check the neighbouring matches for larger scores
        for (s, e) in [
            (m.start + Pos(0, 1), m.end),
            (m.start - Pos(0, 1), m.end),
            (m.start, m.end + Pos(0, 1)),
            (m.start, m.end - Pos(0, 1)),
        ] {
            if self.max_score_for_match(s, e) > m.score() {
                return false;
            }
        }

        true
    }

    /// Returns false when this match should be skipped (i.e. not pruned).
    fn skip_prune_filter(&mut self) -> bool {
        let cnt = &mut self.skip;
        if let Some(skip) = self.pruning.skip_prune {
            *cnt -= 1;
            if *cnt == 0 {
                *cnt = skip;
                false
            } else {
                true
            }
        } else {
            true
        }
    }
}
