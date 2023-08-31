//!
//! TODO: [fill_block] use a single allocation for all blocks in the block. Takes up to 2% of time.
//! TODO: [fill_block] store horizontal deltas in blocks, so that `parent` is more
//!       efficient and doesn't have to use relatively slow `block.index` operations.
//!       (NOTE though that this doesn't actually seem that bad in practice.)
//! TODO: Separate strong types for row `I` and 'block-row' `I*64`.
use std::{
    cmp::{max, min},
    ops::{Index, IndexMut},
    ptr::read_unaligned,
};

use itertools::{izip, Itertools};
use pa_bitpacking::{BitProfile, HEncoding, Profile, B, V, W};
use pa_types::*;
use pa_vis_types::VisualizerInstance;
use serde::{Deserialize, Serialize};

use super::*;
use crate::block::*;

type PA = <BitProfile as Profile>::A;
type PB = <BitProfile as Profile>::B;
type H = (B, B);

/// Parameters for BitBlock.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct BlockParams {
    /// When true, `trace` mode only stores the last column of each block, instead of all columns.
    /// `cost` mode always stores only the last column.
    pub sparse: bool,
    #[serde(default)]
    pub simd: bool,
    #[serde(default)]
    pub incremental_doubling: bool,

    /// Do greedy diagonal-transition traceback.
    #[serde(default)]
    pub dt_trace: bool,

    /// Do traceback up to this distance. 0 disables the limit.
    #[serde(default)]
    pub max_g: Cost,

    /// X-drop for dt-trace.
    #[serde(default)]
    pub x_drop: I,
}

impl Default for BlockParams {
    fn default() -> Self {
        Self {
            sparse: true,
            simd: true,
            incremental_doubling: true,
            dt_trace: false,
            max_g: 40,
            x_drop: 20,
        }
    }
}

/// The main data for bitblocks.
pub struct Blocks {
    // Input/parameters.
    params: BlockParams,
    trace: bool,
    a: Vec<PA>,
    b: Vec<PA>,

    // Pos.
    /// The list of blocks.
    /// NOTE: When using sparse traceback blocks, indices do not correspond to `i`!
    blocks: Vec<Block>,
    /// The index of the current/last active block.
    last_block_idx: usize,
    /// The range corresponding to all blocks.
    i_range: IRange,

    /// Store horizontal differences for row `j_h`.
    /// This allows for incremental band doubling.
    h: Vec<H>,

    // Additional statistics.
    /// The distribution of number of rows in `compute` calls.
    computed_rows: Vec<usize>,
    unique_rows: usize,
}

impl BlockParams {
    pub const BLOCKSIZE: I = 64;
    pub fn new<'a>(&self, trace: bool, a: Seq<'a>, b: Seq<'a>) -> Blocks {
        let (a, b) = BitProfile::build(a, b);
        Blocks {
            params: *self,
            blocks: vec![],
            trace,
            i_range: IRange(-1, 0),
            last_block_idx: 0,
            h: if self.incremental_doubling {
                vec![(0, 0); a.len()]
            } else {
                vec![]
            },
            a,
            b,
            computed_rows: vec![],
            unique_rows: 0,
        }
    }
}

/// Print some statistics.
impl Drop for Blocks {
    fn drop(&mut self) {
        if !DEBUG {
            return;
        }
        let mut cnt = 0;
        let mut total = 0;
        for (i, c) in self.computed_rows.iter().enumerate() {
            cnt += c;
            total += i * c;
            if i % 10 == 0 {
                eprint!("\n{i:>4}");
            }
            eprint!("{c:>7}");
        }
        eprintln!();
        eprintln!("Num blocks: {cnt}");
        // FIXME: Hardcoded blocksize.
        let num_blocks = max(self.a.len().div_ceil(256), 1);
        eprintln!("Total band: {}", total / num_blocks);
        eprintln!("Uniq. band: {}", self.unique_rows / num_blocks);
    }
}

impl IndexMut<usize> for Blocks {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.blocks[index]
    }
}

impl Index<usize> for Blocks {
    type Output = Block;

    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks[index]
    }
}

impl Blocks {
    pub fn init(&mut self, mut initial_j_range: JRange) {
        assert!(initial_j_range.0 == 0);
        self.last_block_idx = 0;
        self.i_range = IRange(-1, 0);

        if let Some(block) = self.blocks.get(0) {
            initial_j_range = initial_j_range.union(*block.j_range);
        }
        let initial_j_range = initial_j_range.round_out();

        let block = if self.trace {
            // First column block, with more blocks pushed after.
            Block::first_col(initial_j_range)
        } else {
            // Block spanning the entire first column.
            Block {
                v: vec![V::one(); self.b.len()],
                i_range: IRange(-1, 0),
                j_range: initial_j_range,
                fixed_j_range: Some(*initial_j_range),
                offset: 0,
                top_val: 0,
                bot_val: initial_j_range.1,
                j_h: None,
            }
        };
        if self.blocks.is_empty() {
            self.blocks.push(block);
        } else {
            self.blocks[0] = block;
        }
    }

    /// Remove the last block and update the i_range.
    pub fn pop_last_block(&mut self) {
        self.i_range.pop(self.blocks[self.last_block_idx].i_range);
        self.last_block_idx -= 1;
    }

    /// The next block can be reused from an earlier iteration.
    /// Simply increment the last_block_idx, update the i_range, and check that
    /// the reused block indeed has the same ranges.
    pub fn reuse_next_block(&mut self, i_range: IRange, j_range: JRange) {
        self.i_range.push(i_range);
        self.last_block_idx += 1;

        let block = &mut self.blocks.get(self.last_block_idx).unwrap();
        assert_eq!(block.i_range, i_range);
        assert_eq!(block.j_range, j_range.round_out());
    }

    /// The main function to compute the next block.
    ///
    /// Contains the implementation for incremental doubling (which is tedious
    /// and needs to maintain a lot of indices).  Dispatches to various lower
    /// level calls of `compute_columns` based on whether incremental doubling
    /// is used and whether traceback is enabled.
    pub fn compute_next_block(
        &mut self,
        i_range: IRange,
        j_range: JRange,
        viz: &mut impl VisualizerInstance,
    ) {
        let j_range = j_range.round_out();

        if let Some(next_block) = self.blocks.get(self.last_block_idx + 1) {
            assert!(
                j_range.contains_range(*next_block.j_range),
                "j_range must grow"
            );
            self.unique_rows -= next_block.j_range.exclusive_len() as usize / W;
        }

        if self.trace && !self.params.sparse {
            // This is extracted to a separate function for reuse during traceback.
            return self.fill_with_blocks(i_range, j_range, viz);
        }

        self.i_range.push(i_range);

        if DEBUG {
            eprintln!("Compute block {:?} {:?}", i_range, j_range);
        }

        let v_range = j_range.v_range();
        self.unique_rows += v_range.len();

        // Get top/bot values in the previous column for the new j_range.
        let prev_top_val = self.last_block().index(j_range.0);
        let prev_bot_val = self.last_block().index(j_range.1);

        if !self.trace && !self.params.incremental_doubling {
            // Update the existing `v` vector in the single block.
            let top_val = prev_top_val + i_range.len();
            let bot_val = prev_bot_val
                + compute_block(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range.clone(),
                    &mut self.blocks[self.last_block_idx].v[v_range.clone()],
                    &mut self.h,
                    &mut self.computed_rows,
                    HMode::None,
                    viz,
                );
            // In this case there is only a single reused block. Overwrite its range.
            let block = &mut self.blocks[self.last_block_idx];
            block.i_range = i_range;
            block.j_range = j_range;
            block.top_val = top_val;
            block.bot_val = bot_val;
            block.check_top_bot_val();
            return;
        }

        assert!(self.params.sparse);

        // Compute the new `v` at the end of the `i_range` and push a new block.

        // Reuse memory from an existing block if possible.
        // Otherwise, push a new block.
        if self.last_block_idx + 1 == self.blocks.len() {
            self.blocks.push(Block::default());
        } else {
            let next_block = &mut self.blocks[self.last_block_idx + 1];
            assert_eq!(next_block.i_range, i_range);
        };

        // Some trickery two access two elements at the same time.
        let [prev_block, next_block] = &mut self.blocks[self.last_block_idx..].split_array_mut().0;
        self.last_block_idx += 1;

        // Update the block properties.
        next_block.i_range = i_range;
        next_block.top_val = prev_top_val + i_range.len();
        // Will be updated with `bottom_delta`.
        next_block.bot_val = prev_bot_val;

        // If no incremental doubling or no fixed_j_range was set, just compute everything.
        // TODO: Also just compute everything if the range is small anyway.
        // Fragmenting into smaller slices breaks SIMD and is slower.
        if !self.params.incremental_doubling
            || prev_block.fixed_j_range.is_none()
            || next_block.fixed_j_range.is_none()
        {
            // Incremental doubling disabled; just compute the entire `j_range`.
            init_v_with_overlap(prev_block, j_range, &mut next_block.v);
            next_block.bot_val += compute_block(
                self.params,
                &self.a,
                &self.b,
                i_range,
                v_range.clone(),
                &mut next_block.v,
                &mut self.h,
                &mut self.computed_rows,
                HMode::None,
                viz,
            );
            next_block.j_range = j_range;
            next_block.offset = j_range.0;
            next_block.fixed_j_range = None;
            next_block.check_top_bot_val();
            return;
        }

        // Do incremental doubling.

        let prev_fixed = prev_block.fixed_j_range.unwrap().round_in();
        let next_fixed = next_block.fixed_j_range.unwrap().round_in();

        // New j_h.
        let new_j_h = prev_fixed.1;

        let offset = j_range.v_range().start;

        // If there is already a fixed range here, a corresponding j_h, and the ranges before/after the fixed part do not overlap, then do a 3-range split:
        // range 0: everything before the fixed part.  h not used.
        // range 1: from previous j_h to new j_h.      h is updated.
        // range 2: from new j_h to end.               h is input.
        //
        // Otherwise, do a 2-range split:
        // range 01: everything before the new j_h.    h is output.
        // range  2: from new j_h to end.              h is output.
        if let Some(old_j_h) = next_block.j_h
                && next_fixed.0 < old_j_h {
                eprintln!("IC: {i_range:?} {j_range:?} old {:?} fixed {:?}", next_block.j_range, next_block.fixed_j_range);
                init_v_with_overlap_preserve_fixed(prev_block, next_block, j_range);

                let v_range_0 = JRange(j_range.0, next_fixed.0).assert_rounded().v_range();
                assert!(v_range_0.start <= v_range_0.end);
                // The part between next_fixed.0 and old_j_h is fixed and skipped!
                let v_range_1 = JRange(old_j_h, new_j_h).assert_rounded().v_range();
                assert!(v_range_1.start <= v_range_1.end, "j_h may only increase! i {i_range:?} old_j_h: {}, new_j_h: {}", old_j_h, new_j_h);
                let v_range_2 = JRange(new_j_h, j_range.1).assert_rounded().v_range();
                assert!(v_range_2.start <= v_range_2.end);

                // Compute the part before the fixed range without using input/output horizontal deltas.
                compute_block(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range_0.clone(),
                    &mut next_block.v[v_range_0.start - offset..v_range_0.end - offset],
                    &mut self.h,
                    &mut self.computed_rows,
                    HMode::None,
                    viz,
                );

                // Update the horizontal deltas from old_j_h to new_j_h.
                compute_block(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range_1.clone(),
                    &mut next_block.v[v_range_1.start - offset..v_range_1.end - offset],
                    &mut self.h,
                    &mut self.computed_rows,
                    HMode::Update,
                    viz
                );

                // Compute the part below new_j_h using the horizontal deltas.
                next_block.bot_val += compute_block(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range_2.clone(),
                    &mut next_block.v[v_range_2.start - offset..v_range_2.end - offset],
                    &mut self.h,
                    &mut self.computed_rows,
                    HMode::Input,
                    viz
                );
            } else {
                init_v_with_overlap(prev_block, j_range, &mut next_block.v);

                let v_range_01 = JRange(j_range.0, new_j_h).assert_rounded().v_range();
                assert!(v_range_01.start <= v_range_01.end);
                let v_range_2 = JRange(new_j_h, j_range.1).assert_rounded().v_range();
                assert!(v_range_2.start <= v_range_2.end);

                compute_block(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range_01.clone(),
                    &mut next_block.v[v_range_01.start - offset..v_range_01.end - offset],
                    &mut self.h,
                    &mut self.computed_rows,
                    HMode::Output,
                    viz
                );

                next_block.bot_val += compute_block(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range_2.clone(),
                    &mut next_block.v[v_range_2.start - offset..v_range_2.end - offset],
                    &mut self.h,
                    &mut self.computed_rows,
                    HMode::Input,
                    viz
                );
            };
        next_block.j_range = j_range;
        next_block.offset = j_range.0;
        next_block.j_h = Some(new_j_h);
        next_block.fixed_j_range = None;
        next_block.check_top_bot_val();

        // Test incremental doubling: Redo the computation without the
        // fixed range and test if they give the same results.
        if cfg!(test) || DEBUG {
            let mut v2 = Vec::default();
            init_v_with_overlap(prev_block, j_range, &mut v2);
            compute_block(
                self.params,
                &self.a,
                &self.b,
                i_range,
                v_range.clone(),
                &mut v2,
                &mut self.h,
                &mut self.computed_rows,
                HMode::None,
                viz,
            );
            assert_eq!(next_block.v.len(), v2.len());
            if next_block.v != v2 {
                for (i, (a, b)) in izip!(&next_block.v, &v2).enumerate() {
                    if a != b {
                        println!("{}+{}={}: {:?} != {:?}", i, offset, i + offset, a, b);
                    }
                }

                panic!("Vectors differ!");
            }
            assert_eq!(next_block.v, v2);
        }
    }

    pub fn last_block(&self) -> &Block {
        &self.blocks[self.last_block_idx]
    }

    pub fn next_block_j_range(&self) -> Option<JRange> {
        self.blocks.get(self.last_block_idx + 1).map(|f| *f.j_range)
    }

    /// Traceback the path from `from` to `to`.
    ///
    /// This requires `self.trace` to be `true`. In case of sparse blocks, this
    /// recomputes blocks when needed (when dt-trace fails).
    pub fn trace(
        &mut self,
        a: Seq,
        b: Seq,
        from: Pos,
        mut to: Pos,
        viz: &mut impl VisualizerInstance,
    ) -> Cigar {
        assert!(self.trace);
        assert!(self.blocks.last().unwrap().i_range.1 == to.0);
        let mut cigar = Cigar { ops: vec![] };
        let mut g = self.blocks[self.last_block_idx].index(to.1);

        if DEBUG {
            eprintln!("Trace from distance {g}");
        }

        let mut dt_trace_tries = 0;
        let mut dt_trace_success = 0;
        let mut dt_trace_fallback = 0;

        let cached_dt_blocks =
            &mut vec![BlockElem::default(); (self.params.max_g + 1).pow(2) as usize];

        while to != from {
            // Remove blocks to the right of `to`.
            while self.last_block_idx > 0 && self.blocks[self.last_block_idx - 1].i_range.1 >= to.0
            {
                if DEBUG {
                    eprintln!(
                        "to {to:?} Pop block at i={:?}",
                        self.blocks[self.last_block_idx].i_range
                    );
                }
                self.pop_last_block();
            }

            // Try a Diagonal Transition based traceback first which should be faster for small distances.
            if self.params.dt_trace && to.0 > 0 {
                let prev_block = &self.blocks[self.last_block_idx - 1];
                if prev_block.i_range.1 < to.0 - 1 {
                    dt_trace_tries += 1;
                    if let Some(new_to) = self.dt_trace_block(
                        a,
                        b,
                        to,
                        &mut g,
                        prev_block,
                        &mut cigar,
                        cached_dt_blocks,
                    ) {
                        dt_trace_success += 1;
                        // eprintln!("To from {:?} to {:?}", to, new_to);
                        to = new_to;
                        continue;
                    }
                    dt_trace_fallback += 1;
                }
            }

            // Fall back to DP based traceback.

            // In case of sparse blocks, fill missing columns by recomputing the
            // block and storing all columns.
            if self.params.sparse && to.0 > 0 {
                let block = &self.blocks[self.last_block_idx];
                let prev_block = &self.blocks[self.last_block_idx - 1];
                assert!(prev_block.i_range.1 < to.0 && to.0 <= block.i_range.1);
                // If the previous block is the correct one, no need for further recomputation.
                if prev_block.i_range.1 < to.0 - 1 || block.i_range.1 > to.0 {
                    if DEBUG {
                        eprintln!(
                            "Expand previous block from {:?} to {}",
                            prev_block.i_range, to.0
                        );
                    }
                    let i_range = IRange(prev_block.i_range.1, to.0);
                    let j_range = JRange(block.j_range.0, to.1);
                    self.pop_last_block();
                    // NOTE: It's unlikely the full (large) `j_range` is needed to trace back through the current block.
                    // 1. We don't need states with `j > to.1`, because the path (in reverse direction) can never go down.
                    // 2. It's unlikely we'll need all states starting at the (possibly much smaller) `j_range.0`.
                    //    Instead, we do an exponential search for the start of the `j_range`, starting at `to.1-2*i_range.len()`.
                    //    The block is high enough once the cost to `to` equals `g`.
                    let mut height = max(j_range.exclusive_len(), i_range.len() * 5 / 4);
                    loop {
                        let j_range = JRange(max(j_range.1 - height, 0), j_range.1).round_out();
                        if DEBUG {
                            eprintln!("Fill block {:?} {:?}", i_range, j_range);
                        }
                        self.fill_with_blocks(i_range, j_range, viz);
                        if self.blocks[self.last_block_idx].index(to.1) == g {
                            break;
                        }
                        if j_range.0 == 0 {
                            panic!("No trace found through block {i_range:?} {j_range:?}");
                        }
                        // Pop all the computed blocks.
                        for _i in i_range.0..i_range.1 {
                            self.pop_last_block();
                        }
                        // Try again with a larger height.
                        height *= 2;
                    }
                }
            }

            if DEBUG && to.0 % 256 == 0 {
                eprintln!(
                    "Parent of {to:?} at distance {g} with range {:?}",
                    self.blocks[self.last_block_idx].j_range
                );
            }
            let (parent, cigar_elem) = self.parent(to, &mut g);
            to = parent;
            cigar.push_elem(cigar_elem);
        }
        if DEBUG {
            eprintln!("dt_trace_tries:    {:>7}", dt_trace_tries);
            eprintln!("dt_trace_success:  {:>7}", dt_trace_success);
            eprintln!("dt_trace_fallback: {:>7}", dt_trace_fallback);
        }
        assert_eq!(g, 0);
        cigar.reverse();
        cigar
    }

    // Update the fixed range, and make sure it only grows.
    pub fn set_last_block_fixed_j_range(&mut self, fixed_j_range: Option<JRange>) {
        if let Some(old) = self.blocks[self.last_block_idx].fixed_j_range
            && let Some(new) = fixed_j_range {
                self.blocks[self.last_block_idx].fixed_j_range = Some(JRange(
                    min(old.0, new.0),
                    max(old.1, new.1),
                ));
            } else {
                self.blocks[self.last_block_idx].fixed_j_range = fixed_j_range;
            }
    }
}

#[derive(Clone, Copy)]
struct BlockElem {
    /// The current column.
    i: I,
    /// The length of the extension to get here.
    ext: I,
    /// The diagonal of the parent relative to this one.
    parent_d: I,
}
impl Default for BlockElem {
    fn default() -> Self {
        BlockElem {
            i: I::MAX,
            ext: 0,
            parent_d: 0,
        }
    }
}
impl BlockElem {
    fn reset(&mut self) {
        *self = BlockElem::default();
    }
}

fn extend_left(i: &mut i32, i0: i32, j: &mut i32, a: &[u8], b: &[u8]) -> I {
    let mut cnt = 0;
    while *i > i0 && *j > 0 && a[*i as usize - 1] == b[*j as usize - 1] {
        *i -= 1;
        *j -= 1;
        cnt += 1;
    }
    cnt
}

fn extend_left_simd(i: &mut i32, i0: i32, j: &mut i32, a: &[u8], b: &[u8]) -> I {
    let mut cnt = 0;
    // Do the first char manually to throw away some easy bad cases before going into SIMD.
    if *i > i0 && *j > 0 && a[*i as usize - 1] == b[*j as usize - 1] {
        *i -= 1;
        *j -= 1;
        cnt += 1;
    } else {
        return cnt;
    }
    while *i >= 8 && *j >= 8 {
        // let simd_a: Simd<u8, 32> = Simd::from_array(*a[*i as usize - 32..].split_array_ref().0);
        // let simd_b: Simd<u8, 32> = Simd::from_array(*b[j as usize - 32..].split_array_ref().0);
        // let eq = simd_a.simd_eq(simd_b).to_bitmask();
        // let cnt2 = if cfg!(target_endian = "little") {
        //     eq.leading_ones() as I
        // } else {
        //     eq.trailing_ones() as I
        // };

        let cmp = unsafe {
            read_unaligned(a[*i as usize - 8..].as_ptr() as *const usize)
                ^ read_unaligned(b[*j as usize - 8..].as_ptr() as *const usize)
        };
        let cnt2 = if cmp == 0 {
            8
        } else {
            (cmp.leading_zeros() / u8::BITS) as I
        };

        *i -= cnt2;
        *j -= cnt2;
        cnt += cnt2;
        if *i <= i0 {
            let overshoot = i0 - *i;
            *i += overshoot;
            *j += overshoot;
            cnt -= overshoot;
            return cnt;
        }
        if cnt2 < 8 {
            return cnt;
        }
    }
    cnt += extend_left(i, i0, j, a, b);
    cnt
}

impl Blocks {
    /// Find the parent of `st`.
    /// NOTE: This assumes that `st.0` is in the last block, and that the block before is for `st.0-1`.
    /// `g`: distance to `st`.
    /// `block_start`: the IRange.0 of the previous block.
    /// ALG: NOTE: Greedy backward matching is OK (it is guaranteed that all
    /// computed cells reached this way have the same score). But note that this
    /// may end up outside the computed area. In that case we use insertions or
    /// deletions as needed to get back.
    fn parent(&self, mut st: Pos, g: &mut Cost) -> (Pos, CigarElem) {
        let block = &self.blocks[self.last_block_idx];
        assert!(
            block.i_range.1 == st.0,
            "Parent of state {st:?} but block.i is {:?}",
            block.i_range
        );

        // Greedy matching.
        let mut cnt = 0;
        // TODO: SIMD using raw A and B.
        while st.0 > 0 && st.1 > 0 && BitProfile::is_match(&self.a, &self.b, st.0 - 1, st.1 - 1) {
            cnt += 1;
            st.0 -= 1;
            st.1 -= 1;
        }
        if cnt > 0 {
            return (
                st,
                CigarElem {
                    op: CigarOp::Match,
                    cnt,
                },
            );
        }

        // Vertical delta (insert).
        // (This is first since it only needs a single delta bit, instead of an index() call.)
        let vd = block.get_diff(st.1 - 1);
        if vd == Some(1) {
            *g -= 1;
            return (
                Pos(st.0, st.1 - 1),
                CigarElem {
                    op: CigarOp::Ins,
                    cnt: 1,
                },
            );
        }

        let prev_block = &self.blocks[self.last_block_idx - 1];
        assert!(prev_block.i_range.1 == st.0 - 1);

        // Horizontal delta (delete).
        let hd = *g - prev_block.index(st.1);
        if hd == 1 {
            *g -= 1;
            return (
                Pos(st.0 - 1, st.1),
                CigarElem {
                    op: CigarOp::Del,
                    cnt: 1,
                },
            );
        }

        // Diagonal delta (substitution).
        // This edge case happens when entering the previous block exactly in
        // the bottom-most row, where no vertical delta is available.
        let dd = if st.1 > prev_block.j_range.1 {
            assert_eq!(st.1, prev_block.j_range.1 + 1);
            1
        } else {
            prev_block.get_diff(st.1 - 1).unwrap() + hd
        };
        if dd == 1 {
            *g -= 1;
            return (
                Pos(st.0 - 1, st.1 - 1),
                CigarElem {
                    op: CigarOp::Sub,
                    cnt: 1,
                },
            );
        }

        panic!("ERROR: PARENT OF {st:?} NOT FOUND IN TRACEBACK");
    }

    /// Trace a path backwards from `st` until `i=block_start`.
    fn dt_trace_block(
        &self,
        a: Seq,
        b: Seq,
        st: Pos,
        g_st: &mut Cost,
        prev_block: &Block,
        cigar: &mut Cigar,
        blocks: &mut Vec<BlockElem>,
    ) -> Option<Pos> {
        // eprintln!(
        //     "DT Trace from {st:?} with g={g_st} back to {}",
        //     prev_block.i
        // );
        let block_start = prev_block.i_range.1;
        // Returns true when `end_i` is reached.
        // The block stores the leftmost reachable column at distance g in diagonal d relative to st.
        // The element for (g,d) is at position g*g+g+d.
        blocks[0] = BlockElem {
            i: st.0,
            ext: 0,
            parent_d: 0,
        };

        fn index(g: Cost, d: I) -> usize {
            (g * g + g + d) as usize
        }
        fn get(blocks: &Vec<BlockElem>, g: Cost, d: I) -> BlockElem {
            blocks[index(g, d)]
        }
        fn get_mut(blocks: &mut Vec<BlockElem>, g: Cost, d: I) -> &mut BlockElem {
            &mut blocks[index(g, d)]
        }

        fn trace(
            blocks: &Vec<BlockElem>,
            mut g: Cost,
            mut d: I,
            st: Pos,
            g_st: &mut Cost,
            block_start: I,
            cigar: &mut Cigar,
        ) -> Pos {
            //eprintln!("TRACE");
            let new_st = Pos(block_start, st.1 - (st.0 - block_start) - d);
            *g_st -= g;
            let mut ops = vec![];
            loop {
                let fr = get(blocks, g, d);
                if fr.ext > 0 {
                    //eprintln!("Ext: {}", fr.ext);
                    ops.push(CigarElem {
                        op: CigarOp::Match,
                        cnt: fr.ext,
                    })
                }
                if g == 0 {
                    break;
                }
                g -= 1;
                d += fr.parent_d;
                let op = match fr.parent_d {
                    -1 => CigarOp::Ins,
                    0 => CigarOp::Sub,
                    1 => CigarOp::Del,
                    _ => panic!(),
                };
                //eprintln!("Op: {:?}", op);
                ops.push(CigarElem { op, cnt: 1 });
            }
            for e in ops.into_iter().rev() {
                cigar.push_elem(e);
            }
            new_st
        }

        let mut g = 0 as Cost;

        // Extend up to the start of the previous block and check if the distance is correct.
        let extend_left_simd_and_check = |elem: &mut BlockElem, mut j: I, target_g: Cost| -> bool {
            elem.ext += extend_left_simd(&mut elem.i, prev_block.i_range.1, &mut j, a, b);
            *(&mut elem.i) == prev_block.i_range.1 && prev_block.get(j) == Some(target_g)
        };

        if extend_left_simd_and_check(&mut blocks[0], st.1, 0) {
            return Some(trace(&blocks, 0, 0, st, g_st, block_start, cigar));
        }
        //eprintln!("extend d=0 from {:?} to {}", st, blocks[0][0].i);

        let mut d_range = (0, 0);
        loop {
            let ng = g + 1;
            // Init next block

            let end_idx = index(ng, d_range.1 + 1);
            if blocks.len() <= end_idx {
                blocks.resize(end_idx + 1, BlockElem::default());
            }
            for fe in &mut blocks[index(ng, d_range.0 - 1)..=end_idx] {
                fe.reset();
            }

            // EXPAND.
            //eprintln!("expand");
            for d in d_range.0..=d_range.1 {
                let fr = get(blocks, g, d);
                //eprintln!("Expand g={} d={} i={}", g, d, fr.i);
                fn update(x: &mut BlockElem, y: I, d: I) {
                    if y < x.i {
                        //eprintln!("update d={d} from {} to {}", x.i, y);
                        x.i = y;
                        x.parent_d = d;
                    }
                }
                update(&mut get_mut(blocks, ng, d - 1), fr.i - 1, 1);
                update(&mut get_mut(blocks, ng, d), fr.i - 1, 0);
                update(&mut get_mut(blocks, ng, d + 1), fr.i, -1);
            }
            g += 1;
            d_range.0 -= 1;
            d_range.1 += 1;

            // Extend.
            let mut min_i = I::MAX;
            for d in d_range.0..=d_range.1 {
                let fr = get_mut(blocks, g, d);
                if fr.i == I::MAX {
                    continue;
                }
                let j = st.1 - (st.0 - fr.i) - d;
                // let old_i = fr.i;
                if extend_left_simd_and_check(fr, j, *g_st - g) {
                    return Some(trace(&blocks, g, d, st, g_st, block_start, cigar));
                }
                // eprintln!("extend d={d} from {} to {}", Pos(old_i, j), fr.i);
                min_i = min(min_i, fr.i);
            }

            if g == self.params.max_g {
                return None;
            }

            // Shrink diagonals more than 5 behind.
            if self.params.x_drop > 0 {
                while d_range.0 < d_range.1
                    && get(blocks, g, d_range.0).i > min_i + self.params.x_drop
                {
                    d_range.0 += 1;
                }
                while d_range.0 < d_range.1
                    && get(blocks, g, d_range.1).i > min_i + self.params.x_drop
                {
                    d_range.1 -= 1;
                }
            }
        }
    }

    /// Store a single block for each column in `i_range`.
    fn fill_with_blocks(
        &mut self,
        i_range: IRange,
        j_range: RoundedOutJRange,
        viz: &mut impl VisualizerInstance,
    ) {
        self.i_range.push(i_range);

        let j_range_rounded = j_range;
        let v_range = j_range_rounded.0 as usize / W..j_range_rounded.1 as usize / W;

        // Get top/bot values in the previous column for the new j_range_rounded.
        let prev_block = &self.blocks[self.last_block_idx];
        assert!(IRange::consecutive(prev_block.i_range, i_range));

        let mut v = Vec::default();
        init_v_with_overlap(prev_block, j_range_rounded, &mut v);

        // 1. Push blocks for all upcoming columns.
        // 2. Take the vectors.
        // 3. Fill
        // 4. Put the vectors back.
        // 5. Compute bot values.

        let mut next_block = Block {
            // Will be resized in fill().
            v: vec![],
            i_range: IRange(i_range.0, i_range.0),
            j_range,
            offset: j_range_rounded.0,
            fixed_j_range: None,
            top_val: prev_block.index(j_range_rounded.0),
            // Will be set later.
            bot_val: 0,
            // bot_val: prev_block.index(j_range_rounded.1),
            // During traceback, we ignore any stored horizontal deltas.
            j_h: None,
        };

        // 1.
        for i in i_range.0..i_range.1 {
            // Along the top row, horizontal deltas are 1.
            next_block.i_range = IRange(i, i + 1);
            next_block.top_val += 1;
            self.last_block_idx += 1;
            if self.last_block_idx == self.blocks.len() {
                self.blocks.push(next_block.clone());
            } else {
                self.blocks[self.last_block_idx].clone_from(&next_block);
            }
        }

        // 2.
        let mut values = vec![vec![]; i_range.len() as usize];
        for (block, vv) in izip!(
            &mut self.blocks
                [self.last_block_idx + 1 - i_range.len() as usize..=self.last_block_idx],
            values.iter_mut()
        ) {
            *vv = std::mem::take(&mut block.v);
        }
        let h = &mut vec![H::one(); i_range.len() as usize];

        // 3.
        viz.expand_block_simple(
            Pos(i_range.0 + 1, j_range_rounded.0),
            Pos(i_range.len(), j_range_rounded.exclusive_len()),
        );
        if self.params.simd {
            pa_bitpacking::simd::fill::<2, H, 4>(
                &self.a[i_range.0 as usize..i_range.1 as usize],
                &self.b[v_range],
                h,
                &mut v,
                true,
                &mut values[..],
            );
        } else {
            pa_bitpacking::scalar::fill::<BitProfile, H>(
                &self.a[i_range.0 as usize..i_range.1 as usize],
                &self.b[v_range],
                h,
                &mut v,
                &mut values[..],
            );
        }

        // 4. 5.
        let mut bot_val =
            self.blocks[self.last_block_idx - i_range.len() as usize].index(j_range_rounded.1);
        for (block, vv, h) in izip!(
            &mut self.blocks
                [self.last_block_idx + 1 - i_range.len() as usize..=self.last_block_idx],
            values.into_iter(),
            h.iter(),
        ) {
            block.v = vv;
            bot_val += h.value();
            block.bot_val = bot_val;
        }
    }
}

#[derive(Debug)]
enum HMode {
    None,
    Input,
    Update,
    Output,
}

/// The main function to compute the right column of a block `i_range` x `v_range`.
/// Uses `v_range` of `v` as input vertical differences on the left and updates it with vertical differences.
/// Does some checks and logging and dispatches to (SIMD) functions in `pa_bitpacking`.
///
/// Returns the sum of horizontal differences along the bottom edge.
///
/// Can run in 4 modes regarding horizontal differences at the top/bottom of the block:
/// - None: assume +1 differences on the top, do not output bottom differences.
/// - Output: assume +1 differences on the top, output bottom differences.
/// - Input: use given horizontal differences on the top, do not output bottom differences.
/// - Update: use given horizontal differences on the top, and update them.
///
/// This is a free function to allow passing in mutable references.
fn compute_block(
    params: BlockParams,
    a: &[PA],
    b: &[PB],
    i_range: IRange,
    v_range: std::ops::Range<usize>,
    v: &mut [V],
    h: &mut [H],
    computed_rows: &mut Vec<usize>,
    mode: HMode,
    viz: &mut impl VisualizerInstance,
) -> i32 {
    viz.expand_block_simple(
        Pos(i_range.0 + 1, v_range.start as I * WI),
        Pos(i_range.len(), v_range.len() as I * WI),
    );

    // Keep statistics on how many rows are computed at a time.
    // Skipped during traceback.
    if i_range.len() > 1 {
        eprintln!("Compute i {i_range:?} x j {v_range:?} in mode {mode:?}");

        if !(v_range.len() < computed_rows.len()) {
            computed_rows.resize(v_range.len() + 1, 0);
        }
        computed_rows[v_range.len()] += 1;
    }

    let run = |h, exact_end| {
        if params.simd {
            pa_bitpacking::simd::compute::<2, H, 4>(
                &a[i_range.0 as usize..i_range.1 as usize],
                &b[v_range],
                h,
                v,
                exact_end,
            ) as I
        } else {
            pa_bitpacking::scalar::row::<BitProfile, H>(
                &a[i_range.0 as usize..i_range.1 as usize],
                &b[v_range],
                h,
                v,
            ) as I
        }
    };
    let i_slice = i_range.0 as usize..i_range.1 as usize;

    match mode {
        HMode::None => {
            // Just create two temporary vectors that are discarded afterwards.
            let h = &mut vec![H::one(); i_slice.len()];
            run(h, false)
        }
        HMode::Input => {
            // Make a copy to prevent overwriting.
            let h = &mut h[i_slice].iter().copied().collect_vec();
            run(h, false)
        }
        HMode::Update => run(&mut h[i_slice], true),
        HMode::Output => {
            // Initialize to +1.
            let h = &mut h[i_slice];
            h.fill(H::one());
            run(h, true)
        }
    }
}

/// This prepares the `v` vector of vertical differences for a new block.
///
/// It copies the overlap with the previous block, and fills the rest with +1.
fn init_v_with_overlap(prev_block: &Block, j_range: RoundedOutJRange, v: &mut Vec<V>) {
    v.clear();
    v.resize(j_range.exclusive_len() as usize / W, V::one());
    // Copy the overlap from the last block.
    for idx in RoundedOutJRange::intersection(j_range, prev_block.j_range).v_range() {
        v[idx - (j_range.0 / WI) as usize] = prev_block.v[idx - (prev_block.offset / WI) as usize];
    }
}

/// This prepares the `v` vector of vertical differences for a new block.
///
/// It copies the overlap with the previous block, and fills the rest with +1.
///
/// Unlike `init_v_with_overlap`, this preserves the existing `fixed_j_range` of the block.
fn init_v_with_overlap_preserve_fixed(
    prev_block: &Block,
    next_block: &mut Block,
    j_range: RoundedOutJRange,
) {
    let v = &mut next_block.v;

    // Some simplifying assumptions.
    assert!(next_block.offset == next_block.j_range.0);
    assert!(prev_block.offset == prev_block.j_range.0);
    assert!(j_range.contains_range(*next_block.j_range));

    let prev_v_range = prev_block.j_range.v_range();
    let old_v_range = next_block.j_range.v_range();
    let v_range = j_range.v_range();
    assert!(prev_v_range.start <= v_range.start);
    assert!(v_range.start <= old_v_range.start);
    let preserve = JRange(next_block.fixed_j_range.unwrap().0, next_block.j_h.unwrap())
        .round_in()
        .v_range();
    assert!(!preserve.is_empty());

    // 1. Resize the v array.
    v.resize(v_range.len(), V::one());

    // 2. Move the fixed range for `next_block` to the right place.
    // NOTE: ALG:
    // It can happen that stored_h is larger than fixed_rounded.1,
    // meaning that the loop below will copy beyond the end of the fixed range.
    // That's OK though, since in this case, the end of the fixed range has
    // shrunk from the previous block. While that means some values there have f(u) > f_max,
    // these values are still guaranteed to be correct.
    assert!(v_range.start <= old_v_range.start);
    if v_range.start != old_v_range.start {
        v.copy_within(
            preserve.start - old_v_range.start..preserve.end - old_v_range.start,
            preserve.start - v_range.start,
        );
    }

    // 3. Copy the prefix and suffix with values from `prev_block`.
    // prefix
    v[..preserve.start - v_range.start].copy_from_slice(
        &prev_block.v[v_range.start - prev_v_range.start..preserve.start - prev_v_range.start],
    );
    // suffix
    let copy_end = min(v_range.end, prev_block.j_range.v_range().end);
    v[preserve.end - v_range.start..copy_end - v_range.start].copy_from_slice(
        &prev_block.v[preserve.end - prev_v_range.start..copy_end - prev_v_range.start],
    );

    // 4. Fill the remainder with 1s.
    v[copy_end - v_range.start..].fill(V::one());
}
