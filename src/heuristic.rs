pub mod bruteforce_csh;
pub mod chained_seed_heuristic;
pub mod distance;
pub mod equal_heuristic;
pub mod print;
pub mod seed_heuristic;

pub use bruteforce_csh::*;
pub use chained_seed_heuristic::*;
pub use distance::*;
pub use equal_heuristic::*;
use rand::{prelude::Distribution, SeedableRng};
use rand_chacha::ChaCha8Rng;
pub use seed_heuristic::*;

use serde::Serialize;

use crate::{matches::Match, prelude::*};

#[derive(Serialize, Default, Clone)]
pub struct HeuristicParams {
    pub name: String,
    pub distance_function: String,
    pub k: I,
    pub max_match_cost: MatchCost,
    pub pruning: bool,
    pub build_fast: bool,
}

#[derive(Serialize, Clone)]
pub struct HeuristicStats {
    pub num_seeds: I,
    pub num_matches: usize,
    pub num_filtered_matches: usize,
    #[serde(skip_serializing)]
    pub matches: Vec<Match>,
    pub pruning_duration: f32,
    pub num_prunes: usize,
}

impl Default for HeuristicStats {
    fn default() -> Self {
        Self {
            num_seeds: 0,
            num_matches: 0,
            num_filtered_matches: 0,
            matches: Default::default(),
            pruning_duration: 0.,
            num_prunes: 0,
        }
    }
}

/// An object containing the settings for a heuristic.
pub trait Heuristic: std::fmt::Debug + Copy {
    type Instance<'a>: HeuristicInstance<'a>;

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::Instance<'a>;

    // Heuristic properties.
    fn name(&self) -> String;

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            ..Default::default()
        }
    }
}

/// An instantiation of a heuristic for a specific pair of sequences.
pub trait HeuristicInstance<'a> {
    fn h(&self, pos: Pos) -> Cost;

    fn h_with_parent(&self, pos: Pos) -> (Cost, Pos) {
        (self.h(pos), Pos::default())
    }

    type Hint: Copy + Default + std::fmt::Debug = ();
    fn h_with_hint(&self, pos: Pos, _hint: Self::Hint) -> (Cost, Self::Hint) {
        (self.h(pos), Default::default())
    }

    fn root_state(&self, _root_pos: Pos) -> Self::Hint {
        Default::default()
    }

    fn root_potential(&self) -> Cost {
        0
    }

    /// A* will checked for consistency whenever this returns true.
    fn is_seed_start_or_end(&self, _pos: Pos) -> bool {
        true
    }

    /// Returns the offset by which all expanded states in the priority queue can be shifted.
    ///
    /// `seed_cost`: The cost made in the seed ending at pos.
    fn prune(&mut self, _pos: Pos, _hint: Self::Hint, _seed_cost: MatchCost) -> Cost {
        0
    }

    /// Tells the heuristic that the position was explored, so it knows which
    /// positions need to be updated when propagating the pruning to the
    /// priority queue.
    fn explore(&mut self, _pos: Pos) {}

    fn stats(&self) -> HeuristicStats {
        Default::default()
    }

    fn terminal_print(&self, target: Pos) {
        if !crate::config::print() {
            return;
        }

        let mut ps = HashMap::default();
        let mut rng = ChaCha8Rng::seed_from_u64(3144);
        let dist = rand::distributions::Uniform::new_inclusive(0u8, 255u8);
        let Pos(a, b) = target;
        let mut pixels = vec![vec![(None, None, false, false); 20 * b as usize]; 20 * a as usize];
        for i in 0..=a {
            for j in 0..=b {
                let p = Pos(i, j);
                let pixel = &mut pixels[p.0 as usize][p.1 as usize];

                let (val, parent_pos) = self.h_with_parent(p);
                let k = ps.len();
                let (_parent_id, color) = ps.entry(parent_pos).or_insert((
                    k,
                    termion::color::Rgb(
                        dist.sample(&mut rng),
                        dist.sample(&mut rng),
                        dist.sample(&mut rng),
                    ),
                ));
                if self.is_seed_start_or_end(p) {
                    pixel.2 = true;
                }
                pixel.0 = Some(*color);
                pixel.1 = Some(val);
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
                termion::color::Bg(pixel.0.unwrap_or(termion::color::Rgb(0, 0, 0))),
                pixel.1.map(|x| format!("{:3}", x)).unwrap_or_default()
            );
            print!(
                "{}{}",
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset)
            );
        };
        for j in 0..=b {
            for i in 0..=a {
                print(i, j);
            }
            println!(
                "{}{}",
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset)
            );
        }
    }
}
