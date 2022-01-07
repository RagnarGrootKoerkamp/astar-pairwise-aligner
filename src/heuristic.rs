pub mod distance;
pub mod equal_heuristic;
pub mod fast_zero_seed_heuristic;
pub mod pathmax;
pub mod seed_heuristic;

pub use distance::*;
pub use equal_heuristic::*;
pub use fast_zero_seed_heuristic::*;
pub use pathmax::*;
pub use seed_heuristic::*;

use serde::Serialize;

use crate::{seeds::Match, util::*};

#[derive(Serialize, Default)]
pub struct HeuristicParams {
    pub name: String,
    pub distance_function: Option<String>,
    pub l: Option<usize>,
    pub max_match_cost: Option<usize>,
    pub pruning: Option<bool>,
    pub build_fast: Option<bool>,
    pub query_fast: Option<QueryMode>,
}

#[derive(Serialize, Default)]
pub struct HeuristicStats {
    pub num_seeds: Option<usize>,
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

pub type NodeH<'a, H> = crate::graph::Node<
    <H as HeuristicInstance<'a>>::Pos,
    <H as HeuristicInstance<'a>>::IncrementalState,
>;
/// An instantiation of a heuristic for a specific pair of sequences.
pub trait HeuristicInstance<'a> {
    type Pos: Eq + Copy + std::fmt::Debug = crate::graph::Pos;
    type IncrementalState: Eq + Copy + Default + std::fmt::Debug = ();

    fn h(&self, pos: NodeH<'a, Self>) -> usize;

    fn incremental_h(
        &self,
        _parent: NodeH<'a, Self>,
        _pos: Self::Pos,
        _cost: usize,
    ) -> Self::IncrementalState {
        Default::default()
    }
    fn root_state(&self, _root_pos: Self::Pos) -> Self::IncrementalState {
        Default::default()
    }

    fn prune(&mut self, _pos: Self::Pos) {}

    fn stats(&self) -> HeuristicStats {
        Default::default()
    }

    fn print(&self, _do_transform: bool) {}
}
