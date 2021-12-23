use std::{cell::RefCell, cmp::Reverse, collections::HashSet, iter::once};

use super::{distance::*, heuristic::*};
use crate::{
    alignment_graph::{AlignmentGraph, Node},
    implicit_graph::{Edge, ImplicitGraphBase},
    prelude::*,
    seeds::{find_matches, Match, SeedMatches},
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

    pub seed_matches: SeedMatches,
    h_at_seeds: HashMap<Pos, usize>,
    h_cache: RefCell<HashMap<Pos, usize>>,
    graph: AlignmentGraph<'a>,
    pruned_positions: HashSet<Pos>,
}

/// The seed heuristic implies a distance function as the maximum of the
/// provided distance function and the potential difference between the two
/// positions.  Assumes that the current position is not a match, and no matches
/// are visited in between `from` and `to`.
impl<'a, DH: DistanceHeuristic> DistanceHeuristicInstance<'a> for SeedHeuristicI<'a, DH> {
    fn distance(&self, from: Pos, to: Pos) -> usize {
        let v = max(
            self.distance_function.distance(from, to),
            self.seed_matches.distance(from, to),
        );
        //println!("Distance from {:?} to {:?} = {}", from, to, v);
        v
    }
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

    fn best_distance<'b, T: Iterator<Item = (&'b Pos, &'b usize)>>(
        &self,
        pos: Pos,
        parents: T,
    ) -> usize {
        parents
            .into_iter()
            .filter(|&(parent, _)| *parent >= pos)
            .map(|(parent, val)| self.distance(pos, *parent) + val)
            .min()
            .unwrap_or_else(|| self.distance(pos, self.target))
    }

    // Returns the best distance, and the parent position where this is obtained.
    fn best_distance_with_parent<'b, T: Iterator<Item = (&'b Pos, &'b usize)>>(
        &self,
        pos: Pos,
        parents: T,
    ) -> (usize, Pos) {
        parents
            .into_iter()
            .filter(|&(parent, _)| *parent >= pos)
            .map(|(parent, val)| (self.distance(pos, *parent) + val, *parent))
            .min_by_key(|&(val, Pos(i, j))| (val, Reverse((i, j))))
            .unwrap_or_else(|| (self.distance(pos, self.target), self.target))
    }

    // TODO: Report some metrics on skipped states.
    fn build(&mut self) {
        if self.params.build_fast {
            return self.build_fast();
        }
        let mut h_at_seeds = HashMap::<Pos, usize>::new();
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
            // Use the match.
            let update_val = match_cost + self.best_distance(*end, h_at_seeds.iter());
            // Skip the match.
            let query_val = self.best_distance(*start, h_at_seeds.iter());
            // Update if using is better than skipping.

            // println!("{:?} {:?}  {} {}", end, start, update_val, query_val);
            if update_val < query_val {
                h_at_seeds.insert(*start, update_val);
            }
        }
        self.h_at_seeds = h_at_seeds;
    }

    /// Build the `h_at_seeds` map in roughly O(#seeds).
    // Implementation:
    // - Loop over seeds from right to left.
    // - Keep a second sorted list going from the bottom to the top.
    // - Keep a front over all match-starts with i>=x-l and j>=y-l, for some x,y.
    // - To determine the value at a position, simply loop over all matches in the front.
    //
    // - Matches B(x,y) are dropped from the front when we are sure they are superseeded:
    //   - The diagonal is covered by another match A, with !(start(A) < start(B))
    //     This ensures that everything that can reach B on the side of A can also reach A.
    //   - That match A is to the left at (i,*) or above at (*,j)
    //   - We have processed all matches M with end(M).0 > i or end(M).1 > j.
    //
    // When the diagonal has sufficiently many matches, this process should lead to
    // a front containing O(1) matches.
    fn build_fast(&mut self) {
        let mut h_at_seeds = HashMap::new();
        h_at_seeds.insert(self.target, 0);

        todo!();

        self.h_at_seeds = h_at_seeds;
    }

    // The base heuristic function, which is not consistent in the following case:
    // pos A is the start of a seed, and pos B is A+(1,1), with edge cost 0.
    // In this case h(A) = P(A)-P(X) <= d(A,B) + h(B) = 0 + P(B)-P(X) = P(A)-P(X)-1
    // is false. consistent_h below fixes this.
    fn base_h(&self, pos: Pos) -> usize {
        self.best_distance(pos, self.h_at_seeds.iter())
    }

    pub fn base_h_with_parent(&self, pos: Pos) -> (usize, Pos) {
        self.best_distance_with_parent(pos, self.h_at_seeds.iter())
    }

    // The distance from the start of the current seed to the current position,
    // capped at `match_cost+1`
    // TODO: Generalize this for overlapping seeds.
    fn consistent_h(&self, pos: Pos) -> usize {
        self.consistent_h_acc(pos, 0)
    }

    // Internal function that also takes the cost already accumulated, and
    // returns early when the total cost is larger than the max_match_cost.
    // Delta is the cost form `pos` to the positions where we are currently
    // evaluating `consistent_h`.
    // TODO: Benchmark whether a full DP is faster than the DFS we do currently.
    fn consistent_h_acc(&self, pos: Pos, delta: usize) -> usize {
        if let Some(h) = self.h_cache.borrow().get(&pos) {
            return *h;
        }
        // If we are currently at the start of a seed, we do not move to the left.
        let is_start_of_seed = self.seed_matches.is_start_of_seed(pos);
        // H is the maximum of the heuristic at this point, and the minimum
        // value implied by consistency.
        let h = once(self.base_h(pos))
            .chain(
                self.graph
                    .edges_directed(pos, petgraph::EdgeDirection::Incoming)
                    .filter_map(|Edge(start, _, edge_cost)| {
                        // Do not move further left from the start of a seed.
                        if is_start_of_seed && start.0 < pos.0 {
                            None
                        } else {
                            // Do not explore states that are too much edit
                            // distance away.
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
        // We can only store the computed value if we are sure the computed
        // value was not capped.
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

        // If the current position is not on a Pareto front, there is no need to
        // rebuild.
        if self.h_at_seeds.remove(&pos).is_none() {
            return;
        }
        self.build();
    }
}

#[cfg(test)]
mod tests {

    use itertools::Itertools;

    use super::*;
    use crate::setup;
    use crate::{align, alignment_graph};

    #[test]
    fn fast_build() {
        for pruning in [false, true] {
            for (l, max_match_cost) in [(4, 0), (7, 1)] {
                let h_slow = SeedHeuristic {
                    l,
                    max_match_cost,
                    distance_function: GapHeuristic,
                    pruning,
                    build_fast: false,
                };
                let h_fast = SeedHeuristic {
                    l,
                    max_match_cost,
                    distance_function: GapHeuristic,
                    pruning,
                    build_fast: true,
                };

                let (a, b, alphabet, stats) = setup(100, 0.20);

                println!("Testing: {:?}", h_fast);
                {
                    let graph = alignment_graph::new_alignment_graph(&a, &b);
                    let h_slow = h_slow.build(&a, &b, &alphabet, &graph);
                    let h_fast = h_fast.build(&a, &b, &alphabet, &graph);
                    let mut h_slow_map = h_slow.h_at_seeds.into_iter().collect_vec();
                    let mut h_fast_map = h_fast.h_at_seeds.into_iter().collect_vec();
                    h_slow_map.sort_by_key(|&(Pos(i, j), _)| (i, j));
                    h_fast_map.sort_by_key(|&(Pos(i, j), _)| (i, j));
                    assert_eq!(h_slow_map, h_fast_map);
                }

                align(
                    &a,
                    &b,
                    &alphabet,
                    stats,
                    EqualHeuristic {
                        h1: h_slow,
                        h2: h_fast,
                    },
                );
            }
        }
    }
}
