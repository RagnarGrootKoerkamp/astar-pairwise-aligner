//! TODO
//! - Store block of fronts in a single allocation. Update `NwFront` to contain multiple columns as once and be reusable.
//! - timings
//! - meet in the middle with A* and pruning on both sides
//! - try jemalloc/mimalloc
//! - Matches:
//!   - Recursively merge matches to find r=2^k matches.
//!     - possibly reduce until no more spurious matches
//!     - tricky: requires many 'shadow' matches. Handle in cleaner way?
//!  - Figure out why pruning up to Layer::MAX gives errors, but pruning up to highest_modified_contour does not.
//! BUG: Figure out why the delta=64 is broken in fixed_j_range.
//! TODO: Traceback using DT
//! TODO: QgramIndex for short k.
//! TODO: Analyze local doubling better
//! TODO: Speed up j_range more???
mod affine;
mod bitpacking;
mod front;

use crate::nw::front::{IRange, JRange, NwFront, NwFronts};
use crate::{exponential_search, Strategy, PRINT};
use crate::{linear_search, Domain};
use pa_affine_types::*;
use pa_heuristic::*;
use pa_types::*;
use pa_vis::*;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

use self::affine::AffineNwFrontsTag;
use self::front::NwFrontsTag;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum FrontType {
    Affine,
    Bit(BitFront),
}

impl Default for FrontType {
    fn default() -> Self {
        FrontType::Bit(BitFront::default())
    }
}

// TODO: Fix these names to be the same.
pub use affine::AffineNwFrontsTag as AffineFront;
pub use bitpacking::BitFrontsTag as BitFront;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct AstarNwParams {
    /// An optional name for the parameter set.
    #[serde(default)]
    pub name: String,

    /// The domain to compute.
    pub domain: Domain<()>,

    /// Heuristic to use for A* domain.
    pub heuristic: HeuristicParams,

    /// The strategy to use to compute the given domain.
    pub strategy: Strategy,

    /// Compute `block_width` columns at a time, to reduce overhead of metadata
    /// computations.
    pub block_width: I,

    /// The front type to use.
    pub front: FrontType,

    /// When true, `j_range` skips querying `h` when it can assuming consistency.
    #[serde(default)]
    pub sparse_h_calls: bool,

    /// Whether pruning is enabled.
    #[serde(default)]
    pub prune: bool,

    /// Whether the visualizer is enabled.
    #[serde(default)]
    pub viz: bool,
}

impl AstarNwParams {
    /// Build an `AstarStatsAligner` instance from
    pub fn make_aligner(&self, trace: bool) -> Box<dyn Aligner> {
        #[cfg(feature = "example")]
        if self.viz {
            use pa_vis::visualizer::{Gradient, When};
            use pa_vis::canvas::RED;
            use std::time::Duration;
            let mut config = pa_vis::visualizer::Config::default();
            config.draw = When::StepBy(1);
            config.save = When::None; //When::LayersStepBy(30);
            config.save_last = false;
            config.delay = Duration::from_secs_f32(0.0001);
            config.cell_size = 0;
            config.downscaler = 0;
            config.style.bg_color = (255, 255, 255, 128);
            config.style.expanded = Gradient::TurboGradient(0.25..0.90);
            config.style.path_width = None;
            config.layer_drawing = false;
            config.style.draw_dt = false;
            config.style.draw_heuristic = false;
            config.style.draw_f = false;
            config.style.draw_h_calls = true;
            config.style.draw_labels = false;
            config.transparent_bmp = true;
            config.draw_old_on_top = false;
            config.paused = true;

            config.style.pruned_match = RED;
            config.style.match_width = 1;
            config.style.draw_matches = true;
            config.filepath = self.name.clone().into();
            return self.make_aligner_with_visualizer(trace, config);
        }
        self.make_aligner_with_visualizer(trace, NoVis)
    }

    /// Build a type-erased aligner object from parameters.
    /// FIXME: Add costmodel support.
    pub fn make_aligner_with_visualizer<V: VisualizerT + 'static>(
        &self,
        trace: bool,
        v: V,
    ) -> Box<dyn Aligner> {
        struct Mapper<V: VisualizerT, F: NwFrontsTag<0>> {
            params: AstarNwParams,
            trace: bool,
            v: V,
            front: F,
        }
        impl<V: VisualizerT + 'static, F: NwFrontsTag<0> + 'static> HeuristicMapper for Mapper<V, F> {
            type R = Box<dyn Aligner>;
            fn call<H: Heuristic + 'static>(self, h: H) -> Box<dyn Aligner> {
                Box::new(NW {
                    cm: AffineCost::unit(),
                    domain: Domain::Astar(h),
                    strategy: self.params.strategy,
                    block_width: self.params.block_width,
                    v: self.v,
                    front: self.front,
                    trace: self.trace,
                    sparse_h: self.params.sparse_h_calls,
                    prune: self.params.prune,
                })
            }
        }
        match (self.domain, self.front) {
            (Domain::Astar(()), FrontType::Affine) => self.heuristic.map(Mapper {
                params: self.clone(),
                trace,
                v,
                front: AffineFront,
            }),
            (Domain::Astar(()), FrontType::Bit(front)) => self.heuristic.map(Mapper {
                params: self.clone(),
                trace,
                v,
                front,
            }),
            (d, FrontType::Affine) => Box::new(NW {
                cm: AffineCost::unit(),
                domain: d.into(),
                strategy: self.strategy,
                block_width: self.block_width,
                v,
                front: AffineFront,
                trace,
                sparse_h: self.sparse_h_calls,
                prune: self.prune,
            }),
            (d, FrontType::Bit(front)) => Box::new(NW {
                cm: AffineCost::unit(),
                domain: d.into(),
                strategy: self.strategy,
                block_width: self.block_width,
                v,
                front,
                trace,
                sparse_h: self.sparse_h_calls,
                prune: self.prune,
            }),
        }
    }
}

/// Needleman-Wunsch aligner.
///
/// NOTE: Heuristics only support unit cost graph for now.
pub struct NW<const N: usize, V: VisualizerT, H: Heuristic, F: NwFrontsTag<N>> {
    /// The cost model to use.
    pub cm: AffineCost<N>,

    /// The domain to compute.
    pub domain: Domain<H>,

    /// The strategy to use to compute the given domain.
    pub strategy: Strategy,

    /// Compute `block_width` columns at a time, to reduce overhead of metadata
    /// computations.
    pub block_width: I,

    /// The visualizer to use.
    pub v: V,

    /// The type of front to use.
    pub front: F,

    /// Whether to return a trace.
    /// `.cost()` always returns cost only, while `.align()` returns a cigar
    /// depending on this.
    pub trace: bool,

    /// When true, `j_range` skips querying `h` when it can assuming consistency.
    pub sparse_h: bool,

    /// Whether pruning is enabled.
    pub prune: bool,
}

impl<const N: usize> NW<N, NoVis, NoCost, AffineNwFrontsTag<N>> {
    // TODO: This is only used in tests.
    pub fn new(cm: AffineCost<N>, use_gap_cost_heuristic: bool, exponential_search: bool) -> Self {
        Self {
            cm,
            domain: if use_gap_cost_heuristic {
                Domain::GapGap
            } else {
                Domain::Full
            },
            strategy: if exponential_search {
                // TODO: Make this more general.
                Strategy::band_doubling()
            } else {
                Strategy::None
            },
            // TODO: Make this more general.
            block_width: 32,
            v: NoVis,
            front: AffineNwFrontsTag::<N>,
            trace: true,
            sparse_h: true,
            prune: true,
        }
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic, F: NwFrontsTag<N>> NW<N, V, H, F> {
    pub fn build<'a>(&'a self, a: Seq<'a>, b: Seq<'a>) -> NWInstance<'a, N, V, H, F> {
        use Domain::*;
        NWInstance {
            a,
            b,
            params: self,
            domain: match self.domain {
                Full => Full,
                GapStart => GapStart,
                GapGap => GapGap,
                Astar(h) => {
                    let h = h.build(a, b);
                    if PRINT {
                        eprintln!("h0: {}", h.h(Pos(0, 0)));
                    }
                    Astar(h)
                }
            },
            hint: Default::default(),
            v: self.v.build(a, b),
        }
    }

    fn band_doubling_params(
        &self,
        start: crate::DoublingStart,
        a: &[u8],
        b: &[u8],
        nw: &NWInstance<N, V, H, F>,
    ) -> (i32, i32) {
        let (start_f, start_increment) = match start {
            crate::DoublingStart::Zero => (0, 1),
            crate::DoublingStart::Gap => {
                let x = self.cm.gap_cost(Pos(0, 0), Pos::target(a, b));
                (x, x)
            }
            crate::DoublingStart::H0 => match nw.domain {
                Domain::Full => (0, 1),
                Domain::GapStart | Domain::GapGap => {
                    let x = self.cm.gap_cost(Pos(0, 0), Pos::target(a, b));
                    (x, x)
                }
                Domain::Astar(_) => (nw.domain.h().unwrap().h(Pos(0, 0)), 1),
            },
        };
        (start_f, max(start_increment, F::BLOCKSIZE))
    }

    fn cost_or_align(&self, a: Seq, b: Seq, trace: bool) -> (Cost, Option<AffineCigar>) {
        let mut nw = self.build(a, b);
        let h0 = nw.domain.h().map_or(0, |h| h.h(Pos(0, 0)));
        let (cost, cigar) = match self.strategy {
            Strategy::LocalDoubling => {
                assert!(self.prune, "Local doubling requires pruning.");
                let (cost, cigar) = nw.local_doubling();
                (cost, Some(cigar))
            }
            Strategy::BandDoubling { start, factor } => {
                let (start_f, start_increment) = self.band_doubling_params(start, a, b, &nw);
                let mut fronts = self.front.new(trace, a, b, &self.cm);
                exponential_search(start_f, start_increment, factor, |s| {
                    nw.align_for_bounded_dist(Some(s), trace, Some(&mut fronts))
                        .map(|x @ (c, _)| (c, x))
                })
                .1
            }
            Strategy::None => {
                // FIXME: Allow single-shot alignment with bounded dist.
                assert!(matches!(self.domain, Domain::Full));
                nw.align_for_bounded_dist(None, trace, None).unwrap()
            }
            Strategy::LinearSearch { start, delta } => {
                let start_f = self.band_doubling_params(start, a, b, &nw).0;
                let mut fronts = self.front.new(trace, a, b, &self.cm);
                linear_search(start_f, delta as Cost, |s| {
                    nw.align_for_bounded_dist(Some(s), trace, Some(&mut fronts))
                        .map(|x @ (c, _)| (c, x))
                })
                .1
            }
        };
        nw.v.last_frame(cigar.as_ref(), None, nw.domain.h());
        assert!(h0 <= cost, "Heuristic at start {h0} > final cost {cost}.");
        (cost, cigar)
    }

    pub fn cost(&self, a: Seq, b: Seq) -> Cost {
        self.cost_or_align(a, b, false).0
    }

    pub fn align(&self, a: Seq, b: Seq) -> (Cost, Option<AffineCigar>) {
        let (cost, cigar) = self.cost_or_align(a, b, self.trace);
        (cost, cigar)
    }

    pub fn cost_for_bounded_dist(&self, a: Seq, b: Seq, f_max: Cost) -> Option<Cost> {
        self.build(a, b)
            .align_for_bounded_dist(Some(f_max), false, None)
            .map(|c| c.0)
    }

    pub fn align_for_bounded_dist(
        &self,
        a: Seq,
        b: Seq,
        f_max: Cost,
    ) -> Option<(Cost, AffineCigar)> {
        self.build(a, b)
            .align_for_bounded_dist(Some(f_max), true, None)
            .map(|(c, cigar)| (c, cigar.unwrap()))
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic, F: NwFrontsTag<N>> AffineAligner
    for NW<N, V, H, F>
{
    fn align_affine(&mut self, a: Seq, b: Seq) -> (Cost, Option<AffineCigar>) {
        self.cost_or_align(a, b, true)
    }
}

impl<V: VisualizerT, H: Heuristic, F: NwFrontsTag<0>> Aligner for NW<0, V, H, F> {
    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Option<Cigar>) {
        let (cost, cigar) = NW::align(self, a, b);
        (cost, cigar.map(|c| c.into()))
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic, F: NwFrontsTag<N>> std::fmt::Debug
    for NW<N, V, H, F>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NW").field("domain", &self.domain).finish()
    }
}

pub struct NWInstance<'a, const N: usize, V: VisualizerT, H: Heuristic, F: NwFrontsTag<N>> {
    // NOTE: `a` and `b` are padded sequences and hence owned.
    a: Seq<'a>,
    b: Seq<'a>,

    params: &'a NW<N, V, H, F>,

    /// The instantiated heuristic to use.
    domain: Domain<H::Instance<'a>>,

    /// Hint for the heuristic, cached between `j_range` calls.
    hint: <H::Instance<'a> as HeuristicInstance<'a>>::Hint,

    /// The instantiated visualizer to use.
    v: V::Instance,
}

impl<'a, const N: usize, V: VisualizerT, H: Heuristic, F: NwFrontsTag<N>> Drop
    for NWInstance<'a, N, V, H, F>
{
    fn drop(&mut self) {
        if PRINT {
            if let Domain::Astar(h) = &mut self.domain {
                eprintln!("h0 end: {}", h.h(Pos(0, 0)));
            }
        }
    }
}

impl<'a, const N: usize, V: VisualizerT, H: Heuristic, F: NwFrontsTag<N>>
    NWInstance<'a, N, V, H, F>
{
    /// The range of rows `j` to consider for columns `i_range.0 .. i_range.1`, when the cost is bounded by `f_bound`.
    ///
    /// For A*, this also returns the range of rows in column `i_range.0` that are 'fixed', ie have `f <= f_max`.
    /// TODO: We could actually also return such a range in non-A* cases.
    ///
    /// `i_range`: `[start, end)` range of characters of `a` to process. Ends with column `end` of the DP matrix.
    /// Pass `-1..0` for the range of the first column. `prev` is not used.
    /// Pass `i..i+1` to move 1 front, with `prev` the front for column `i`,
    /// Pass `i..i+W` to compute a block of `W` columns `i .. i+W`.
    ///
    ///
    /// `old_range`: The old j_range at the end of the current interval, to ensure it only grows.
    ///
    /// ALG: We must continue from the old_j_range to ensure things work well after pruning:
    /// Pruning is only allowed if we guarantee that the range never shrinks,
    /// and it can happen that we 'run out' of `f(u) <= f_max` states inside the
    /// `old_range`, while extending the `old_range` from the bottom could grow
    /// more.
    fn j_range(
        &mut self,
        i_range: IRange,
        f_max: Option<Cost>,
        prev: &<F::Fronts<'a> as NwFronts<N>>::Front,
        old_range: Option<JRange>,
    ) -> JRange {
        // Without a bound on the distance, we can only return the full range.
        let Some(f_max) = f_max else {
            return JRange(0, self.b.len() as I);
        };

        // Inclusive start column of the new block.
        let is = i_range.0;
        // Inclusive end column of the new block.
        let ie = i_range.1;

        match &self.domain {
            Domain::Full => JRange(0, self.b.len() as I),
            Domain::GapStart => {
                // range: the max number of diagonals we can move up/down from the start with cost f.
                let range = JRange(
                    -(self.params.cm.max_del_for_cost(f_max) as I),
                    self.params.cm.max_ins_for_cost(f_max) as I,
                );
                // crop
                JRange(
                    max(is + 1 + range.0, 0),
                    min(ie + range.1, self.b.len() as I),
                )
            }
            Domain::GapGap => {
                let d = self.b.len() as I - self.a.len() as I;
                // We subtract the cost needed to bridge the gap from the start to the end.
                let s = f_max
                    - self
                        .params
                        .cm
                        .gap_cost(Pos(0, 0), Pos::target(&self.a, &self.b));
                // Each extra diagonal costs one insertion and one deletion.
                let extra_diagonals =
                    s / (self.params.cm.min_ins_extend + self.params.cm.min_del_extend);
                // NOTE: The range could be reduced slightly further by considering gap open costs.
                let range = JRange(
                    min(d, 0) - extra_diagonals as I,
                    max(d, 0) + extra_diagonals as I,
                );

                // crop
                JRange(
                    max(is + 1 + range.0, 0),
                    min(ie + range.1, self.b.len() as I),
                )
            }
            Domain::Astar(h) => {
                // Get the range of rows with fixed states `f(u) <= f_max`.
                let JRange(mut fixed_start, mut fixed_end) = if i_range.1 == 0 {
                    JRange(-1, -1)
                } else {
                    prev.fixed_j_range()
                        .expect("With A* Domain, fixed_j_range should always be set.")
                };

                if PRINT {
                    eprintln!("j_range for {i_range:?}\t\told {old_range:?}\t\t fixed @ {is}\t {fixed_start}..{fixed_end}");
                }

                // Early return for empty range.
                if fixed_start > fixed_end {
                    return JRange(fixed_start, fixed_end);
                }

                if let Some(old_range) = old_range {
                    fixed_start = min(fixed_start, old_range.0);
                    fixed_end = max(fixed_end, old_range.1);
                }

                // The start of the j_range we will compute for this block is the `fixed_start` of the previous column.
                // The end of the j_range is extrapolated from `fixed_end`.

                // `u` is the bottom most fixed state in prev col.
                let u = Pos(is, fixed_end);
                let gu = if is < 0 { 0 } else { prev.index(fixed_end) };
                // in the end, `v` will be the bottom most state in column
                // i_range.1 that could possibly have `f(v) <= f_max`.
                let mut v = u;

                // Wrapper to use h with hint.
                let mut h = |pos| {
                    let (h, new_hint) = h.h_with_hint(pos, self.hint);
                    self.hint = new_hint;
                    self.v.h_call(pos);
                    h
                };
                // A lower bound of `f` values estimated from `gu`, valid for states `v` below the diagonal of `u`.
                let mut f = |v: Pos| {
                    assert!(v.1 - u.1 >= v.0 - u.0);
                    // eprintln!("f({})", v);
                    gu + self.params.cm.extend_cost(u, v) + h(v)
                };

                // Extend `v` diagonally one column at a time towards `ie`.
                // In each column, find the lowest `v` such that
                // `f(v) = g(v) + h(v) <= gu + extend_cost(u, v) + h(v) <= s`.
                //
                // NOTE: We can not directly go to the last column, since
                // the optimal path could then 'escape' through the bottom.
                // Without further reasoning, we must evaluate `h` at least
                // once per column.

                if self.params.sparse_h {
                    v += Pos(1, 1);
                    // ALG:
                    // First go down by block size.
                    // (This is important; f doesn't work or `v` above the diagonal of `u`.)
                    // Then, go right, until in-scope using exponential steps.
                    // Then down until out-of-scope.
                    // Repeat.
                    // In the end, go up to in-scope.
                    // NOTE: We add a small additional buffer to prevent doing v.1 += 1 in the loop below.
                    v.1 += self.params.block_width + 8;
                    v.1 = min(v.1, self.b.len() as I);
                    while v.0 <= ie && v.1 < self.b.len() as I {
                        let fv = f(v);
                        if fv <= f_max {
                            // TODO: Make this number larger. Outside the scope,
                            // we can make bigger jumps.
                            v.1 += 1;
                        } else {
                            v.0 += (fv - f_max).div_ceil(2 * self.params.cm.min_del_extend);
                        }
                    }
                    v.0 = ie;
                    loop {
                        // Stop in the edge case where `f(v)` would be invalid (`v.1<0`)
                        // or when the bottom of the grid was reached, in which
                        // case `v` may not be below the diagonal of `u`, and
                        // simply computing everything won't loose much anyway.
                        if v.1 < 0 || v.1 == self.b.len() as I {
                            break;
                        }
                        let fv = f(v);
                        if fv <= f_max {
                            break;
                        } else {
                            v.1 -= (fv - f_max).div_ceil(2 * self.params.cm.min_ins_extend);
                            // Don't go above the diagonal.
                            // This could happen after pruning we if don't check explicitly.
                            if v.1 < v.0 - u.0 + u.1 {
                                v.1 = v.0 - u.0 + u.1;
                                break;
                            }
                        }
                    }
                } else {
                    while v.0 < ie {
                        // Extend diagonally.
                        v += Pos(1, 1);

                        // Check if cell below is out-of-reach.
                        v.1 += 1;
                        while v.1 <= self.b.len() as I && f(v) <= f_max {
                            v.1 += 1;
                        }
                        v.1 -= 1;
                    }
                }
                JRange(max(fixed_start, 0), min(v.1, self.b.len() as I))
            }
        }
    }

    /// Compute the j_range of `front` `i` with `f(u) <= f_max`.
    /// BUG: This should take into account potential non-consistency of `h`.
    /// In particular, with inexact matches, we can only fix states with `f(u) <= f_max - r`.
    fn fixed_j_range(
        &mut self,
        i: I,
        f_max: Option<Cost>,
        front: &<F::Fronts<'a> as NwFronts<N>>::Front,
    ) -> Option<JRange> {
        let Domain::Astar(h) = &self.domain else {
            return None;
        };
        let Some(f_max) = f_max else {
            return None;
        };

        // Wrapper to use h with hint.
        let mut h = |pos| {
            let (h, new_hint) = h.h_with_hint(pos, self.hint);
            self.hint = new_hint;
            h
        };
        let mut f = |j| front.index(j) + h(Pos(i, j));

        // Start: increment the start of the range until f<=f_max is satisfied.
        // End: decrement the end of the range until f<=f_max is satisfied.
        //
        // ALG: Sparse h-calls:
        // Set u = (i, start), and compute f(u).
        // For v = (i, j), (j>start) we have
        // - g(v) >= g(u) - (j - start), by triangle inequality
        // - h(u) <= (j - start) + h(v), by 'column-wise-consistency'
        // => f(u) = g(u) + h(u) <= g(v) + h(v) + 2*(j - start) = f(v) + 2*(j - start)
        // => f(v) >= f(u) - 2*(j - start)
        // We want f(v) <= f_max, so we can stop when f(u) - 2*(j - start) <= f_max, ie
        // j >= start + (f(u) - f_max) / 2
        // Thus, both for increasing `start` and decreasing `end`, we can jump ahead if the difference is too large.
        let mut start = front.j_range().0;
        let mut end = front.j_range().1;
        while start <= end {
            let f = f(start);
            if f <= f_max {
                break;
            }
            start += if self.params.sparse_h {
                // TODO: Increase by steps of 64.
                (f - f_max).div_ceil(2 * self.params.cm.min_ins_extend)
            } else {
                1
            };
        }

        while end >= start {
            let f = f(end);
            if f <= f_max {
                break;
            }
            end -= if self.params.sparse_h {
                // TODO: Decrease by steps of 64.
                (f - f_max).div_ceil(2 * self.params.cm.min_ins_extend)
            } else {
                1
            };
        }
        Some(JRange(start, end))
    }

    /// Test whether the cost is at most s.
    /// Returns None if no path was found.
    /// It may happen that a path is found, but the cost is larger than s.
    /// In this case no cigar is returned.
    /// TODO: Reuse fronts between iterations.
    fn align_for_bounded_dist(
        &mut self,
        f_max: Option<Cost>,
        trace: bool,
        fronts: Option<&mut F::Fronts<'a>>,
    ) -> Option<(Cost, Option<AffineCigar>)> {
        // Update contours for any pending prunes.
        if self.params.prune
            && let Domain::Astar(h) = &mut self.domain
        {
            h.update_contours(Pos(0, 0));
            if PRINT {
                eprintln!("Test dist {} h0 {}", f_max.unwrap_or(0), h.h(Pos(0, 0)));
            }
        }

        // Make a local front variable if not passed in.
        let mut local_fronts = if fronts.is_none() {
            Some(
                self.params
                    .front
                    .new(trace, self.a, self.b, &self.params.cm),
            )
        } else {
            None
        };
        let fronts = if let Some(fronts) = fronts {
            fronts
        } else {
            local_fronts.as_mut().unwrap()
        };

        assert!(f_max.unwrap_or(0) >= 0);
        let initial_j_range = self.j_range(
            IRange::first_col(),
            f_max,
            &Default::default(),
            fronts.next_front_j_range(),
        );
        if initial_j_range.is_empty() {
            return None;
        }
        // eprintln!("Bound: {f_max:?} {initial_j_range:?}");
        fronts.init(initial_j_range);
        fronts.set_last_front_fixed_j_range(Some(initial_j_range));

        self.v.expand_block(
            Pos(0, fronts.last_front().j_range_rounded().0),
            Pos(1, fronts.last_front().j_range_rounded().len()),
            0,
            f_max.unwrap_or(0),
            self.domain.h(),
        );

        let mut all_fronts_reused = true;

        for i in (0..self.a.len() as I).step_by(self.params.block_width as _) {
            let i_range = IRange(i, min(i + self.params.block_width, self.a.len() as I));
            let mut j_range = self.j_range(
                i_range,
                f_max,
                fronts.last_front(),
                fronts.next_front_j_range(),
            );
            if j_range.is_empty() && fronts.next_front_j_range().is_none() {
                // eprintln!("Empty range at i {i}");
                self.v.new_layer(self.domain.h());
                return None;
            }
            let mut reuse = false;
            if let Some(old_j_range) = fronts.next_front_j_range() {
                j_range = JRange(min(j_range.0, old_j_range.0), max(j_range.1, old_j_range.1));
                // If this front doesn't grow, and previous fronts also didn't grow, reuse this front.
                if all_fronts_reused && j_range == old_j_range {
                    reuse = true;
                }
            }
            all_fronts_reused &= reuse;
            let prev_fixed_j_range = fronts.last_front().fixed_j_range();
            // eprintln!("{i}: Prev fixed range {prev_fixed_j_range:?}");
            if reuse {
                // eprintln!("{i}: Reuse block for {i_range:?} x {j_range:?}");
                fronts.reuse_next_block(i_range, j_range);
            } else {
                // eprintln!("{i}: compute block {i_range:?} {j_range:?}");
                fronts.compute_next_block(i_range, j_range, &mut self.v);
                if self.params.strategy == Strategy::None {
                    self.v.new_layer(self.domain.h());
                }
            }
            // Compute the range of fixed states.
            let next_fixed_j_range = self.fixed_j_range(i_range.1, f_max, fronts.last_front());
            // if PRINT {
            //     eprintln!("{i}: New fixed range {next_fixed_j_range:?}");
            // }
            fronts.set_last_front_fixed_j_range(next_fixed_j_range);
            let next_fixed_j_range = fronts.last_front().fixed_j_range();

            // Prune matches in the fixed range.
            if self.params.prune
                && let Domain::Astar(h) = &mut self.domain
                && let Some(prev_fixed_j_range) = prev_fixed_j_range
                && let Some(next_fixed_j_range) = next_fixed_j_range
            {
                let fixed_j_range = max(prev_fixed_j_range.0, next_fixed_j_range.0)
                    ..min(prev_fixed_j_range.1, next_fixed_j_range.1);
                if !fixed_j_range.is_empty() {
                    h.prune_block(i_range.0..i_range.1, fixed_j_range);
                }
            }

            // Only draw a new expanded block if the front was actually recomputed.
            if !reuse {}
        }
        self.v.new_layer(self.domain.h());

        let Some(dist) = fronts.last_front().get(self.b.len() as I) else {
            return None;
        };
        if trace && dist <= f_max.unwrap_or(I::MAX) {
            let cigar = fronts.trace(
                self.a,
                self.b,
                State {
                    i: 0,
                    j: 0,
                    layer: None,
                },
                State {
                    i: self.a.len() as I,
                    j: self.b.len() as I,
                    layer: None,
                },
                &mut self.v,
            );
            Some((dist, Some(cigar)))
        } else {
            Some((dist, None))
        }
    }

    pub fn local_doubling(&mut self) -> (Cost, AffineCigar) {
        let h = self.domain.h().unwrap();
        let h0 = h.h(Pos(0, 0));

        // For block-width B:
        // idx 0: i_range 0 .. 0
        // idx i: i_range (B-1)*i .. B*i
        // idx max: i_range (B-1)*max .. a.len()
        let mut fronts = self.params.front.new(true, self.a, self.b, &self.params.cm);

        // Add the front for i_range 0..0
        {
            let initial_j_range = self.j_range(
                IRange::first_col(),
                Some(h0),
                &Default::default(),
                fronts.next_front_j_range(),
            );
            fronts.init(initial_j_range);
            fronts.set_last_front_fixed_j_range(Some(initial_j_range));
        }

        // Fronts have been computed up to this f.
        // TODO: Move f_max and f_delta into the front datastructure.
        let mut f_max = vec![h0];

        // Each time a front is grown, it grows to the least multiple of delta that is large enough.
        // Delta doubles after each grow.
        // TODO: Make this customizable.
        type Delta = (Cost, u8);
        let delta0 = (self.params.block_width * 2, 0);
        let delta_growth = 2;
        let mut f_delta = vec![delta0];

        // The end of the current front.
        let mut i = 0;
        // The index into f_max and f_delta of the current front.
        let mut last_idx = 0;

        let update_delta = |delta: &mut Delta| match delta.1 {
            0 => delta.1 += 1,
            1 => {
                delta.0 *= delta_growth;
                delta.0 = min(delta.0, 4 * 1024);
                delta.1 = 0;
            }
            _ => panic!(),
        };
        let grow_to = |f: &mut Cost, f_target: Cost, delta: &mut Delta| {
            // *f = max(*f + *delta, f_target);
            *f = (f_target).next_multiple_of(delta.0);
            assert!(*f >= f_target);
            update_delta(delta);
            // eprintln!("Grow front idx {start_idx} to f {}", f_max[start_idx]);
        };

        // This is a for loop over `i`, but once `i` reaches `a.len()`, the last
        // front is grown instead of increasing `i`.
        loop {
            if fronts.last_front().fixed_j_range().unwrap().is_empty() {
                // Fixed_j_range is empty; grow last front.
                let delta = &mut f_delta[last_idx];
                f_max[last_idx] = (f_max[last_idx] + 1).next_multiple_of(delta.0);
                update_delta(delta);
                // eprintln!("Grow last front idx {last_idx} f {}", f_max[last_idx]);
                fronts.pop_last_front();
            } else if i < self.a.len() as I {
                let i_range = IRange(i, min(i + self.params.block_width, self.a.len() as I));

                // The value of f at the tip. When going to the next front, this is
                // incremented until the range is non-empty.
                let mut next_f = f_max[last_idx];
                // Add a new front.
                loop {
                    let j_range = self.j_range(
                        i_range,
                        Some(next_f),
                        fronts.last_front(),
                        fronts.next_front_j_range(),
                    );
                    if !j_range.is_empty() {
                        break;
                    }
                    // TODO: Make the growth of f_tip customizable.
                    next_f += self.params.block_width;
                    // eprintln!("Grow next_f to {next_f}");
                }
                i = i_range.1;
                last_idx += 1;
                f_max.push(next_f);
                f_delta.push(delta0);
                assert!(f_max.len() == last_idx + 1);
                assert!(f_delta.len() == last_idx + 1);
                // eprintln!(
                // "Push new front idx {last_idx} i {i_range:?} f {}",
                // f_max[last_idx]
                // );
            } else {
                // Grow the last front.
                let f = &mut f_max[last_idx];
                let f_target = *f + 1;
                grow_to(f, f_target, &mut f_delta[last_idx]);
                // eprintln!("Grow last front idx {last_idx} f {}", f_max[last_idx]);
                fronts.pop_last_front();
            }

            // Grow previous front sizes as long as their f_max is not large enough.
            let mut start_idx = last_idx;
            let mut last_grow = 0;
            while start_idx > 0 && f_max[start_idx - 1] < f_max[start_idx] {
                start_idx -= 1;

                let f_target = f_max[start_idx + 1];
                let old_f = f_max[start_idx];
                let old_delta = f_delta[start_idx];
                grow_to(&mut f_max[start_idx], f_target, &mut f_delta[start_idx]);
                if f_max[start_idx] > last_grow {
                    if PRINT {
                        eprintln!(
                            "Grow  front idx {start_idx:>5} to {:>6} by {:>6} for {old_delta:>5?} and shortage {:>6}",
                            f_max[start_idx],
                            f_max[start_idx] - old_f,
                            f_target - old_f
                        );
                    }
                    last_grow = f_max[start_idx];
                }

                fronts.pop_last_front();
            }

            if start_idx < last_idx {
                if PRINT {
                    eprintln!("START front idx {start_idx:>5} to {:>6}", f_max[start_idx]);
                }
                let h = self.domain.h_mut().unwrap();
                h.update_contours(Pos((start_idx as I - 1) * self.params.block_width, 0));
            }

            if start_idx == 0 {
                let initial_j_range = self.j_range(
                    IRange::first_col(),
                    Some(h0),
                    &Default::default(),
                    fronts.next_front_j_range(),
                );
                fronts.init(initial_j_range);
                fronts.set_last_front_fixed_j_range(Some(initial_j_range));
                // eprintln!("Reset front idx 0 to {initial_j_range:?}");

                start_idx += 1;
            }

            // Recompute all fronts from start_idx upwards for their new f_max.
            // As long as j_range doesn't grow, existing results are reused.
            let mut all_fronts_reused = true;
            for idx in start_idx..=last_idx {
                // eprintln!("Compute front idx {}", idx);
                let f_max = Some(f_max[idx]);

                let i_range = IRange(
                    (idx as I - 1) * self.params.block_width,
                    min(idx as I * self.params.block_width, self.a.len() as I),
                );
                let mut j_range = self.j_range(
                    i_range,
                    f_max,
                    fronts.last_front(),
                    fronts.next_front_j_range(),
                );
                assert!(!j_range.is_empty());

                let mut reuse = false;
                if let Some(old_j_range) = fronts.next_front_j_range() {
                    j_range = JRange(min(j_range.0, old_j_range.0), max(j_range.1, old_j_range.1));
                    // If this front doesn't grow, and previous fronts also didn't grow, reuse this front.
                    if all_fronts_reused && j_range == old_j_range {
                        reuse = true;
                    }
                }
                all_fronts_reused &= reuse;

                let prev_fixed_j_range = fronts.last_front().fixed_j_range().unwrap();
                if reuse {
                    // eprintln!("Reuse   front idx {idx} i {i_range:?} j {j_range:?} f {f_max:?}");
                    fronts.reuse_next_block(i_range, j_range);
                } else {
                    // eprintln!("Compute front idx {idx} i {i_range:?} j {j_range:?} f {f_max:?}");
                    fronts.compute_next_block(i_range, j_range, &mut self.v);
                }
                // Compute the range of fixed states.
                let next_fixed_j_range = self.fixed_j_range(i_range.1, f_max, fronts.last_front());
                // eprintln!("{i}: New fixed range {next_fixed_j_range:?}");
                fronts.set_last_front_fixed_j_range(next_fixed_j_range);
                let next_fixed_j_range = fronts.last_front().fixed_j_range().unwrap();

                // eprintln!("Prune matches..");

                // Prune matches in the fixed range.
                let fixed_j_range = max(prev_fixed_j_range.0, next_fixed_j_range.0)
                    ..min(prev_fixed_j_range.1, next_fixed_j_range.1);
                if !fixed_j_range.is_empty() {
                    let h = self.domain.h_mut().unwrap();
                    h.prune_block(i_range.0..i_range.1, fixed_j_range);
                }
                // eprintln!("Prune matches done");
            }

            self.v.new_layer(self.domain.h());
            if i == self.a.len() as I && fronts[last_idx].j_range().contains(self.b.len() as I) {
                break;
            }
        } // end loop over i

        if PRINT {
            let mut delta = 0;
            for (idx, d) in f_delta.iter().enumerate() {
                if delta != d.0 {
                    delta = d.0;
                    eprintln!("Delta {idx:>6} => {delta:>5}");
                }
            }
        }

        // eprintln!("TRACE..");
        let dist = fronts.last_front().get(self.b.len() as I).unwrap();
        let cigar = fronts.trace(
            self.a,
            self.b,
            State {
                i: 0,
                j: 0,
                layer: None,
            },
            State {
                i: self.a.len() as I,
                j: self.b.len() as I,
                layer: None,
            },
            &mut self.v,
        );
        (dist, cigar)
    }
}

#[cfg(test)]
mod test {
    use pa_affine_types::AffineCost;
    use pa_heuristic::{MatchConfig, Pruning, GCSH};
    use pa_vis::NoVis;

    use crate::{Domain, Strategy};

    use super::{BitFront, NW};

    #[test]
    fn nw() {
        let (a, b) =
            pa_generate::generate_model(10000, 0.1, pa_generate::ErrorModel::Uniform, 31415);
        let d = NW {
            cm: AffineCost::unit(),
            strategy: Strategy::band_doubling(),
            domain: Domain::Astar(GCSH::new(MatchConfig::exact(15), Pruning::start())),
            block_width: 256,
            v: NoVis,
            front: BitFront::default(),
            trace: true,
            sparse_h: true,
            prune: false,
        }
        .align(&a, &b)
        .0;
        let d2 = triple_accel::levenshtein_exp(&a, &b) as _;
        assert_eq!(d, d2);
    }

    #[test]
    fn nw_prune() {
        let (a, b) =
            pa_generate::generate_model(10000, 0.1, pa_generate::ErrorModel::Uniform, 31415);
        let d = NW {
            cm: AffineCost::unit(),
            strategy: Strategy::band_doubling(),
            domain: Domain::Astar(GCSH::new(MatchConfig::exact(15), Pruning::start())),
            block_width: 256,
            v: NoVis,
            front: BitFront::default(),
            trace: true,
            sparse_h: true,
            prune: true,
        }
        .align(&a, &b)
        .0;
        let d2 = triple_accel::levenshtein_exp(&a, &b) as _;
        assert_eq!(d, d2);
    }

    #[test]
    fn local_doubling() {
        let (a, b) =
            pa_generate::generate_model(10000, 0.1, pa_generate::ErrorModel::Uniform, 31415);
        let d = NW {
            cm: AffineCost::unit(),
            strategy: Strategy::LocalDoubling,
            domain: Domain::Astar(GCSH::new(MatchConfig::exact(15), Pruning::start())),
            block_width: 256,
            v: NoVis,
            front: BitFront::default(),
            trace: true,
            sparse_h: true,
            prune: true,
        }
        .align(&a, &b)
        .0;
        let d2 = triple_accel::levenshtein_exp(&a, &b) as _;
        assert_eq!(d, d2);
    }

    #[test]
    fn dt_trace() {
        let (a, b) =
            pa_generate::generate_model(10000, 0.1, pa_generate::ErrorModel::Uniform, 31415);
        let d = NW {
            cm: AffineCost::unit(),
            strategy: Strategy::LocalDoubling,
            domain: Domain::Astar(GCSH::new(MatchConfig::exact(15), Pruning::start())),
            block_width: 256,
            v: NoVis,
            front: {
                let mut f = BitFront::default();
                f.dt_trace = true;
                f
            },
            trace: true,
            sparse_h: true,
            prune: true,
        }
        .align(&a, &b)
        .0;
        let d2 = triple_accel::levenshtein_exp(&a, &b) as _;
        assert_eq!(d, d2);
    }
}
