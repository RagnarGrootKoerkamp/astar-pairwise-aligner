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
use super::*;
use crate::{block::Block, blocks::Blocks};
use pa_affine_types::AffineCost;
use pa_heuristic::*;
use pa_types::*;
use pa_vis_types::*;
use std::cmp::{max, min};
use Domain::*;

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
}

impl<V: VisualizerT, H: Heuristic> Aligner for AstarPa2<V, H> {
    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Option<Cigar>) {
        self.cost_or_align(a, b, true)
    }
}

impl<'a, V: VisualizerT, H: Heuristic> Drop for AstarPa2Instance<'a, V, H> {
    fn drop(&mut self) {
        if DEBUG {
            if let Astar(h) = &mut self.domain {
                eprintln!("h0 end: {}", h.h(Pos(0, 0)));
            }
        }
    }
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

        match &self.domain {
            Full => JRange(0, self.b.len() as I),
            GapStart => {
                // range: the max number of diagonals we can move up/down from the start with cost f.
                let range = JRange(
                    -(AffineCost::unit().max_del_for_cost(f_max) as I),
                    AffineCost::unit().max_ins_for_cost(f_max) as I,
                );
                // crop
                JRange(
                    max(is + 1 + range.0, 0),
                    min(ie + range.1, self.b.len() as I),
                )
            }
            GapGap => {
                let d = self.b.len() as I - self.a.len() as I;
                // We subtract the cost needed to bridge the gap from the start to the end.
                let s =
                    f_max - AffineCost::unit().gap_cost(Pos(0, 0), Pos::target(&self.a, &self.b));
                // Each extra diagonal costs one insertion and one deletion.
                let extra_diagonals =
                    s / (AffineCost::unit().min_ins_extend + AffineCost::unit().min_del_extend);
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
            Astar(h) => {
                // Get the range of rows with fixed states `f(u) <= f_max`.
                let JRange(mut fixed_start, mut fixed_end) = if i_range.1 == 0 {
                    JRange(-1, -1)
                } else {
                    *prev
                        .fixed_j_range
                        .as_deref()
                        .expect("With A* Domain, fixed_j_range should always be set.")
                };

                if DEBUG {
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
                    gu + AffineCost::unit().extend_cost(u, v) + h(v)
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
                            v.0 += (fv - f_max).div_ceil(2 * AffineCost::unit().min_del_extend);
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
                            v.1 -= (fv - f_max).div_ceil(2 * AffineCost::unit().min_ins_extend);
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

    /// Compute the j_range of `block` `i` with `f(u) <= f_max`.
    /// BUG: This should take into account potential non-consistency of `h`.
    /// In particular, with inexact matches, we can only fix states with `f(u) <= f_max - r`.
    fn fixed_j_range(&mut self, i: I, f_max: Option<Cost>, block: &Block) -> Option<JRange> {
        let Astar(h) = &self.domain else {
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
        let mut f = |j| block.index(j) + h(Pos(i, j));

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
        let mut start = block.j_range.0;
        let mut end = block.j_range.1;
        while start <= end {
            let f = f(start);
            if f <= f_max {
                break;
            }
            start += if self.params.sparse_h {
                // TODO: Increase by steps of 64.
                (f - f_max).div_ceil(2 * AffineCost::unit().min_ins_extend)
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
                (f - f_max).div_ceil(2 * AffineCost::unit().min_ins_extend)
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
    /// TODO: Reuse blocks between iterations.
    pub fn align_for_bounded_dist(
        &mut self,
        f_max: Option<Cost>,
        trace: bool,
        blocks: Option<&mut Blocks>,
    ) -> Option<(Cost, Option<Cigar>)> {
        // Update contours for any pending prunes.
        if self.params.prune && let Astar(h) = &mut self.domain {
            h.update_contours(Pos(0,0));
            if DEBUG {
                eprintln!("Test dist {} h0 {}", f_max.unwrap_or(0), h.h(Pos(0,0)));
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
        let initial_j_range = self.j_range(
            IRange::first_col(),
            f_max,
            &Block::default(), // unused
            blocks.next_block_j_range(),
        );
        if initial_j_range.is_empty() {
            return None;
        }
        // eprintln!("Bound: {f_max:?} {initial_j_range:?}");
        blocks.init(initial_j_range);
        blocks.set_last_block_fixed_j_range(Some(initial_j_range));

        self.v.expand_block(
            Pos(0, 0),
            Pos(1, blocks.last_block().j_range.len()),
            0,
            f_max.unwrap_or(0),
            self.domain.h(),
        );

        let mut all_blocks_reused = true;

        for i in (0..self.a.len() as I).step_by(self.params.block_width as _) {
            let i_range = IRange(i, min(i + self.params.block_width, self.a.len() as I));
            let mut j_range = self.j_range(
                i_range,
                f_max,
                blocks.last_block(),
                blocks.next_block_j_range(),
            );
            if j_range.is_empty() && blocks.next_block_j_range().is_none() {
                // eprintln!("Empty range at i {i}");
                self.v.new_layer(self.domain.h());
                return None;
            }
            let mut reuse = false;
            if let Some(old_j_range) = blocks.next_block_j_range() {
                j_range = JRange(min(j_range.0, old_j_range.0), max(j_range.1, old_j_range.1));
                // If this block doesn't grow, and previous blocks also didn't grow, reuse this block.
                if all_blocks_reused && j_range == old_j_range {
                    reuse = true;
                }
            }
            all_blocks_reused &= reuse;
            let prev_fixed_j_range = blocks.last_block().fixed_j_range;
            // eprintln!("{i}: Prev fixed range {prev_fixed_j_range:?}");
            if reuse {
                // eprintln!("{i}: Reuse block for {i_range:?} x {j_range:?}");
                blocks.reuse_next_block(i_range, j_range);
            } else {
                // eprintln!("{i}: compute block {i_range:?} {j_range:?}");
                blocks.compute_next_block(i_range, j_range, &mut self.v);
                if self.params.doubling == DoublingType::None {
                    self.v.new_layer(self.domain.h());
                }
            }
            // Compute the range of fixed states.
            let next_fixed_j_range = self.fixed_j_range(i_range.1, f_max, blocks.last_block());
            // if PRINT {
            //     eprintln!("{i}: New fixed range {next_fixed_j_range:?}");
            // }
            blocks.set_last_block_fixed_j_range(next_fixed_j_range);
            let next_fixed_j_range = blocks.last_block().fixed_j_range;

            // Prune matches in the fixed range.
            if self.params.prune
                && let Astar(h) = &mut self.domain
                && let Some(prev_fixed_j_range) = prev_fixed_j_range
                && let Some(next_fixed_j_range) = next_fixed_j_range
            {
                let fixed_j_range = max(prev_fixed_j_range.0, next_fixed_j_range.0)..min(
                    prev_fixed_j_range.1,
                    next_fixed_j_range.1,
                );
                if !fixed_j_range.is_empty() {
                    h.prune_block(i_range.0..i_range.1, fixed_j_range);
                }
            }

            // Only draw a new expanded block if the block was actually recomputed.
            if !reuse {}
        }
        self.v.new_layer(self.domain.h());

        let Some(dist) = blocks.last_block().get(self.b.len() as I) else {
            return None;
        };
        if trace && dist <= f_max.unwrap_or(I::MAX) {
            let cigar = blocks.trace(
                self.a,
                self.b,
                Pos(0, 0),
                Pos(self.a.len() as I, self.b.len() as I),
                &mut self.v,
            );
            Some((dist, Some(cigar)))
        } else {
            Some((dist, None))
        }
    }

    pub fn local_doubling(&mut self) -> (Cost, Cigar) {
        let h = self.domain.h().unwrap();
        let h0 = h.h(Pos(0, 0));

        // For block-width B:
        // idx 0: i_range 0 .. 0
        // idx i: i_range (B-1)*i .. B*i
        // idx max: i_range (B-1)*max .. a.len()
        let mut blocks = self.params.block.new(true, self.a, self.b);

        // Add the block for i_range 0..0
        {
            let initial_j_range = self.j_range(
                IRange::first_col(),
                Some(h0),
                &Default::default(),
                blocks.next_block_j_range(),
            );
            blocks.init(initial_j_range);
            blocks.set_last_block_fixed_j_range(Some(initial_j_range));
        }

        // Blocks have been computed up to this f.
        // TODO: Move f_max and f_delta into the block datastructure.
        let mut f_max = vec![h0];

        // Each time a block is grown, it grows to the least multiple of delta that is large enough.
        // Delta doubles after each grow.
        // TODO: Make this customizable.
        type Delta = (Cost, u8);
        let delta0 = (self.params.block_width * 2, 0);
        let delta_growth = 2;
        let mut f_delta = vec![delta0];

        // The end of the current block.
        let mut i = 0;
        // The index into f_max and f_delta of the current block.
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
            // eprintln!("Grow block idx {start_idx} to f {}", f_max[start_idx]);
        };

        // This is a for loop over `i`, but once `i` reaches `a.len()`, the last
        // block is grown instead of increasing `i`.
        loop {
            if blocks.last_block().fixed_j_range.unwrap().is_empty() {
                // Fixed_j_range is empty; grow last block.
                let delta = &mut f_delta[last_idx];
                f_max[last_idx] = (f_max[last_idx] + 1).next_multiple_of(delta.0);
                update_delta(delta);
                // eprintln!("Grow last block idx {last_idx} f {}", f_max[last_idx]);
                blocks.pop_last_block();
            } else if i < self.a.len() as I {
                let i_range = IRange(i, min(i + self.params.block_width, self.a.len() as I));

                // The value of f at the tip. When going to the next block, this is
                // incremented until the range is non-empty.
                let mut next_f = f_max[last_idx];
                // Add a new block.
                loop {
                    let j_range = self.j_range(
                        i_range,
                        Some(next_f),
                        blocks.last_block(),
                        blocks.next_block_j_range(),
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
                // "Push new block idx {last_idx} i {i_range:?} f {}",
                // f_max[last_idx]
                // );
            } else {
                // Grow the last block.
                let f = &mut f_max[last_idx];
                let f_target = *f + 1;
                grow_to(f, f_target, &mut f_delta[last_idx]);
                // eprintln!("Grow last block idx {last_idx} f {}", f_max[last_idx]);
                blocks.pop_last_block();
            }

            // Grow previous block sizes as long as their f_max is not large enough.
            let mut start_idx = last_idx;
            let mut last_grow = 0;
            while start_idx > 0 && f_max[start_idx - 1] < f_max[start_idx] {
                start_idx -= 1;

                let f_target = f_max[start_idx + 1];
                let old_f = f_max[start_idx];
                let old_delta = f_delta[start_idx];
                grow_to(&mut f_max[start_idx], f_target, &mut f_delta[start_idx]);
                if f_max[start_idx] > last_grow {
                    if DEBUG {
                        eprintln!(
                            "Grow  block idx {start_idx:>5} to {:>6} by {:>6} for {old_delta:>5?} and shortage {:>6}",
                            f_max[start_idx],
                            f_max[start_idx] - old_f,
                            f_target - old_f
                        );
                    }
                    last_grow = f_max[start_idx];
                }

                blocks.pop_last_block();
            }

            if start_idx < last_idx {
                if DEBUG {
                    eprintln!("START block idx {start_idx:>5} to {:>6}", f_max[start_idx]);
                }
                let h = self.domain.h_mut().unwrap();
                h.update_contours(Pos((start_idx as I - 1) * self.params.block_width, 0));
            }

            if start_idx == 0 {
                let initial_j_range = self.j_range(
                    IRange::first_col(),
                    Some(h0),
                    &Default::default(),
                    blocks.next_block_j_range(),
                );
                blocks.init(initial_j_range);
                blocks.set_last_block_fixed_j_range(Some(initial_j_range));
                // eprintln!("Reset block idx 0 to {initial_j_range:?}");

                start_idx += 1;
            }

            // Recompute all blocks from start_idx upwards for their new f_max.
            // As long as j_range doesn't grow, existing results are reused.
            let mut all_blocks_reused = true;
            for idx in start_idx..=last_idx {
                // eprintln!("Compute block idx {}", idx);
                let f_max = Some(f_max[idx]);

                let i_range = IRange(
                    (idx as I - 1) * self.params.block_width,
                    min(idx as I * self.params.block_width, self.a.len() as I),
                );
                let mut j_range = self.j_range(
                    i_range,
                    f_max,
                    blocks.last_block(),
                    blocks.next_block_j_range(),
                );
                assert!(!j_range.is_empty());

                let mut reuse = false;
                if let Some(old_j_range) = blocks.next_block_j_range() {
                    j_range = JRange(min(j_range.0, old_j_range.0), max(j_range.1, old_j_range.1));
                    // If this block doesn't grow, and previous blocks also didn't grow, reuse this block.
                    if all_blocks_reused && j_range == old_j_range {
                        reuse = true;
                    }
                }
                all_blocks_reused &= reuse;

                let prev_fixed_j_range = blocks.last_block().fixed_j_range.unwrap();
                if reuse {
                    // eprintln!("Reuse   block idx {idx} i {i_range:?} j {j_range:?} f {f_max:?}");
                    blocks.reuse_next_block(i_range, j_range);
                } else {
                    // eprintln!("Compute block idx {idx} i {i_range:?} j {j_range:?} f {f_max:?}");
                    blocks.compute_next_block(i_range, j_range, &mut self.v);
                }
                // Compute the range of fixed states.
                let next_fixed_j_range = self.fixed_j_range(i_range.1, f_max, blocks.last_block());
                // eprintln!("{i}: New fixed range {next_fixed_j_range:?}");
                blocks.set_last_block_fixed_j_range(next_fixed_j_range);
                let next_fixed_j_range = blocks.last_block().fixed_j_range.unwrap();

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
            if i == self.a.len() as I && blocks[last_idx].j_range.contains(self.b.len() as I) {
                break;
            }
        } // end loop over i

        if DEBUG {
            let mut delta = 0;
            for (idx, d) in f_delta.iter().enumerate() {
                if delta != d.0 {
                    delta = d.0;
                    eprintln!("Delta {idx:>6} => {delta:>5}");
                }
            }
        }

        // eprintln!("TRACE..");
        let dist = blocks.last_block().get(self.b.len() as I).unwrap();
        let cigar = blocks.trace(
            self.a,
            self.b,
            Pos(0, 0),
            Pos::target(self.a, self.b),
            &mut self.v,
        );
        (dist, cigar)
    }
}
