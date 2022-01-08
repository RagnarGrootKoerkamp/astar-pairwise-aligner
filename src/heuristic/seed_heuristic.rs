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
pub struct SeedHeuristic<DH: DistanceHeuristic> {
    pub match_config: MatchConfig,
    pub distance_function: DH,
    pub pruning: bool,
    pub prune_fraction: f32,
}

impl<DH: DistanceHeuristic> Default for SeedHeuristic<DH> {
    fn default() -> Self {
        Self {
            match_config: Default::default(),
            distance_function: DH::default(),
            pruning: false,
            prune_fraction: 1.0,
        }
    }
}

impl<DH: DistanceHeuristic> Heuristic for SeedHeuristic<DH>
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
                < self.match_config.length.l().unwrap_or(usize::MAX) / 3
        );
        SimpleSeedHeuristicI::new(a, b, alphabet, *self)
    }

    fn name(&self) -> String {
        "SimpleSeed".into()
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

pub struct SimpleSeedHeuristicI<'a, DH: DistanceHeuristic> {
    params: SeedHeuristic<DH>,
    distance_function: DH::DistanceInstance<'a>,
    target: Pos,

    pub matches: SeedMatches,
    // The lowest cost match starting at each position.
    h_at_seeds: HashMap<Pos, usize>,
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
impl<'a, DH: DistanceHeuristic> DistanceHeuristicInstance<'a> for SimpleSeedHeuristicI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceHeuristicInstance<'a, Pos = Pos>,
{
    fn distance(&self, from: Self::Pos, to: Self::Pos) -> usize {
        max(
            self.distance_function.distance(from, to),
            self.matches.distance(from, to),
        )
    }
}

impl<'a, DH: DistanceHeuristic> SimpleSeedHeuristicI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceHeuristicInstance<'a, Pos = Pos>,
{
    fn new(
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
        params: SeedHeuristic<DH>,
    ) -> Self {
        let mut h = SimpleSeedHeuristicI::<'a> {
            params,
            distance_function: DistanceHeuristic::build(&params.distance_function, a, b, alphabet),
            target: Pos(a.len(), b.len()),
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
        let mut h_at_seeds = HashMap::<Pos, usize>::default();
        h_at_seeds.insert(self.target, 0);
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
            let update_val = match_cost + self.h(Node(*end, ()));
            // Skip the match.
            let query_val = self.h(Node(*start, ()));

            // Update if using is better than skipping.
            // TODO: Report some metrics on skipped states.
            if update_val < query_val {
                h_at_seeds.insert(*start, update_val);
            }
        }
        self.h_at_seeds = h_at_seeds;
    }

    pub fn h_with_parent(&self, Node(pos, _): NodeH<'a, Self>) -> (usize, Pos) {
        self.h_at_seeds
            .iter()
            .into_iter()
            .filter(|&(parent, _)| *parent >= pos)
            .map(|(parent, val)| (self.distance(pos, *parent) + val, *parent))
            .min_by_key(|(val, pos)| (*val, Reverse(LexPos(*pos))))
            .unwrap()
    }
}

impl<'a, DH: DistanceHeuristic> HeuristicInstance<'a> for SimpleSeedHeuristicI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceHeuristicInstance<'a, Pos = Pos>,
{
    type Pos = crate::graph::Pos;
    fn h(&self, Node(pos, _): NodeH<'a, Self>) -> usize {
        self.h_at_seeds
            .iter()
            .into_iter()
            .filter(|&(parent, _)| *parent >= pos)
            .map(|(parent, val)| self.distance(pos, *parent) + val)
            .min()
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
        if self.matches.is_start_of_seed(pos) {
            // When we don't ensure consistency, starts of seeds should still only be expanded once.
            assert!(
                self.expanded.insert(pos),
                "Double expanded start of seed {:?}",
                pos
            );
        }

        self.num_tried_pruned += 1;
        if self.num_actual_pruned as f32
            >= self.num_tried_pruned as f32 * self.params.prune_fraction
        {
            return;
        }
        self.num_actual_pruned += 1;

        //Prune the current position.
        self.pruned_positions.insert(pos);
        if self.h_at_seeds.remove(&pos).is_none() {
            // Nothing to do.
            return;
        }

        let start = time::Instant::now();
        self.build();
        self.pruning_duration += start.elapsed();
    }

    fn print(&self, _transform: bool, wait_for_user: bool) {
        super::print::print(self, self.matches.iter(), self.target, wait_for_user);
    }
}
