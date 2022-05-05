use std::{
    cmp::Reverse,
    time::{self, Duration},
};

use itertools::Itertools;

use super::{distance::*, *};
use crate::{
    matches::{find_matches, Match, MatchConfig, Seeds},
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

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::Instance<'a> {
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

    pub seeds: Seeds,
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
    fn new(
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
        params: BruteForceCSH<DH>,
    ) -> Self {
        let mut h = BruteForceCSHI::<'a> {
            params,
            distance_function: Distance::build(&params.distance_function, a, b, alphabet),
            target: Pos::from_length(a, b),
            seeds: find_matches(a, b, alphabet, params.match_config),
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
        let arrows_iterator = h.seeds.matches.iter().map(
            |&Match {
                 start,
                 end,
                 match_cost,
                 seed_potential,
             }| {
                Arrow {
                    start,
                    end,
                    len: seed_potential - match_cost,
                }
            },
        );

        h.arrows = arrows_iterator
            .clone()
            .group_by(|a| a.start)
            .into_iter()
            .map(|(start, pos_arrows)| (start, pos_arrows.collect_vec()))
            .collect();

        h.build();
        h.print(false, false);
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
            ..
        } in self.seeds.matches.iter().rev()
        {
            if !self.arrows.contains_key(start) {
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

    fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        self.seeds.is_seed_start_or_end(pos)
    }

    // TODO: This is copied from CSH::prune. It would be better to have a single implementation for this.
    fn prune(&mut self, pos: Pos, _hint: Self::Hint, _seed_cost: MatchCost) -> Cost {
        const D: bool = false;
        if !self.params.pruning {
            return 0;
        }

        let start = time::Instant::now();

        // Maximum length arrow at given pos.
        let a = if let Some(arrows) = self.arrows.get(&pos) {
            arrows.iter().max_by_key(|a| a.len).unwrap().clone()
        } else {
            self.pruning_duration += start.elapsed();
            return 0;
        };

        // Make sure that h remains consistent: never prune positions with larger neighbouring arrows.
        // TODO: Make this smarter and allow pruning long arrows even when pruning short arrows is not possible.
        // The minimum length required for consistency here.
        let mut min_len = 0;
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

        if a.len <= min_len {
            return 0;
        }

        if D || print() {
            println!("PRUNE GAP SEED HEURISTIC {pos} to {min_len}: {a}");
        }

        // If there is an exact match here, also prune neighbouring states for which all arrows end in the same position.
        // TODO: Make this more precise for larger inexact matches.
        if PRUNE_INEXACT_MATCHES_BY_END && a.len == self.params.match_config.max_match_cost + 1 {
            // See if there are neighbouring points that can now be fully pruned.
            for d in 1..=self.params.match_config.max_match_cost {
                let mut check = |pos: Pos| {
                    if !self.arrows.contains_key(&pos) {
                        println!("Did not find nb arrow at {pos} while pruning {a} at {pos}");
                    }
                    let arrows = self.arrows.get(&pos).expect("Arrows are not consistent!");
                    if arrows.iter().all(|a2| a2.end == a.end) {
                        self.num_pruned += 1;
                        self.arrows.remove(&pos);
                    }
                };
                if pos.0 >= d as Cost {
                    check(Pos(pos.0, pos.1 - d as I));
                }
                check(Pos(pos.0, pos.1 + d as I));
            }
        }

        if min_len == 0 {
            self.arrows.remove(&pos).unwrap();
        } else {
            // If we only remove a subset of arrows, do no actual pruning.
            // TODO: Also update contours on partial pruning.
            let arrows = self.arrows.get_mut(&pos).unwrap();
            if D {
                println!("Remove arrows of length > {min_len} at pos {pos}.");
            }
            arrows.drain_filter(|a| a.len > min_len).count();
            assert!(arrows.len() > 0);
        };

        // Rebuild the datastructure.
        self.build();

        self.pruning_duration += start.elapsed();

        self.num_pruned += 1;
        if print() {
            self.print(false, false);
        }
        return 0;
    }

    fn print(&self, _transform: bool, wait_for_user: bool) {
        super::print::terminal_print(self, self.target, wait_for_user);
    }
}
