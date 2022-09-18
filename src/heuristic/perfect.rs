use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct PerfectHeuristic;

pub struct PerfectHeuristicI<'a> {
    a: Seq<'a>,
    b: Seq<'a>,
}

impl Heuristic for PerfectHeuristic {
    type Instance<'a> = PerfectHeuristicI<'a>;

    fn name(&self) -> String {
        "perfect".to_owned()
    }

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>, _alphabet: &Alphabet) -> Self::Instance<'a> {
        PerfectHeuristicI { a, b }
    }
}

impl<'a> HeuristicInstance<'a> for PerfectHeuristicI<'a> {
    fn h(&self, Pos(i, j): Pos) -> Cost {
        bio::alignment::distance::simd::levenshtein(&self.a[i as usize..], &self.b[j as usize..])
    }
    type Hint = ();
}
