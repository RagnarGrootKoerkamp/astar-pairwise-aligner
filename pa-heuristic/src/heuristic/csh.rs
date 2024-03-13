use itertools::Itertools;
use smallvec::SmallVec;

use super::*;
use crate::contour::rotate_to_front::RotateToFrontContour;
use crate::prune::MatchPruner;
use crate::util::Timer;
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

pub type DefaultCSH = CSH<HintContours<RotateToFrontContour>>;

impl CSH<HintContours<RotateToFrontContour>> {
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
    ) -> CSH<HintContours<RotateToFrontContour>> {
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

    fn build_with_filter<'a>(
        &self,
        a: Seq<'a>,
        b: Seq<'a>,
        filter: Option<impl FnMut(&Match, Cost) -> bool>,
    ) -> Self::Instance<'a> {
        CSHI::new(a, b, filter, *self)
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

    /// For block-based pruning, the lowest and highest contour/layer from which matches were removed.
    lowest_modified_contour: Layer,
    highest_modified_contour: Layer,

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
    fn new(
        a: Seq,
        b: Seq,
        filter: Option<impl FnMut(&Match, Cost) -> bool>,
        params: CSH<C>,
    ) -> Self {
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

        let mut pruner = MatchPruner::new(params.pruning, params.use_gap_cost, matches, &seeds);

        // Matches are sorted by reversed start (the order needed to construct contours).
        // TODO: Can we get rid of this ugly temporary copy somehow?
        let copied_matches = pruner.iter().cloned().collect_vec();
        let arrows = copied_matches
            .iter()
            .rev()
            .filter(|m| m.is_active())
            .map(match_to_arrow)
            .filter(|a| a.end <= t_target);
        let contours = if let Some(mut filter) = filter {
            // NOTE: This `filter` is only used an path-pruning experiment.
            C::new_with_filter(arrows, params.match_config.r as I, |arrow, layer| {
                let m = Match {
                    start: if params.use_gap_cost {
                        seeds.transform_back(arrow.start)
                    } else {
                        arrow.start
                    },
                    end: if params.use_gap_cost {
                        seeds.transform_back(arrow.end)
                    } else {
                        arrow.start
                    },
                    match_cost: params.match_config.r - arrow.score,
                    seed_potential: params.match_config.r,
                    pruned: MatchStatus::Active,
                };
                let f = filter(&m, seeds.potential(m.start) - layer);
                if !f {
                    pruner.mut_match_start(&m).unwrap().filter();
                    pruner.mut_match_end(&m).unwrap().filter();
                }
                f
            })
        } else {
            C::new(arrows, params.match_config.r as I)
        };

        let mut h = CSHI {
            params,
            gap_distance: Distance::build(&GapCost, a, b),
            target,
            t_target,
            seeds,
            matches: pruner,
            stats: HeuristicStats::default(),

            // For pruning propagation
            max_transformed_pos: Pos(I::MIN, I::MIN),

            contours,
            lowest_modified_contour: Layer::MAX,
            highest_modified_contour: Layer::MIN,
        };
        h.stats.h0 = h.h(Pos(0, 0));
        h.stats.num_seeds = h.seeds.seeds.len() as _;
        h.stats.num_matches = num_matches;
        h.stats.num_filtered_matches = num_filtered_matches;
        // eprintln!("#matches:          {}", num_matches);
        // eprintln!("#filtered matches: {}", num_filtered_matches);
        // eprintln!(
        //     "#flt matches/seed: {}",
        //     num_filtered_matches as f32 / h.seeds.seeds.len() as f32
        // );
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

    /// Same as `transform`, but doesn't take `self` for better borrowing.
    fn transform_2(params: &CSH<C>, seeds: &Seeds, pos: Pos) -> Pos {
        if params.use_gap_cost {
            seeds.transform(pos)
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
        let ans = if val == 0 {
            (self.distance(pos, self.target), new_hint)
        } else {
            (p - val, new_hint)
        };
        ans
    }

    fn h_with_hint_timed(&mut self, pos: Pos, hint: Self::Hint) -> ((Cost, Self::Hint), f64) {
        let timer = Timer::new(&mut self.stats.h_calls);
        let ans = self.h_with_hint(pos, hint);
        let t = timer.end(&mut self.stats.h_duration);
        (ans, t)
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

        // Time the duration of pruning and consistency checks.
        let timer = Timer::each(64, &mut self.stats.prune_calls);

        let tpos = self.transform(pos);

        let mut pruned_start_positions: SmallVec<[Pos; 5]> = Default::default();
        let (p_start, p_end) = self.matches.prune(&self.seeds, pos, |m| {
            if !pruned_start_positions.contains(&m.start) {
                pruned_start_positions.push(m.start)
            }
        });
        timer.end(&mut self.stats.prune_duration);
        if p_start + p_end == 0 {
            return (0, pos);
        }

        // Time the duration updating the contours.
        // Each of them is timed individually, since the variance can be high.
        let timer = Timer::each(1, &mut self.stats.contours_calls);

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
        let mut change = 0;
        for p in pruned_start_positions {
            let pt = self.transform(p);
            let c = self.contours.prune_with_hint(pt, hint, |pt| {
                let p = if self.params.use_gap_cost {
                    self.seeds.transform_back(*pt)
                } else {
                    *pt
                };
                self.matches.matches_for_start(p).map(|ms| {
                    ms.iter()
                        .filter(|m| m.is_active())
                        .map(match_to_arrow)
                        .filter(|a| a.end <= self.t_target)
                })
            });
            if p == pos {
                // For CSH, propagating just works.
                // For GCSH, we manually ensure here that all states in the queue are <= the pruned pos.
                if !self.params.use_gap_cost || self.max_transformed_pos <= tpos {
                    change = c.1;
                }
            }
        }
        timer.end(&mut self.stats.contours_duration);

        (change, pos)
    }

    /// Prune all matches in a block.
    /// NOTE that this does not update `h` or the contours yet; call `update_contours` for that.
    fn prune_block(&mut self, i_range: Range<I>, j_range: Range<I>) {
        let start = instant::Instant::now();
        let mut hint = Self::Hint::default();
        let mut lowest_modified_contour = self.lowest_modified_contour;
        let mut highest_modified_contour = self.highest_modified_contour;
        self.matches.prune_block(i_range, j_range, |m| {
            let (layer, new_hint) = self
                .contours
                .score_with_hint(Self::transform_2(&self.params, &self.seeds, m.start), hint);
            if PRINT {
                eprintln!("Prune match {m:?} in layer {layer}");
            }
            // eprintln!("Prune match {m:?} in layer {layer}");
            lowest_modified_contour = min(lowest_modified_contour, layer as Layer);
            highest_modified_contour = max(highest_modified_contour, layer as Layer);
            hint = new_hint;
        });
        self.lowest_modified_contour = lowest_modified_contour;
        self.highest_modified_contour = highest_modified_contour;

        self.stats.prune_duration += start.elapsed().as_secs_f64();
    }

    /// Update contours from `lowest_modified_contour` to `highest_modified_contour`.
    /// Stop when the entire contour is *left of* `_pos.0`.
    fn update_contours(&mut self, pos: Pos) {
        let start = instant::Instant::now();

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

        // eprintln!(
        //     "Prune contours from {} to {} right of {}",
        //     self.lowest_modified_contour, self.highest_modified_contour, pos.0
        // );
        // FIXME Figure out why pruning up to Layer::MAX gives errors.
        // Pruning up to highest_modified_contour also errors, which is
        // explained by leaving the heuristic in an inconsistent state.
        self.contours.update_layers(
            self.lowest_modified_contour,
            // continue to exactly the highest modified contour.
            // self.highest_modified_contour,
            Layer::MAX,
            &|pt: &Pos| {
                let p = if self.params.use_gap_cost {
                    self.seeds.transform_back(*pt)
                } else {
                    *pt
                };
                self.matches.matches_for_start(p).map(|ms| {
                    ms.iter()
                        .filter(|m| m.is_active())
                        .map(match_to_arrow)
                        .filter(|a| a.end <= self.t_target)
                })
            },
            // None::<(_, fn(_) -> _)>,
            Some((pos.0, |pt: Pos| {
                if self.params.use_gap_cost {
                    self.seeds.transform_back(pt)
                } else {
                    pt
                }
            })),
        );
        // self.lowest_modified_contour = Layer::MAX;
        self.highest_modified_contour = Layer::MIN;
        if PRINT {
            eprintln!("h0 after  update: {}", self.h(Pos(0, 0)));
        }
        self.stats.contours_duration += start.elapsed().as_secs_f64();
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
        Some(self.matches.iter().cloned().collect_vec())
    }

    fn seeds(&self) -> Option<&Seeds> {
        Some(&self.seeds)
    }

    fn params_string(&self) -> String {
        format!("{:?}", self.params)
    }
}
