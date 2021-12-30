use std::marker::PhantomData;

use super::*;
use crate::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct PathMax<H: Heuristic> {
    heuristic: H,
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
            heuristic: self.heuristic.build(a, b, alphabet),
            phantom: &PhantomData,
        }
    }

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            ..self.heuristic.params()
        }
    }
}

pub struct PathMaxI<'a, HI: HeuristicInstance<'a>> {
    heuristic: HI,
    #[allow(dead_code)]
    phantom: &'a PhantomData<()>,
}

impl<'a, HI: HeuristicInstance<'a>> HeuristicInstance<'a> for PathMaxI<'a, HI> {
    type IncrementalState = (usize, HI::IncrementalState);
    type Pos = HI::Pos;

    fn h(&self, Node(_, state): NodeH<'a, Self>) -> usize {
        state.0
    }

    fn prune(&mut self, pos: Self::Pos) {
        self.heuristic.prune(pos);
    }

    fn incremental_h(
        &self,
        Node(parent, (parent_h, state)): NodeH<'a, Self>,
        pos: Self::Pos,
        cost: usize,
    ) -> Self::IncrementalState {
        let cur_state = self.heuristic.incremental_h(Node(parent, state), pos, cost);

        (
            max(
                max(parent_h, cost) - cost,
                self.heuristic.h(Node(pos, cur_state)),
            ),
            cur_state,
        )
    }

    fn root_state(&self, root_pos: Self::Pos) -> Self::IncrementalState {
        let cur_state = self.heuristic.root_state(root_pos);
        (
            self.heuristic.h(crate::graph::Node(root_pos, cur_state)),
            cur_state,
        )
    }

    fn stats(&self) -> HeuristicStats {
        self.heuristic.stats()
    }
}
