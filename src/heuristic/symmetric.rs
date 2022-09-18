use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct SymmetricHeuristic<H: Heuristic>(pub H);

type SymmetricHeuristicI<'a, H> = MaxHeuristicI<'a, H, MirrorHeuristic<H>>;

impl<H: Heuristic> Heuristic for SymmetricHeuristic<H>
where
    for<'a> H::Instance<'a>: HeuristicInstance<'a>,
    Pos: Copy + Eq + std::fmt::Debug + Default,
{
    type Instance<'a> = SymmetricHeuristicI<'a, H>;

    fn name(&self) -> String {
        "symm(".to_owned() + &self.0.name() + ")"
    }

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> Self::Instance<'a> {
        let max_config = MaxHeuristic::<H, MirrorHeuristic<H>> {
            h1: self.0,
            h2: MirrorHeuristic(self.0),
        };
        max_config.build(a, b)
    }

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            ..self.0.params()
        }
    }
}
