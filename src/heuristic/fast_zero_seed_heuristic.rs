use itertools::Itertools;

use super::*;
use crate::{
    increasing_function::IncreasingFunction2D,
    prelude::*,
    seeds::{find_matches, Match, SeedMatches},
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
    fn l(&self) -> Option<usize> {
        Some(self.l)
    }
    fn distance(&self) -> Option<String> {
        Some("Zero".into())
    }
}
pub struct FastZeroSeedHeuristicI {
    seed_matches: SeedMatches,
    f: IncreasingFunction2D<usize>,
    // TODO: Replace this by params: SeedHeuristic
}

impl FastZeroSeedHeuristicI {
    pub fn new(
        a: &Sequence,
        b: &Sequence,
        alphabet: &Alphabet,
        params: &FastZeroSeedHeuristic,
    ) -> Self {
        let seed_matches = find_matches(a, b, alphabet, params.l, 0);

        // The increasing function goes back from the end, and uses (0,0) for the final state.
        let f = IncreasingFunction2D::new(
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
    fn num_seeds(&self) -> Option<usize> {
        Some(self.seed_matches.num_seeds)
    }
    fn matches(&self) -> Option<&Vec<Match>> {
        Some(&self.seed_matches.matches)
    }
    fn num_matches(&self) -> Option<usize> {
        Some(self.seed_matches.matches.len())
    }
}
