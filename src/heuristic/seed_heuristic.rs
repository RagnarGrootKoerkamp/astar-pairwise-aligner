use std::cell::RefCell;

use super::{distance::*, heuristic::*};
use crate::{
    alignment_graph::{AlignmentGraph, Node},
    implicit_graph::{Edge, ImplicitGraphBase},
    seeds::{find_matches, Match, SeedMatches},
    util::*,
};

#[derive(Debug, Clone, Copy)]
pub struct SeedHeuristic<DH: DistanceHeuristic> {
    pub l: usize,
    pub match_distance: usize,
    pub distance_function: DH,
    pub pruning: bool,
}
impl<DH: DistanceHeuristic> Heuristic for SeedHeuristic<DH> {
    type Instance<'a> = SeedHeuristicI<'a, DH>;

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
        graph: &AlignmentGraph<'a>,
    ) -> Self::Instance<'a> {
        SeedHeuristicI::new(a, b, alphabet, graph, &self)
    }
    fn l(&self) -> Option<usize> {
        Some(self.l)
    }
    fn match_distance(&self) -> Option<usize> {
        Some(self.match_distance)
    }
    fn pruning(&self) -> Option<bool> {
        Some(self.pruning)
    }
    fn distance(&self) -> Option<String> {
        Some(self.distance_function.name())
    }
    fn name(&self) -> String {
        "Seed".into()
    }
}
pub struct SeedHeuristicI<'a, DH: DistanceHeuristic> {
    seed_matches: SeedMatches,
    h_map: HashMap<Pos, usize>,
    distance_function: DH::DistanceInstance<'a>,
    target: Pos,
    // TODO: Replace this by params: SeedHeuristic
    pruning: bool,
    max_match_distance: usize,
    graph: AlignmentGraph<'a>,
}

impl<'a, DH: DistanceHeuristic> SeedHeuristicI<'a, DH> {
    fn new(
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
        graph: &AlignmentGraph<'a>,
        params: &SeedHeuristic<DH>,
    ) -> Self {
        let seed_matches = find_matches(a, b, alphabet, params.l, params.match_distance);

        let distance_function =
            DistanceHeuristic::build(&params.distance_function, a, b, alphabet, graph);

        let mut h_map = HashMap::new();
        h_map.insert(Pos(a.len(), b.len()), 0);
        for Match {
            start,
            end,
            match_distance,
        } in seed_matches.iter().rev()
        {
            let update_val = h_map
                .iter()
                .filter(|&(parent, _)| parent >= end)
                .map(|(&parent, &val)| {
                    val + max(
                        distance_function.distance(*start, parent),
                        seed_matches.potential(*start) - seed_matches.potential(parent)
                            + match_distance
                            - (params.match_distance + 1),
                    )
                })
                .min()
                .unwrap();
            let query_val = h_map
                .iter()
                .filter(|&(parent, _)| parent >= start)
                .map(|(&parent, &val)| -> usize {
                    val + max(
                        distance_function.distance(*start, parent),
                        seed_matches.potential(*start) - seed_matches.potential(parent),
                    )
                })
                .min()
                .unwrap();

            // TODO: Report number of inserted and skipped matches
            if update_val < query_val {
                h_map.insert(*start, update_val);
            }
        }
        SeedHeuristicI::<'a> {
            seed_matches,
            h_map,
            distance_function,
            target: Pos(a.len(), b.len()),
            pruning: params.pruning,
            max_match_distance: params.match_distance,
            graph: graph.clone(),
        }
    }
}

impl<'a, DH: DistanceHeuristic> HeuristicInstance<'a> for SeedHeuristicI<'a, DH> {
    fn h(&self, Node(pos, _): Node<Self::IncrementalState>) -> usize {
        self.h_map
            .iter()
            .filter(|&(&parent, &_)| parent >= pos)
            .map(|(&parent, &val)| {
                val + max(
                    self.distance_function.distance(pos, parent),
                    self.seed_matches.potential(pos) - self.seed_matches.potential(parent),
                )
            })
            .min()
            .unwrap_or(self.distance_function.distance(pos, self.target)) as usize
    }
    fn num_seeds(&self) -> Option<usize> {
        Some(self.seed_matches.num_seeds)
    }
    fn matches(&self) -> Option<&Vec<Match>> {
        Some(&self.seed_matches.matches)
    }
    fn num_matches(&self) -> Option<usize> {
        Some(self.seed_matches.matches.len())
    }
    fn prune(&mut self, pos: Pos) {
        if !self.pruning {
            return;
        }
        // TODO: Efficient pruning
        // If this is not a matching position, no need to rebuild the heuristic.
        if self.h_map.remove(&pos).is_none() {
            return;
        }

        let mut h_map = HashMap::new();
        h_map.insert(self.target, 0);
        for Match {
            start,
            end,
            match_distance,
        } in self.seed_matches.matches.iter().rev()
        {
            if !self.h_map.contains_key(&start) {
                continue;
            }

            let update_val = h_map
                .iter()
                .filter(|&(parent, _)| parent >= end)
                .map(|(&parent, &val)| {
                    val + max(
                        self.distance_function.distance(*start, parent),
                        self.seed_matches.potential(*start) - self.seed_matches.potential(parent)
                            + match_distance
                            - (self.max_match_distance + 1),
                    )
                })
                .min()
                .unwrap();
            let query_val = h_map
                .iter()
                .filter(|&(parent, _)| parent >= start)
                .map(|(&parent, &val)| {
                    val + max(
                        self.distance_function.distance(*start, parent),
                        self.seed_matches.potential(*start) - self.seed_matches.potential(parent),
                    )
                })
                .min()
                .unwrap();

            if update_val < query_val {
                h_map.insert(*start, update_val);
            }
        }
        self.h_map = h_map;
    }
}
