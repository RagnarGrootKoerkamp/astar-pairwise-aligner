use std::{
    cmp::Reverse,
    time::{self, Duration},
};

use itertools::Itertools;

use super::{distance::*, *};
use crate::{
    matches::{find_matches, Match, MatchConfig, SeedMatches},
    prelude::*,
};

#[derive(Debug, Copy, Clone)]
pub struct BruteForceCSH<DH: Distance> {
    pub match_config: MatchConfig,
    pub distance_function: DH,
    pub pruning: bool,
}

impl<DH: Distance> Heuristic for BruteForceCSH<DH>
where
    for<'a> DH::DistanceInstance<'a>: HeuristicInstance<'a>,
{
    type Instance<'a> = BruteForceCSHI<'a, DH>;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>, alphabet: &Alphabet) -> Self::Instance<'a> {
        assert!(
            self.match_config.max_match_cost
                <= self.match_config.length.k().unwrap_or(I::MAX) as MatchCost / 3
        );
        BruteForceCSHI::new(a, b, alphabet, *self)
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

pub struct BruteForceCSHI<'a, DH: Distance> {
    params: BruteForceCSH<DH>,
    distance_function: DH::DistanceInstance<'a>,
    target: Pos,

    pub seeds: SeedMatches,
    // The lowest cost match starting at each position.
    h_at_seeds: HashMap<Pos, Cost>,
    // Remaining arrows/matches
    arrows: HashMap<Pos, Vec<Arrow>>,
    num_pruned: usize,

    // Statistics
    pub pruning_duration: Duration,
}

/// The seed heuristic implies a distance function as the maximum of the
/// provided distance function and the potential difference between the two
/// positions.  Assumes that the current position is not a match, and no matches
/// are visited in between `from` and `to`.
impl<'a, DH: Distance> DistanceInstance<'a> for BruteForceCSHI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceInstance<'a>,
{
    default fn distance(&self, from: Pos, to: Pos) -> Cost {
        max(
            self.distance_function.distance(from, to),
            self.seeds.potential_distance(from, to),
        )
    }
}

impl<'a, DH: Distance> BruteForceCSHI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceInstance<'a>,
{
    fn new(a: Seq<'a>, b: Seq<'a>, alphabet: &Alphabet, params: BruteForceCSH<DH>) -> Self {
        let mut h = BruteForceCSHI::<'a> {
            params,
            distance_function: Distance::build(&params.distance_function, a, b, alphabet),
            target: Pos::from_lengths(a, b),
            seeds: find_matches(
                a,
                b,
                alphabet,
                params.match_config,
                params.distance_function.name() == "Gap",
            ),
            h_at_seeds: Default::default(),
            arrows: Default::default(),
            pruning_duration: Default::default(),
            num_pruned: 0,
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
            len: m.seed_potential - m.match_cost,
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
                len: seed_potential - match_cost,
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

impl<'a, DH: Distance> HeuristicInstance<'a> for BruteForceCSHI<'a, DH>
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
        self.h_at_seeds
            .iter()
            .into_iter()
            .filter(|&(parent, _)| *parent >= pos)
            .map(|(parent, val)| (self.distance(pos, *parent).saturating_add(*val), *parent))
            .min_by_key(|(val, pos)| (*val, Reverse(LexPos(*pos))))
            .unwrap()
    }

    fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        self.seeds.is_seed_start_or_end(pos)
    }

    /// TODO: This is copied from CSH::prune. It would be better to have a single implementation for this.
    fn prune(&mut self, pos: Pos, _hint: Self::Hint) -> Cost {
        const D: bool = false;
        if !self.params.pruning {
            return 0;
        }

        let start = time::Instant::now();

        // Maximum length arrow at given pos.
        let tpos = pos;
        let max_match_cost = self.params.match_config.max_match_cost;

        // Prune any matches ending here.
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
                        self.num_pruned += 1;
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
            self.build();
            return 0;
        };

        // Make sure that h remains consistent: never prune positions with larger neighbouring arrows.
        // TODO: Make this smarter and allow pruning long arrows even when pruning short arrows is not possible.
        // The minimum length required for consistency here.
        let mut min_len = 0;
        if CHECK_MATCH_CONSISTENCY || self.params.distance_function.name() == "Gap" {
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
            self.build();
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
                    let tp = pos;
                    if let Some(arrows) = self.arrows.get(&tp) {
                        if arrows.iter().all(|a2| a2.end == a.end) {
                            self.num_pruned += 1;
                            self.arrows.remove(&tp);
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
            if min_len == 0 {
                self.arrows.remove(&tpos).unwrap();
            } else {
                // If we only remove a subset of arrows, do no actual pruning.
                let arrows = self.arrows.get_mut(&tpos).unwrap();
                if D {
                    println!("Remove arrows of length > {min_len} at pos {pos}.");
                }
                arrows.drain_filter(|a| a.len > min_len).count();
                assert!(arrows.len() > 0);
            };
        }

        self.pruning_duration += start.elapsed();

        self.num_pruned += 1;
        self.build();
        return 0;
    }

    fn stats(&self) -> HeuristicStats {
        HeuristicStats {
            num_seeds: self.seeds.seeds.len() as I,
            num_matches: self.seeds.matches.len(),
            num_filtered_matches: self.seeds.matches.len(),
            matches: self.seeds.matches.clone(),
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
