use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct MirrorHeuristic<H: Heuristic>(pub H);

pub struct MirrorHeuristicI<'a, H: Heuristic> {
    h: H::Instance<'a>,
}

impl<H: Heuristic> Heuristic for MirrorHeuristic<H>
where
    for<'a> H::Instance<'a>: HeuristicInstance<'a>,
    Pos: Copy + Eq + std::fmt::Debug + Default,
{
    type Instance<'a> = MirrorHeuristicI<'a, H>;

    fn name(&self) -> String {
        "mirror(".to_owned() + &self.0.name() + ")"
    }

    fn build<'a>(
        &self,
        a: Seq<'a>,
        b: Seq<'a>,
        alphabet: &bio::alphabets::Alphabet,
    ) -> Self::Instance<'a> {
        MirrorHeuristicI {
            h: self.0.build(b, a, alphabet),
        }
    }

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            ..self.0.params()
        }
    }
}

impl<'a, H: Heuristic> HeuristicInstance<'a> for MirrorHeuristicI<'a, H>
where
    H::Instance<'a>: HeuristicInstance<'a>,
{
    fn h(&self, pos: Pos) -> Cost {
        self.h.h(pos.mirror())
    }

    type Hint = <<H as Heuristic>::Instance<'a> as HeuristicInstance<'a>>::Hint;

    fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        self.h.is_seed_start_or_end(pos.mirror())
    }

    fn prune(&mut self, pos: Pos, hint: Self::Hint, seed_cost: MatchCost) -> Cost {
        self.h.prune(pos.mirror(), hint, seed_cost)
    }

    fn h_with_hint(&self, pos: Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        self.h.h_with_hint(pos.mirror(), hint)
    }

    fn root_state(&self, root_pos: Pos) -> Self::Hint {
        self.h.root_state(root_pos.mirror())
    }

    fn stats(&self) -> HeuristicStats {
        self.h.stats()
    }

    fn root_potential(&self) -> Cost {
        self.h.root_potential()
    }

    fn explore(&mut self, pos: Pos) {
        self.h.explore(pos.mirror())
    }
}
