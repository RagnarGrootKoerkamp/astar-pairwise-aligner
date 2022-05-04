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

/// For GapCost, we can show that it's never optimal to actually pay for a gap (unless going to the target)
/// -- the potential difference to the parent will always be smaller.
impl<'a> DistanceInstance<'a> for BruteForceCSHI<'a, GapCost> {
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        let gap = self.distance_function.distance(from, to);
        let pot = self.seeds.potential_distance(from, to);
        if gap <= pot {
            pot
        } else if to == self.target {
            gap
        } else {
            Cost::MAX
        }
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

    fn prune(&mut self, pos: Pos, _hint: Self::Hint, _seed_cost: MatchCost) -> Cost {
        if !self.params.pruning {
            return 0;
        }

        let start = time::Instant::now();

        if !self.arrows.contains_key(&pos) {
            self.pruning_duration += start.elapsed();
            return 0;
        }

        // Make sure that h remains consistent: never prune positions with larger neighbouring arrows.
        // FIXME: Replace this logic by the version from ChainedSeedsHeuristic
        for d in 1..=self.params.match_config.max_match_cost {
            if pos.1 >= d as Cost {
                if let Some(pos_arrows) = self.arrows.get(&Pos(pos.0, pos.1 - d as Cost)) {
                    if pos_arrows.iter().map(|a| a.len).max().unwrap() > d {
                        self.pruning_duration += start.elapsed();
                        return 0;
                    }
                }
            }
            if pos.1 + d as Cost <= self.target.1 {
                if let Some(pos_arrows) = self.arrows.get(&Pos(pos.0, pos.1 + d as Cost)) {
                    if pos_arrows.iter().map(|a| a.len).max().unwrap() > d {
                        self.pruning_duration += start.elapsed();
                        return 0;
                    }
                }
            }
        }

        //Prune the current position.
        if print() {
            println!("PRUNE SEED HEURISTIC: {}", pos);
        }

        self.arrows.remove(&pos).unwrap();

        if self.h_at_seeds.remove(&pos).is_none() {
            // No need to rebuild.
            return 0;
        }
        self.build();
        self.pruning_duration += start.elapsed();
        self.print(false, false);
        0
    }

    fn print(&self, _transform: bool, wait_for_user: bool) {
        super::print::print(self, self.target, wait_for_user);
    }
}
