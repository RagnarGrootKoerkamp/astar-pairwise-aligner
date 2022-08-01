pub mod bruteforce_csh;
pub mod chained_seed;
pub mod distance;
pub mod equal;
pub mod max;
pub mod mirror;
pub mod perfect;
pub mod seed;
pub mod symmetric;

pub use bruteforce_csh::*;
pub use chained_seed::*;
pub use distance::*;
pub use equal::*;
pub use max::*;
pub use mirror::*;
pub use perfect::*;

pub use seed::*;
pub use symmetric::*;

use crate::{matches::Match, prelude::*};

#[derive(Default, Clone)]
pub struct HeuristicParams {
    pub name: String,
    pub distance_function: String,
    pub k: I,
    pub max_match_cost: MatchCost,
    pub pruning: Pruning,
    pub build_fast: bool,
}

#[derive(Clone)]
pub struct HeuristicStats {
    pub num_seeds: I,
    pub num_matches: usize,
    pub num_filtered_matches: usize,
    pub pruning_duration: f32,
    pub num_prunes: usize,
}

impl Default for HeuristicStats {
    fn default() -> Self {
        Self {
            num_seeds: 0,
            num_matches: 0,
            num_filtered_matches: 0,
            pruning_duration: 0.,
            num_prunes: 0,
        }
    }
}

/// An object containing the settings for a heuristic.
pub trait Heuristic: std::fmt::Debug + Copy {
    type Instance<'a>: HeuristicInstance<'a>;
    const IS_DEFAULT: bool = false;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>, alphabet: &Alphabet) -> Self::Instance<'a>;

    // Heuristic properties.
    fn name(&self) -> String;

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub struct DisplayOptions {}

/// An instantiation of a heuristic for a specific pair of sequences.
pub trait HeuristicInstance<'a> {
    fn h(&self, pos: Pos) -> Cost;

    /// The internal contour value at the given position, if available.
    fn layer(&self, _pos: Pos) -> Option<Cost> {
        None
    }

    /// The internal contour value at the given position, if available.
    fn layer_with_hint(&self, _pos: Pos, _hint: Self::Hint) -> Option<(Cost, Self::Hint)> {
        None
    }

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
        false
    }

    /// Returns the offset by which all expanded states in the priority queue can be shifted.
    ///
    /// `seed_cost`: The cost made in the seed ending at pos.
    fn prune(&mut self, _pos: Pos, _hint: Self::Hint) -> Cost {
        0
    }

    /// Tells the heuristic that the position was explored, so it knows which
    /// positions need to be updated when propagating the pruning to the
    /// priority queue.
    fn explore(&mut self, _pos: Pos) {}

    fn stats(&self) -> HeuristicStats {
        Default::default()
    }

    fn matches(&self) -> Option<Vec<Match>> {
        None
    }

    fn seeds(&self) -> Option<&Vec<Seed>> {
        None
    }

    fn params_string(&self) -> String {
        "".into()
    }
}
