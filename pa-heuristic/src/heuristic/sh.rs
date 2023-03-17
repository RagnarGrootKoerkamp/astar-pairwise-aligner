use itertools::Itertools;
use std::cmp::Ordering;

use super::*;
use crate::{contour::Layer, split_vec::SplitVec};

#[derive(Debug, Copy, Clone)]
pub struct SH {
    pub match_config: MatchConfig,
    pub pruning: Pruning,
}

impl SH {
    pub fn new(match_config: MatchConfig, pruning: Pruning) -> Self {
        Self {
            match_config,
            pruning,
        }
    }
}

impl Heuristic for SH {
    type Instance<'a> = SHI;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        SHI::new(a, b, *self)
    }

    fn name(&self) -> String {
        "SH".into()
    }
}

pub struct SHI {
    params: SH,
    _target: Pos,
    matches: SeedMatches,

    // TODO: Do not use vectors inside a hashmap.
    // TODO: Instead, store a Vec<Array>, and attach a slice to each contour point.
    arrows: HashMap<Pos, Vec<Arrow>>,

    /// index `[l][seed_idx]`: number of arrows for the seed with given `seed_idx` length `l`.
    num_arrows_per_length: Vec<Vec<usize>>,

    /// For each score `s`, this is the largest index `i` where total score `s` is still available.
    /// ```text
    /// layer_start[0] = n
    /// layer_start[1] = start of first seed with a match
    /// ...
    /// ```
    /// Values in this vector are decreasing, and the layer of position `i` is the
    /// largest index that has a value at least `i`.
    layer_starts: SplitVec<I>,

    /// The maximum position explored so far.
    max_explored_pos: Pos,

    stats: HeuristicStats,
}

type Hint = Layer;

impl SHI {
    fn new(a: Seq, b: Seq, params: SH) -> Self {
        // First find all matches.
        let matches = find_matches(a, b, params.match_config, false);

        // Initialize layers.
        let mut layer_starts = SplitVec::default();
        // Layer 0 starts at the end of A.
        layer_starts.push(a.len() as I);
        for seed in matches.seeds.iter().rev() {
            let weight = seed.seed_potential - seed.seed_cost;
            for _ in 0..weight {
                layer_starts.push(seed.start);
            }
        }

        let arrows: HashMap<Pos, Vec<Arrow>> = matches
            .matches
            .iter()
            .map(match_to_arrow)
            .group_by(|a| a.start)
            .into_iter()
            .map(|(start, pos_arrows)| (start, pos_arrows.collect_vec()))
            .collect();

        // Count number of matches per seed of each length.
        // The max length is r=1+max_match_cost.
        let mut num_arrows_per_length =
            vec![vec![0; matches.seeds.len()]; params.match_config.max_match_cost as usize + 2];

        for (start, arrows) in &arrows {
            for a in arrows {
                assert!(0 < a.score && a.score <= params.match_config.max_match_cost + 1);
                num_arrows_per_length[a.score as usize]
                    [matches.seed_at[start.0 as usize].unwrap() as usize] += 1;
            }
        }

        let mut h = SHI {
            params,
            _target: Pos::target(a, b),
            arrows,
            num_arrows_per_length,
            layer_starts,
            max_explored_pos: Pos(0, 0),
            stats: HeuristicStats {
                num_seeds: matches.seeds.len() as I,
                num_matches: matches.matches.len(),
                num_filtered_matches: matches.matches.len(),
                pruning_duration: 0.0,
                num_pruned: 0,
                h0: 0,
                h0_end: 0,
                prune_count: 0,
            },
            matches,
        };
        h.stats.h0 = h.h(Pos(0, 0));
        h
    }

    /// The layer of position i is the largest index that has a value at least i.
    fn value(&self, Pos(i, _): Pos) -> Cost {
        // FIXME: Make sure this is still up-to-date!
        self.layer_starts
            .binary_search_by(|start| {
                if *start >= i {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .unwrap_err() as Cost
            - 1
    }

    /// Hint is the total weight _before_ the position, since this will change
    /// less than the weight _after_ the position.
    fn value_with_hint(&self, pos: Pos, layers_before: Hint) -> (Cost, Hint) {
        let hint_layer = (self.layer_starts.len() as Layer).saturating_sub(max(layers_before, 1));

        const SEARCH_RANGE: Layer = 8;

        // Do a linear search for some steps, starting at contour v.
        let layer = 'outer: {
            if self.layer_starts[hint_layer as usize] >= pos.0 {
                // Go up.
                for layer in hint_layer + 1
                    ..min(
                        hint_layer + 1 + SEARCH_RANGE,
                        self.layer_starts.len() as Layer,
                    )
                {
                    if self.layer_starts[layer as usize] < pos.0 {
                        break 'outer layer - 1;
                    }
                }
            } else {
                // Go down.
                for layer in (hint_layer.saturating_sub(SEARCH_RANGE)..hint_layer).rev() {
                    if self.layer_starts[layer as usize] >= pos.0 {
                        break 'outer layer;
                    }
                }
            }

            // Fall back to binary search if not found close to the hint.
            self.value(pos) as Layer
        };
        assert!(pos.0 <= self.layer_starts[layer as usize]);
        if layer as usize + 1 < self.layer_starts.len() {
            assert!(pos.0 > self.layer_starts[layer as usize + 1]);
        }
        let hint = self.layer_starts.len() as Layer - layer;
        (layer as Cost, hint)
    }

    /// When pruning a match/arrow:
    /// 1. Lower the corresponding count.
    /// 2. If the count for the current length and all larger lengths are 0:
    /// 3. Remove as many layers as possible.
    ///
    /// This takes the arrow by value, to ensure it is removed from the hashmap before passing it here.
    ///
    /// Returns the number of layers removed.
    fn update_layers_on_pruning_arrow(&mut self, a: Arrow, hint: Hint) -> Cost {
        let seed_idx = self.matches.seed_at[a.start.0 as usize].unwrap() as usize;
        if SH_MARK_MATCH_AS_PRUNED {
            for m in &mut self.matches.matches {
                if match_to_arrow(m) == a {
                    m.pruned = MatchStatus::Pruned;
                }
            }
        }
        self.num_arrows_per_length[a.score as usize][seed_idx] -= 1;
        if self.num_arrows_per_length[a.score as usize][seed_idx] != 0 {
            // Remaining matches; nothing to prune.
            return 0;
        }
        // Make sure all larger lengths are also 0.
        for l in a.score as usize + 1..self.num_arrows_per_length.len() {
            if self.num_arrows_per_length[l][seed_idx] > 0 {
                return 0;
            }
        }
        // No seeds of length a.len remain, so we remove the layer.
        let mut removed = 0;
        let mut layer = self.value_with_hint(a.start, hint).0;
        // NOTE: we don't actually have arrows of length 0.
        for l in (1..=a.score).rev() {
            if self.num_arrows_per_length[l as usize][seed_idx] > 0 {
                break;
            }
            assert_eq!(self.layer_starts[layer as usize], a.start.0);
            self.layer_starts.remove(layer as usize);
            removed += 1;
            layer -= 1;
        }

        removed
    }
}

fn match_to_arrow(m: &Match) -> Arrow {
    Arrow {
        start: m.start,
        end: m.end,
        score: m.seed_potential - m.match_cost,
    }
}

impl<'a> HeuristicInstance<'a> for SHI {
    /// The index of the next match, from the end of the splitvec.
    type Hint = Hint;

    fn h(&self, pos: Pos) -> Cost {
        let p = self.matches.potential(pos);
        let m = self.value(pos);
        p - m
    }

    fn layer(&self, pos: Pos) -> Option<Cost> {
        Some(self.value(pos))
    }

    fn layer_with_hint(&self, pos: Pos, hint: Self::Hint) -> Option<(Cost, Self::Hint)> {
        Some(self.value_with_hint(pos, hint))
    }

    fn h_with_hint(&self, pos: Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        let p = self.matches.potential(pos);
        let (m, h) = self.value_with_hint(pos, hint);
        (p - m, h)
    }

    fn root_potential(&self) -> Cost {
        self.matches.potential[0]
    }

    fn seed_matches(&self) -> Option<&SeedMatches> {
        Some(&self.matches)
    }

    type Order = I;

    /// FIXME: This code is copied from CSH. Should be extracted into a pruning module.
    fn prune(&mut self, pos: Pos, hint: Self::Hint) -> (Cost, I) {
        const D: bool = false;
        if !self.params.pruning.is_enabled() {
            return (0, 0);
        }

        // Maximum length arrow at given pos.
        let max_match_cost = self.params.match_config.max_match_cost;

        // Prune any matches ending here.
        // TODO: Shifting for prune by end.
        if self.params.pruning.prune_end() {
            'prune_by_end: {
                // Check all possible start positions of a match ending here.
                if let Some(s) = self.matches.seed_ending_at(pos) {
                    assert_eq!(pos.0, s.end);
                    if s.start + pos.1 < pos.0 {
                        break 'prune_by_end;
                    }
                    let match_start = Pos(s.start, s.start + pos.1 - pos.0);
                    let mut try_prune_pos = |startpos: Pos| {
                        let Some(mut matches) = self.arrows.get_mut(&startpos).map(std::mem::take) else { return; };
                        // Filter arrows starting in the current position.
                        let mut delta = false;
                        for a in matches.drain_filter(|a| a.end == pos) {
                            self.update_layers_on_pruning_arrow(a, hint);
                            delta = true;
                        }
                        if !delta {
                            *self.arrows.get_mut(&startpos).unwrap() = matches;
                            return;
                        }
                        self.stats.num_pruned += 1;
                        if matches.is_empty() {
                            self.arrows.remove(&startpos).unwrap();
                        } else {
                            *self.arrows.get_mut(&startpos).unwrap() = matches;
                        }
                    };
                    // First try pruning neighbouring start states, and prune the diagonal start state last.
                    for d in 1..=max_match_cost {
                        if (d as Cost) <= match_start.1 {
                            try_prune_pos(Pos(match_start.0, match_start.1 - d as I));
                        }
                        try_prune_pos(Pos(match_start.0, match_start.1 + d as I));
                    }
                    try_prune_pos(match_start);
                }
            }
        }
        let a = if let Some(matches) = self.arrows.get(&pos) {
            matches.iter().max_by_key(|a| a.score).unwrap().clone()
        } else {
            return (0, 0);
        };

        let mut change = 0;

        // Make sure that h remains consistent: never prune positions with larger neighbouring arrows.
        // TODO: Make this smarter and allow pruning long arrows even when pruning short arrows is not possible.
        // The minimum length required for consistency here.
        let mut min_len = 0;
        if CHECK_MATCH_CONSISTENCY {
            for d in 1..=self.params.match_config.max_match_cost {
                let mut check = |pos: Pos| {
                    if let Some(pos_arrows) = self.arrows.get(&pos) {
                        min_len = max(
                            min_len,
                            pos_arrows.iter().map(|a| a.score).max().unwrap() - d,
                        );
                    }
                };
                if pos.0 >= d as Cost {
                    check(Pos(pos.0, pos.1 - d as I));
                }
                check(Pos(pos.0, pos.1 + d as I));
            }
        }

        if a.score <= min_len {
            return (0, 0);
        }

        if D {
            println!("PRUNE GAP SEED HEURISTIC {pos} to {min_len}: {a}");
        }

        if self.params.pruning.prune_start() {
            if min_len == 0 {
                for a in self.arrows.remove(&pos).unwrap() {
                    change += self.update_layers_on_pruning_arrow(a, hint);
                }
            } else {
                // If we only remove a subset of arrows, do no actual pruning.
                // TODO: Also update contours on partial pruning.
                let mut arrows = std::mem::take(self.arrows.get_mut(&pos).unwrap());
                assert!(arrows.len() > 0);
                if D {
                    println!("Remove arrows of length > {min_len} at pos {pos}.");
                }
                for a in arrows.drain_filter(|a| a.score > min_len) {
                    change += self.update_layers_on_pruning_arrow(a, hint);
                }
                *self.arrows.get_mut(&pos).unwrap() = arrows;
            };
        }

        self.stats.num_pruned += 1;
        return if pos >= self.max_explored_pos {
            (change, pos.0)
        } else {
            (0, 0)
        };
    }

    fn explore(&mut self, pos: Pos) {
        self.max_explored_pos.0 = max(self.max_explored_pos.0, pos.0);
        self.max_explored_pos.1 = max(self.max_explored_pos.1, pos.1);
    }

    fn stats(&mut self) -> HeuristicStats {
        self.stats.h0_end = self.h(Pos(0, 0));
        self.stats
    }

    fn matches(&self) -> Option<Vec<Match>> {
        Some(self.matches.matches.clone())
    }

    fn seeds(&self) -> Option<&Vec<Seed>> {
        Some(&self.matches.seeds)
    }

    fn params_string(&self) -> String {
        format!("{:?}", self.params)
    }
}
