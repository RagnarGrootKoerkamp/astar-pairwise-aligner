use std::{
    cmp::Reverse,
    io,
    marker::PhantomData,
    time::{self, Duration},
};

use itertools::Itertools;
use rand::{prelude::Distribution, SeedableRng};

use super::{distance::*, *};
use crate::{
    contour::{Arrow, Contours},
    prelude::*,
    seeds::{find_matches, Match, MatchConfig, SeedMatches},
};

pub struct GapSeedHeuristic<C: Contours> {
    pub match_config: MatchConfig,
    pub pruning: bool,
    pub prune_fraction: f32,
    pub c: PhantomData<C>,
}

impl<C: Contours> GapSeedHeuristic<C> {
    pub fn as_seed_heuristic(&self) -> SeedHeuristic<GapCost> {
        SeedHeuristic {
            match_config: self.match_config,
            distance_function: GapCost,
            pruning: self.pruning,
            prune_fraction: self.prune_fraction,
        }
    }
    pub fn as_bruteforce_contours(&self) -> GapSeedHeuristic<BruteForceContours> {
        GapSeedHeuristic {
            match_config: self.match_config,
            pruning: self.pruning,
            prune_fraction: self.prune_fraction,
            c: Default::default(),
        }
    }
    pub fn as_naive_brutefore_contour(&self) -> GapSeedHeuristic<NaiveContours<BruteForceContour>> {
        GapSeedHeuristic {
            match_config: self.match_config,
            pruning: self.pruning,
            prune_fraction: self.prune_fraction,
            c: Default::default(),
        }
    }
}

// Manual implementations because C is not Debug, Clone, or Copy.
impl<C: Contours> std::fmt::Debug for GapSeedHeuristic<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GapSeedHeuristic")
            .field("match_config", &self.match_config)
            .field("pruning", &self.pruning)
            .field("prune_fraction", &self.prune_fraction)
            .finish()
    }
}
impl<C: Contours> Clone for GapSeedHeuristic<C> {
    fn clone(&self) -> Self {
        Self {
            match_config: self.match_config.clone(),
            pruning: self.pruning.clone(),
            prune_fraction: self.prune_fraction.clone(),
            c: self.c.clone(),
        }
    }
}
impl<C: Contours> Copy for GapSeedHeuristic<C> {}

impl<C: Contours> Default for GapSeedHeuristic<C> {
    fn default() -> Self {
        Self {
            match_config: Default::default(),
            pruning: false,
            prune_fraction: 1.0,
            c: PhantomData,
        }
    }
}

impl<C: Contours> Heuristic for GapSeedHeuristic<C> {
    type Instance<'a> = GapSeedHeuristicI<C>;

    fn build<'a>(&self, a: &'a Sequence, b: &'a Sequence, alph: &Alphabet) -> Self::Instance<'a> {
        assert!(
            self.match_config.max_match_cost
                <= self.match_config.length.l().unwrap_or(usize::MAX) / 3
        );
        GapSeedHeuristicI::new(a, b, alph, *self)
    }

    fn name(&self) -> String {
        "GapSeed".into()
    }

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            l: Some(self.match_config.length.l().unwrap_or(0)),
            max_match_cost: Some(self.match_config.max_match_cost),
            pruning: Some(self.pruning),
            distance_function: Some("Gap".to_string()),
            ..Default::default()
        }
    }
}

pub struct GapSeedHeuristicI<C: Contours> {
    params: GapSeedHeuristic<C>,
    gap_distance: GapCostI,
    target: Pos,

    pub seed_matches: SeedMatches,
    // The lowest cost match starting at each position.
    //active_matches: HashMap<Pos, Match>,
    pruned_positions: HashSet<Pos>,

    // For partial pruning.
    num_tried_pruned: usize,
    num_actual_pruned: usize,

    // For the fast version
    transform_target: Pos,
    //contour_graph: ContourGraph<usize>,
    contours: C,

    // For debugging
    expanded: HashSet<Pos>,
    pub pruning_duration: Duration,
}

/// The seed heuristic implies a distance function as the maximum of the
/// provided distance function and the potential difference between the two
/// positions.  Assumes that the current position is not a match, and no matches
/// are visited in between `from` and `to`.
impl<'a, C: Contours> DistanceInstance<'a> for GapSeedHeuristicI<C> {
    fn distance(&self, from: Self::Pos, to: Self::Pos) -> usize {
        max(
            self.gap_distance.distance(from, to),
            self.seed_matches.distance(from, to),
        )
    }
}

impl<'a, C: Contours> Drop for GapSeedHeuristicI<C> {
    fn drop(&mut self) {
        self.contours.print_stats();
    }
}

impl<'a, C: Contours> GapSeedHeuristicI<C> {
    fn new(a: &'a Sequence, b: &'a Sequence, alph: &Alphabet, params: GapSeedHeuristic<C>) -> Self {
        let seed_matches = find_matches(a, b, alph, params.match_config);

        let mut h = GapSeedHeuristicI {
            params,
            gap_distance: Distance::build(&GapCost, a, b, alph),
            target: Pos(a.len(), b.len()),
            seed_matches,
            //active_matches: Default::default(),
            pruned_positions: Default::default(),
            transform_target: Pos(0, 0),
            // Filled below.
            contours: C::default(),
            expanded: HashSet::default(),
            pruning_duration: Default::default(),
            num_tried_pruned: 0,
            num_actual_pruned: 0,
        };
        h.transform_target = h.transform(h.target);
        h.build();
        //h.print(true, false);
        h.contours.print_stats();
        h
    }

    // TODO: Report some metrics on skipped states.
    fn build(&mut self) {
        // Filter matches by transformed start position.
        let filtered_matches = self
            .seed_matches
            .iter()
            .filter(|Match { start, end, .. }| {
                self.transform(*end) <= self.transform_target
                    && !self.pruned_positions.contains(start)
            })
            .collect_vec();
        // Update active_matches.
        // for &m in &filtered_matches {
        //     match self.active_matches.entry(m.start) {
        //         std::collections::hash_map::Entry::Occupied(mut entry) => {
        //             if m.match_cost < entry.get().match_cost {
        //                 entry.insert(m.clone());
        //             }
        //         }
        //         std::collections::hash_map::Entry::Vacant(entry) => {
        //             entry.insert(m.clone());
        //         }
        //     }
        // }
        // Transform to Arrows.
        let mut arrows = filtered_matches
            .into_iter()
            .map(
                |&Match {
                     start,
                     end,
                     match_cost,
                     seed_potential,
                 }| {
                    Arrow {
                        start: self.transform(start),
                        end: self.transform(end),
                        len: seed_potential - match_cost,
                    }
                },
            )
            .collect_vec();
        //println!("{:?}", self.seed_matches.matches);
        //println!("{:?}", arrows);
        // Sort revered by start.
        arrows.sort_by_key(|Arrow { start, .. }| Reverse(LexPos(*start)));
        self.contours = C::new(arrows, self.params.match_config.max_match_cost + 1);
        //println!("{:?}", self.contours);
    }

    fn transform(&self, pos @ Pos(i, j): Pos) -> Pos {
        let a = self.target.0;
        let b = self.target.1;
        let pot = |pos| self.seed_matches.potential(pos);
        Pos(
            i + b - j + pot(Pos(0, 0)) - pot(pos),
            j + a - i + pot(Pos(0, 0)) - pot(pos),
        )
    }
}

impl<'a, C: Contours> HeuristicInstance<'a> for GapSeedHeuristicI<C> {
    type Pos = crate::graph::Pos;

    fn h(&self, Node(pos, ()): NodeH<'a, Self>) -> usize {
        let p = self.seed_matches.potential(pos);
        let val = self.contours.value(self.transform(pos));
        if val == 0 {
            self.distance(pos, self.target)
        } else {
            p - val
        }
    }

    // TODO: Move the pruning code to Contours.
    // NOTE: This still has a small bug/difference with the bruteforce implementation:
    // When two exact matches are neighbours, it can happen that one
    // suffices as parent/root for the region, but the fast implementation doesn't detect this and uses both.
    // This means that the spurious match will be prunes in the fast
    // case, and not in the slow case, leading to small differences.
    // Either way, both behaviours are correct.
    fn prune(&mut self, pos: Pos) {
        if !self.params.pruning {
            return;
        }

        // Check that we don't double expand start-of-seed states.
        if self.seed_matches.is_start_of_seed(pos) {
            // Starts of seeds should still only be expanded once.
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

        // let _m = if let Some(m) = self.active_matches.get(&pos) {
        //     m
        // } else {
        //     return;
        // };

        // Skip pruning when this is an inexact match neighbouring a strictly better still active exact match.
        // TODO: This feels hacky doing the manual position manipulation, but oh well... .
        /*
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
        */

        // self.active_matches
        //     .remove(&pos)
        //     .expect("Already checked that this positions is a match.");

        // Prune the current position.
        self.pruned_positions.insert(pos);
        if !self.seed_matches.is_start_of_seed(pos) {
            return;
        }

        let start = time::Instant::now();
        //println!("PRUNE INCREMENT {} / {}", pos, self.transform(pos));
        self.contours.prune(self.transform(pos));
        self.pruning_duration += start.elapsed();
        //self.print(false, false);
    }

    fn stats(&self) -> HeuristicStats {
        HeuristicStats {
            num_seeds: Some(self.seed_matches.num_seeds),
            num_matches: Some(self.seed_matches.matches.len()),
            matches: Some(self.seed_matches.matches.clone()),
            pruning_duration: Some(self.pruning_duration.as_secs_f32()),
        }
    }

    // TODO: Unify this with the base print function.
    fn print(&self, do_transform: bool, wait_for_user: bool) {
        let l = self.params.match_config.length.l().unwrap();
        let max_match_cost = self.params.match_config.max_match_cost;
        let mut ps = HashMap::default();
        // ps.insert(1, termion::color::Rgb(255, 0, 0));
        // ps.insert(2, termion::color::Rgb(0, 255, 0));
        // ps.insert(0, termion::color::Rgb(0, 0, 255));
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(31413);
        let dist = rand::distributions::Uniform::new_inclusive(0u8, 255u8);
        let Pos(a, b) = self.target;
        let mut pixels = vec![vec![(None, None, false, false); 20 * b]; 20 * a];
        let start_i = Pos(
            b * (max_match_cost + 1) / (l + max_match_cost + 1) + a - b,
            0,
        );
        let start_j = Pos(0, a * (max_match_cost + 1) / l + b - a);
        let start = Pos(self.transform(start_j).0, self.transform(start_i).1);
        let target = self.transform(Pos(a, b));
        for i in 0..=a {
            for j in 0..=b {
                let p = Pos(i, j);
                // Transformation: draw (i,j) at ((l+1)*i + l*(B-j), l*j + (A-i)*(l-1))
                // scaling: divide draw coordinate by l, using the right offset.
                let draw_pos = if do_transform { self.transform(p) } else { p };
                let pixel = &mut pixels[draw_pos.0][draw_pos.1];

                let layer = self.contours.value(self.transform(p));
                let (_val, _parent_pos) = self.h_with_parent(Node(p, ()));
                let color = ps.entry(layer).or_insert(termion::color::Rgb(
                    dist.sample(&mut rng),
                    dist.sample(&mut rng),
                    dist.sample(&mut rng),
                ));
                let is_start_of_match = self.seed_matches.iter().find(|m| m.start == p).is_some();
                let is_end_of_match = self.seed_matches.iter().find(|m| m.end == p).is_some();
                if is_start_of_match {
                    pixel.2 = true;
                } else if is_end_of_match {
                    pixel.3 = true;
                }
                pixel.0 = Some(*color);
                pixel.1 = Some(layer);
            }
        }
        let print = |i: usize, j: usize| {
            let pixel = pixels[i][j];
            if pixel.2 {
                print!(
                    "{}{}",
                    termion::color::Fg(termion::color::Black),
                    termion::style::Bold
                );
            } else if pixel.3 {
                print!(
                    "{}{}",
                    termion::color::Fg(termion::color::Rgb(100, 100, 100)),
                    termion::style::Bold
                );
            }
            print!(
                "{}{:3} ",
                termion::color::Bg(pixel.0.unwrap_or(termion::color::Rgb(0, 0, 0))),
                pixel.1.map(|x| format!("{:3}", x)).unwrap_or_default()
            );
            print!(
                "{}{}",
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset)
            );
        };
        if do_transform {
            for j in start.1..=target.1 {
                for i in start.0..=target.0 {
                    print(i, j);
                }
                print!(
                    "{}{}\n",
                    termion::color::Fg(termion::color::Reset),
                    termion::color::Bg(termion::color::Reset)
                );
            }
        } else {
            for j in 0 * b..=1 * b {
                for i in 0 * a..=1 * a {
                    print(i, j);
                }
                print!(
                    "{}{}\n",
                    termion::color::Fg(termion::color::Reset),
                    termion::color::Bg(termion::color::Reset)
                );
            }
        };
        if wait_for_user {
            let mut ret = String::new();
            io::stdin().read_line(&mut ret).unwrap();
        }
    }
}
