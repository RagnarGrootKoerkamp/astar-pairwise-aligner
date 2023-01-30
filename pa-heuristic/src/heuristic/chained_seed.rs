use super::*;
use crate::{contour::*, wrappers::EqualHeuristic};
use itertools::Itertools;
use std::marker::PhantomData;

pub struct CSH<C: Contours> {
    pub match_config: MatchConfig,
    pub pruning: Pruning,
    // When false, gaps are free and only the max chain of matches is found.
    pub use_gap_cost: bool,
    pub c: PhantomData<C>,
}

impl CSH<HintContours<BruteForceContour>> {
    pub fn new(match_config: MatchConfig, pruning: Pruning, use_gap_cost: bool) -> Self {
        Self {
            match_config,
            pruning,
            use_gap_cost,
            c: PhantomData,
        }
    }
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

    pub fn to_zero_cost_seed_heuristic(&self) -> BruteForceCSH<NoCost> {
        assert!(!self.use_gap_cost);
        BruteForceCSH {
            match_config: self.match_config,
            distance_function: NoCost,
            pruning: self.pruning,
        }
    }

    pub fn equal_to_seed_heuristic(&self) -> EqualHeuristic<BruteForceCSH<GapCost>, Self> {
        EqualHeuristic {
            h1: self.to_seed_heuristic(),
            h2: *self,
        }
    }

    pub fn equal_to_zero_cost_seed_heuristic(&self) -> EqualHeuristic<BruteForceCSH<NoCost>, Self> {
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
            .field("use_gap_cost", &self.use_gap_cost)
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

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        // TODO: Warning
        assert!(
            self.match_config.max_match_cost
                <= self.match_config.length.k().unwrap_or(I::MAX) as MatchCost / 3
        );
        CSHI::new(a, b, *self)
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

    stats: HeuristicStats,
    seeds: SeedMatches,

    /// The max transformed position.
    max_transformed_pos: Pos,
    transform_target: Pos,
    contours: C,

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
    /// `filter` is currently only used for pre-pruning when an optimal path is guessed and all matches on it are pruned directly.
    /// This is not in the paper yet.
    fn new(a: Seq, b: Seq, params: CSH<C>) -> Self {
        let matches = find_matches(a, b, params.match_config, params.use_gap_cost);
        //println!("\nfind matches.. done: {}", matches.matches.len());
        let mut h = CSHI {
            params,
            gap_distance: Distance::build(&GapCost, a, b),
            target: Pos::target(a, b),
            seeds: matches,
            stats: HeuristicStats::default(),

            // For pruning propagation
            max_transformed_pos: Pos(0, 0),

            // Filled below.
            transform_target: Pos(0, 0),
            contours: C::default(),
            arrows: Default::default(),
        };
        h.transform_target = h.transform(h.target);

        // Filter the matches.
        // NOTE: Matches is already sorted by start.
        assert!(h
            .seeds
            .matches
            .is_sorted_by_key(|Match { start, .. }| LexPos(*start)));

        h.stats.num_matches = h.seeds.matches.len();
        if params.use_gap_cost {
            // Need to take it out of h.seeds because transform also uses this.
            let mut matches = std::mem::take(&mut h.seeds.matches);
            matches.retain(|Match { end, .. }| h.transform(*end) <= h.transform_target);
            h.seeds.matches = matches;
        }
        h.stats.num_filtered_matches = h.seeds.matches.len();

        // Transform to Arrows.
        // For arrows with length > 1, also make arrows for length down to 1.
        let match_to_arrow = |m: &Match| Arrow {
            start: h.transform(m.start),
            end: h.transform(m.end),
            score: m.seed_potential - m.match_cost,
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

        // Sort reversed by start (the order needed to construct contours).
        // TODO: Can we get away without sorting? It's probably possible if seeds
        // TODO: Fix the units here -- unclear whether it should be I or cost.
        // FIXME: Re-add the removed filter here?
        h.contours = C::new(
            h.seeds.matches.iter().rev().map(match_to_arrow),
            h.params.match_config.max_match_cost as I + 1,
        );
        h.arrows = arrows;
        h.contours.print_stats();
        h.stats.h0 = h.h(Pos(0, 0));
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
                // Units here are a lie. All should be converted to cost, instead of position really.
                i + b - j + pot(Pos(0, 0)) as I - pot(pos) as I,
                j + a - i + pot(Pos(0, 0)) as I - pot(pos) as I,
            )
        } else {
            pos
        }
    }

    /// True when the next position should be pruned.
    /// Returns false once every `params.pruning.skip_prune` steps, if set.
    fn add_prune(&mut self) -> bool {
        self.stats.num_pruned += 1;
        if let Some(skip) = self.params.pruning.skip_prune {
            self.stats.num_pruned % skip != 0
        } else {
            true
        }
    }
}

impl<'a, C: Contours> HeuristicInstance<'a> for CSHI<C> {
    fn h(&self, pos: Pos) -> Cost {
        let p = self.seeds.potential(pos);
        let val = self.contours.score(self.transform(pos));
        // FIXME: Why not max(self.distance, p-val)?
        if val == 0 {
            self.distance(pos, self.target)
        } else {
            p - val
        }
    }

    fn layer(&self, pos: Pos) -> Option<Cost> {
        Some(self.contours.score(self.transform(pos)))
    }

    fn layer_with_hint(&self, pos: Pos, hint: Self::Hint) -> Option<(Cost, Self::Hint)> {
        Some(self.contours.score_with_hint(self.transform(pos), hint))
    }

    fn h_with_hint(&self, pos: Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        let p = self.seeds.potential(pos);
        let (val, new_hint) = self.contours.score_with_hint(self.transform(pos), hint);
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

    fn seed_matches(&self) -> Option<&SeedMatches> {
        Some(&self.seeds)
    }

    /// `seed_cost` can be used to filter out lookups for states that won't have a match ending there.
    /// TODO: Separate into one step removing as many arrows as needed, and a separate step updating the contours.
    type Order = Pos;
    fn prune(&mut self, pos: Pos, hint: Self::Hint) -> (Cost, Pos) {
        const D: bool = false;
        if !self.params.pruning.enabled {
            return (0, Pos::default());
        }
        self.stats.prune_count += 1;

        // Time the duration of retrying once in this many iterations.
        const TIME_EACH: usize = 64;
        let start = if self.stats.prune_count % TIME_EACH == 0 {
            Some(instant::Instant::now())
        } else {
            None
        };

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
                        if !self.add_prune() {
                            return;
                        }
                        let tp = self.transform(startpos);
                        let Some(arrows) = self.arrows.get_mut(&tp) else { return; };
                        // Filter arrows starting in the current position.
                        if arrows.drain_filter(|a| a.end == tpos).count() == 0 {
                            return;
                        }
                        if arrows.is_empty() {
                            self.arrows.remove(&tp).unwrap();
                        }
                        self.stats.num_pruned += 1;
                        // FIXME: Propagate this change.
                        self.contours.prune_with_hint(tp, hint, &self.arrows).1;
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
        let a = if let Some(arrows) = self.arrows.get(&tpos) {
            arrows.iter().max_by_key(|a| a.score).unwrap().clone()
        } else {
            if let Some(start) = start {
                self.stats.pruning_duration += TIME_EACH as f32 * start.elapsed().as_secs_f32();
            }
            // FIXME: Fix queue shifting with gapcost.
            return (change, pos);
        };

        // Make sure that h remains consistent: never prune positions with larger neighbouring arrows.
        // TODO: Make this smarter and allow pruning long arrows even when pruning short arrows is not possible.
        // The minimum length required for consistency here.
        let mut min_len = 0;
        if CHECK_MATCH_CONSISTENCY || self.params.use_gap_cost {
            for d in 1..=self.params.match_config.max_match_cost {
                let mut check = |pos: Pos| {
                    if let Some(pos_arrows) = self.arrows.get(&self.transform(pos)) {
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
            return (0, Pos::default());
        }

        if D {
            println!("PRUNE GAP SEED HEURISTIC {pos} to {min_len}: {a}");
        }

        if PRUNE_MATCHES_BY_START {
            change += if min_len == 0 {
                if self.add_prune() {
                    self.arrows.remove(&tpos).unwrap();
                    self.contours.prune_with_hint(tpos, hint, &self.arrows).1
                } else {
                    0
                }
            } else {
                if self.add_prune() {
                    // If we only remove a subset of arrows, do no actual pruning.
                    // TODO: Also update contours on partial pruning.
                    let arrows = self.arrows.get_mut(&tpos).unwrap();
                    if D {
                        println!("Remove arrows of length > {min_len} at pos {pos}.");
                    }
                    arrows.drain_filter(|a| a.score > min_len).count();
                    assert!(arrows.len() > 0);
                    self.contours.prune_with_hint(tpos, hint, &self.arrows).1
                } else {
                    0
                }
            };
        }

        if let Some(start) = start {
            self.stats.pruning_duration += TIME_EACH as f32 * start.elapsed().as_secs_f32();
        }

        if self.params.use_gap_cost {
            // FIXME: Return `change`, and `tpos` instead of `pos`.
            change = 0;
        }
        (change, pos)
    }

    /// Update the max_explored_pos, so we know when the priority queue can be shifted after a prune.
    fn explore(&mut self, pos: Pos) {
        let tpos = self.transform(pos);
        self.max_transformed_pos.0 = max(self.max_transformed_pos.0, tpos.0);
        self.max_transformed_pos.1 = max(self.max_transformed_pos.1, tpos.1);
    }

    fn stats(&mut self) -> HeuristicStats {
        self.stats.h0_end = self.h(Pos(0, 0));
        self.stats
    }

    fn matches(&self) -> Option<Vec<Match>> {
        Some(
            self.seeds
                .matches
                .iter()
                .map(|m| {
                    let mut m = m.clone();
                    m.pruned = if self.arrows.contains_key(&self.transform(m.start)) {
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
