use std::cmp::Reverse;

use itertools::Itertools;
use rand::Rng;

use super::*;
use crate::matches::*;

#[derive(Debug, Copy, Clone)]
pub struct BruteForceGCSH<DH: Distance> {
    pub match_config: MatchConfig,
    pub distance_function: DH,
    pub pruning: Pruning,
}

impl<DH: Distance> Heuristic for BruteForceGCSH<DH>
where
    for<'a> DH::DistanceInstance<'a>: HeuristicInstance<'a>,
{
    type Instance<'a> = BruteForceGCSHI<'a, DH>;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        assert!(
            self.match_config.max_match_cost
                <= self.match_config.length.k().unwrap_or(I::MAX) as MatchCost / 3
        );
        BruteForceGCSHI::new(a, b, *self)
    }

    fn name(&self) -> String {
        "Seed".into()
    }

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            k: self.match_config.length.k().unwrap_or(0),
            max_match_cost: self.match_config.max_match_cost,
            pruning: self.pruning,
            distance_function: self.distance_function.name(),
            ..Default::default()
        }
    }
}

pub struct BruteForceGCSHI<'a, DH: Distance> {
    params: BruteForceGCSH<DH>,
    distance_function: DH::DistanceInstance<'a>,
    target: Pos,

    pub seeds: SeedMatches,
    // The lowest cost match starting at each position.
    h_at_seeds: HashMap<Pos, Cost>,
    // Remaining arrows/matches
    arrows: HashMap<Pos, Vec<Arrow>>,

    stats: HeuristicStats,
}

/// The seed heuristic implies a distance function as the maximum of the
/// provided distance function and the potential difference between the two
/// positions.  Assumes that the current position is not a match, and no matches
/// are visited in between `from` and `to`.
impl<'a, DH: Distance> DistanceInstance<'a> for BruteForceGCSHI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceInstance<'a>,
{
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        max(
            self.distance_function.distance(from, to),
            self.seeds.potential_distance(from, to),
        )
    }
}

impl<'a, DH: Distance> BruteForceGCSHI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceInstance<'a>,
{
    fn new(a: Seq<'a>, b: Seq<'a>, params: BruteForceGCSH<DH>) -> Self {
        let mut h = BruteForceGCSHI::<'a> {
            params,
            distance_function: Distance::build(&params.distance_function, a, b),
            target: Pos::target(a, b),
            seeds: find_matches(
                a,
                b,
                params.match_config,
                params.distance_function.name() == "Gap",
            ),
            h_at_seeds: Default::default(),
            arrows: Default::default(),
            stats: Default::default(),
        };
        assert!(h
            .seeds
            .matches
            .is_sorted_by_key(|Match { start, .. }| LexPos(*start)));

        // Transform to Arrows.
        // For arrows with length > 1, also make arrows for length down to 1.
        let match_to_arrow = |m: &Match| Arrow {
            start: m.start,
            end: m.end,
            score: m.seed_potential - m.match_cost,
        };

        h.arrows = h
            .seeds
            .matches
            .iter()
            .map(match_to_arrow)
            .group_by(|a| a.start)
            .into_iter()
            .map(|(start, pos_arrows)| (start, pos_arrows.collect_vec()))
            .collect();

        h.build();
        h.stats = HeuristicStats {
            num_seeds: h.seeds.seeds.len() as I,
            num_matches: h.seeds.matches.len(),
            num_filtered_matches: h.seeds.matches.len(),
            ..Default::default()
        };
        h
    }

    // A separate function that can be reused with pruning.
    fn build(&mut self) {
        self.h_at_seeds.clear();
        self.h_at_seeds.insert(self.target, 0);
        for Match {
            start,
            end,
            match_cost,
            seed_potential,
            ..
        } in self.seeds.matches.iter().rev()
        {
            let Some(arrows) = self.arrows.get(start) else {continue;};

            if !arrows.contains(&Arrow {
                start: *start,
                end: *end,
                score: seed_potential - match_cost,
            }) {
                continue;
            }

            // Use the match.
            let update_val = *match_cost as Cost + self.h(*end);
            // Skip the match.
            let query_val = self.h(*start);

            // Update if using is better than skipping.
            // TODO: Report some metrics on skipped states.
            if update_val < query_val {
                self.h_at_seeds.insert(*start, update_val);
            }
        }
    }
}

impl<'a, DH: Distance> HeuristicInstance<'a> for BruteForceGCSHI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceInstance<'a>,
{
    fn h(&self, pos: Pos) -> Cost {
        self.h_at_seeds
            .iter()
            .into_iter()
            .filter(|&(parent, _)| *parent >= pos)
            .map(|(parent, val)| self.distance(pos, *parent).saturating_add(*val))
            .min()
            .unwrap()
    }

    fn h_with_parent(&self, pos: Pos) -> (Cost, Pos) {
        let rng = &mut rand::thread_rng();
        self.h_at_seeds
            .iter()
            .into_iter()
            .filter(|&(parent, _)| *parent >= pos)
            .map(|(parent, val)| (self.distance(pos, *parent).saturating_add(*val), *parent))
            .min_by_key(|(val, pos)| (*val, rng.gen_range(0..u64::MAX), Reverse(LexPos(*pos))))
            .unwrap()
    }

    fn seed_matches(&self) -> Option<&SeedMatches> {
        Some(&self.seeds)
    }

    /// TODO: This is copied from CSH::prune. It would be better to have a single implementation for this.
    fn prune(&mut self, pos: Pos, _hint: Self::Hint) -> (Cost, ()) {
        const D: bool = false;
        if self.params.pruning.is_enabled() {
            return (0, ());
        }

        let start = instant::Instant::now();

        // Maximum length arrow at given pos.
        let tpos = pos;
        let max_match_cost = self.params.match_config.max_match_cost;

        // Prune any matches ending here.
        if self.params.pruning.end() {
            'prune_by_end: {
                // Check all possible start positions of a match ending here.
                if let Some(s) = self.seeds.seed_ending_at(pos) {
                    assert_eq!(pos.0, s.end);
                    if s.start + pos.1 < pos.0 {
                        break 'prune_by_end;
                    }
                    let match_start = Pos(s.start, s.start + pos.1 - pos.0);
                    let mut try_prune_pos = |startpos: Pos| {
                        let tp = startpos;
                        let Some(arrows) = self.arrows.get_mut(&tp) else { return; };
                        // Filter arrows starting in the current position.
                        if arrows
                            .drain_filter(|a| {
                                if a.end == tpos {
                                    //println!("B: Remove {a:?}");
                                    true
                                } else {
                                    false
                                }
                            })
                            .count()
                            == 0
                        {
                            return;
                        }
                        if arrows.is_empty() {
                            self.arrows.remove(&tp).unwrap();
                            //println!("B: empty {tp}");
                        }
                        self.stats.num_pruned += 1;
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
            self.stats.pruning_duration += start.elapsed().as_secs_f32();
            self.build();
            return (0, ());
        };

        // Make sure that h remains consistent: never prune positions with larger neighbouring arrows.
        // TODO: Make this smarter and allow pruning long arrows even when pruning short arrows is not possible.
        // The minimum length required for consistency here.
        let mut min_len = 0;
        if CHECK_MATCH_CONSISTENCY || self.params.distance_function.name() == "Gap" {
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
            self.build();
            return (0, ());
        }

        if D {
            println!("PRUNE GAP SEED HEURISTIC {pos} to {min_len}: {a}");
        }

        if self.params.pruning.start() {
            if min_len == 0 {
                self.arrows.remove(&tpos).unwrap();
            } else {
                // If we only remove a subset of arrows, do no actual pruning.
                let arrows = self.arrows.get_mut(&tpos).unwrap();
                if D {
                    println!("Remove arrows of length > {min_len} at pos {pos}.");
                }
                arrows.drain_filter(|a| a.score > min_len).count();
                assert!(arrows.len() > 0);
            };
        }

        self.stats.pruning_duration += start.elapsed().as_secs_f32();

        self.stats.num_pruned += 1;
        self.build();
        return (0, ());
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
