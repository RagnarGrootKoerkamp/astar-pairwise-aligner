use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct EqualHeuristic<H1: Heuristic, H2: Heuristic> {
    pub h1: H1,
    pub h2: H2,
}

pub struct EqualHeuristicI<'a, H1: Heuristic, H2: Heuristic> {
    h1: H1::Instance<'a>,
    h2: H2::Instance<'a>,
}

impl<H1: Heuristic, H2: Heuristic> Heuristic for EqualHeuristic<H1, H2> {
    type Instance<'a> = EqualHeuristicI<'a, H1, H2>;

    fn name(&self) -> String {
        self.h1.name() + "+" + &self.h2.name()
    }

    fn build<'a>(
        &self,
        a: &'a bio_types::sequence::Sequence,
        b: &'a bio_types::sequence::Sequence,
        alphabet: &bio::alphabets::Alphabet,
    ) -> Self::Instance<'a> {
        EqualHeuristicI {
            h1: self.h1.build(a, b, alphabet),
            h2: self.h2.build(a, b, alphabet),
        }
    }

    fn l(&self) -> Option<usize> {
        self.h1.l()
    }

    fn max_match_cost(&self) -> Option<usize> {
        self.h1.max_match_cost()
    }

    fn pruning(&self) -> Option<bool> {
        self.h1.pruning()
    }

    fn distance(&self) -> Option<String> {
        self.h1.distance()
    }
}

impl<'a, H1: Heuristic, H2: Heuristic> HeuristicInstance<'a> for EqualHeuristicI<'a, H1, H2> {
    fn h(&self, Node(pos, (s1, s2)): Node<Self::IncrementalState>) -> usize {
        let h1 = self.h1.h(Node(pos, s1));
        let h2 = self.h2.h(Node(pos, s2));
        // h1 is the slow accurate one, h2 the fast inaccurate one.
        assert!(
            h1 <= h2 + 1 && h2 <= h1,
            "Values differ at {:?}: {} {}",
            pos,
            h1,
            h2
        );
        h1
    }

    type IncrementalState = (
        <<H1 as Heuristic>::Instance<'a> as HeuristicInstance<'a>>::IncrementalState,
        <<H2 as Heuristic>::Instance<'a> as HeuristicInstance<'a>>::IncrementalState,
    );

    fn prune(&mut self, pos: Pos) {
        self.h1.prune(pos);
        self.h2.prune(pos);
    }

    fn incremental_h(
        &self,
        Node(parent, (s1, s2)): Node<Self::IncrementalState>,
        pos: Pos,
    ) -> Self::IncrementalState {
        (
            self.h1.incremental_h(Node(parent, s1), pos),
            self.h2.incremental_h(Node(parent, s2), pos),
        )
    }

    fn root_state(&self) -> Self::IncrementalState {
        (self.h1.root_state(), self.h2.root_state())
    }

    fn num_seeds(&self) -> Option<usize> {
        self.h1.num_seeds()
    }

    fn matches(&self) -> Option<&Vec<Match>> {
        self.h1.matches()
    }

    fn num_matches(&self) -> Option<usize> {
        self.h1.num_matches()
    }
}
