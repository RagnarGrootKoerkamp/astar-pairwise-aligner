use super::{distance::*, *};
use crate::{
    contour::{Arrow, Contours},
    matches::{find_matches, Match, MatchConfig, SeedMatches},
    prelude::*,
};
use itertools::Itertools;
use std::{
    marker::PhantomData,
    time::{self, Duration},
};

pub struct CSH<C: Contours> {
    pub match_config: MatchConfig,
    pub pruning: bool,
    // When false, gaps are free and only the max chain of matches is found.
    pub use_gap_cost: bool,
    pub c: PhantomData<C>,
}

impl<C: Contours> CSH<C> {
    pub fn to_seed_heuristic(&self) -> BruteForceCSH<GapCost> {
        assert!(self.use_gap_cost);
        BruteForceCSH {
            match_config: self.match_config,
            distance_function: GapCost,
            pruning: self.pruning,
        }
    }

    pub fn to_zero_cost_seed_heuristic(&self) -> BruteForceCSH<ZeroCost> {
        assert!(!self.use_gap_cost);
        BruteForceCSH {
            match_config: self.match_config,
            distance_function: ZeroCost,
            pruning: self.pruning,
        }
    }

    pub fn equal_to_seed_heuristic(&self) -> EqualHeuristic<BruteForceCSH<GapCost>, Self> {
        EqualHeuristic {
            h1: self.to_seed_heuristic(),
            h2: *self,
        }
    }

    pub fn equal_to_zero_cost_seed_heuristic(
        &self,
    ) -> EqualHeuristic<BruteForceCSH<ZeroCost>, Self> {
        EqualHeuristic {
            h1: self.to_zero_cost_seed_heuristic(),
            h2: *self,
        }
    }

    pub fn equal_to_bruteforce_contours(&self) -> EqualHeuristic<CSH<BruteForceContours>, Self> {
        EqualHeuristic {
            h1: CSH {
                match_config: self.match_config,
                pruning: self.pruning,
                use_gap_cost: self.use_gap_cost,
                c: Default::default(),
            },
            h2: *self,
        }
    }
}

// Manual implementations because C is not Debug, Clone, or Copy.
impl<C: Contours> std::fmt::Debug for CSH<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainedSeedsHeuristic")
            .field("match_config", &self.match_config)
            .field("pruning", &self.pruning)
            .field("contours", &std::any::type_name::<C>())
            .finish()
    }
}
impl<C: Contours> Clone for CSH<C> {
    fn clone(&self) -> Self {
        Self {
            match_config: self.match_config,
            pruning: self.pruning,
            use_gap_cost: self.use_gap_cost,
            c: self.c,
        }
    }
}
impl<C: Contours> Copy for CSH<C> {}

impl<C: Contours> Heuristic for CSH<C> {
    type Instance<'a> = CSHI<C>;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>, alph: &Alphabet) -> Self::Instance<'a> {
        // TODO: Warning
        assert!(
            self.match_config.max_match_cost
                <= self.match_config.length.k().unwrap_or(I::MAX) as MatchCost / 3
        );
        CSHI::new(a, b, alph, *self)
    }

    fn name(&self) -> String {
        "CSH".into()
    }

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            k: self.match_config.length.k().unwrap_or(0),
            max_match_cost: self.match_config.max_match_cost,
            pruning: self.pruning,
            distance_function: (if self.use_gap_cost { "Gap" } else { "Zero" }).to_string(),
            ..Default::default()
        }
    }
}

pub struct CSHI<C: Contours> {
    params: CSH<C>,
    gap_distance: GapCostI,
    target: Pos,

    seeds: SeedMatches,
    num_matches: usize,
    num_filtered_matches: usize,

    // TODO: Put statistics into a separate struct.
    num_pruned: usize,

    /// The max transformed position.
    max_transformed_pos: Pos,
    transform_target: Pos,
    contours: C,

    // For debugging
    pruning_duration: Duration,
    // TODO: Do not use vectors inside a hashmap.
    // TODO: Instead, store a Vec<Array>, and attach a slice to each contour point.
    arrows: HashMap<Pos, Vec<Arrow>>,
}

/// The seed heuristic implies a distance function as the maximum of the
/// provided distance function and the potential difference between the two
/// positions.  Assumes that the current position is not a match, and no matches
/// are visited in between `from` and `to`.
impl<'a, C: Contours> DistanceInstance<'a> for CSHI<C> {
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        if self.params.use_gap_cost {
            max(
                self.gap_distance.distance(from, to),
                self.seeds.potential_distance(from, to),
            )
        } else {
            self.seeds.potential_distance(from, to)
        }
    }
}

// TODO: Get rid of this.
impl<C: Contours> Drop for CSHI<C> {
    fn drop(&mut self) {
        self.contours.print_stats();
    }
}

impl<C: Contours> CSHI<C> {
    fn new(a: Seq, b: Seq, alph: &Alphabet, params: CSH<C>) -> Self {
        let matches = find_matches(a, b, alph, params.match_config, params.use_gap_cost);
        //println!("\nfind matches.. done: {}", matches.matches.len());
        let mut h = CSHI {
            params,
            gap_distance: Distance::build(&GapCost, a, b, alph),
            target: Pos::from_lengths(a, b),
            seeds: matches,
            num_matches: 0,
            num_filtered_matches: 0,
            num_pruned: 0,

            // For pruning propagation
            max_transformed_pos: Pos(0, 0),

            // Filled below.
            transform_target: Pos(0, 0),
            contours: C::default(),
            pruning_duration: Default::default(),
            arrows: Default::default(),
        };
        h.transform_target = h.transform(h.target);

        // Filter the matches.
        // NOTE: Matches is already sorted by start.
        assert!(h
            .seeds
            .matches
            .is_sorted_by_key(|Match { start, .. }| LexPos(*start)));

        h.num_matches = h.seeds.matches.len();
        {
            // Need to take it out of h.seeds because transform also uses this.
            let mut matches = std::mem::take(&mut h.seeds.matches);
            matches.retain(|Match { end, .. }| h.transform(*end) <= h.transform_target);
            h.seeds.matches = matches;
        }
        h.num_filtered_matches = h.seeds.matches.len();

        // Transform to Arrows.
        // For arrows with length > 1, also make arrows for length down to 1.
        let match_to_arrow = |m: &Match| Arrow {
            start: h.transform(m.start),
            end: h.transform(m.end),
            len: m.seed_potential - m.match_cost,
        };

        let arrows = h
            .seeds
            .matches
            .iter()
            .map(match_to_arrow)
            .group_by(|a| a.start)
            .into_iter()
            .map(|(start, pos_arrows)| (start, pos_arrows.collect_vec()))
            .collect();

        // Sort revered by start (the order needed to construct contours).
        // TODO: Can we get away without sorting? It's probably possible if seeds
        // TODO: Fix the units here -- unclear whether it should be I or cost.
        h.contours = C::new(
            h.seeds.matches.iter().rev().map(match_to_arrow),
            h.params.match_config.max_match_cost as I + 1,
        );
        h.arrows = arrows;
        h.contours.print_stats();
        h
    }

    // TODO: Transform maps from position domain into cost domain.
    // Contours should take a template for the type of point they deal with.
    fn transform(&self, pos @ Pos(i, j): Pos) -> Pos {
        if self.params.use_gap_cost {
            let a = self.target.0;
            let b = self.target.1;
            let pot = |pos| self.seeds.potential(pos);
            Pos(
                // This is a lie. All should be converted to cost, instead of position really.
                i + b - j + pot(Pos(0, 0)) as I - pot(pos) as I,
                j + a - i + pot(Pos(0, 0)) as I - pot(pos) as I,
            )
        } else {
            pos
        }
    }
}

impl<'a, C: Contours> HeuristicInstance<'a> for CSHI<C> {
    fn h(&self, pos: Pos) -> Cost {
        let p = self.seeds.potential(pos);
        let val = self.contours.value(self.transform(pos));
        if val == 0 {
            self.distance(pos, self.target)
        } else {
            p - val
        }
    }

    fn contour_value(&self, pos: Pos) -> Option<Cost> {
        Some(self.contours.value(self.transform(pos)))
    }

    fn h_with_hint(&self, pos: Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        let p = self.seeds.potential(pos);
        let (val, new_hint) = self.contours.value_with_hint(self.transform(pos), hint);
        if val == 0 {
            (self.distance(pos, self.target), new_hint)
        } else {
            (p - val, new_hint)
        }
    }

    type Hint = C::Hint;
    fn root_potential(&self) -> Cost {
        self.seeds.potential(Pos(0, 0))
    }

    fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        self.seeds.is_seed_start_or_end(pos)
    }

    /// `seed_cost` can be used to filter out lookups for states that won't have a match ending there.
    /// TODO: Separate into one step removing as many arrows as needed, and a separate step updating the contours.
    fn prune(&mut self, pos: Pos, hint: Self::Hint) -> Cost {
        const D: bool = false;
        if !self.params.pruning {
            return 0;
        }

        let start = time::Instant::now();

        // Maximum length arrow at given pos.
        let tpos = self.transform(pos);
        let max_match_cost = self.params.match_config.max_match_cost;

        // Prune any matches ending here.
        let mut change = 0;
        if PRUNE_MATCHES_BY_END {
            'prune_by_end: {
                // Check all possible start positions of a match ending here.
                if let Some(s) = self.seeds.seed_ending_at(pos) {
                    assert_eq!(pos.0, s.end);
                    if s.start + pos.1 < pos.0 {
                        break 'prune_by_end;
                    }
                    let match_start = Pos(s.start, s.start + pos.1 - pos.0);
                    let mut try_prune_pos = |startpos: Pos| {
                        let tp = self.transform(startpos);
                        let Some(arrows) = self.arrows.get_mut(&tp) else { return; };
                        // Filter arrows starting in the current position.
                        if arrows.drain_filter(|a| a.end == tpos).count() == 0 {
                            return;
                        }
                        if arrows.is_empty() {
                            self.arrows.remove(&tp).unwrap();
                        }
                        self.num_pruned += 1;
                        // TODO: Propagate this change.
                        self.contours.prune_with_hint(tp, hint, &self.arrows).1;
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
        let a = if let Some(arrows) = self.arrows.get(&tpos) {
            arrows.iter().max_by_key(|a| a.len).unwrap().clone()
        } else {
            self.pruning_duration += start.elapsed();
            return if tpos >= self.max_transformed_pos {
                change
            } else {
                0
            };
        };

        // Make sure that h remains consistent: never prune positions with larger neighbouring arrows.
        // TODO: Make this smarter and allow pruning long arrows even when pruning short arrows is not possible.
        // The minimum length required for consistency here.
        let mut min_len = 0;
        if CHECK_MATCH_CONSISTENCY || self.params.use_gap_cost {
            for d in 1..=self.params.match_config.max_match_cost {
                let mut check = |pos: Pos| {
                    if let Some(pos_arrows) = self.arrows.get(&self.transform(pos)) {
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
                    let tp = self.transform(pos);
                    if let Some(arrows) = self.arrows.get(&tp) {
                        if arrows.iter().all(|a2| a2.end == a.end) {
                            self.num_pruned += 1;
                            self.arrows.remove(&tp);
                            self.contours.prune_with_hint(tp, hint, &self.arrows);
                        }
                    } else {
                        if CHECK_MATCH_CONSISTENCY {
                            println!("Did not find nb arrow at {tp} while pruning {a} at {pos}");
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
            change += if min_len == 0 {
                self.arrows.remove(&tpos).unwrap();
                self.contours.prune_with_hint(tpos, hint, &self.arrows).1
            } else {
                // If we only remove a subset of arrows, do no actual pruning.
                // TODO: Also update contours on partial pruning.
                let arrows = self.arrows.get_mut(&tpos).unwrap();
                if D {
                    println!("Remove arrows of length > {min_len} at pos {pos}.");
                }
                arrows.drain_filter(|a| a.len > min_len).count();
                assert!(arrows.len() > 0);
                self.contours.prune_with_hint(tpos, hint, &self.arrows).1
            };
        }

        self.pruning_duration += start.elapsed();

        self.num_pruned += 1;
        return if tpos >= self.max_transformed_pos {
            change
        } else {
            0
        };
    }

    /// Update the max_explored_pos, so we know when the priority queue can be shifted after a prune.
    fn explore(&mut self, pos: Pos) {
        let tpos = self.transform(pos);
        self.max_transformed_pos.0 = max(self.max_transformed_pos.0, tpos.0);
        self.max_transformed_pos.1 = max(self.max_transformed_pos.1, tpos.1);
    }

    fn stats(&self) -> HeuristicStats {
        HeuristicStats {
            num_seeds: self.seeds.seeds.len() as I,
            num_matches: self.num_matches,
            num_filtered_matches: self.num_filtered_matches,
            matches: if DEBUG {
                self.seeds.matches.clone()
            } else {
                Default::default()
            },
            pruning_duration: self.pruning_duration.as_secs_f32(),
            num_prunes: self.num_pruned,
        }
    }

    fn matches(&self) -> Option<Vec<Match>> {
        Some(
            self.seeds
                .matches
                .iter()
                .map(|m| {
                    let mut m = m.clone();
                    m.pruned = if self.arrows.contains_key(&m.start) {
                        MatchStatus::Active
                    } else {
                        MatchStatus::Pruned
                    };
                    m
                })
                .collect(),
        )
    }

    fn seeds(&self) -> Option<&Vec<Seed>> {
        Some(&self.seeds.seeds)
    }

    fn params_string(&self) -> String {
        format!("{:?}", self.params)
    }
}
