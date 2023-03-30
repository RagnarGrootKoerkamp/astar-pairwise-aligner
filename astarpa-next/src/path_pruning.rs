#![allow(unused)]
use std::fmt::Debug;

use pa_heuristic::{matches::Match, Heuristic};
use pa_types::{seq_to_string, Cigar, Cost, CostModel, Seq};

/// The Path heuristic takes as input a path, and builds a heuristic that
/// 'simulates' the pruning of the SH/CSH by doing it up-front where possible.
///
/// Basically, as long as the path as unaccounted-for errors, the first match on
/// the path preceding the error is pruned in advance.
///
/// NOTE: H should have pruning disabled.
#[derive(Copy, Clone, Debug)]
pub struct PathHeuristic<H: Heuristic> {
    pub h: H,
}

impl<H: Heuristic> PathHeuristic<H> {
    pub fn build_with_cost<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> (Cost, H::Instance<'a>) {
        let start = instant::Instant::now();
        let (path_cost, cigar): (Cost, Cigar) = astarpa::astarpa(a, b);
        println!("Inner alignment: {}", start.elapsed().as_secs_f32());
        let path = cigar.to_path_with_costs(CostModel::unit());
        let cigar_cost = path.last().unwrap().1;
        assert_eq!(
            cigar_cost,
            path_cost,
            "
a: {}
b: {}
cigar: {}
cost:       {path_cost}
cigar cost: {cigar_cost}
path {path:?}",
            seq_to_string(a),
            seq_to_string(b),
            cigar.to_string()
        );
        let mut p = path.iter().rev().peekable();

        let h = self.h.build_with_filter(
            a,
            b,
            Some(|m: &Match, h: Cost| {
                while m.start < p.peek().unwrap().0 {
                    p.next();
                }
                let (pos, pos_cost) = **p.peek().unwrap();
                if m.start == pos {
                    assert!(h <= path_cost - pos_cost);
                    // Filter away the match when h < cost_to_end
                    if h < path_cost - pos_cost {
                        return false;
                    }
                }
                true
            }),
        );
        (path_cost, h)
    }
}

impl<H: Heuristic> Heuristic for PathHeuristic<H> {
    type Instance<'a> = H::Instance<'a>;

    fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> H::Instance<'a> {
        self.build_with_cost(a, b).1
    }

    fn name(&self) -> String {
        format!("Path({})", self.h.name())
    }
}
