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
    active_matches: HashMap<Pos, Match>,
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

impl<'a, C: Contours> GapSeedHeuristicI<C> {
    fn new(a: &'a Sequence, b: &'a Sequence, alph: &Alphabet, params: GapSeedHeuristic<C>) -> Self {
        let seed_matches = find_matches(a, b, alph, params.match_config);

        let mut h = GapSeedHeuristicI {
            params,
            gap_distance: Distance::build(&GapCost, a, b, alph),
            target: Pos(a.len(), b.len()),
            seed_matches,
            active_matches: Default::default(),
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
        self.contours = C::new(arrows);
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

    fn stats(&self) -> HeuristicStats {
        HeuristicStats {
            num_seeds: Some(self.seed_matches.num_seeds),
            num_matches: Some(self.seed_matches.matches.len()),
            matches: Some(self.seed_matches.matches.clone()),
            pruning_duration: Some(self.pruning_duration.as_secs_f32()),
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

        // Prune the current position.
        self.pruned_positions.insert(pos);
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

        let start = time::Instant::now();
        self.build();
        self.pruning_duration += start.elapsed();
    }

    fn print(&self, do_transform: bool, wait_for_user: bool) {
        let l = self.params.match_config.length.l().unwrap();
        let max_match_cost = self.params.match_config.max_match_cost;
        let mut ps = HashMap::default();
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(3144);
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

                let (val, parent_pos) = self.h_with_parent(Node(p, ()));
                let l = ps.len();
                let (_parent_id, color) = ps.entry(parent_pos).or_insert((
                    l,
                    termion::color::Rgb(
                        dist.sample(&mut rng),
                        dist.sample(&mut rng),
                        dist.sample(&mut rng),
                    ),
                ));
                let is_start_of_match = self.seed_matches.iter().find(|m| m.start == p).is_some();
                let is_end_of_match = self.seed_matches.iter().find(|m| m.end == p).is_some();
                if is_start_of_match {
                    pixel.2 = true;
                } else if is_end_of_match {
                    pixel.3 = true;
                }
                pixel.0 = Some(*color);
                pixel.1 = Some(val);
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::align;
    use crate::setup;
    use crate::SequenceStats;
    use crate::Source;

    #[allow(unused)]
    fn print<C: Contours>(h: GapSeedHeuristic<C>, a: &Vec<u8>, b: &Vec<u8>, alph: &Alphabet) {
        h.as_seed_heuristic().build(a, b, alph).print(false, false);
        let h = h.build(a, b, alph);
        println!("{:?}", h.contours);
        h.print(false, false);
    }

    #[test]
    fn exact_no_pruning() {
        for l in [4, 5] {
            for n in [40, 100, 200, 500] {
                for e in [0.1, 0.3, 1.0] {
                    let h = GapSeedHeuristic {
                        match_config: MatchConfig {
                            length: Fixed(l),
                            max_match_cost: 0,
                            ..MatchConfig::default()
                        },
                        pruning: false,
                        c: PhantomData::<NaiveContours<NaiveContour>>,
                        ..GapSeedHeuristic::default()
                    };
                    let (a, b, alph, stats) = setup(n, e);
                    println!("TESTING n {} e {}: {:?}", n, e, h);
                    align(
                        &a,
                        &b,
                        &alph,
                        stats,
                        EqualHeuristic {
                            h1: h.as_seed_heuristic(),
                            h2: h,
                        },
                    );
                }
            }
        }
    }

    #[test]
    fn inexact_no_pruning() {
        for l in [6, 7] {
            for n in [40, 100, 200, 500] {
                for e in [0.1, 0.3, 1.0] {
                    let h = GapSeedHeuristic {
                        match_config: MatchConfig {
                            length: Fixed(l),
                            max_match_cost: 1,
                            ..MatchConfig::default()
                        },
                        pruning: false,
                        c: PhantomData::<BruteforceContours>,
                        ..GapSeedHeuristic::default()
                    };
                    let (a, b, alph, stats) = setup(n, e);
                    //print(h, &a, &b, &alph);
                    println!("TESTING n {} e {}: {:?}", n, e, h);
                    align(
                        &a,
                        &b,
                        &alph,
                        stats,
                        EqualHeuristic {
                            h1: h.as_seed_heuristic(),
                            h2: h,
                        },
                    );
                }
            }
        }
    }

    #[test]
    fn exact_pruning() {
        for l in [4, 5] {
            for n in [40, 100, 200, 500] {
                for e in [0.1, 0.3, 1.0] {
                    let h = GapSeedHeuristic {
                        match_config: MatchConfig {
                            length: Fixed(l),
                            max_match_cost: 0,
                            ..MatchConfig::default()
                        },
                        pruning: true,
                        c: PhantomData::<BruteforceContours>,
                        ..GapSeedHeuristic::default()
                    };
                    let (a, b, alph, stats) = setup(n, e);
                    println!("TESTING n {} e {}: {:?}", n, e, h);
                    align(
                        &a,
                        &b,
                        &alph,
                        stats,
                        EqualHeuristic {
                            h1: h.as_seed_heuristic(),
                            h2: h,
                        },
                    );
                }
            }
        }
    }

    #[test]
    fn inexact_pruning() {
        for l in [6, 7] {
            for n in [40, 100, 200, 500] {
                for e in [0.1, 0.3, 1.0] {
                    let h = GapSeedHeuristic {
                        match_config: MatchConfig {
                            length: Fixed(l),
                            max_match_cost: 1,
                            ..MatchConfig::default()
                        },
                        pruning: true,
                        c: PhantomData::<BruteforceContours>,
                        ..GapSeedHeuristic::default()
                    };
                    let (a, b, alph, stats) = setup(n, e);
                    println!("TESTING n {} e {}: {:?}", n, e, h);
                    align(
                        &a,
                        &b,
                        &alph,
                        stats,
                        EqualHeuristic {
                            h1: h.as_seed_heuristic(),
                            h2: h,
                        },
                    );
                }
            }
        }
    }

    #[test]
    fn incremental_pruning_bruteforce() {
        for (l, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
            for n in [40, 100, 200, 500] {
                for e in [0.1, 0.3, 1.0] {
                    let h = GapSeedHeuristic {
                        match_config: MatchConfig {
                            length: Fixed(l),
                            max_match_cost,
                            ..MatchConfig::default()
                        },
                        pruning: true,
                        incremental_pruning: true,
                        c: PhantomData::<BruteforceContours>,
                        ..GapSeedHeuristic::default()
                    };
                    let (a, b, alph, stats) = setup(n, e);
                    println!("TESTING n {} e {}: {:?}", n, e, h);
                    align(
                        &a,
                        &b,
                        &alph,
                        stats,
                        EqualHeuristic {
                            h1: GapSeedHeuristic {
                                incremental_pruning: false,
                                ..h
                            },
                            h2: h,
                        },
                    );
                }
            }
        }
    }

    #[test]
    fn incremental_pruning_naive_naive() {
        for (l, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
            for n in [40, 100, 200, 500, 1000] {
                for e in [0.1, 0.3, 1.0] {
                    let h = GapSeedHeuristic {
                        match_config: MatchConfig {
                            length: Fixed(l),
                            max_match_cost,
                            ..MatchConfig::default()
                        },
                        pruning: true,
                        incremental_pruning: true,
                        c: PhantomData::<NaiveContours<NaiveContour>>,
                        ..GapSeedHeuristic::default()
                    };
                    let (a, b, alph, stats) = setup(n, e);
                    println!("TESTING n {} e {}: {:?}", n, e, h);
                    align(
                        &a,
                        &b,
                        &alph,
                        stats,
                        EqualHeuristic {
                            h1: GapSeedHeuristic {
                                incremental_pruning: false,
                                ..h
                            },
                            h2: h,
                        },
                    );
                }
            }
        }
    }

    #[test]
    fn incremental_pruning_naive_log() {
        for (l, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
            for n in [40, 100, 200, 500, 1000] {
                for e in [0.1, 0.3, 1.0] {
                    let h = GapSeedHeuristic {
                        match_config: MatchConfig {
                            length: Fixed(l),
                            max_match_cost,
                            ..MatchConfig::default()
                        },
                        pruning: true,
                        incremental_pruning: true,
                        c: PhantomData::<NaiveContours<LogContour>>,
                        ..GapSeedHeuristic::default()
                    };
                    let h_base = GapSeedHeuristic {
                        match_config: MatchConfig {
                            length: Fixed(l),
                            max_match_cost,
                            ..MatchConfig::default()
                        },
                        pruning: true,
                        incremental_pruning: true,
                        c: PhantomData::<NaiveContours<NaiveContour>>,
                        ..GapSeedHeuristic::default()
                    };
                    let (a, b, alph, stats) = setup(n, e);
                    println!("TESTING n {} e {}: {:?}", n, e, h);
                    align(&a, &b, &alph, stats, EqualHeuristic { h1: h_base, h2: h });
                }
            }
        }
    }

    #[test]
    fn contour_graph() {
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
                let h_slow = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning,
                    c: PhantomData::<NaiveContours<NaiveContour>>,
                    ..GapSeedHeuristic::default()
                };
                let h_fast = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning,
                    c: PhantomData::<NaiveContours<NaiveContour>>,
                    ..GapSeedHeuristic::default()
                };

                let (_, _, alph, stats) = setup(0, 0.0);

                align(
                    &a,
                    &b,
                    &alph,
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
        let (l, max_match_cost) = (7, 1);
        let h_slow = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(l),
                max_match_cost,
                ..MatchConfig::default()
            },
            pruning,
            c: PhantomData::<NaiveContours<NaiveContour>>,
            ..GapSeedHeuristic::default()
        };
        let h_fast = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(l),
                max_match_cost,
                ..MatchConfig::default()
            },
            pruning,
            c: PhantomData::<NaiveContours<NaiveContour>>,
            ..GapSeedHeuristic::default()
        };

        let n = 1000;
        let e: f32 = 0.3;
        let (a, b, alph, stats) = setup(n, e);
        let start = 679;
        let end = 750;
        let a = &a[start..end].to_vec();
        let b = &b[start..end].to_vec();

        println!("\n\n\nALIGN");
        align(
            &a,
            &b,
            &alph,
            stats,
            EqualHeuristic {
                h1: h_slow,
                h2: h_fast,
            },
        );
    }

    #[test]
    fn needs_leftover() {
        let h_slow = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(7),
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            pruning: false,
            c: PhantomData::<NaiveContours<NaiveContour>>,
            ..GapSeedHeuristic::default()
        };
        let h_fast = GapSeedHeuristic { ..h_slow };

        let n = 1000;
        let e: f32 = 0.3;
        let (a, b, alph, stats) = setup(n, e);
        let start = 909;
        let end = 989;
        let a = &a[start..end].to_vec();
        let b = &b[start..end].to_vec();

        println!("TESTING: {:?}", h_fast);
        println!("{}\n{}", to_string(a), to_string(b));

        println!("ALIGN");
        align(
            &a,
            &b,
            &alph,
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
                let h_slow = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning,
                    c: PhantomData::<NaiveContours<NaiveContour>>,
                    ..GapSeedHeuristic::default()
                };
                let h_fast = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning,
                    c: PhantomData::<NaiveContours<NaiveContour>>,
                    ..GapSeedHeuristic::default()
                };

                let n = 1000;
                let e: f32 = 0.3;
                let (a, b, alph, stats) = setup(n, e);
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
                        &alph,
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

    #[test]
    fn small_test() {
        let alphabet = &Alphabet::new(b"ACTG");

        let _n = 25;
        let _e = 0.2;
        let l = 4;
        let pattern = "AGACGTCC".as_bytes().to_vec();
        let ___text = "AGACGTCCA".as_bytes().to_vec();
        let text = ___text;

        let stats = SequenceStats {
            len_a: pattern.len(),
            len_b: text.len(),
            error_rate: 0.,
            source: Source::Manual,
        };

        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(l),
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            pruning: false,
            c: PhantomData::<NaiveContours<NaiveContour>>,
            ..GapSeedHeuristic::default()
        };
        let r = align(&pattern, &text, &alphabet, stats, h);
        assert!(r.heuristic_stats2.root_h <= r.answer_cost);
    }
}
