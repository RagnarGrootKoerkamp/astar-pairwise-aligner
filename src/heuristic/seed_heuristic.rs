use std::{
    cmp::Reverse,
    time::{self, Duration},
};

use super::{distance::*, *};
use crate::{
    prelude::*,
    seeds::{find_matches, Match, MatchConfig, SeedMatches},
};

#[derive(Debug, Copy, Clone)]
pub struct SeedHeuristic<DH: Distance> {
    pub match_config: MatchConfig,
    pub distance_function: DH,
    pub pruning: bool,
    pub prune_fraction: f32,
}

impl<DH: Distance> Default for SeedHeuristic<DH> {
    fn default() -> Self {
        Self {
            match_config: Default::default(),
            distance_function: DH::default(),
            pruning: false,
            prune_fraction: 1.0,
        }
    }
}

impl<DH: Distance> Heuristic for SeedHeuristic<DH>
where
    for<'a> DH::DistanceInstance<'a>: HeuristicInstance<'a, Pos = Pos>,
{
    type Instance<'a> = SimpleSeedHeuristicI<'a, DH>;

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::Instance<'a> {
        assert!(
            self.match_config.max_match_cost
                <= self.match_config.length.l().unwrap_or(I::MAX) as Cost / 3
        );
        SimpleSeedHeuristicI::new(a, b, alphabet, *self)
    }

    fn name(&self) -> String {
        "Seed".into()
    }

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            l: Some(self.match_config.length.l().unwrap_or(0)),
            max_match_cost: Some(self.match_config.max_match_cost),
            pruning: Some(self.pruning),
            distance_function: Some(self.distance_function.name()),
            ..Default::default()
        }
    }
}

pub struct SimpleSeedHeuristicI<'a, DH: Distance> {
    params: SeedHeuristic<DH>,
    distance_function: DH::DistanceInstance<'a>,
    target: Pos,

    pub matches: SeedMatches,
    // The lowest cost match starting at each position.
    h_at_seeds: HashMap<Pos, Cost>,
    // State for pruning.
    pruned_positions: HashSet<Pos>,
    // Partial pruning.
    num_tried_pruned: usize,
    num_actual_pruned: usize,
    // Make sure we don't expand a pruned state twice.
    expanded: HashSet<Pos>,

    // Statistics
    pub pruning_duration: Duration,
}

/// The seed heuristic implies a distance function as the maximum of the
/// provided distance function and the potential difference between the two
/// positions.  Assumes that the current position is not a match, and no matches
/// are visited in between `from` and `to`.
impl<'a, DH: Distance> DistanceInstance<'a> for SimpleSeedHeuristicI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceInstance<'a, Pos = Pos>,
{
    default fn distance(&self, from: Self::Pos, to: Self::Pos) -> Cost {
        max(
            self.distance_function.distance(from, to),
            self.matches.distance(from, to),
        )
    }
}

/// For GapCost, we can show that it's never optimal to actually pay for a gap (unless going to the target)
/// -- the potential difference to the parent will always be smaller.
impl<'a> DistanceInstance<'a> for SimpleSeedHeuristicI<'a, GapCost> {
    fn distance(&self, from: Self::Pos, to: Self::Pos) -> Cost {
        let gap = self.distance_function.distance(from, to);
        let pot = self.matches.distance(from, to);
        if gap <= pot {
            pot
        } else if to == self.target {
            gap
        } else {
            Cost::MAX
        }
    }
}

impl<'a, DH: Distance> SimpleSeedHeuristicI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceInstance<'a, Pos = Pos>,
{
    fn new(
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
        params: SeedHeuristic<DH>,
    ) -> Self {
        let mut h = SimpleSeedHeuristicI::<'a> {
            params,
            distance_function: Distance::build(&params.distance_function, a, b, alphabet),
            target: Pos::from_length(a, b),
            matches: find_matches(a, b, alphabet, params.match_config),
            h_at_seeds: Default::default(),
            pruned_positions: Default::default(),
            expanded: HashSet::default(),
            pruning_duration: Default::default(),
            num_tried_pruned: 0,
            num_actual_pruned: 0,
        };
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
            ..
        } in self.matches.iter().rev()
        {
            if self.pruned_positions.contains(start) {
                continue;
            }
            // Use the match.
            let update_val = match_cost + self.h(*end);
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

impl<'a, DH: Distance> HeuristicInstance<'a> for SimpleSeedHeuristicI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceInstance<'a, Pos = Pos>,
{
    type Pos = crate::graph::Pos;
    fn h(&self, pos: Self::Pos) -> Cost {
        self.h_at_seeds
            .iter()
            .into_iter()
            .filter(|&(parent, _)| *parent >= pos)
            .map(|(parent, val)| self.distance(pos, *parent).saturating_add(*val))
            .min()
            .unwrap()
    }

    fn h_with_parent(&self, pos: Self::Pos) -> (Cost, Pos) {
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
            num_seeds: Some(self.matches.num_seeds),
            num_matches: Some(self.matches.matches.len()),
            matches: Some(self.matches.matches.clone()),
            pruning_duration: Some(self.pruning_duration.as_secs_f32()),
        }
    }

    fn prune(&mut self, pos: Pos) {
        if !self.params.pruning {
            return;
        }

        // Check that we don't double expand start-of-seed states.
        if !self.matches.is_start_of_seed(pos) {
            return;
        }

        // When we don't ensure consistency, starts of seeds should still only be expanded once.
        assert!(
            self.expanded.insert(pos),
            "Double expanded start of seed {:?}",
            pos
        );

        self.num_tried_pruned += 1;
        if self.num_actual_pruned as f32
            >= self.num_tried_pruned as f32 * self.params.prune_fraction
        {
            return;
        }
        self.num_actual_pruned += 1;

        // Make sure that h remains consistent, by never pruning if it would make the new value >1 larger than it's neighbours above/below.
        {
            // Compute the new value. Can be linear time loop since we are going to rebuild anyway.
            let cur_val = self.h(pos);
            if pos.1 > 0 {
                let nb_val = self.h(Pos(pos.0, pos.1 - 1));
                assert!(cur_val + 1 >= nb_val, "cur {} nb {}", cur_val, nb_val);
                if cur_val > nb_val {
                    return;
                }
            }
            if pos.1 < self.target.1 {
                let nb_val = self.h(Pos(pos.0, pos.1 + 1));
                assert!(cur_val + 1 >= nb_val, "cur {} nb {}", cur_val, nb_val);
                if cur_val > nb_val {
                    return;
                }
            }
        }

        //Prune the current position.
        self.pruned_positions.insert(pos);
        if self.h_at_seeds.remove(&pos).is_none() {
            // Nothing to do.
            return;
        }

        if print() {
            println!("PRUNE SEED HEURISTIC: {}", pos);
        }
        let start = time::Instant::now();
        self.build();
        self.pruning_duration += start.elapsed();
        self.print(false, false);
    }

    fn print(&self, _transform: bool, wait_for_user: bool) {
        super::print::print(self, self.matches.iter(), self.target, wait_for_user);
    }
}
