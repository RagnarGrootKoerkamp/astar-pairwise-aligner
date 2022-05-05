use super::{distance::*, *};
use crate::{
    contour::{Arrow, Contours},
    matches::{find_matches, Match, MatchConfig, Seeds},
    prelude::*,
};
use itertools::Itertools;
use rand::{prelude::Distribution, SeedableRng};
use std::{
    io,
    marker::PhantomData,
    time::{self, Duration},
};

pub struct CSH<C: Contours> {
    pub match_config: MatchConfig,
    pub pruning: bool,
    // When false, gaps are free and only the max chain of matches is found.
    pub use_gap_cost: bool,
    pub c: PhantomData<C>,
}

impl<C: 'static + Contours> CSH<C> {
    pub fn to_seed_heuristic(&self) -> BruteForceCSH<GapCost> {
        assert!(self.use_gap_cost);
        BruteForceCSH {
            match_config: self.match_config,
            distance_function: GapCost,
            pruning: self.pruning,
        }
    }

    pub fn to_zero_cost_seed_heuristic(&self) -> BruteForceCSH<ZeroCost> {
        assert!(!self.use_gap_cost);
        BruteForceCSH {
            match_config: self.match_config,
            distance_function: ZeroCost,
            pruning: self.pruning,
        }
    }

    pub fn equal_to_seed_heuristic(&self) -> EqualHeuristic<BruteForceCSH<GapCost>, Self> {
        EqualHeuristic {
            h1: self.to_seed_heuristic(),
            h2: *self,
        }
    }

    pub fn equal_to_zero_cost_seed_heuristic(
        &self,
    ) -> EqualHeuristic<BruteForceCSH<ZeroCost>, Self> {
        EqualHeuristic {
            h1: self.to_zero_cost_seed_heuristic(),
            h2: *self,
        }
    }

    pub fn equal_to_bruteforce_contours(&self) -> EqualHeuristic<CSH<BruteForceContours>, Self> {
        EqualHeuristic {
            h1: CSH {
                match_config: self.match_config,
                pruning: self.pruning,
                use_gap_cost: self.use_gap_cost,
                c: Default::default(),
            },
            h2: *self,
        }
    }
}

// Manual implementations because C is not Debug, Clone, or Copy.
impl<C: 'static + Contours> std::fmt::Debug for CSH<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainedSeedsHeuristic")
            .field("match_config", &self.match_config)
            .field("pruning", &self.pruning)
            .field("contours", &std::any::type_name::<C>())
            .finish()
    }
}
impl<C: Contours> Clone for CSH<C> {
    fn clone(&self) -> Self {
        Self {
            match_config: self.match_config,
            pruning: self.pruning,
            use_gap_cost: self.use_gap_cost,
            c: self.c,
        }
    }
}
impl<C: Contours> Copy for CSH<C> {}

impl<C: 'static + Contours> Heuristic for CSH<C> {
    type Instance<'a> = CSHI<C>;

    fn build<'a>(&self, a: &'a Sequence, b: &'a Sequence, alph: &Alphabet) -> Self::Instance<'a> {
        // TODO: Warning
        assert!(
            self.match_config.max_match_cost
                <= self.match_config.length.k().unwrap_or(I::MAX) as MatchCost / 3
        );
        CSHI::new(a, b, alph, *self)
    }

    fn name(&self) -> String {
        "GapSeed".into()
    }

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            k: self.match_config.length.k().unwrap_or(0),
            max_match_cost: self.match_config.max_match_cost,
            pruning: self.pruning,
            distance_function: (if self.use_gap_cost { "Gap" } else { "Zero" }).to_string(),
            ..Default::default()
        }
    }
}

pub struct CSHI<C: Contours> {
    params: CSH<C>,
    gap_distance: GapCostI,
    target: Pos,

    seeds: Seeds,
    num_matches: usize,
    num_filtered_matches: usize,

    // TODO: Put statistics into a separate struct.
    num_pruned: usize,

    /// The max transformed position.
    max_transformed_pos: Pos,
    transform_target: Pos,
    contours: C,

    // For debugging
    pruning_duration: Duration,
    // TODO: Do not use vectors inside a hashmap.
    // TODO: Instead, store a Vec<Array>, and attach a slice to each contour point.
    arrows: HashMap<Pos, Vec<Arrow>>,
}

/// The seed heuristic implies a distance function as the maximum of the
/// provided distance function and the potential difference between the two
/// positions.  Assumes that the current position is not a match, and no matches
/// are visited in between `from` and `to`.
impl<'a, C: Contours> DistanceInstance<'a> for CSHI<C> {
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        if self.params.use_gap_cost {
            max(
                self.gap_distance.distance(from, to),
                self.seeds.potential_distance(from, to),
            )
        } else {
            self.seeds.potential_distance(from, to)
        }
    }
}

// TODO: Get rid of this.
impl<C: Contours> Drop for CSHI<C> {
    fn drop(&mut self) {
        self.contours.print_stats();
    }
}

impl<C: Contours> CSHI<C> {
    fn new(a: &Sequence, b: &Sequence, alph: &Alphabet, params: CSH<C>) -> Self {
        let matches = find_matches(a, b, alph, params.match_config);
        //println!("\nfind matches.. done: {}", matches.matches.len());
        let mut h = CSHI {
            params,
            gap_distance: Distance::build(&GapCost, a, b, alph),
            target: Pos::from_length(a, b),
            seeds: matches,
            num_matches: 0,
            num_filtered_matches: 0,
            num_pruned: 0,

            // For pruning propagation
            max_transformed_pos: Pos(0, 0),

            // Filled below.
            transform_target: Pos(0, 0),
            contours: C::default(),
            pruning_duration: Default::default(),
            arrows: Default::default(),
        };
        h.transform_target = h.transform(h.target);

        // Filter the matches.
        // NOTE: Matches is already sorted by start.
        assert!(h
            .seeds
            .matches
            .is_sorted_by_key(|Match { start, .. }| LexPos(*start)));
        {
            // Need to take it out of h.seeds because transform also uses this.
            let mut matches = std::mem::take(&mut h.seeds.matches);
            h.num_matches = matches.len();
            matches.retain(|Match { end, .. }| h.transform(*end) <= h.transform_target);
            h.num_filtered_matches = matches.len();
            h.seeds.matches = matches;
        }

        // Transform to Arrows.
        // For arrows with length > 1, also make arrows for length down to 1.
        let match_to_arrow = |m: &Match| Arrow {
            start: h.transform(m.start),
            end: h.transform(m.end),
            len: m.seed_potential - m.match_cost,
        };

        let arrows = h
            .seeds
            .matches
            .iter()
            .map(match_to_arrow)
            .group_by(|a| a.start)
            .into_iter()
            .map(|(start, pos_arrows)| (start, pos_arrows.collect_vec()))
            .collect();

        // Sort revered by start (the order needed to construct contours).
        // TODO: Can we get away without sorting? It's probably possible if seeds
        // TODO: Fix the units here -- unclear whether it should be I or cost.
        h.contours = C::new(
            h.seeds.matches.iter().rev().map(match_to_arrow),
            h.params.match_config.max_match_cost as I + 1,
        );
        h.arrows = arrows;
        h.terminal_print(h.target);
        h.contours.print_stats();
        h
    }

    // TODO: Transform maps from position domain into cost domain.
    // Contours should take a template for the type of point they deal with.
    fn transform(&self, pos @ Pos(i, j): Pos) -> Pos {
        if self.params.use_gap_cost {
            let a = self.target.0;
            let b = self.target.1;
            let pot = |pos| self.seeds.potential(pos);
            Pos(
                // This is a lie. All should be converted to cost, instead of position really.
                i + b - j + pot(Pos(0, 0)) as I - pot(pos) as I,
                j + a - i + pot(Pos(0, 0)) as I - pot(pos) as I,
            )
        } else {
            pos
        }
    }

    // TODO: Unify this with the base print function.
    #[allow(unused)]
    fn print_transformed(&self, do_transform: bool, wait_for_user: bool) {
        if !print() {
            return;
        }
        let k = self.params.match_config.length.k().unwrap();
        let max_match_cost = self.params.match_config.max_match_cost as I;
        let reset = termion::color::Rgb(230, 230, 230);
        let mut ps = HashMap::default();
        // ps.insert(1, termion::color::Rgb(255, 0, 0));
        //ps.insert(1, termion::color::Rgb(255, 255, 200));
        //ps.insert(0, termion::color::Rgb(2, 255, 210));
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(31415);
        let dist = rand::distributions::Uniform::new_inclusive(0u8, 255u8);
        let Pos(a, b) = self.target;
        let mut pixels = vec![vec![(None, None, false, false); 20 * b as usize]; 20 * a as usize];
        let start_i = Pos(
            b * (max_match_cost + 1) / (k + max_match_cost + 1) + a - b,
            0,
        );
        let start_j = Pos(0, a * (max_match_cost + 1) / k + b - a);
        let start = Pos(self.transform(start_j).0, self.transform(start_i).1);
        let target = self.transform(Pos(a, b));
        for i in 0..=a {
            for j in 0..=b {
                let p = Pos(i, j);
                // Transformation: draw (i,j) at ((k+1)*i + k*(B-j), k*j + (A-i)*(k-1))
                // scaling: divide draw coordinate by k, using the right offset.
                let draw_pos = if do_transform { self.transform(p) } else { p };
                let pixel = &mut pixels[draw_pos.0 as usize][draw_pos.1 as usize];

                let layer = self.contours.value(self.transform(p));
                let (_val, _parent_pos) = self.h_with_parent(p);
                let color = ps.entry(layer).or_insert_with(|| {
                    termion::color::Rgb(
                        dist.sample(&mut rng),
                        dist.sample(&mut rng),
                        dist.sample(&mut rng),
                    )
                });
                let is_start_of_match = self.seeds.matches.iter().any(|m| m.start == p);
                let is_end_of_match = self.seeds.matches.iter().any(|m| m.end == p);
                if is_start_of_match {
                    pixel.2 = true;
                } else if is_end_of_match {
                    pixel.3 = true;
                }
                pixel.0 = Some(*color);
                pixel.1 = Some(_val); // _val, layer
            }
        }
        let print = |i: I, j: I| {
            let pixel = pixels[i as usize][j as usize];
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
                termion::color::Bg(pixel.0.unwrap_or(reset)),
                pixel.1.map(|x| format!("{:3}", x)).unwrap_or_default()
            );
            print!("{}{}", termion::color::Fg(reset), termion::color::Bg(reset));
        };
        if do_transform {
            for j in start.1..=target.1 {
                for i in start.0..=target.0 {
                    print(i, j);
                }
                println!("{}{}", termion::color::Fg(reset), termion::color::Bg(reset));
            }
        } else {
            for j in 0..=b {
                for i in 0..=a {
                    print(i, j);
                }
                println!("{}{}", termion::color::Fg(reset), termion::color::Bg(reset));
            }
        };
        print!(
            "{}{}",
            termion::color::Fg(termion::color::Reset),
            termion::color::Bg(termion::color::Reset)
        );
        if wait_for_user {
            let mut ret = String::new();
            io::stdin().read_line(&mut ret).unwrap();
        }
    }
}

impl<'a, C: Contours> HeuristicInstance<'a> for CSHI<C> {
    fn h(&self, pos: Pos) -> Cost {
        let p = self.seeds.potential(pos);
        let val = self.contours.value(self.transform(pos));
        if val == 0 {
            self.distance(pos, self.target)
        } else {
            p - val
        }
    }

    type Hint = C::Hint;
    fn h_with_hint(&self, pos: Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        let p = self.seeds.potential(pos);
        let (val, new_hint) = self.contours.value_with_hint(self.transform(pos), hint);
        if val == 0 {
            (self.distance(pos, self.target), new_hint)
        } else {
            (p - val, new_hint)
        }
    }

    fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        self.seeds.is_seed_start_or_end(pos)
    }

    // TODO: Prune by end pos as well (or instead of) start pos.
    // In this case, `seed_cost` can be used to filter out lookups for states that won't have a match ending there.
    fn prune(&mut self, pos: Pos, hint: Self::Hint, _seed_cost: MatchCost) -> Cost {
        const D: bool = false;
        if !self.params.pruning {
            return 0;
        }

        let start = time::Instant::now();

        let tpos = self.transform(pos);

        // Maximum length arrow at given pos.
        let a = if let Some(arrows) = self.arrows.get(&tpos) {
            arrows.iter().max_by_key(|a| a.len).unwrap().clone()
        } else {
            self.pruning_duration += start.elapsed();
            return 0;
        };

        // Make sure that h remains consistent: never prune positions with larger neighbouring arrows.
        // TODO: Make this smarter and allow pruning long arrows even when pruning short arrows is not possible.
        // The minimum length required for consistency here.
        let mut min_len = 0;
        for d in 1..=self.params.match_config.max_match_cost {
            let mut check = |pos: Pos| {
                if let Some(pos_arrows) = self.arrows.get(&self.transform(pos)) {
                    min_len = max(min_len, pos_arrows.iter().map(|a| a.len).max().unwrap() - d);
                }
            };
            if pos.0 >= d as Cost {
                check(Pos(pos.0, pos.1 - d as I));
            }
            check(Pos(pos.0, pos.1 + d as I));
        }

        if a.len <= min_len {
            return 0;
        }

        if D || print() {
            println!("PRUNE GAP SEED HEURISTIC {pos} to {min_len}: {a}");
        }

        // If there is an exact match here, also prune neighbouring states for which all arrows end in the same position.
        // TODO: Make this more precise for larger inexact matches.
        if PRUNE_INEXACT_MATCHES_BY_END && a.len == self.params.match_config.max_match_cost + 1 {
            // See if there are neighbouring points that can now be fully pruned.
            for d in 1..=self.params.match_config.max_match_cost {
                let mut check = |pos: Pos| {
                    let tp = self.transform(pos);
                    if !self.arrows.contains_key(&tp) {
                        println!("Did not find nb arrow at {tp} while pruning {a} at {pos}");
                    }
                    let arrows = self.arrows.get(&tp).expect("Arrows are not consistent!");
                    if arrows.iter().all(|a2| a2.end == a.end) {
                        self.num_pruned += 1;
                        self.arrows.remove(&tp);
                        self.contours.prune_with_hint(tp, hint, &self.arrows);
                    }
                };
                if pos.0 >= d as Cost {
                    check(Pos(pos.0, pos.1 - d as I));
                }
                check(Pos(pos.0, pos.1 + d as I));
            }
        }

        let change = if min_len == 0 {
            self.arrows.remove(&tpos).unwrap();
            self.contours.prune_with_hint(tpos, hint, &self.arrows).1
        } else {
            // If we only remove a subset of arrows, do no actual pruning.
            // TODO: Also update contours on partial pruning.
            let arrows = self.arrows.get_mut(&tpos).unwrap();
            if D {
                println!("Remove arrows of length > {min_len} at pos {pos}.");
            }
            arrows.drain_filter(|a| a.len > min_len).count();
            assert!(arrows.len() > 0);
            self.contours.prune_with_hint(tpos, hint, &self.arrows).1
        };

        self.pruning_duration += start.elapsed();

        self.num_pruned += 1;
        if print() {
            self.terminal_print(self.target);
        }
        if tpos >= self.max_transformed_pos {
            return change;
        } else {
            return 0;
        }
    }

    /// Update the max_explored_pos, so we know when the priority queue can be shifted after a prune.
    fn explore(&mut self, pos: Pos) {
        let tpos = self.transform(pos);
        if tpos.0 >= self.max_transformed_pos.0 {
            if tpos.0 > self.max_transformed_pos.0 {
                self.max_transformed_pos.0 = tpos.0;
                //self.max_i_pos.clear();
            }
            //self.max_i_pos.push(pos);
        }
        if tpos.1 >= self.max_transformed_pos.1 {
            if tpos.1 > self.max_transformed_pos.1 {
                self.max_transformed_pos.1 = tpos.1;
                //self.max_j_pos.clear();
            }
            //self.max_j_pos.push(pos);
        }
    }

    fn stats(&self) -> HeuristicStats {
        HeuristicStats {
            num_seeds: self.seeds.seeds.len() as I,
            num_matches: self.num_matches,
            num_filtered_matches: self.num_filtered_matches,
            matches: if DEBUG {
                self.seeds.matches.clone()
            } else {
                Default::default()
            },
            pruning_duration: self.pruning_duration.as_secs_f32(),
            num_prunes: self.num_pruned,
        }
    }

    fn root_potential(&self) -> Cost {
        self.seeds.potential(Pos(0, 0))
    }
}
