pub mod distance;
pub mod equal_heuristic;
//pub mod fast_zero_seed_heuristic;
pub mod gap_seed_heuristic;
pub mod print;
pub mod seed_heuristic;

pub use distance::*;
pub use equal_heuristic::*;
//pub use fast_zero_seed_heuristic::*;
pub use gap_seed_heuristic::*;
pub use seed_heuristic::*;

use serde::Serialize;

use crate::{seeds::Match, util::*};

#[derive(Serialize, Default)]
pub struct HeuristicParams {
    pub name: String,
    pub distance_function: Option<String>,
    pub l: Option<I>,
    pub max_match_cost: Option<Cost>,
    pub pruning: Option<bool>,
    pub build_fast: Option<bool>,
}

#[derive(Serialize, Default)]
pub struct HeuristicStats {
    pub num_seeds: Option<I>,
    pub num_matches: Option<usize>,
    #[serde(skip_serializing)]
    pub matches: Option<Vec<Match>>,
    pub pruning_duration: Option<f32>,
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
    type Pos: Eq + Copy + std::fmt::Debug + Default = crate::graph::Pos;

    fn h(&self, pos: Self::Pos) -> Cost;

    fn h_with_parent(&self, pos: Self::Pos) -> (Cost, Self::Pos) {
        (self.h(pos), Self::Pos::default())
    }

    type Hint: Copy + Default + std::fmt::Debug = ();
    fn h_with_hint(&self, pos: Self::Pos, _hint: Self::Hint) -> (Cost, Self::Hint) {
        (self.h(pos), Default::default())
    }

    fn root_state(&self, _root_pos: Self::Pos) -> Self::Hint {
        Default::default()
    }

    fn prune(&mut self, _pos: Self::Pos) {}
    fn prune_with_hint(&mut self, pos: Self::Pos, _hint: Self::Hint) {
        self.prune(pos)
    }

    fn stats(&self) -> HeuristicStats {
        Default::default()
    }

    fn print(&self, _do_transform: bool, _wait_for_user: bool) {}
}
