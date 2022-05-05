use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct SymmetricHeuristic<H: Heuristic> {
    pub h: H,
}

type SymmetricHeuristicI<'a, H> = MaxHeuristicI<'a, H, MirrorHeuristic<H>>;

impl<H: Heuristic> Heuristic for SymmetricHeuristic<H>
where
    for<'a> H::Instance<'a>: HeuristicInstance<'a>,
    Pos: Copy + Eq + std::fmt::Debug + Default,
{
    type Instance<'a> = SymmetricHeuristicI<'a, H>;

    fn name(&self) -> String {
        "symm(".to_owned() + &self.h.name() + ")"
    }

    fn build<'a>(
        &self,
        a: &'a bio_types::sequence::Sequence,
        b: &'a bio_types::sequence::Sequence,
        alphabet: &bio::alphabets::Alphabet,
    ) -> Self::Instance<'a> {
        let max_config = MaxHeuristic::<H, MirrorHeuristic<H>> {
            h1: self.h,
            h2: MirrorHeuristic { h: self.h },
        };
        max_config.build(a, b, alphabet)
    }

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            ..self.h.params()
        }
    }
}
