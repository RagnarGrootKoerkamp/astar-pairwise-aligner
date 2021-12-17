use serde::Serialize;

use crate::{alignment_graph::Node, seeds::Match, util::*};

#[derive(Serialize)]
pub struct HeuristicParams {
    pub heuristic: String,
    pub distance_function: Option<String>,
    pub l: Option<usize>,
    pub match_distance: Option<usize>,
    pub pruning: Option<bool>,
}

/// An object containing the settings for a heuristic.
pub trait Heuristic: std::fmt::Debug + Copy {
    type Instance<'a>: HeuristicInstance<'a>;

    // Heuristic properties.
    fn name(&self) -> String;
    fn l(&self) -> Option<usize> {
        None
    }
    fn match_distance(&self) -> Option<usize> {
        None
    }
    fn pruning(&self) -> Option<bool> {
        None
    }
    fn distance(&self) -> Option<String> {
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
            match_distance: self.match_distance(),
            pruning: self.pruning(),
        }
    }
}

/// An instantiation of a heuristic for a specific pair of sequences.
pub trait HeuristicInstance<'a> {
    fn h(&self, pos: Node<Self::IncrementalState>) -> usize;
    fn expand(&mut self, _pos: Pos) {}

    // TODO: Simplify this, and just use a map inside the heuristic.
    type IncrementalState: std::hash::Hash + Eq + Copy + Default = ();
    fn incremental_h(
        &self,
        _parent: Node<Self::IncrementalState>,
        _pos: Pos,
    ) -> Self::IncrementalState {
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
