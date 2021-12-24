use std::{cell::RefCell, cmp::Reverse, collections::HashSet, iter::once};

use itertools::Itertools;

use super::{distance::*, heuristic::*};
use crate::{
    alignment_graph::{self, Node},
    implicit_graph::Edge,
    increasing_function::IncreasingFunction2D,
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
    pub query_fast: bool,
    pub make_consistent: bool,
}
impl<DH: DistanceHeuristic> Heuristic for SeedHeuristic<DH> {
    type Instance<'a> = SeedHeuristicI<'a, DH>;

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::Instance<'a> {
        assert!(self.max_match_cost < self.l);
        SeedHeuristicI::new(a, b, alphabet, *self)
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
    a: &'a Sequence,
    b: &'a Sequence,
    params: SeedHeuristic<DH>,
    distance_function: DH::DistanceInstance<'a>,
    target: Pos,

    pub seed_matches: SeedMatches,
    active_matches: HashSet<Pos>,
    h_at_seeds: HashMap<Pos, usize>,
    h_cache: RefCell<HashMap<Pos, usize>>,
    pruned_positions: HashSet<Pos>,

    // For the fast version
    transform_target: Pos,
    increasing_function: IncreasingFunction2D<usize>,
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
        params: SeedHeuristic<DH>,
    ) -> Self {
        let seed_matches = find_matches(a, b, alphabet, params.l, params.max_match_cost);

        let distance_function = DistanceHeuristic::build(&params.distance_function, a, b, alphabet);

        if params.build_fast {
            assert!(params.distance_function.name() == "Gap");
        }
        if params.query_fast {
            assert!(
                params.build_fast,
                "Query_fast only works when build_fast is enabled"
            );
        }

        let mut h = SeedHeuristicI::<'a> {
            a,
            b,
            params,
            distance_function,
            target: Pos(a.len(), b.len()),
            seed_matches,
            h_at_seeds: HashMap::default(),
            active_matches: HashSet::default(),
            h_cache: RefCell::new(HashMap::new()),
            pruned_positions: HashSet::new(),
            increasing_function: Default::default(),
            // Filled below.
            transform_target: Pos(0, 0),
        };
        h.transform_target = if h.params.build_fast {
            h.transform(h.target)
        } else {
            h.target
        };
        h.build();
        //println!("{:?}", h.h_at_seeds);
        //println!("{:?}", h.active_matches);
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

    pub fn transform(&self, pos @ Pos(i, j): Pos) -> Pos {
        let a = self.target.0;
        let b = self.target.1;
        let pot = |pos| self.seed_matches.potential(pos);
        Pos(
            i + b - j + pot(Pos(0, 0)) - pot(pos),
            j + a - i + pot(Pos(0, 0)) - pot(pos),
        )
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
        // The bottom right of the transformed region.
        let transform_target = self.transform(self.target);
        let leftover_at_end = self.seed_matches.start_of_seed[self.target.0] < self.target.0;

        // println!("Bot ri: xxx / {:?}", transform_target);
        // println!(
        //     "Target: {:?} / {:?}",
        //     self.target,
        //     self.transform(self.target)
        // );
        // println!("Start : {:?} / {:?}", Pos(0, 0), transform_start);

        let filtered_matches = self
            .seed_matches
            .iter()
            .filter(|Match { end, .. }| self.transform(*end) <= transform_target)
            // Filter matches by transformed start position.
            .filter(|Match { start, .. }| !self.pruned_positions.contains(start))
            .collect_vec();
        self.active_matches = filtered_matches
            .iter()
            .map(|Match { start, .. }| *start)
            .collect();
        let mut transformed_matches = filtered_matches
            .into_iter()
            .map(
                |&Match {
                     start,
                     end,
                     match_cost,
                 }| {
                    //println!("Match: {:?}", m);
                    // println!(
                    //     "Filter match {:?} / {:?}: {}",
                    //     start,
                    //     self.transform(start),
                    //     !self.pruned_positions.contains(&self.transform(start))
                    // );
                    Match {
                        start: self.transform(start),
                        end: self.transform(end),
                        match_cost,
                    }
                },
            )
            .collect_vec();
        transformed_matches
            .sort_by_key(|Match { end, start, .. }| (start.0, start.1, end.0, end.1));
        // for x in &transformed_matches {
        //     println!("Match: {:?}", x);
        // }
        //dbg!(&transformed_matches);
        self.increasing_function = IncreasingFunction2D::new(
            transform_target,
            self.params.max_match_cost,
            leftover_at_end,
            transformed_matches,
        );
        self.h_at_seeds = self.increasing_function.to_map();

        let mut h_map = self.h_at_seeds.iter().collect_vec();
        h_map.sort_by_key(|&(Pos(i, j), _)| (i, j));
        //println!("H: {:?}", h_map);
    }

    // The base heuristic function, which is not consistent in the following case:
    // pos A is the start of a seed, and pos B is A+(1,1), with edge cost 0.
    // In this case h(A) = P(A)-P(X) <= d(A,B) + h(B) = 0 + P(B)-P(X) = P(A)-P(X)-1
    // is false. consistent_h below fixes this.
    fn base_h(&self, Node(pos, parent): HNode<'a, Self>) -> usize {
        let d = if self.params.query_fast {
            let p = self.seed_matches.potential(pos);
            let val = self.increasing_function.val(parent);
            if parent == 0 {
                self.distance(pos, self.target)
            } else {
                p - val
            }
        } else if self.params.build_fast {
            let pos_transformed = self.transform(pos);
            let p = self.seed_matches.potential(pos);
            self.h_at_seeds
                .iter()
                .filter(|&(parent, _)| *parent >= pos_transformed)
                .map(|(_parent, val)| p - *val)
                .min()
                .unwrap_or(self.distance(pos, self.target))
        } else {
            self.best_distance(pos, self.h_at_seeds.iter())
        };
        //println!("h {:?} -> {:?}", pos, self.base_h_with_parent(pos.0));
        d
    }

    pub fn base_h_with_parent(&self, pos: Pos) -> (usize, Pos) {
        if self.params.build_fast {
            let pos_transformed = self.transform(pos);
            let to_end = (self.distance(pos, self.target), self.target);
            let (val, parent) = self
                .h_at_seeds
                .iter()
                .filter(|&(parent, _)| *parent >= pos_transformed)
                .map(|(parent, val)| {
                    // println!(
                    //     "pos {:?} parent {:?} pot {} val {}",
                    //     pos,
                    //     parent,
                    //     self.seed_matches.potential(pos),
                    //     val
                    // );
                    (self.seed_matches.potential(pos) - *val, *parent)
                })
                .min_by_key(|&(val, Pos(i, j))| (val, Reverse((i, j))))
                .unwrap_or(to_end);
            (val, parent)
            // println!(
            //     "H at {:?} / {:?}: {} - {} \t for parent {:?}",
            //     pos,
            //     pos_transformed,
            //     self.seed_matches.potential(pos),
            //     val,
            //     parent,
            // );
        } else {
            self.h_at_seeds
                .iter()
                .filter(|&(parent, _)| *parent >= pos)
                .map(|(parent, val)| (self.distance(pos, *parent) + val, *parent))
                .min_by_key(|&(val, Pos(i, j))| (val, Reverse((i, j))))
                .unwrap_or_else(|| (self.distance(pos, self.target), self.target))
        }
    }

    // The distance from the start of the current seed to the current position,
    // capped at `match_cost+1`
    // TODO: Generalize this for overlapping seeds.
    fn consistent_h(&self, pos: HNode<'a, Self>) -> usize {
        self.consistent_h_acc(pos, 0)
    }

    // Internal function that also takes the cost already accumulated, and
    // returns early when the total cost is larger than the max_match_cost.
    // Delta is the cost form `pos` to the positions where we are currently
    // evaluating `consistent_h`.
    // TODO: Benchmark whether a full DP is faster than the DFS we do currently.
    fn consistent_h_acc(&self, pos: HNode<'a, Self>, delta: usize) -> usize {
        if let Some(h) = self.h_cache.borrow().get(&pos.0) {
            return *h;
        }
        // If we are currently at the start of a seed, we do not move to the left.
        let is_start_of_seed = self.seed_matches.is_start_of_seed(pos.0);
        // H is the maximum of the heuristic at this point, and the minimum
        // value implied by consistency.
        let h = once(self.base_h(pos))
            .chain(
                alignment_graph::incremental_edges(
                    &self.a,
                    &self.b,
                    self,
                    pos,
                    petgraph::EdgeDirection::Incoming,
                )
                .filter_map(|Edge(start, _, edge_cost)| {
                    // Do not move further left from the start of a seed.
                    if is_start_of_seed && start.0 .0 < pos.0 .0 {
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
        // if delta == 0 {
        //     self.h_cache.borrow_mut().insert(pos.0, h);
        // }
        //println!("{:?} {} -> {}", pos, delta, h);
        h
    }
}

impl<'a, DH: DistanceHeuristic> HeuristicInstance<'a> for SeedHeuristicI<'a, DH> {
    fn h(&self, pos: Node<Self::IncrementalState>) -> usize {
        if self.params.make_consistent {
            self.consistent_h(pos)
        } else {
            self.base_h(pos)
        }
    }

    // TODO: Get rid of Option here?
    type IncrementalState = crate::increasing_function::NodeIndex;

    fn incremental_h(
        &self,
        parent: Node<Self::IncrementalState>,
        pos: Pos,
    ) -> Self::IncrementalState {
        if self.params.query_fast {
            self.increasing_function.incremental(
                self.transform(pos),
                parent.1,
                self.transform(parent.0),
            )
        } else {
            parent.1
        }
    }

    fn root_state(&self) -> Self::IncrementalState {
        self.increasing_function.root()
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

        // If the current position is not on a Pareto front, there is no need to
        // rebuild.
        if self.params.build_fast {
            //println!(
            //"{} PRUNE POINT {:?} / {:?}",
            //self.params.build_fast as u8,
            //pos,
            //self.transform(pos)
            //);
            // This doesn't work after all...
            //let tpos = self.transform(pos);
            // Prune the current position.
            self.pruned_positions.insert(pos);
            // NOTE: This still has a small bug/difference with the bruteforce implementation:
            // When two exact matches are neighbours, it can happen that one
            // suffices as parent/root for the region, but the fast implementation doesn't detect this and uses both.
            // This means that the spurious match will be prunes in the fast
            // case, and not in the slow case, leading to small differences.
            // Either way, both behaviours are correct.
            if !self.seed_matches.is_start_of_seed(pos) || !self.active_matches.remove(&pos) {
                return;
            }
        } else {
            //println!("{} PRUNE POINT {:?}", self.params.build_fast as u8, pos);
            //Prune the current position.
            self.pruned_positions.insert(pos);
            if self.h_at_seeds.remove(&pos).is_none() {
                return;
            }
        }
        //println!("REBUILD");
        self.build();
        //println!("{:?}", self.h_at_seeds);
    }
}

#[cfg(test)]
mod tests {

    use itertools::Itertools;

    use super::*;
    use crate::align;
    use crate::setup;

    #[test]
    fn fast_build() {
        // TODO: check pruning
        for (l, max_match_cost) in [(4, 0), (5, 0), (7, 1), (8, 1)] {
            for n in [100, 200, 500, 1000] {
                for e in [0.1, 0.3, 1.0] {
                    for pruning in [false, true] {
                        let h_slow = SeedHeuristic {
                            l,
                            max_match_cost,
                            distance_function: GapHeuristic,
                            pruning,
                            build_fast: false,
                            query_fast: false,
                            make_consistent: true,
                        };
                        let h_fast = SeedHeuristic {
                            l,
                            max_match_cost,
                            distance_function: GapHeuristic,
                            pruning,
                            build_fast: true,
                            query_fast: false,
                            make_consistent: true,
                        };

                        let (a, b, alphabet, stats) = setup(n, e);

                        println!("\n\n\nTESTING n {} e {}: {:?}", n, e, h_fast);
                        if false {
                            let h_slow = h_slow.build(&a, &b, &alphabet);
                            let h_fast = h_fast.build(&a, &b, &alphabet);
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
    }

    #[test]
    fn increasing_function() {
        let tests = [
            (
                "GATCGCAGCAGAACTGTGCCCATTTTGTGCCT",
                "CGGATCGGCGCAGAACATGTGGTCCAATTTTGCTGCC",
            ),
            (
                "GCCTAAATGCGAACGTAGATTCGTTGTTCC",
                "GTGCCTCGCCTAAACGGGAACGTAGTTCGTTGTTC",
            ),
        ];
        for build_fast in [false, true] {
            for (a, b) in tests {
                println!("TEST:\n{}\n{}", a, b);
                let a = a.as_bytes().to_vec();
                let b = b.as_bytes().to_vec();
                let l = 7;
                let max_match_cost = 1;
                let pruning = true;
                let h_slow = SeedHeuristic {
                    l,
                    max_match_cost,
                    distance_function: GapHeuristic,
                    pruning,
                    build_fast,
                    query_fast: build_fast,
                    make_consistent: true,
                };
                let h_fast = SeedHeuristic {
                    l,
                    max_match_cost,
                    distance_function: GapHeuristic,
                    pruning,
                    build_fast,
                    query_fast: build_fast,
                    make_consistent: true,
                };

                let (_, _, alphabet, stats) = setup(0, 0.0);

                if false {
                    let h_slow = h_slow.build(&a, &b, &alphabet);
                    let h_fast = h_fast.build(&a, &b, &alphabet);
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
