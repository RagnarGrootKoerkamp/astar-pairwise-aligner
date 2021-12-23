use super::heuristic::*;
use crate::{alignment_graph::Node, seeds::Match, util::*};

#[derive(Debug, Copy, Clone)]
pub struct PathMax<H: Heuristic> {
    pub heuristic: H,
}

impl<H: Heuristic> Heuristic for PathMax<H> {
    type Instance<'a> = PathMaxI<'a, H::Instance<'a>>;

    fn name(&self) -> String {
        self.heuristic.name() + "-PM"
    }

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::Instance<'a> {
        PathMaxI {
            a,
            b,
            heuristic: self.heuristic.build(a, b, alphabet),
        }
    }

    fn l(&self) -> Option<usize> {
        self.heuristic.l()
    }

    fn max_match_cost(&self) -> Option<usize> {
        self.heuristic.max_match_cost()
    }

    fn pruning(&self) -> Option<bool> {
        self.heuristic.pruning()
    }

    fn distance(&self) -> Option<String> {
        self.heuristic.distance()
    }
}

pub struct PathMaxI<'a, HI: HeuristicInstance<'a>> {
    a: &'a Sequence,
    b: &'a Sequence,
    heuristic: HI,
}

impl<'a, HI: HeuristicInstance<'a>> HeuristicInstance<'a> for PathMaxI<'a, HI> {
    fn h(&self, Node(_, state): Node<Self::IncrementalState>) -> usize {
        state.0
    }

    type IncrementalState = (usize, HI::IncrementalState);

    fn prune(&mut self, pos: Pos) {
        self.heuristic.prune(pos);
    }

    fn incremental_h(
        &self,
        Node(parent, (parent_h, state)): Node<Self::IncrementalState>,
        pos: Pos,
    ) -> Self::IncrementalState {
        assert!(parent.0 <= pos.0 && parent.1 <= pos.1);
        // The length of the edge. Supports edges of length 1, and longer diagonals are assumed to be all equal value.
        let d = if pos.1 - parent.1 != pos.0 - parent.0 {
            assert!(pos.0 <= parent.0 + 1 && pos.1 <= parent.1 + 1);
            1
        } else {
            if self.a[parent.0] == self.b[parent.1] {
                1 //0
            } else {
                assert!(pos.0 == parent.0 + 1 && pos.1 == parent.1 + 1);
                1
            }
        };
        let cur_state = self.heuristic.incremental_h(Node(parent, state), pos);

        (
            max(
                1 * (max(parent_h, d) - d),
                self.heuristic.h(Node(pos, cur_state)),
            ),
            cur_state,
        )
    }

    fn root_state(&self) -> Self::IncrementalState {
        let cur_state = self.heuristic.root_state();
        (self.heuristic.h(Node(Pos(0, 0), cur_state)), cur_state)
    }

    fn num_seeds(&self) -> Option<usize> {
        self.heuristic.num_seeds()
    }

    fn matches(&self) -> Option<&Vec<Match>> {
        self.heuristic.matches()
    }

    fn num_matches(&self) -> Option<usize> {
        self.heuristic.num_matches()
    }
}
