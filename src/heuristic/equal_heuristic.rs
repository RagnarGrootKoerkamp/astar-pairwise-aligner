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
        h2
    }

    type Hint = (
        <<H1 as Heuristic>::Instance<'a> as HeuristicInstance<'a>>::Hint,
        <<H2 as Heuristic>::Instance<'a> as HeuristicInstance<'a>>::Hint,
    );

    fn is_start_of_seed(&mut self, pos: Self::Pos) -> bool {
        let s1 = self.h1.is_start_of_seed(pos);
        let s2 = self.h2.is_start_of_seed(pos);
        assert_eq!(s1, s2);
        s2
    }

    fn prune(&mut self, pos: Pos) {
        self.h1.prune(pos);
        self.h2.prune(pos);
    }

    fn prune_with_hint(&mut self, pos: Self::Pos, hint: Self::Hint) -> Cost {
        let c1 = self.h1.prune_with_hint(pos, hint.0);
        let c2 = self.h2.prune_with_hint(pos, hint.1);
        c2
    }

    fn h_with_hint(&self, pos: Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        let (c1, hint1) = self.h1.h_with_hint(pos, hint.0);
        let (c2, hint2) = self.h2.h_with_hint(pos, hint.1);
        assert!(c1 == c2, "Values differ at {:?}: {} {}", pos, c1, c2);
        (c2, (hint1, hint2))
    }

    fn root_state(&self, root_pos: Self::Pos) -> Self::Hint {
        (self.h1.root_state(root_pos), self.h2.root_state(root_pos))
    }

    fn stats(&self) -> HeuristicStats {
        self.h2.stats()
    }

    fn root_potential(&self) -> Cost {
        self.h2.root_potential()
    }

    fn explore(&mut self, pos: Self::Pos) {
        self.h1.explore(pos);
        self.h2.explore(pos);
    }
}
