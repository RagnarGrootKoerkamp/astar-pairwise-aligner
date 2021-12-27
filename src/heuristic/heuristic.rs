use serde::Serialize;

use crate::{alignment_graph::Node, seeds::Match, util::*};

#[derive(Serialize)]
pub struct HeuristicParams {
    pub heuristic: String,
    pub distance_function: Option<String>,
    pub l: Option<usize>,
    pub max_match_cost: Option<usize>,
    pub pruning: Option<bool>,
    pub consistent: Option<bool>,
    pub build_fast: Option<bool>,
    pub query_fast: Option<bool>,
}

/// An object containing the settings for a heuristic.
pub trait Heuristic: std::fmt::Debug + Copy {
    type Instance<'a>: HeuristicInstance<'a>;

    // Heuristic properties.
    fn name(&self) -> String;
    fn l(&self) -> Option<usize> {
        None
    }
    fn max_match_cost(&self) -> Option<usize> {
        None
    }
    fn pruning(&self) -> Option<bool> {
        None
    }
    fn distance(&self) -> Option<String> {
        None
    }
    fn consistent(&self) -> Option<bool> {
        None
    }
    fn build_fast(&self) -> Option<bool> {
        None
    }
    fn query_fast(&self) -> Option<bool> {
        None
    }

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::Instance<'a>;

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            heuristic: self.name().to_string(),
            distance_function: self.distance().map(|x| x.to_string()),
            l: self.l(),
            max_match_cost: self.max_match_cost(),
            pruning: self.pruning(),
            consistent: self.consistent(),
            build_fast: self.build_fast(),
            query_fast: self.query_fast(),
        }
    }
}

pub type HNode<'a, H> = Node<<H as HeuristicInstance<'a>>::IncrementalState>;

/// An instantiation of a heuristic for a specific pair of sequences.
pub trait HeuristicInstance<'a> {
    fn h(&self, pos: Node<Self::IncrementalState>) -> usize;
    fn prune(&mut self, _pos: Pos) {}

    // TODO: Simplify this, and just use a map inside the heuristic.
    type IncrementalState: std::hash::Hash + Eq + Copy + Default + std::fmt::Debug = ();
    fn incremental_h(&self, _parent: HNode<'a, Self>, _pos: Pos) -> Self::IncrementalState {
        Default::default()
    }
    fn root_state(&self) -> Self::IncrementalState {
        Default::default()
    }

    // Some statistics of the heuristic.
    fn num_seeds(&self) -> Option<usize> {
        None
    }
    fn matches(&self) -> Option<&Vec<Match>> {
        None
    }
    fn num_matches(&self) -> Option<usize> {
        None
    }
}
