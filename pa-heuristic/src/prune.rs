use crate::{
    matches::{Match, MatchStatus},
    prelude::*,
    seeds::MatchCost,
};
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

pub struct MatchPruner {
    pruning: Pruning,
    check_consistency: bool,
    /// Skip a prune when this reaches 0 and `skip_prune` is set.
    skip: usize,
    // TODO: Do not use vectors inside a hashmap.
    // Instead, store a Vec<Array>, and attach a slice to each contour point.
    pub by_start: HashMap<Pos, Vec<Match>>,
    pub by_end: HashMap<Pos, Vec<Match>>,
}

impl MatchPruner {
    pub fn new(pruning: Pruning, check_consistency: bool, mut matches: Vec<Match>) -> MatchPruner {
        // Sort by start, then by  match cost.
        // This ensures that matches are pruned from low cost to high cost.
        matches.sort_unstable_by_key(|m| (LexPos(m.start), m.match_cost));
        let by_start = matches
            .iter()
            .cloned()
            .group_by(|m| m.start)
            .into_iter()
            .map(|(start, pos_arrows)| (start, pos_arrows.collect_vec()))
            .collect();

        // Sort by end, then by *decreasing* match cost.
        matches.sort_unstable_by_key(|m| (LexPos(m.end), m.match_cost));
        let by_end = matches
            .into_iter()
            .group_by(|m| m.start)
            .into_iter()
            .map(|(start, pos_arrows)| (start, pos_arrows.collect_vec()))
            .collect();

        MatchPruner {
            pruning,
            check_consistency,
            skip: 1,
            by_start,
            by_end,
        }
    }

    /// Returns number of matches pruned by start (succeeding this pos) and by end (preceding this pos).
    pub fn prune(&mut self, seeds: &Seeds, pos: Pos, mut f: impl FnMut(&Match)) -> (usize, usize) {
        let mut cnt = (0, 0);
        if self.pruning.prune_start() && seeds.is_seed_start(pos) {
            if let Some(ms) = self.by_start.get_mut(&pos) {
                let mut ms = std::mem::take(ms);
                cnt.0 = self.prune_vec(&mut ms, &mut f);
                *self.by_start.get_mut(&pos).unwrap() = ms;
            }
        };
        if self.pruning.prune_end() && seeds.is_seed_end(pos) {
            if let Some(ms) = self.by_end.get_mut(&pos) {
                let mut ms = std::mem::take(ms);
                cnt.0 = self.prune_vec(&mut ms, &mut f);
                *self.by_end.get_mut(&pos).unwrap() = ms;
            }
        };
        cnt
    }

    /// Returns the number of pruned matches.
    fn prune_vec(&mut self, ms: &mut Vec<Match>, f: &mut impl FnMut(&Match)) -> usize {
        ms.iter_mut()
            .filter(|m| m.is_active() && self.check_consistency(m) && self.skip_prune_filter())
            .map(|m| {
                m.pruned = MatchStatus::Pruned;
                f(m);
            })
            .count()
    }

    /// Returns false when this match should be skipped (i.e. not pruned).
    fn skip_prune_filter(&mut self) -> bool {
        if let Some(skip) = self.pruning.skip_prune {
            self.skip -= 1;
            if self.skip == 0 {
                self.skip = skip;
                false
            } else {
                true
            }
        } else {
            true
        }
    }

    fn max_score_for_match(&self, start: Pos, end: Pos) -> MatchCost {
        let Some(ms) = self.by_start.get(&start) else { return 0; };
        ms.iter()
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

    pub fn collect_vec(&self) -> Vec<Match> {
        self.by_start
            .iter()
            .flat_map(|(_start, ms)| ms)
            .cloned()
            .collect_vec()
    }
}
