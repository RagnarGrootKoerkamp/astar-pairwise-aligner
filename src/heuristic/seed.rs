use std::cmp::Ordering;

use itertools::Itertools;

use crate::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct SH {
    pub match_config: MatchConfig,
    pub pruning: Pruning,
}

impl Heuristic for SH {
    type Instance<'a> = SHI;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>, alphabet: &Alphabet) -> Self::Instance<'a> {
        SHI::new(a, b, alphabet, *self)
    }

    fn name(&self) -> String {
        "SH".into()
    }

    fn params(&self) -> HeuristicParams {
        // TODO
        HeuristicParams {
            name: self.name(),
            k: self.match_config.length.k().unwrap_or(0),
            max_match_cost: self.match_config.max_match_cost,
            pruning: self.pruning,
            distance_function: "Zero".to_string(),
            ..Default::default()
        }
    }
}

pub struct SHI {
    params: SH,
    _target: Pos,
    matches: SeedMatches,

    // TODO: Do not use vectors inside a hashmap.
    // TODO: Instead, store a Vec<Array>, and attach a slice to each contour point.
    arrows: HashMap<Pos, Vec<Arrow>>,

    /// [l][seed_idx] = number of arrows for the seed with given `seed_idx` length `l`.
    num_arrows_per_length: Vec<Vec<usize>>,

    /// For each score `s`, this is the largest index `i` where total score `s` is still available.
    /// layer_start[0] = n
    /// layer_start[1] = start of first seed with a match
    /// ...
    /// Values in this vector are decreasing, and the layer of position i is the
    /// largest index that has a value at least i.
    layer_starts: SplitVec<I>,

    /// The maximum position explored so far.
    max_explored_pos: Pos,

    // TODO: Put statistics into a separate struct.
    num_pruned: usize,
}

type Hint = Cost;

impl SHI {
    fn new(a: Seq, b: Seq, alph: &Alphabet, params: SH) -> Self {
        // First find all matches.
        let matches = find_matches(a, b, alph, params.match_config, false);

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

        // Transform to Arrows.
        // For arrows with length > 1, also make arrows for length down to 1.
        let match_to_arrow = |m: &Match| Arrow {
            start: m.start,
            end: m.end,
            len: m.seed_potential - m.match_cost,
        };

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
                assert!(0 < a.len && a.len <= params.match_config.max_match_cost + 1);
                num_arrows_per_length[a.len as usize]
                    [matches.seed_at[start.0 as usize].unwrap() as usize] += 1;
            }
        }

        if print() {
            println!("Starts: {layer_starts:?}");
        }

        let h = SHI {
            params,
            _target: Pos::from_lengths(a, b),
            matches,
            arrows,
            num_arrows_per_length,
            layer_starts,
            max_explored_pos: Pos(0, 0),
            num_pruned: 0,
        };
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
        let hint_layer = (self.layer_starts.len() as Cost).saturating_sub(max(layers_before, 1));

        const SEARCH_RANGE: Cost = 8;

        // Do a linear search for some steps, starting at contour v.
        let layer = 'outer: {
            if self.layer_starts[hint_layer as usize] >= pos.0 {
                // Go up.
                for layer in hint_layer + 1
                    ..min(
                        hint_layer + 1 + SEARCH_RANGE,
                        self.layer_starts.len() as Cost,
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
            self.value(pos)
        };
        assert!(pos.0 <= self.layer_starts[layer as usize]);
        if layer as usize + 1 < self.layer_starts.len() {
            assert!(pos.0 > self.layer_starts[layer as usize + 1]);
        }
        let hint = self.layer_starts.len() as Cost - layer;
        (layer, hint)
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
        self.num_arrows_per_length[a.len as usize][seed_idx] -= 1;
        if self.num_arrows_per_length[a.len as usize][seed_idx] != 0 {
            // Remaining matches; nothing to prune.
            return 0;
        }
        // Make sure all larger lengths are also 0.
        for l in a.len as usize + 1..self.num_arrows_per_length.len() {
            if self.num_arrows_per_length[l][seed_idx] > 0 {
                return 0;
            }
        }
        // No seeds of length a.len remain, so we remove the layer.
        let mut removed = 0;
        let mut layer = self.value_with_hint(a.start, hint).0;
        // NOTE: we don't actually have arrows of length 0.
        for l in (1..=a.len).rev() {
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

    fn h_with_parent(&self, pos: Pos) -> (Cost, Pos) {
        (self.h(pos), Pos::default())
    }

    fn h_with_hint(&self, pos: Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        let p = self.matches.potential(pos);
        let (m, h) = self.value_with_hint(pos, hint);
        (p - m, h)
    }

    fn root_state(&self, _root_pos: Pos) -> Self::Hint {
        Default::default()
    }

    fn root_potential(&self) -> Cost {
        self.matches.potential[0]
    }

    fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        self.matches.is_seed_start_or_end(pos)
    }

    /// FIXME: This code is copied from CSH. Should be extracted into a pruning module.
    fn prune(&mut self, pos: Pos, hint: Self::Hint) -> Cost {
        const D: bool = false;
        if !self.params.pruning.enabled {
            return 0;
        }

        // Maximum length arrow at given pos.
        let max_match_cost = self.params.match_config.max_match_cost;

        // Prune any matches ending here.
        let mut change = 0;
        if PRUNE_MATCHES_BY_END {
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
                        self.num_pruned += 1;
                        if matches.is_empty() {
                            self.arrows.remove(&startpos).unwrap();
                        } else {
                            *self.arrows.get_mut(&startpos).unwrap() = matches;
                        }
                    };
                    // First try pruning neighbouring start states, and prune the diagonal start state last.
                    for d in 1..=max_match_cost {
                        if d as Cost <= match_start.1 {
                            try_prune_pos(Pos(match_start.0, match_start.1 - d as I));
                        }
                        try_prune_pos(Pos(match_start.0, match_start.1 + d as I));
                    }
                    try_prune_pos(match_start);
                }
            }
        }
        let a = if let Some(matches) = self.arrows.get(&pos) {
            matches.iter().max_by_key(|a| a.len).unwrap().clone()
        } else {
            return if pos >= self.max_explored_pos {
                change
            } else {
                0
            };
        };

        // Make sure that h remains consistent: never prune positions with larger neighbouring arrows.
        // TODO: Make this smarter and allow pruning long arrows even when pruning short arrows is not possible.
        // The minimum length required for consistency here.
        let mut min_len = 0;
        if CHECK_MATCH_CONSISTENCY {
            for d in 1..=self.params.match_config.max_match_cost {
                let mut check = |pos: Pos| {
                    if let Some(pos_arrows) = self.arrows.get(&pos) {
                        min_len = max(min_len, pos_arrows.iter().map(|a| a.len).max().unwrap() - d);
                    }
                };
                if pos.0 >= d as Cost {
                    check(Pos(pos.0, pos.1 - d as I));
                }
                check(Pos(pos.0, pos.1 + d as I));
            }
        }

        if a.len <= min_len {
            return 0;
        }

        if D || print() {
            println!("PRUNE GAP SEED HEURISTIC {pos} to {min_len}: {a}");
        }

        // If there is an exact match here, also prune neighbouring states for which all arrows end in the same position.
        // TODO: Make this more precise for larger inexact matches.
        if PRUNE_NEIGHBOURING_INEXACT_MATCHES_BY_END
            && a.len == self.params.match_config.max_match_cost + 1
        {
            // See if there are neighbouring points that can now be fully pruned.
            for d in 1..=self.params.match_config.max_match_cost {
                let mut check = |pos: Pos| {
                    if let Some(arrows) = self.arrows.get(&pos) {
                        if arrows.iter().all(|a2| a2.end == a.end) {
                            self.num_pruned += 1;
                            for a in self.arrows.remove(&pos).unwrap() {
                                // TODO: Increment change here?
                                self.update_layers_on_pruning_arrow(a, hint);
                            }
                        }
                    } else {
                        if CHECK_MATCH_CONSISTENCY {
                            println!("Did not find nb arrow at {pos} while pruning {a}");
                            panic!("Arrows are not consistent!");
                        }
                    }
                };
                if pos.1 >= d as Cost {
                    check(Pos(pos.0, pos.1 - d as I));
                }
                check(Pos(pos.0, pos.1 + d as I));
            }
        }

        if PRUNE_MATCHES_BY_START {
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
                for a in arrows.drain_filter(|a| a.len > min_len) {
                    change += self.update_layers_on_pruning_arrow(a, hint);
                }
                *self.arrows.get_mut(&pos).unwrap() = arrows;
            };
        }

        self.num_pruned += 1;
        return if pos >= self.max_explored_pos {
            change
        } else {
            0
        };
    }

    fn explore(&mut self, pos: Pos) {
        self.max_explored_pos.0 = max(self.max_explored_pos.0, pos.0);
        self.max_explored_pos.1 = max(self.max_explored_pos.1, pos.1);
    }

    fn stats(&self) -> HeuristicStats {
        let num_matches = self
            .matches
            .seeds
            .iter()
            .filter(|seed| seed.seed_cost < seed.seed_potential)
            .count();
        HeuristicStats {
            num_seeds: self.matches.seeds.len() as I,
            num_matches,
            num_filtered_matches: num_matches,
            pruning_duration: Default::default(),
            num_prunes: self.num_pruned,
        }
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
