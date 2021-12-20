use std::{cell::RefCell, collections::HashSet, iter::once};

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
    pub max_match_cost: usize,
    pub distance_function: DH,
    pub pruning: bool,
    pub build_fast: bool,
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
        assert!(self.max_match_cost < self.l);
        SeedHeuristicI::new(a, b, alphabet, graph, *self)
    }
    fn l(&self) -> Option<usize> {
        Some(self.l)
    }
    fn max_match_cost(&self) -> Option<usize> {
        Some(self.max_match_cost)
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
    params: SeedHeuristic<DH>,
    distance_function: DH::DistanceInstance<'a>,
    target: Pos,

    seed_matches: SeedMatches,
    h_at_seeds: HashMap<Pos, usize>,
    h_cache: RefCell<HashMap<Pos, usize>>,
    graph: AlignmentGraph<'a>,
    pruned_positions: HashSet<Pos>,
}

impl<'a, DH: DistanceHeuristic> SeedHeuristicI<'a, DH> {
    fn new(
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
        graph: &AlignmentGraph<'a>,
        params: SeedHeuristic<DH>,
    ) -> Self {
        let seed_matches = find_matches(a, b, alphabet, params.l, params.max_match_cost);

        let distance_function =
            DistanceHeuristic::build(&params.distance_function, a, b, alphabet, graph);

        let mut h = SeedHeuristicI::<'a> {
            params,
            distance_function,
            target: Pos(a.len(), b.len()),
            seed_matches,
            h_at_seeds: HashMap::default(),
            h_cache: RefCell::new(HashMap::new()),
            graph: graph.clone(),
            pruned_positions: HashSet::new(),
        };
        h.build();
        h
    }

    fn build(&mut self) {
        if self.params.build_fast {
            return self.build_fast();
        }
        let mut h_at_seeds = HashMap::new();
        h_at_seeds.insert(self.target, 0);
        for Match {
            start,
            end,
            match_cost,
        } in self.seed_matches.iter().rev()
        {
            if self.pruned_positions.contains(start) {
                continue;
            }
            let update_val = h_at_seeds
                .iter()
                .filter(|&(parent, _)| parent >= end)
                .map(|(&parent, &val)| {
                    val + max(
                        self.distance_function.distance(*start, parent),
                        self.seed_matches.potential(*start) - self.seed_matches.potential(parent)
                            + match_cost
                            - (self.params.max_match_cost + 1),
                    )
                })
                .min()
                .unwrap();
            let query_val = h_at_seeds
                .iter()
                .filter(|&(parent, _)| parent >= start)
                .map(|(&parent, &val)| -> usize {
                    val + max(
                        self.distance_function.distance(*start, parent),
                        self.seed_matches.potential(*start) - self.seed_matches.potential(parent),
                    )
                })
                .min()
                .unwrap();

            // TODO: Report number of inserted and skipped matches
            assert!(
                update_val <= query_val + self.params.max_match_cost,
                "At {:?} update {} query {}",
                start,
                update_val,
                query_val
            );
            if update_val < query_val {
                h_at_seeds.insert(*start, update_val);
            }
        }
        self.h_at_seeds = h_at_seeds;
    }

    /// Build the `h_at_seeds` map in roughly O(#seeds).
    // Implementation:
    // - Loop over seeds from right to left.
    // - Keep a second
    fn build_fast(&mut self) {
        let mut h_at_seeds = HashMap::new();
        h_at_seeds.insert(self.target, 0);
        for Match {
            start,
            end,
            match_cost,
        } in self.seed_matches.iter().rev()
        {
            if self.pruned_positions.contains(start) {
                continue;
            }
            let update_val = h_at_seeds
                .iter()
                .filter(|&(parent, _)| parent >= end)
                .map(|(&parent, &val)| {
                    val + max(
                        self.distance_function.distance(*start, parent),
                        self.seed_matches.potential(*start) - self.seed_matches.potential(parent)
                            + match_cost
                            - (self.params.max_match_cost + 1),
                    )
                })
                .min()
                .unwrap();
            let query_val = h_at_seeds
                .iter()
                .filter(|&(parent, _)| parent >= start)
                .map(|(&parent, &val)| -> usize {
                    val + max(
                        self.distance_function.distance(*start, parent),
                        self.seed_matches.potential(*start) - self.seed_matches.potential(parent),
                    )
                })
                .min()
                .unwrap();

            // TODO: Report number of inserted and skipped matches
            assert!(
                update_val <= query_val + self.params.max_match_cost,
                "At {:?} update {} query {}",
                start,
                update_val,
                query_val
            );
            if update_val < query_val {
                h_at_seeds.insert(*start, update_val);
            }
        }
        self.h_at_seeds = h_at_seeds;
    }

    // The base heuristic function, which is not consistent in the following case:
    // pos A is the start of a seed, and pos B is A+(1,1), with edge cost 0.
    // In this case h(A) = P(A)-P(X) <= d(A,B) + h(B) = 0 + P(B)-P(X) = P(A)-P(X)-1
    // is false. consistent_h below fixes this.
    fn base_h(&self, pos: Pos) -> usize {
        self.h_at_seeds
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

    // The distance from the start of the current seed to the current position, capped at `match_cost+1`
    // TODO: Generalize this for overlapping seeds.
    fn consistent_h(&self, pos: Pos) -> usize {
        self.consistent_h_acc(pos, 0)
    }

    // Internal function that also takes the cost already accumulated, and returns early when the total cost is larger than the max_match_cost.
    // Delta is the cost form `pos` to the positions where we are currently evaluating `consistent_h`.
    // TODO: Benchmark whether a full DP is faster than the DFS we do currently.
    fn consistent_h_acc(&self, pos: Pos, delta: usize) -> usize {
        if let Some(h) = self.h_cache.borrow().get(&pos) {
            return *h;
        }
        // If we are currently at the start of a seed, we do not move to the left.
        let is_start_of_seed = self.seed_matches.is_start_of_seed(pos);
        // H is the maximum of the heuristic at this point, and the minimum value implied by consistency.
        let h = once(self.base_h(pos))
            .chain(
                self.graph
                    .edges_directed(pos, petgraph::EdgeDirection::Incoming)
                    .filter_map(|Edge(start, _, edge_cost)| {
                        // Do not move further left from the start of a seed.
                        if is_start_of_seed && start.0 < pos.0 {
                            None
                        } else {
                            // Do not explore states that are too much edit distance away.
                            let new_delta = edge_cost + delta;
                            if new_delta >= self.params.max_match_cost + 1 {
                                None
                            } else {
                                Some(
                                    self.consistent_h_acc(start, new_delta)
                                        .saturating_sub(edge_cost),
                                )
                            }
                        }
                    }),
            )
            .max()
            .unwrap();
        // We can only store the computed value if we are sure the computed value was not capped.
        // TODO: Reuse the computed value more often.
        if delta == 0 {
            self.h_cache.borrow_mut().insert(pos, h);
        }
        h
    }
}

impl<'a, DH: DistanceHeuristic> HeuristicInstance<'a> for SeedHeuristicI<'a, DH> {
    fn h(&self, Node(pos, _): Node<Self::IncrementalState>) -> usize {
        self.consistent_h(pos)
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
        if !self.params.pruning {
            return;
        }
        // Prune the current position.
        self.pruned_positions.insert(pos);

        // If the current position is not on a Pareto front, there is no need to rebuild.
        if self.h_at_seeds.remove(&pos).is_none() {
            return;
        }
        self.build();
    }
}
