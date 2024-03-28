// TODO
// - Store block of blocks in a single allocation. Update `NwBlock` to contain multiple columns as once and be reusable.
// - timings
// - meet in the middle with A* and pruning on both sides
// - try jemalloc/mimalloc
// - Matches:
//   - Recursively merge matches to find r=2^k matches.
//     - possibly reduce until no more spurious matches
//     - tricky: requires many 'shadow' matches. Handle in cleaner way?
//  - Figure out why pruning up to Layer::MAX gives errors, but pruning up to highest_modified_contour does not.
// - QgramIndex for short k.
// - Analyze local doubling better
// - Speed up j_range more???
// BUG: Figure out why the delta=64 is broken in fixed_j_range.
mod local_doubling;

use self::blocks::{trace::TraceStats, BlockStats};

use super::*;
use crate::{block::Block, blocks::Blocks};
use pa_affine_types::AffineCost;
use pa_heuristic::*;
use pa_types::*;
use pa_vis::*;
use std::{
    cmp::{max, min},
    time::Duration,
};
use Domain::*;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct AstarPa2Stats {
    pub block_stats: BlockStats,
    pub trace_stats: TraceStats,

    pub f_max_tries: usize,

    pub t_precomp: Duration,
    pub t_j_range: Duration,
    pub t_fixed_j_range: Duration,
    pub t_pruning: Duration,
    pub t_contours_update: Duration,
}

pub struct AstarPa2Instance<'a, V: VisualizerT, H: Heuristic> {
    // NOTE: `a` and `b` are padded sequences and hence owned.
    pub a: Seq<'a>,
    pub b: Seq<'a>,

    pub params: &'a AstarPa2<V, H>,

    /// The instantiated heuristic to use.
    pub domain: Domain<H::Instance<'a>>,

    /// Hint for the heuristic, cached between `j_range` calls.
    pub hint: <H::Instance<'a> as HeuristicInstance<'a>>::Hint,

    /// The instantiated visualizer to use.
    pub v: V::Instance,

    pub stats: AstarPa2Stats,
}

impl<'a, V: VisualizerT, H: Heuristic> AstarPa2Instance<'a, V, H> {
    /// The range of rows `j` to consider for columns `i_range.0 .. i_range.1`, when the cost is bounded by `f_bound`.
    ///
    /// For A*, this also returns the range of rows in column `i_range.0` that are 'fixed', ie have `f <= f_max`.
    /// TODO: We could actually also return such a range in non-A* cases.
    ///
    /// `i_range`: `[start, end)` range of characters of `a` to process. Ends with column `end` of the DP matrix.
    /// Pass `-1..0` for the range of the first column. `prev` is not used.
    /// Pass `i..i+1` to move 1 block, with `prev` the block for column `i`,
    /// Pass `i..i+W` to compute a block of `W` columns `i .. i+W`.
    ///
    ///
    /// `old_range`: The old j_range at the end of the current interval, to ensure it only grows.
    fn j_range(
        &mut self,
        i_range: IRange,
        f_max: Option<Cost>,
        prev: &Block,
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

        let unit_cost = AffineCost::unit();

        let mut range = match &self.domain {
            Full => JRange(0, self.b.len() as I),
            GapStart => {
                // range: the max number of diagonals we can move up/down from the start with cost f.
                JRange(
                    is + 1 + -(unit_cost.max_del_for_cost(f_max) as I),
                    ie + unit_cost.max_ins_for_cost(f_max) as I,
                )
            }
            GapGap => {
                let d = self.b.len() as I - self.a.len() as I;
                // We subtract the cost needed to bridge the gap from the start to the end.
                let s = f_max - unit_cost.gap_cost(Pos(0, 0), Pos::target(&self.a, &self.b));
                // Each extra diagonal costs one insertion and one deletion.
                let extra_diagonals = s / (unit_cost.min_ins_extend + unit_cost.min_del_extend);
                // NOTE: The range could be reduced slightly further by considering gap open costs.
                JRange(
                    is + 1 + min(d, 0) - extra_diagonals as I,
                    ie + max(d, 0) + extra_diagonals as I,
                )
            }
            Astar(h) => {
                let t_start = std::time::Instant::now();
                let stats = &mut self.stats;
                scopeguard::defer! {
                    stats.t_j_range += t_start.elapsed();
                }
                // TODO FIXME Return already-rounded jrange. More precision isn't needed, and this will save some time.

                // Get the range of rows with fixed states `f(u) <= f_max`.
                let JRange(fixed_start, fixed_end) = prev
                    .fixed_j_range
                    .expect("With A* Domain, fixed_j_range should always be set.");
                if DEBUG {
                    eprintln!("j_range for   {i_range:?}");
                    eprintln!("\told j_range {old_range:?}");
                    eprintln!("\told fixed   {:?} @ {is}", prev.fixed_j_range.unwrap());
                }
                assert!(fixed_start <= fixed_end, "Fixed range must not be empty");

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
                    h
                };
                // A lower bound of `f` values estimated from `gu`, valid for states `v` below the diagonal of `u`.
                let mut f = |v: Pos| {
                    assert!(v.1 - u.1 >= v.0 - u.0);
                    let f = gu + unit_cost.extend_cost(u, v) + h(v);
                    self.v.f_call(v, f <= f_max, false);
                    f
                };

                // Extend `v` diagonally one column at a time towards `ie`.
                // In each column, find the lowest `v` such that
                // `f(v) = g(v) + h(v) <= gu + extend_cost(u, v) + h(v) <= s`.
                //
                // NOTE: We can not directly go to the last column, since
                // the optimal path could then 'escape' through the bottom.
                // Without further reasoning, we must evaluate `h` at least
                // once per column.

                if !self.params.sparse_h {
                    while v.0 < ie {
                        // Extend diagonally.
                        v += Pos(1, 1);

                        // Extend down while cell below is in-reach.
                        v.1 += 1;
                        while v.1 <= self.b.len() as I && f(v) <= f_max {
                            v.1 += 1;
                        }
                        v.1 -= 1;
                    }
                } else {
                    // FIXME: Can we drop this??
                    v += Pos(1, 1);
                    // ALG:
                    // First go down by block width, anticipating that extending diagonally will not increase f.
                    // (This is important; f doesn't work for `v` above the diagonal of `u`.)
                    // Then repeat:
                    // - Go right until in-scope using exponential steps.
                    // - Go down until out-of-scope using steps of size 8.
                    // Finally, go up to in-scope.
                    // NOTE: We start with a small additional buffer to prevent doing v.1 += 1 in the loop below.
                    v.1 += self.params.block_width;
                    v.1 = min(v.1, self.b.len() as I);
                    loop {
                        // Don't go above the diagonal.
                        if v.1 < v.0 - u.0 + u.1 {
                            v.1 = v.0 - u.0 + u.1;
                            break;
                        }
                        let fv = f(v);
                        if fv <= f_max {
                            if v.1 == self.b.len() as I {
                                break;
                            }
                            v.1 += 8;
                            if v.1 >= self.b.len() as I {
                                v.1 = self.b.len() as I;
                            }
                        } else {
                            // By consistency of `f`, it can only change value by at most `2` per step in the unit cost setting.
                            // When `f(v) > f_max`, this means we have to make at least `ceil((fv - f_max)/2)` steps to possibly get at a cell with `f(v) <= f_max`.
                            v.0 += (fv - f_max).div_ceil(2 * unit_cost.min_del_extend);
                            if v.0 > ie {
                                v.0 = ie;
                                break;
                            }
                        }
                    }
                    v.0 = ie;
                    loop {
                        // Don't go above the diagonal.
                        if v.1 < v.0 - u.0 + u.1 {
                            v.1 = v.0 - u.0 + u.1;
                            break;
                        }
                        let fv = f(v);
                        if fv <= f_max {
                            break;
                        } else {
                            v.1 -= (fv - f_max).div_ceil(2 * unit_cost.min_ins_extend);
                        }
                    }
                }
                JRange(fixed_start, v.1)
            }
        };
        // Size at least old_range.
        if let Some(old_range) = old_range {
            range = range.union(old_range);
        }
        // crop
        let j_range = range.intersection(JRange(0, self.b.len() as I));

        self.v.j_range(Pos(is, j_range.0), Pos(ie, j_range.1));
        j_range
    }

    /// Compute the j_range of `block` `i` with `f(u) <= f_max`.
    /// BUG: This should take into account potential non-consistency of `h`.
    /// In particular, with inexact matches, we can only fix states with `f(u) <= f_max - r`.
    fn fixed_j_range(
        &mut self,
        i: I,
        f_max: Option<Cost>,
        prev_fixed_j_range: Option<JRange>,
        block: &Block,
    ) -> Option<JRange> {
        let Astar(h) = &self.domain else {
            return None;
        };
        let Some(f_max) = f_max else {
            return None;
        };

        let t_start = std::time::Instant::now();
        let stats = &mut self.stats;
        scopeguard::defer! {
            stats.t_fixed_j_range += t_start.elapsed();
        }

        // Wrapper to use h with hint.
        let mut h = |pos| {
            let (h, new_hint) = h.h_with_hint(pos, self.hint);
            self.hint = new_hint;
            h
        };

        // Compute values at the end of each lane.
        let mut f = |j| {
            let f = block.index(j) + h(Pos(i, j));
            self.v.f_call(Pos(i, j), f <= f_max, true);
            f
        };

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
        // TODO: It may be sufficient to only compute this with rounded-to-64 precision.
        let prev_fixed_j_range = prev_fixed_j_range.unwrap();
        assert!(block.j_range.0 <= prev_fixed_j_range.0);
        let mut start = prev_fixed_j_range.0;
        let mut end = block.original_j_range.1.min(self.b.len() as I);

        let unit_cost = AffineCost::unit();

        while start <= end {
            let f = f(start);
            if f <= f_max {
                break;
            }
            start += if self.params.sparse_h {
                (f - f_max).div_ceil(2 * unit_cost.min_ins_extend)
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
                (f - f_max).div_ceil(2 * unit_cost.min_ins_extend)
            } else {
                1
            };
        }
        let mut fixed_j_range = JRange(start, end);
        if DEBUG {
            eprintln!("initial fixed_j_range for {i} {fixed_j_range:?}");
            eprintln!("old     fixed_j_range for {i} {:?}", block.fixed_j_range);
        }
        if let Some(old_fixed_j_range) = block.fixed_j_range {
            if fixed_j_range.is_empty() {
                fixed_j_range = old_fixed_j_range;
            } else {
                fixed_j_range = fixed_j_range.union(old_fixed_j_range);
            }
        }
        if DEBUG {
            eprintln!("updated fixed_j_range for {i} {fixed_j_range:?}");
        }

        if !fixed_j_range.is_empty() {
            self.v
                .fixed_j_range(Pos(i, fixed_j_range.0), Pos(i, fixed_j_range.1));
        }
        Some(fixed_j_range)
    }

    /// Test whether the cost is at most s.
    /// Returns None if no path was found.
    /// It may happen that a path is found, but the cost is larger than s.
    /// In this case no cigar is returned.
    pub fn align_for_bounded_dist(
        &mut self,
        f_max: Option<Cost>,
        trace: bool,
        blocks: Option<&mut Blocks>,
    ) -> Option<(Cost, Option<Cigar>)> {
        self.stats.f_max_tries += 1;

        // Update contours for any pending prunes.
        if self.params.prune
            && let Astar(h) = &mut self.domain
        {
            let start = std::time::Instant::now();
            h.update_contours(Pos(0, 0));
            self.stats.t_contours_update += start.elapsed();
            if DEBUG {
                eprintln!("\nTEST DIST {} h0 {}\n", f_max.unwrap_or(0), h.h(Pos(0, 0)));
            }
        } else {
            if DEBUG {
                eprintln!("\nTEST DIST {}\n", f_max.unwrap_or(0));
            }
        }

        // Make a local block variable if not passed in.
        let mut local_blocks = if blocks.is_none() {
            Some(self.params.block.new(trace, self.a, self.b))
        } else {
            None
        };
        let blocks = if let Some(blocks) = blocks {
            blocks
        } else {
            local_blocks.as_mut().unwrap()
        };

        assert!(f_max.unwrap_or(0) >= 0);

        // Set up initial block for column 0.
        let initial_j_range = self.j_range(
            IRange::first_col(),
            f_max,
            &Block {
                fixed_j_range: Some(JRange(-1, -1)),
                ..Block::default()
            },
            blocks.next_block_j_range(),
        );

        // If 0 is not included in the initial range, no path can be found.
        // This can happen for e.g. the GapGap heuristic when the threshold is too small.
        // Note that the range never shrinks, so even after pruning it should still start at 0.
        if initial_j_range.is_empty() || initial_j_range.0 > 0 {
            return None;
        }

        blocks.init(initial_j_range);
        blocks.set_last_block_fixed_j_range(Some(initial_j_range));

        self.v.expand_block(
            Pos(-1, 0),
            Pos(1, blocks.last_block().j_range.len()),
            0,
            f_max.unwrap_or(0),
            self.domain.h(),
        );

        self.v
            .fixed_j_range(Pos(0, initial_j_range.0), Pos(0, initial_j_range.1));

        let mut all_blocks_reused = true;

        for i in (0..self.a.len() as I).step_by(self.params.block_width as _) {
            // The i_range of the new block.
            let i_range = IRange(i, min(i + self.params.block_width, self.a.len() as I));
            // The j_range of the new block.
            let j_range = self.j_range(
                i_range,
                f_max,
                // The last block is needed to query `g(u)` in the last column.
                blocks.last_block(),
                // An existing `j_range` for a previous iteration may be
                // present, in which case we ensure the `j_range` does not
                // shrink.
                blocks.next_block_j_range(),
            );

            if j_range.is_empty() {
                assert!(blocks.next_block_j_range().is_none());
                self.v.new_layer(self.domain.h());
                return None;
            }

            // If the new `j_range` is the same as the old one, and all previous
            // blocks were reused, we can also reuse this new block.
            let mut reuse = false;
            if blocks.next_block_j_range() == Some(j_range) && all_blocks_reused {
                reuse = true;
            }
            all_blocks_reused &= reuse;

            // Store before appending a new block.
            let prev_fixed_j_range = blocks.last_block().fixed_j_range;

            {
                if let Some(prev_fixed_j_range) = prev_fixed_j_range {
                    let j_h = prev_fixed_j_range.round_in().1;
                    self.v
                        .next_fixed_h(Pos(i_range.0 + 1, j_h), Pos(i_range.1, j_h));
                }
            }

            // Reuse or compute the next block.
            if reuse {
                blocks.reuse_next_block(i_range, j_range);
            } else {
                blocks.compute_next_block(i_range, j_range, &mut self.v);
                if self.params.doubling == DoublingType::None {
                    self.v.new_layer(self.domain.h());
                }
            }

            // Compute the new range of fixed states.
            let next_fixed_j_range =
                self.fixed_j_range(i_range.1, f_max, prev_fixed_j_range, blocks.last_block());

            // If there are no fixed states, break.
            if next_fixed_j_range.is_some_and(|r| r.is_empty()) {
                if DEBUG {
                    eprintln!("fixed_j_range is empty! Increasing f_max!");
                }
                self.v.new_layer(self.domain.h());
                return None;
            }
            blocks.set_last_block_fixed_j_range(next_fixed_j_range);

            // If the stored h_j is actually fixed, draw it.
            {
                if let Some(j_h) = blocks.last_block().j_h
                    && let Some(next_fixed_j_range) = next_fixed_j_range
                    && let Some(prev_fixed_j_range) = prev_fixed_j_range
                {
                    if j_h >= next_fixed_j_range.0 && j_h >= prev_fixed_j_range.0 {
                        self.v.fixed_h(Pos(i_range.0 + 1, j_h), Pos(i_range.1, j_h));
                    }
                }
            }

            // Prune matches in the intersection of the previous and next fixed range.
            if self.params.prune
                && let Astar(h) = &mut self.domain
            {
                let start = std::time::Instant::now();
                let intersection =
                    JRange::intersection(prev_fixed_j_range.unwrap(), next_fixed_j_range.unwrap());
                if !intersection.is_empty() {
                    h.prune_block(i_range.0..i_range.1, intersection.0..intersection.1);
                }
                self.stats.t_pruning += start.elapsed();
            }
        }

        self.v.new_layer(self.domain.h());

        let Some(dist) = blocks.last_block().get(self.b.len() as I) else {
            return None;
        };

        // If dist is at most the assumed bound, do a traceback.
        if trace && dist <= f_max.unwrap_or(I::MAX) {
            let (cigar, trace_stats) = blocks.trace(
                self.a,
                self.b,
                Pos(0, 0),
                Pos(self.a.len() as I, self.b.len() as I),
                &mut self.v,
            );
            self.stats.trace_stats = trace_stats;
            Some((dist, Some(cigar)))
        } else {
            // NOTE: A distance is always returned, even if it is larger than
            // the assumed bound, since this can be used as an upper bound on the
            // distance in further iterations.
            Some((dist, None))
        }
    }
}
