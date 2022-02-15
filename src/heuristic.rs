pub mod distance;
pub mod equal_heuristic;
pub mod gap_seed_heuristic;
pub mod print;
pub mod seed_heuristic;

pub use distance::*;
pub use equal_heuristic::*;
pub use gap_seed_heuristic::*;
pub use seed_heuristic::*;

use serde::Serialize;

use crate::{matches::Match, prelude::*};

#[derive(Serialize, Default, Clone)]
pub struct HeuristicParams {
    pub name: String,
    pub distance_function: String,
    pub k: I,
    pub max_match_cost: Cost,
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
        println!("base root potential");
        0
    }

    /// A* will checked for consistency whenever this returns true.
    fn is_start_of_seed(&mut self, _pos: Pos) -> bool {
        true
    }

    /// Returns the offset by which all expanded states in the priority queue can be shifted.
    fn prune(&mut self, pos: Pos, _hint: Self::Hint) -> Cost {
        0
    }

    /// Tells the heuristic that the position was explored, so it knows which
    /// positions need to be updated when propagating the pruning to the
    /// priority queue.
    fn explore(&mut self, _pos: Pos) {}

    fn stats(&self) -> HeuristicStats {
        Default::default()
    }

    fn print(&self, _do_transform: bool, _wait_for_user: bool) {}
}
