#![feature(
    let_chains,
    step_trait,
    int_roundings,
    portable_simd,
    type_changing_struct_update
)]

mod band;
mod block;
mod blocks;
mod domain;
mod params;
mod ranges;
#[cfg(test)]
mod tests;

pub use band::{DoublingStart, DoublingType};
use domain::AstarPa2Stats;
use pa_bitpacking::W;
pub use params::*;

pub use blocks::BlockParams;
use pa_affine_types::AffineCigar;
use pa_heuristic::{Heuristic, HeuristicInstance, NoCostI};
use pa_types::*;
use pa_vis::{VisualizerInstance, VisualizerT};
use ranges::*;

use crate::domain::AstarPa2Instance;

const DEBUG: bool = false;

/// Block height 64.
pub const WI: I = W as I;

/// Align two sequences using A*PA2-simple.
pub fn astarpa2_simple(a: Seq, b: Seq) -> (Cost, Cigar) {
    let (cost, cigar) = AstarPa2Params::simple().make_aligner(true).align(a, b);
    (cost, cigar.unwrap())
}

/// Align two sequences using A*PA2-full.
pub fn astarpa2_full(a: Seq, b: Seq) -> (Cost, Cigar) {
    let (cost, cigar) = AstarPa2Params::full().make_aligner(true).align(a, b);
    (cost, cigar.unwrap())
}

/// Typed parameters for A*PA2 containing heuristic and visualizer.
#[derive(Debug)]
pub struct AstarPa2<V: VisualizerT, H: Heuristic> {
    /// The domain to compute.
    pub domain: Domain<H>,

    /// The strategy to use to compute the given domain.
    pub doubling: DoublingType,

    /// Compute `block_width` columns at a time, to reduce overhead of metadata
    /// computations.
    pub block_width: I,

    /// The visualizer to use.
    pub v: V,

    /// The type of front to use.
    pub block: BlockParams,

    /// Whether to return a trace.
    /// `.cost()` always returns cost only, while `.align()` returns a cigar
    /// depending on this.
    pub trace: bool,

    /// When true, `j_range` skips querying `h` when it can assuming consistency.
    pub sparse_h: bool,

    /// Whether pruning is enabled.
    pub prune: bool,
}

impl<V: VisualizerT, H: Heuristic> AstarPa2<V, H> {
    pub fn build<'a>(&'a self, a: Seq<'a>, b: Seq<'a>) -> AstarPa2Instance<'a, V, H> {
        use Domain::*;

        // init V
        let v = self.v.build(a, b);

        // build H
        let start = std::time::Instant::now();
        let domain = match self.domain {
            Full => Full,
            GapStart => GapStart,
            GapGap => GapGap,
            Astar(h) => {
                let h = h.build(a, b);
                if DEBUG {
                    eprintln!("h0: {}", h.h(Pos(0, 0)));
                }
                Astar(h)
            }
        };

        AstarPa2Instance {
            a,
            b,
            params: self,
            domain,
            hint: Default::default(),
            v,
            stats: AstarPa2Stats {
                t_precomp: start.elapsed(),
                ..Default::default()
            },
        }
    }

    fn cost_or_align(&self, a: Seq, b: Seq, trace: bool) -> (Cost, Option<Cigar>, AstarPa2Stats) {
        let mut nw = self.build(a, b);
        let h0 = nw.domain.h().map_or(0, |h| h.h(Pos(0, 0)));
        let (cost, cigar) = match self.doubling {
            DoublingType::None => {
                // FIXME: Allow single-shot alignment with bounded dist.
                assert!(matches!(self.domain, Domain::Full));
                nw.align_for_bounded_dist(None, trace, None).unwrap()
            }
            DoublingType::LinearSearch { start, delta } => {
                let start_f = start.initial_values(a, b, h0).0;
                let mut blocks = self.block.new(trace, a, b);
                band::linear_search(start_f, delta as Cost, |s| {
                    nw.align_for_bounded_dist(Some(s), trace, Some(&mut blocks))
                        .map(|x @ (c, _)| (c, x))
                })
                .1
            }
            DoublingType::BandDoubling { start, factor }
            | DoublingType::BandDoublingStartIncrement { start, factor, .. } => {
                let (start_f, mut start_increment) = start.initial_values(a, b, h0);
                start_increment = start_increment.max(self.block_width as i32);
                if let DoublingType::BandDoublingStartIncrement {
                    start_increment: si,
                    ..
                } = self.doubling
                {
                    start_increment = si;
                }
                let mut blocks = self.block.new(trace, a, b);
                let r = band::exponential_search(start_f, start_increment, factor, |s| {
                    nw.align_for_bounded_dist(Some(s), trace, Some(&mut blocks))
                        .map(|x @ (c, _)| (c, x))
                })
                .1;
                nw.stats.block_stats = blocks.stats;
                r
            }
            // NOTE: This is not in the paper since it does not yet work much
            // better than (global) band doubling in practice.
            DoublingType::LocalDoubling => {
                assert!(self.prune, "Local doubling requires pruning.");
                let (cost, cigar) = nw.local_doubling();
                (cost, Some(cigar))
            }
        };
        nw.v.last_frame::<NoCostI>(
            cigar.as_ref().map(|c| AffineCigar::from(c)).as_ref(),
            None,
            None,
        );
        assert!(h0 <= cost, "Heuristic at start {h0} > final cost {cost}.");
        (cost, cigar, nw.stats)
    }

    pub fn cost(&self, a: Seq, b: Seq) -> Cost {
        self.cost_or_align(a, b, false).0
    }

    pub fn align(&self, a: Seq, b: Seq) -> (Cost, Option<Cigar>) {
        let (cost, cigar, _stats) = self.cost_or_align(a, b, self.trace);
        (cost, cigar)
    }

    pub fn cost_for_bounded_dist(&self, a: Seq, b: Seq, f_max: Cost) -> Option<Cost> {
        self.build(a, b)
            .align_for_bounded_dist(Some(f_max), false, None)
            .map(|c| c.0)
    }

    pub fn align_for_bounded_dist(&self, a: Seq, b: Seq, f_max: Cost) -> Option<(Cost, Cigar)> {
        self.build(a, b)
            .align_for_bounded_dist(Some(f_max), true, None)
            .map(|(c, cigar)| (c, cigar.unwrap()))
    }
}

/// Helper trait to erase the type of the heuristic that additionally returns alignment statistics.
pub trait AstarPa2StatsAligner: Aligner {
    fn align_with_stats(&mut self, a: Seq, b: Seq) -> (Cost, Option<Cigar>, AstarPa2Stats);
}

impl<V: VisualizerT, H: Heuristic> AstarPa2StatsAligner for AstarPa2<V, H> {
    fn align_with_stats(&mut self, a: Seq, b: Seq) -> (Cost, Option<Cigar>, AstarPa2Stats) {
        self.cost_or_align(a, b, self.trace)
    }
}

impl<V: VisualizerT, H: Heuristic> Aligner for AstarPa2<V, H> {
    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Option<Cigar>) {
        let (cost, cigar, _stats) = self.cost_or_align(a, b, self.trace);
        (cost, cigar)
    }
}
