use super::heuristic::*;
use crate::{
    alignment_graph::{AlignmentGraph, Node},
    increasing_function::IncreasingFunction2D,
    seeds::{find_matches, Match, SeedMatches},
    util::*,
};

// TODO: Make this work for the other distance functions.
// TODO: Inherit this from SeedHeuristic
#[derive(Debug, Clone, Copy)]
pub struct FastSeedHeuristic {
    pub l: usize,
    pub pruning: bool,
    pub match_distance: usize,
}
impl Heuristic for FastSeedHeuristic {
    type Instance<'a> = FastSeedHeuristicI;
    fn name(&self) -> String {
        "FastSeed".into()
    }

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
        graph: &'a AlignmentGraph,
    ) -> Self::Instance<'_> {
        FastSeedHeuristicI::new(a, b, alphabet, graph, self)
    }
    fn l(&self) -> Option<usize> {
        Some(self.l)
    }
    fn distance(&self) -> Option<String> {
        Some("Zero".into())
    }
}
pub struct FastSeedHeuristicI {
    seed_matches: SeedMatches,
    target: Pos,
    f: IncreasingFunction2D<usize>,
    // TODO: Replace this by params: SeedHeuristic
    pruning: bool,
    match_distance: usize,
}

impl FastSeedHeuristicI {
    pub fn new(
        a: &Sequence,
        b: &Sequence,
        alphabet: &Alphabet,
        _graph: &AlignmentGraph,
        params: &FastSeedHeuristic,
    ) -> Self {
        let seed_matches = find_matches(a, b, alphabet, params.l, params.match_distance);

        // The increasing function goes back from the end, and uses (0,0) for the final state.
        let f =
            IncreasingFunction2D::new(Pos(a.len(), b.len()), seed_matches.iter().rev().cloned());

        FastSeedHeuristicI {
            seed_matches,
            target: Pos(a.len(), b.len()),
            f,
            pruning: params.pruning,
            match_distance: params.match_distance,
        }
    }
}
impl HeuristicInstance<'_> for FastSeedHeuristicI {
    fn h(&self, Node(pos, parent): Node<Self::IncrementalState>) -> usize {
        self.seed_matches.potential(pos) - self.f.val(parent)
    }

    type IncrementalState = crate::increasing_function::NodeIndex;

    fn incremental_h(
        &self,
        parent: Node<Self::IncrementalState>,
        pos: Pos,
    ) -> Self::IncrementalState {
        // We can unwrap because (0,0) is part of the map.
        self.f.get_jump(pos, parent.1).unwrap()
    }

    fn root_state(&self) -> Self::IncrementalState {
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
