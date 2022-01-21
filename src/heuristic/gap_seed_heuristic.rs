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

impl<C: 'static + Contours> GapSeedHeuristic<C> {
    pub fn to_seed_heuristic(&self) -> SeedHeuristic<GapCost> {
        SeedHeuristic {
            match_config: self.match_config,
            distance_function: GapCost,
            pruning: self.pruning,
            prune_fraction: self.prune_fraction,
        }
    }

    pub fn equal_to_seed_heuristic(&self) -> EqualHeuristic<SeedHeuristic<GapCost>, Self> {
        EqualHeuristic {
            h1: self.to_seed_heuristic(),
            h2: *self,
        }
    }

    pub fn equal_to_bruteforce_contours(
        &self,
    ) -> EqualHeuristic<GapSeedHeuristic<BruteForceContours>, Self> {
        EqualHeuristic {
            h1: GapSeedHeuristic {
                match_config: self.match_config,
                pruning: self.pruning,
                prune_fraction: self.prune_fraction,
                c: Default::default(),
            },
            h2: *self,
        }
    }
    pub fn equal_to_naive_brutefore_contour(
        &self,
    ) -> EqualHeuristic<GapSeedHeuristic<NaiveContours<BruteForceContour>>, Self> {
        EqualHeuristic {
            h1: GapSeedHeuristic {
                match_config: self.match_config,
                pruning: self.pruning,
                prune_fraction: self.prune_fraction,
                c: Default::default(),
            },
            h2: *self,
        }
    }
}

// Manual implementations because C is not Debug, Clone, or Copy.
impl<C: 'static + Contours> std::fmt::Debug for GapSeedHeuristic<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GapSeedHeuristic")
            .field("match_config", &self.match_config)
            .field("pruning", &self.pruning)
            .field("prune_fraction", &self.prune_fraction)
            .field("contours", &std::any::type_name::<C>())
            .finish()
    }
}
impl<C: Contours> Clone for GapSeedHeuristic<C> {
    fn clone(&self) -> Self {
        Self {
            match_config: self.match_config,
            pruning: self.pruning,
            prune_fraction: self.prune_fraction,
            c: self.c,
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

impl<C: 'static + Contours> Heuristic for GapSeedHeuristic<C> {
    type Instance<'a> = GapSeedHeuristicI<C>;

    fn build<'a>(&self, a: &'a Sequence, b: &'a Sequence, alph: &Alphabet) -> Self::Instance<'a> {
        // TODO: Warning
        assert!(
            self.match_config.max_match_cost
                <= self.match_config.length.l().unwrap_or(I::MAX) as Cost / 3
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

    // For partial pruning.
    // TODO: Put statistics into a separate struct.
    num_tried_pruned: usize,
    num_actual_pruned: usize,

    // For the fast version
    transform_target: Pos,
    //contour_graph: ContourGraph<usize>,
    contours: C,

    // For debugging
    pub pruning_duration: Duration,
}

/// The seed heuristic implies a distance function as the maximum of the
/// provided distance function and the potential difference between the two
/// positions.  Assumes that the current position is not a match, and no matches
/// are visited in between `from` and `to`.
impl<'a, C: Contours> DistanceInstance<'a> for GapSeedHeuristicI<C> {
    fn distance(&self, from: Self::Pos, to: Self::Pos) -> Cost {
        max(
            self.gap_distance.distance(from, to),
            self.seed_matches.distance(from, to),
        )
    }
}

// TODO: Get rid of this.
impl<C: Contours> Drop for GapSeedHeuristicI<C> {
    fn drop(&mut self) {
        self.contours.print_stats();
    }
}

impl<C: Contours> GapSeedHeuristicI<C> {
    fn new(a: &Sequence, b: &Sequence, alph: &Alphabet, params: GapSeedHeuristic<C>) -> Self {
        let seed_matches = find_matches(a, b, alph, params.match_config);

        let mut h = GapSeedHeuristicI {
            params,
            gap_distance: Distance::build(&GapCost, a, b, alph),
            target: Pos::from_length(a, b),
            seed_matches,
            transform_target: Pos(0, 0),

            // Filled below.
            contours: C::default(),
            pruning_duration: Default::default(),
            num_tried_pruned: 0,
            num_actual_pruned: 0,
        };
        h.transform_target = h.transform(h.target);
        h.build();
        h.print(false, false);
        h.contours.print_stats();
        h
    }

    // TODO: Report some metrics on skipped states.
    fn build(&mut self) {
        // Filter matches by transformed start position.
        let filtered_matches = self
            .seed_matches
            .iter()
            .filter(|Match { end, .. }| self.transform(*end) <= self.transform_target)
            .collect_vec();
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
        // TODO: Fix the units here -- unclear whether it should be I or cost.
        self.contours = C::new(arrows, self.params.match_config.max_match_cost as I + 1);
        //println!("{:?}", self.contours);
    }

    // TODO: Transform maps from position domain into cost domain.
    // Contours should take a template for the type of point they deal with.
    fn transform(&self, pos @ Pos(i, j): Pos) -> Pos {
        let a = self.target.0;
        let b = self.target.1;
        let pot = |pos| self.seed_matches.potential(pos);
        Pos(
            // This is a lie. All should be converted to cost, instead of position really.
            i + b - j + pot(Pos(0, 0)) as I - pot(pos) as I,
            j + a - i + pot(Pos(0, 0)) as I - pot(pos) as I,
        )
    }
}

impl<'a, C: Contours> HeuristicInstance<'a> for GapSeedHeuristicI<C> {
    type Pos = crate::graph::Pos;

    fn h(&self, pos: Self::Pos) -> Cost {
        let p = self.seed_matches.potential(pos);
        let val = self.contours.value(self.transform(pos));
        if val == 0 {
            self.distance(pos, self.target)
        } else {
            p - val
        }
    }

    type Hint = C::Hint;
    fn h_with_hint(&self, pos: Self::Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        let p = self.seed_matches.potential(pos);
        let (val, new_hint) = self.contours.value_with_hint(self.transform(pos), hint);
        if val == 0 {
            (self.distance(pos, self.target), new_hint)
        } else {
            (p - val, new_hint)
        }
    }

    fn is_start_of_seed(&mut self, pos: Self::Pos) -> bool {
        self.seed_matches.is_start_of_seed(pos)
    }

    fn prune(&mut self, pos: Pos) {
        self.prune_with_hint(pos, Self::Hint::default())
    }

    fn prune_with_hint(&mut self, pos: Self::Pos, hint: Self::Hint) {
        if !self.params.pruning {
            return;
        }

        // Make sure that h remains consistent, by never pruning if it would make the new value >1 larger than it's neighbours above/below.
        {
            // Compute the new value. Can be linear time loop since we are going to rebuild anyway.
            // TODO: Cur_val could be passed in from the parent instead.
            // TODO: Should we be looking at h or contours.value_with_hint here?
            let (cur_val, hint) = self.h_with_hint(pos, hint);
            if pos.1 > 0 {
                let nb_val = self.h_with_hint(Pos(pos.0, pos.1 - 1), hint).0;
                // FIXME: Re-enable this assertion.
                //assert!(cur_val + 1 >= nb_val, "cur {} nb {}", cur_val, nb_val);
                if cur_val > nb_val {
                    return;
                }
            }
            if pos.1 < self.target.1 {
                let nb_val = self.h_with_hint(Pos(pos.0, pos.1 + 1), hint).0;
                // FIXME: Re-enable this assertion.
                //assert!(cur_val + 1 >= nb_val, "cur {} nb {}", cur_val, nb_val);
                if cur_val > nb_val {
                    return;
                }
            }
        }

        self.num_tried_pruned += 1;
        if self.num_actual_pruned as f32
            >= self.num_tried_pruned as f32 * self.params.prune_fraction
        {
            return;
        }
        self.num_actual_pruned += 1;

        if print() {
            println!("PRUNE INCREMENT {} / {}", pos, self.transform(pos));
            self.print(false, false);
        }
        let start = time::Instant::now();
        self.contours.prune_with_hint(self.transform(pos), hint);
        self.pruning_duration += start.elapsed();
    }

    fn stats(&self) -> HeuristicStats {
        HeuristicStats {
            num_seeds: Some(self.seed_matches.num_seeds),
            num_matches: Some(self.seed_matches.matches.len()),
            matches: if DEBUG {
                Some(self.seed_matches.matches.clone())
            } else {
                None
            },
            pruning_duration: Some(self.pruning_duration.as_secs_f32()),
        }
    }

    // TODO: Unify this with the base print function.
    fn print(&self, do_transform: bool, wait_for_user: bool) {
        if !print() {
            return;
        }
        let l = self.params.match_config.length.l().unwrap();
        let max_match_cost = self.params.match_config.max_match_cost as I;
        let reset = termion::color::Rgb(230, 230, 230);
        let mut ps = HashMap::default();
        // ps.insert(1, termion::color::Rgb(255, 0, 0));
        ps.insert(1, termion::color::Rgb(255, 255, 200));
        ps.insert(0, termion::color::Rgb(2, 255, 210));
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(31414);
        let dist = rand::distributions::Uniform::new_inclusive(0u8, 255u8);
        let Pos(a, b) = self.target;
        let mut pixels = vec![vec![(None, None, false, false); 20 * b as usize]; 20 * a as usize];
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
                let is_start_of_match = self.seed_matches.iter().any(|m| m.start == p);
                let is_end_of_match = self.seed_matches.iter().any(|m| m.end == p);
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
