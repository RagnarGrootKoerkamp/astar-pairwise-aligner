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

impl<H1: Heuristic, H2: Heuristic, Pos> Heuristic for EqualHeuristic<H1, H2>
where
    for<'a> H1::Instance<'a>: HeuristicInstance<'a, Pos = Pos>,
    for<'a> H2::Instance<'a>: HeuristicInstance<'a, Pos = Pos>,
    Pos: Copy + Eq + std::fmt::Debug + Default,
{
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

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            ..self.h1.params()
        }
    }
}

impl<'a, H1: Heuristic, H2: Heuristic, Pos> HeuristicInstance<'a> for EqualHeuristicI<'a, H1, H2>
where
    H1::Instance<'a>: HeuristicInstance<'a, Pos = Pos>,
    H2::Instance<'a>: HeuristicInstance<'a, Pos = Pos>,
    Pos: Eq + Copy + std::fmt::Debug + Default,
{
    type Pos = Pos;

    fn h(&self, pos: Self::Pos) -> Cost {
        let h1 = self.h1.h(pos);
        let h2 = self.h2.h(pos);
        // h1 is the slow accurate one, h2 the fast inaccurate one.
        assert!(h1 == h2, "Values differ at {:?}: {} {}", pos, h1, h2);
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

    fn incremental_h(&self, parent: Self::Pos, pos: Pos, cost: Cost) -> Self::IncrementalState {
        (
            self.h1.incremental_h(parent, pos, cost),
            self.h2.incremental_h(parent, pos, cost),
        )
    }

    fn root_state(&self, root_pos: Self::Pos) -> Self::IncrementalState {
        (self.h1.root_state(root_pos), self.h2.root_state(root_pos))
    }

    fn stats(&self) -> HeuristicStats {
        self.h1.stats()
    }
}
