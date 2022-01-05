use std::{
    cmp::Reverse,
    time::{self, Duration},
};

use itertools::Itertools;

use super::{distance::*, *};
use crate::{
    contour_graph::ContourGraph,
    prelude::*,
    seeds::{find_matches, Match, MatchConfig, SeedMatches},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QueryMode {
    // No fast querying
    Off,
    // Incremental querying
    On,
    // Incremental querying, and check gap to matches starting in neighbours.
    // This is needed when not generating matches with indels at the start/end.
    Nbs,
}

impl QueryMode {
    pub fn enabled(&self) -> bool {
        *self != QueryMode::Off
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SeedHeuristic<DH: DistanceHeuristic> {
    pub match_config: MatchConfig,
    pub distance_function: DH,
    pub pruning: bool,
    pub build_fast: bool,
    pub query_fast: QueryMode,
}
impl<DH: DistanceHeuristic> Heuristic for SeedHeuristic<DH>
where
    for<'a> DH::DistanceInstance<'a>: HeuristicInstance<'a, Pos = Pos>,
{
    type Instance<'a> = SeedHeuristicI<'a, DH>;

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::Instance<'a> {
        assert!(self.match_config.max_match_cost < self.match_config.l);
        SeedHeuristicI::new(a, b, alphabet, *self)
    }

    fn name(&self) -> String {
        "Seed".into()
    }

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            l: Some(self.match_config.l),
            max_match_cost: Some(self.match_config.max_match_cost),
            pruning: Some(self.pruning),
            distance_function: Some(self.distance_function.name()),
            build_fast: Some(self.build_fast),
            query_fast: Some(self.query_fast),
            ..Default::default()
        }
    }
}
pub struct SeedHeuristicI<'a, DH: DistanceHeuristic> {
    params: SeedHeuristic<DH>,
    distance_function: DH::DistanceInstance<'a>,
    target: Pos,

    pub seed_matches: SeedMatches,
    // The lowest cost match starting at each position.
    active_matches: HashMap<Pos, Match>,
    h_at_seeds: HashMap<Pos, usize>,
    pruned_positions: HashSet<Pos>,

    // For the fast version
    transform_target: Pos,
    contour_graph: ContourGraph<usize>,

    // For debugging
    expanded: HashSet<Pos>,
    pub pruning_duration: Duration,
}

/// The seed heuristic implies a distance function as the maximum of the
/// provided distance function and the potential difference between the two
/// positions.  Assumes that the current position is not a match, and no matches
/// are visited in between `from` and `to`.
impl<'a, DH: DistanceHeuristic> DistanceHeuristicInstance<'a> for SeedHeuristicI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceHeuristicInstance<'a, Pos = Pos>,
{
    fn distance(&self, from: Self::Pos, to: Self::Pos) -> usize {
        max(
            self.distance_function.distance(from, to),
            self.seed_matches.distance(from, to),
        )
    }
}

impl<'a, DH: DistanceHeuristic> SeedHeuristicI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceHeuristicInstance<'a, Pos = Pos>,
{
    fn new(
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
        params: SeedHeuristic<DH>,
    ) -> Self {
        let seed_matches = find_matches(a, b, alphabet, params.match_config);

        let distance_function = DistanceHeuristic::build(&params.distance_function, a, b, alphabet);

        if params.build_fast {
            assert!(params.distance_function.name() == "Gap");
        }
        if params.query_fast.enabled() {
            assert!(
                params.build_fast,
                "Query_fast only works when build_fast is enabled"
            );
        }

        let mut h = SeedHeuristicI::<'a> {
            params,
            distance_function,
            target: Pos(a.len(), b.len()),
            seed_matches,
            active_matches: Default::default(),
            h_at_seeds: Default::default(),
            pruned_positions: Default::default(),
            transform_target: Pos(0, 0),
            // Filled below.
            contour_graph: Default::default(),
            expanded: HashSet::default(),
            pruning_duration: Default::default(),
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
        let mut h_at_seeds = HashMap::<Pos, usize>::default();
        h_at_seeds.insert(self.target, 0);
        for Match {
            start,
            end,
            match_cost,
            ..
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

        //println!("Target: {:?} / {:?}", self.target, transform_target);
        //println!("MATCHES: {:?}", self.seed_matches.iter().collect_vec());
        let filtered_matches = self
            .seed_matches
            .iter()
            // Filter matches by transformed start position.
            .filter(|Match { start, .. }| !self.pruned_positions.contains(start))
            .collect_vec();
        //println!("FLT MS : {:?}", filtered_matches);
        for &m in &filtered_matches {
            match self.active_matches.entry(m.start) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    if m.match_cost < entry.get().match_cost {
                        entry.insert(m.clone());
                    }
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(m.clone());
                }
            }
        }
        let transformed_matches = filtered_matches
            .into_iter()
            .map(
                |&Match {
                     start,
                     end,
                     match_cost,
                     max_match_cost,
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
                        max_match_cost,
                    }
                },
            )
            .collect_vec();
        //println!("TRS MS : {:?}", transformed_matches);
        let mut transformed_matches = transformed_matches
            .into_iter()
            .filter(|Match { end, .. }| *end <= transform_target)
            .collect_vec();
        //println!("FILTER: {:?}", transformed_matches);
        transformed_matches
            .sort_by_key(|Match { end, start, .. }| (start.0, start.1, end.0, end.1));
        // for x in &transformed_matches {
        //     println!("Match: {:?}", x);
        // }
        //dbg!(&transformed_matches);
        self.contour_graph =
            ContourGraph::new(transform_target, leftover_at_end, transformed_matches);
        if !self.params.query_fast.enabled() {
            self.h_at_seeds = self.contour_graph.to_map();
        }

        //let mut h_map = self.h_at_seeds.iter().collect_vec();
        //h_map.sort_by_key(|&(Pos(i, j), _)| (i, j));
        //println!("H: {:?}", h_map);
    }

    // The base heuristic function, which is not consistent in the following case:
    // pos A is the start of a seed, and pos B is A+(1,1), with edge cost 0.
    // In this case h(A) = P(A)-P(X) <= d(A,B) + h(B) = 0 + P(B)-P(X) = P(A)-P(X)-1
    // is false.
    fn base_h(&self, Node(pos, parent): NodeH<'a, Self>) -> usize {
        let d = if self.params.query_fast == QueryMode::On {
            let p = self.seed_matches.potential(pos);
            let val = self.contour_graph.val(parent);
            if parent == self.contour_graph.bot() {
                self.distance(pos, self.target)
            } else {
                p - val
            }
        } else if self.params.query_fast == QueryMode::Nbs {
            todo!();
        } else if self.params.build_fast {
            let pos_transformed = self.transform(pos);
            let p = self.seed_matches.potential(pos);
            self.h_at_seeds
                .iter()
                .filter(|&(parent, _)| *parent >= pos_transformed)
                .map(|(_parent, val)| p - *val)
                .min()
                .unwrap_or_else(|| self.distance(pos, self.target))
        } else {
            self.best_distance(pos, self.h_at_seeds.iter())
        };
        //println!("h {:?} -> {:?}", pos, self.base_h_with_parent(pos.0));
        d
    }

    // Used for debugging.
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
                .min_by_key(|&(val, Pos(i, j))| (val, ((i, Reverse(j)))))
                .unwrap_or_else(|| (self.distance(pos, self.target), self.target))
        }
    }
}

impl<'a, DH: DistanceHeuristic> HeuristicInstance<'a> for SeedHeuristicI<'a, DH>
where
    DH::DistanceInstance<'a>: DistanceHeuristicInstance<'a, Pos = Pos>,
{
    type Pos = crate::graph::Pos;
    fn h(&self, pos: NodeH<'a, Self>) -> usize {
        self.base_h(pos)
    }

    // TODO: Get rid of Option here?
    type IncrementalState = crate::contour_graph::NodeIndex;

    // TODO: Use cost for more efficient incremental function.
    fn incremental_h(
        &self,
        parent: NodeH<'a, Self>,
        pos: Pos,
        _cost: usize,
    ) -> Self::IncrementalState {
        if self.params.query_fast.enabled() {
            self.contour_graph
                .incremental(self.transform(pos), parent.1, self.transform(parent.0))
        } else {
            parent.1
        }
    }

    fn root_state(&self, _root_pos: Self::Pos) -> Self::IncrementalState {
        self.contour_graph.max()
    }

    fn stats(&self) -> HeuristicStats {
        HeuristicStats {
            num_seeds: Some(self.seed_matches.num_seeds),
            num_matches: Some(self.seed_matches.matches.len()),
            matches: Some(self.seed_matches.matches.clone()),
            pruning_duration: Some(self.pruning_duration.as_secs_f32()),
        }
    }

    fn prune(&mut self, pos: Pos) {
        if !self.params.pruning {
            return;
        }

        // Check that we don't double expand start-of-seed states.
        if self.seed_matches.is_start_of_seed(pos) {
            // When we don't ensure consistency, starts of seeds should still only be expanded once.
            assert!(
                self.expanded.insert(pos),
                "Double expanded start of seed {:?}",
                pos
            );
        }

        // If the current position is not on a Pareto front, there is no need to
        // rebuild.
        if self.params.build_fast {
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
            if !self.seed_matches.is_start_of_seed(pos) {
                return;
            }

            let m = if let Some(m) = self.active_matches.get(&pos) {
                m
            } else {
                return;
            };

            // Skip pruning when this is an inexact match neighbouring a strictly better still active exact match.
            // TODO: This feels hacky doing the manual position manipulation, but oh well... .
            let nbs = {
                let mut nbs = Vec::new();
                if pos.1 > 0 {
                    nbs.push(Pos(pos.0, pos.1 - 1));
                }
                if pos.1 < self.target.1 {
                    nbs.push(Pos(pos.0, pos.1 + 1));
                }
                nbs
            };
            for nb in nbs {
                if self
                    .active_matches
                    .get(&nb)
                    .map_or(false, |m2| m2.match_cost < m.match_cost)
                {
                    return;
                }
            }

            self.active_matches
                .remove(&pos)
                .expect("Already checked that this positions is a match.");
            // println!(
            //     "{} PRUNE POINT {:?} / {:?}",
            //     self.params.build_fast as u8,
            //     pos,
            //     self.transform(pos)
            // );
        } else {
            //println!("{} PRUNE POINT {:?}", self.params.build_fast as u8, pos);
            //Prune the current position.
            self.pruned_positions.insert(pos);
            if self.h_at_seeds.remove(&pos).is_none() {
                return;
            }
        }
        //println!("REBUILD");
        let start = time::Instant::now();
        self.build();
        self.pruning_duration += start.elapsed();
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
                            match_config: MatchConfig {
                                l,
                                max_match_cost,
                                ..MatchConfig::default()
                            },
                            distance_function: GapHeuristic,
                            pruning,
                            build_fast: false,
                            query_fast: QueryMode::Off,
                        };
                        let h_fast = SeedHeuristic {
                            match_config: MatchConfig {
                                l,
                                max_match_cost,
                                ..MatchConfig::default()
                            },
                            distance_function: GapHeuristic,
                            pruning,
                            build_fast: true,
                            query_fast: QueryMode::Off,
                        };

                        let (a, b, alphabet, stats) = setup(n, e);

                        println!("\n\n\nTESTING n {} e {}: {:?}", n, e, h_fast);
                        if false {
                            let h_slow = h_slow.build(&a, &b, &alphabet);
                            let h_fast = h_fast.build(&a, &b, &alphabet);
                            let mut h_slow_map = h_slow.h_at_seeds.into_iter().collect_vec();
                            let mut h_fast_map = h_fast.h_at_seeds.into_iter().collect_vec();
                            h_slow_map.sort_unstable_by_key(|&(Pos(i, j), _)| (i, j));
                            h_fast_map.sort_unstable_by_key(|&(Pos(i, j), _)| (i, j));
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
            // Fails with alternating [(4,0),(7,1)] seeds on something to do with leftover_at_end.
            ("GAAGGGTAACAGTGCTCG", "AGGGTAACAGTGCTCGTA"),
        ];
        for build_fast in [false, true] {
            for (a, b) in tests {
                println!("TEST:\n{}\n{}", a, b);
                let a = a.as_bytes().to_vec();
                let b = b.as_bytes().to_vec();
                let l = 7;
                let max_match_cost = 1;
                let pruning = false;
                let h_slow = SeedHeuristic {
                    match_config: MatchConfig {
                        l,
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    distance_function: GapHeuristic,
                    pruning,
                    build_fast,
                    query_fast: if build_fast {
                        QueryMode::On
                    } else {
                        QueryMode::Off
                    },
                };
                let h_fast = SeedHeuristic {
                    match_config: MatchConfig {
                        l,
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    distance_function: GapHeuristic,
                    pruning,
                    build_fast,
                    query_fast: if build_fast {
                        QueryMode::On
                    } else {
                        QueryMode::Off
                    },
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

    #[test]
    fn no_leftover() {
        let pruning = true;
        let build_fast = true;
        let (l, max_match_cost) = (7, 1);
        let h_slow = SeedHeuristic {
            match_config: MatchConfig {
                l,
                max_match_cost,
                ..MatchConfig::default()
            },
            distance_function: GapHeuristic,
            pruning,
            build_fast: false,
            query_fast: QueryMode::Off,
        };
        let h_fast = SeedHeuristic {
            match_config: MatchConfig {
                l,
                max_match_cost,
                ..MatchConfig::default()
            },
            distance_function: GapHeuristic,
            pruning,
            build_fast,
            query_fast: QueryMode::Off,
        };

        let n = 1000;
        let e: f32 = 0.3;
        let (a, b, alphabet, stats) = setup(n, e);
        let start = 679;
        let end = 750;
        let a = &a[start..end].to_vec();
        let b = &b[start..end].to_vec();

        println!("\n\n\nALIGN");
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

    #[test]
    fn needs_leftover() {
        let pruning = true;
        let (l, max_match_cost) = (7, 1);
        let build_fast = true;
        let h_slow = SeedHeuristic {
            match_config: MatchConfig {
                l,
                max_match_cost,
                ..MatchConfig::default()
            },
            distance_function: GapHeuristic,
            pruning,
            build_fast: false,
            query_fast: QueryMode::Off,
        };
        let h_fast = SeedHeuristic {
            match_config: MatchConfig {
                l,
                max_match_cost,
                ..MatchConfig::default()
            },
            distance_function: GapHeuristic,
            pruning,
            build_fast,
            query_fast: QueryMode::Off,
        };

        let n = 1000;
        let e: f32 = 0.3;
        let (a, b, alphabet, stats) = setup(n, e);
        let start = 909;
        let end = 989;
        let a = &a[start..end].to_vec();
        let b = &b[start..end].to_vec();
        // let a = &"GAAGGGTAACAGTGCTCG".as_bytes().to_vec();
        // let b = &"AGGGTAACAGTGCTCGTA".as_bytes().to_vec();
        // let (a, b) = (
        //     &"GATCGCAGCAGAACTGTGCCCATTTTGTGCCT".as_bytes().to_vec(),
        //     &"CGGATCGGCGCAGAACATGTGGTCCAATTTTGCTGCC".as_bytes().to_vec(),
        // );
        // let (a, b) = (
        //     &"GCCTAAATGCGAACGTAGATTCGTTGTTCC".as_bytes().to_vec(),
        //     &"GTGCCTCGCCTAAACGGGAACGTAGTTCGTTGTTC".as_bytes().to_vec(),
        // );

        println!("\n\n\nTESTING: {:?}", h_fast);
        println!("{}\n{}", to_string(a), to_string(b));

        println!("\n\n\nALIGN");
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

    #[test]
    fn pruning_and_inexact_matches() {
        let pruning = true;
        let (l, max_match_cost) = (7, 1);
        for do_transform in [false, true] {
            for build_fast in [false, true] {
                let h_slow = SeedHeuristic {
                    match_config: MatchConfig {
                        l,
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    distance_function: GapHeuristic,
                    pruning,
                    build_fast: false,
                    query_fast: QueryMode::Off,
                };
                let h_fast = SeedHeuristic {
                    match_config: MatchConfig {
                        l,
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    distance_function: GapHeuristic,
                    pruning,
                    build_fast,
                    query_fast: QueryMode::Off,
                };

                let n = 1000;
                let e: f32 = 0.3;
                let (a, b, alphabet, stats) = setup(n, e);
                let start = 951;
                let end = 986;
                let a = &a[start..end].to_vec();
                let b = &b[start..end].to_vec();
                // let a = &"GAAGGGTAACAGTGCTCG".as_bytes().to_vec();
                // let b = &"AGGGTAACAGTGCTCGTA".as_bytes().to_vec();
                // let (a, b) = (
                //     &"GATCGCAGCAGAACTGTGCCCATTTTGTGCCT".as_bytes().to_vec(),
                //     &"CGGATCGGCGCAGAACATGTGGTCCAATTTTGCTGCC".as_bytes().to_vec(),
                // );
                // let (a, b) = (
                //     &"GCCTAAATGCGAACGTAGATTCGTTGTTCC".as_bytes().to_vec(),
                //     &"GTGCCTCGCCTAAACGGGAACGTAGTTCGTTGTTC".as_bytes().to_vec(),
                // );

                println!("TESTING: {:?}", h_fast);
                println!("{}\n{}", to_string(a), to_string(b));

                if do_transform {
                    println!("ALIGN");
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
