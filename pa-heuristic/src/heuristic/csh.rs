use itertools::Itertools;
use smallvec::SmallVec;

use super::*;
use crate::prune::MatchPruner;
use crate::seeds::Seeds;
use crate::*;
use crate::{contour::*, wrappers::EqualHeuristic};
use std::marker::PhantomData;

pub struct CSH<C: Contours> {
    pub match_config: MatchConfig,
    pub pruning: Pruning,
    // When false, gaps are free and only the max chain of matches is found.
    pub use_gap_cost: bool,
    c: PhantomData<C>,
}

impl CSH<HintContours<BruteForceContour>> {
    pub fn new(match_config: MatchConfig, pruning: Pruning) -> Self {
        Self {
            match_config,
            pruning,
            use_gap_cost: false,
            c: PhantomData,
        }
    }
}

impl CSH<BruteForceContours> {
    pub fn new_bruteforce(match_config: MatchConfig, pruning: Pruning) -> Self {
        Self {
            match_config,
            pruning,
            use_gap_cost: false,
            c: PhantomData,
        }
    }
}

/// TODO: Make a version of GCSH that stores arrows in the original <i,j>
/// domain, and only applies the transformation at the time when states are
/// compared via $\preceq_T$.
pub struct GCSH;
impl GCSH {
    pub fn new(
        match_config: MatchConfig,
        pruning: Pruning,
    ) -> CSH<HintContours<BruteForceContour>> {
        CSH {
            match_config,
            pruning,
            use_gap_cost: true,
            c: PhantomData,
        }
    }
}

impl<C: Contours> CSH<C> {
    pub fn to_bruteforce_gcsh(&self) -> BruteForceGCSH<GapCost> {
        assert!(self.use_gap_cost);
        BruteForceGCSH {
            match_config: self.match_config,
            distance_function: GapCost,
            pruning: self.pruning,
        }
    }

    pub fn to_bruteforce_csh(&self) -> BruteForceGCSH<NoCost> {
        assert!(!self.use_gap_cost);
        BruteForceGCSH {
            match_config: self.match_config,
            distance_function: NoCost,
            pruning: self.pruning,
        }
    }

    pub fn to_bruteforce_contours(&self) -> CSH<BruteForceContours> {
        CSH {
            match_config: self.match_config,
            pruning: self.pruning,
            use_gap_cost: self.use_gap_cost,
            c: Default::default(),
        }
    }

    pub fn equal_to_bruteforce_gcsh(&self) -> EqualHeuristic<BruteForceGCSH<GapCost>, Self> {
        EqualHeuristic {
            h1: self.to_bruteforce_gcsh(),
            h2: *self,
        }
    }

    pub fn equal_to_bruteforce_csh(&self) -> EqualHeuristic<BruteForceGCSH<NoCost>, Self> {
        EqualHeuristic {
            h1: self.to_bruteforce_csh(),
            h2: *self,
        }
    }

    pub fn equal_to_bruteforce_contours(&self) -> EqualHeuristic<CSH<BruteForceContours>, Self> {
        EqualHeuristic {
            h1: self.to_bruteforce_contours(),
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
}

pub struct CSHI<C: Contours> {
    params: CSH<C>,
    gap_distance: GapCostI,
    target: Pos,
    t_target: Pos,

    seeds: Seeds,
    matches: MatchPruner,

    /// The max transformed position.
    max_transformed_pos: Pos,
    contours: C,

    stats: HeuristicStats,
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
        let Matches { seeds, mut matches } =
            find_matches(a, b, params.match_config, params.use_gap_cost);
        let target = Pos::target(a, b);
        let t_target = if params.use_gap_cost {
            seeds.transform(target)
        } else {
            target
        };

        // Filter matches: only keep matches with m.start <= target.
        // NOTE: Only matches m.end <= target can be used in chains and
        // forwarded to the Contours, but the ones with m.start <= target are
        // still needed for consistency.
        let num_matches = matches.len();
        if params.use_gap_cost {
            matches.retain(|m| seeds.transform(m.start) <= t_target);
        }
        let num_filtered_matches = matches.len();

        // Transform to Arrows.
        // For arrows with length > 1, also make arrows for length down to 1.
        let match_to_arrow = |m: &Match| Arrow {
            start: if params.use_gap_cost {
                seeds.transform(m.start)
            } else {
                m.start
            },
            end: if params.use_gap_cost {
                seeds.transform(m.end)
            } else {
                m.end
            },
            score: m.score(),
        };

        // Sort reversed by start (the order needed to construct contours).
        matches.sort_by_key(|m| LexPos(m.start));

        // TODO: Fix the units here -- unclear whether it should be I or cost.
        let contours = C::new(
            matches
                .iter()
                .rev()
                .map(match_to_arrow)
                .filter(|a| a.end <= t_target),
            params.match_config.max_match_cost as I + 1,
        );

        let mut h = CSHI {
            params,
            gap_distance: Distance::build(&GapCost, a, b),
            target,
            t_target,
            seeds,
            matches: MatchPruner::new(params.pruning, params.use_gap_cost, matches),
            stats: HeuristicStats::default(),

            // For pruning propagation
            max_transformed_pos: Pos(0, 0),

            contours,
        };
        h.stats.h0 = h.h(Pos(0, 0));
        h.stats.num_seeds = h.seeds.seeds.len() as _;
        h.stats.num_matches = num_matches;
        h.stats.num_filtered_matches = num_filtered_matches;
        h.contours.print_stats();
        h
    }

    // TODO: Transform maps from position domain into cost domain.
    // Contours should take a template for the type of point they deal with.
    fn transform(&self, pos: Pos) -> Pos {
        if self.params.use_gap_cost {
            self.seeds.transform(pos)
        } else {
            pos
        }
    }

    // TODO: Transform maps from position domain into cost domain.
    // Contours should take a template for the type of point they deal with.
    fn transform_back(&self, pos: Pos) -> Pos {
        if self.params.use_gap_cost {
            self.seeds.transform_back(pos)
        } else {
            pos
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

    fn h_with_parent(&self, pos: Pos) -> (Cost, Pos) {
        (
            self.h(pos),
            self.transform_back(self.contours.parent(self.transform(pos)).1),
        )
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

    /// `seed_cost` can be used to filter out lookups for states that won't have a match ending there.
    /// TODO: Separate into one step removing as many arrows as needed, and a separate step updating the contours.
    type Order = Pos;
    fn prune(&mut self, pos: Pos, hint: Self::Hint) -> (Cost, Pos) {
        if !self.params.pruning.is_enabled() {
            return (0, Pos::default());
        }
        self.stats.prune_count += 1;

        // Time the duration of retrying once in this many iterations.
        const TIME_EACH: usize = 64;
        let start_time = if self.stats.prune_count % TIME_EACH == 0 {
            Some(instant::Instant::now())
        } else {
            None
        };

        let tpos = self.transform(pos);
        let (start_layer, hint) = self.contours.score_with_hint(tpos, hint);

        let mut pruned_start_positions: SmallVec<[Pos; 5]> = Default::default();
        let (p_start, p_end) = self.matches.prune(&self.seeds, pos, |m| {
            if !pruned_start_positions.contains(&m.start) {
                pruned_start_positions.push(m.start)
            }
        });

        self.stats.num_pruned += p_start + p_end;

        let match_to_arrow = |m: &Match| Arrow {
            start: if self.params.use_gap_cost {
                self.seeds.transform(m.start)
            } else {
                m.start
            },
            end: if self.params.use_gap_cost {
                self.seeds.transform(m.end)
            } else {
                m.end
            },
            score: m.score(),
        };

        // Remove from contour from left to right.
        // Without this, the pruning of the right vertex (p itself) changes the
        // layers and presence of the earlier vertex (at the start of the match
        // ending in p), breaking the subsequent pruning step.
        pruned_start_positions.sort_by_key(|p| LexPos(*p));

        // TODO: This should be optimized to a single `contours.prune` call.
        for p in pruned_start_positions {
            let pt = self.transform(p);
            self.contours.prune_with_hint(pt, hint, |pt| {
                let p = if self.params.use_gap_cost {
                    self.seeds.transform_back(*pt)
                } else {
                    *pt
                };
                self.matches.by_start.get(&p).map(|ms| {
                    ms.iter()
                        .filter(|m| m.is_active())
                        .map(match_to_arrow)
                        .filter(|a| a.end <= self.t_target)
                })
            });
        }

        let change = if p_start > 0 && false {
            let end_layer = self.contours.score_with_hint(tpos, hint).0;
            start_layer - end_layer
        } else {
            0
        };
        if let Some(start_time) = start_time {
            self.stats.pruning_duration += TIME_EACH as f32 * start_time.elapsed().as_secs_f32();
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
        Some(self.matches.iter().collect_vec())
    }

    fn seeds(&self) -> Option<&Seeds> {
        Some(&self.seeds)
    }

    fn params_string(&self) -> String {
        format!("{:?}", self.params)
    }
}
