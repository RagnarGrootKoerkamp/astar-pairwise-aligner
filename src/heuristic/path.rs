#![allow(unused)]
use std::fmt::Debug;

use crate::{
    aligners::cigar::Cigar,
    prelude::{Cost, LinearCost, Seq},
};

use super::Heuristic;

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
        // Find a candidate path
        // let mut aligner = DiagonalTransition::new(
        //     LinearCost::new_unit(),
        //     GapCostHeuristic::Disable,
        //     NoCost,
        //     true,
        //     NoVisualizer,
        // );
        let start = instant::Instant::now();
        let (path_cost, cigar): (Cost, Cigar) = todo!();
        println!("Inner alignment: {}", start.elapsed().as_secs_f32());
        let path = cigar.to_path_with_cost(LinearCost::new_unit());
        assert_eq!(path.last().unwrap().1, path_cost);
        let mut p = path.iter().rev().peekable();

        let h = self.h.build_with_filter(a, b, |m, h| {
            while m.start < p.peek().unwrap().0 {
                p.next();
            }
            let (pos, pos_cost) = **p.peek().unwrap();
            if m.start == pos {
                assert!(h <= path_cost - pos_cost);
                // Filter away the match when h < cost_to_end
                if h < path_cost - pos_cost {
                    return true;
                }
            }
            false
        });
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
