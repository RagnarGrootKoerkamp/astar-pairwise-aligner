use itertools::Itertools;

use super::*;
use crate::{
    increasing_function::ContourGraph,
    prelude::*,
    seeds::{find_matches, MatchConfig, SeedMatches},
};

// TODO: Make this work for the other distance functions.
// TODO: Inherit this from SeedHeuristic.
// TODO: Support pruning.
// TODO: Support inexact matches.
#[derive(Debug, Clone, Copy)]
pub struct FastZeroSeedHeuristic {
    pub l: usize,
    pub max_match_cost: usize,
}
impl Heuristic for FastZeroSeedHeuristic {
    type Instance<'a> = FastZeroSeedHeuristicI;
    fn name(&self) -> String {
        "FastZeroSeed".into()
    }

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::Instance<'_> {
        FastZeroSeedHeuristicI::new(a, b, alphabet, self)
    }

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            l: Some(self.l),
            distance_function: Some("Zero".into()),
            ..Default::default()
        }
    }
}
pub struct FastZeroSeedHeuristicI {
    seed_matches: SeedMatches,
    f: ContourGraph<usize>,
    // TODO: Replace this by params: SeedHeuristic
}

impl FastZeroSeedHeuristicI {
    pub fn new(
        a: &Sequence,
        b: &Sequence,
        alphabet: &Alphabet,
        params: &FastZeroSeedHeuristic,
    ) -> Self {
        let seed_matches = find_matches(
            a,
            b,
            alphabet,
            MatchConfig {
                l: params.l,
                ..Default::default()
            },
        );

        // The increasing function goes back from the end, and uses (0,0) for the final state.
        let f = ContourGraph::new(
            Pos(a.len(), b.len()),
            false,
            seed_matches.iter().cloned().collect_vec(),
        );

        FastZeroSeedHeuristicI { seed_matches, f }
    }
}
impl HeuristicInstance<'_> for FastZeroSeedHeuristicI {
    fn h(&self, Node(pos, parent_state): NodeH<Self>) -> usize {
        self.seed_matches.potential(pos) - self.f.val(parent_state)
    }

    type IncrementalState = crate::increasing_function::NodeIndex;

    fn incremental_h(
        &self,
        parent: NodeH<Self>,
        pos: Self::Pos,
        _cost: usize,
    ) -> Self::IncrementalState {
        // TODO: Forward the cost of the edge.
        self.f.incremental_forward(pos, parent.1)
    }

    fn root_state(&self, _: Self::Pos) -> Self::IncrementalState {
        self.f.root()
    }

    fn stats(&self) -> HeuristicStats {
        HeuristicStats {
            num_seeds: Some(self.seed_matches.num_seeds),
            num_matches: Some(self.seed_matches.matches.len()),
            matches: Some(self.seed_matches.matches.clone()),
            ..Default::default()
        }
    }
}
