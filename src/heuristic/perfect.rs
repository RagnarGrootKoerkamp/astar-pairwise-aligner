use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct PerfectHeuristic;

pub struct PerfectHeuristicI<'a> {
    a: &'a Sequence,
    b: &'a Sequence,
}

impl Heuristic for PerfectHeuristic {
    type Instance<'a> = PerfectHeuristicI<'a>;

    fn name(&self) -> String {
        "perfect".to_owned()
    }

    fn build<'a>(
        &self,
        a: &'a bio_types::sequence::Sequence,
        b: &'a bio_types::sequence::Sequence,
        _alphabet: &bio::alphabets::Alphabet,
    ) -> Self::Instance<'a> {
        PerfectHeuristicI { a, b }
    }
}

impl<'a> HeuristicInstance<'a> for PerfectHeuristicI<'a> {
    fn h(&self, Pos(i, j): Pos) -> Cost {
        bio::alignment::distance::simd::levenshtein(&self.a[i as usize..], &self.b[j as usize..])
    }
    type Hint = ();
}
