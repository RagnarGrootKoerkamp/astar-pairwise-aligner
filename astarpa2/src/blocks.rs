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
        let num_blocks = self.a.len() / 256;
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

        // eprintln!("Init first block for {:?}", initial_j_range);
        if let Some(block) = self.blocks.get(0) {
            initial_j_range = JRange(
                min(block.j_range.0, initial_j_range.0),
                max(block.j_range.1, initial_j_range.1),
            );
            // eprintln!("Upated initial range to {:?}", initial_j_range);
        }
        let initial_j_range = initial_j_range.round_out();

        let block = if self.trace {
            // First column block, with more blocks pushed after.
            Block::first_col(initial_j_range)
        } else {
            // Block spanning the entire first column.
            Block {
                v: vec![V::one(); self.b.len()],
                i: 0,
                j_range: initial_j_range,
                fixed_j_range: Some(initial_j_range.round_in()),
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
        //self.computed_rows.fill(0);
    }

    // TODO: Maybe we should at some point drop the unused blocks?
    pub fn pop_last_block(&mut self) {
        assert!(self.i_range.1 == self.blocks[self.last_block_idx].i);
        self.last_block_idx -= 1;
        self.i_range.1 = self.blocks.get(self.last_block_idx).map_or(-1, |f| f.i);
    }

    pub fn reuse_next_block(&mut self, i_range: IRange, j_range: JRange) {
        let j_range = j_range.round_out();
        assert_eq!(self.i_range.1, i_range.0);
        self.i_range.1 = i_range.1;

        self.last_block_idx += 1;
        assert!(self.last_block_idx < self.blocks.len());
        let block = &mut self.blocks[self.last_block_idx];
        assert!(block.i == i_range.1);
        assert!(block.j_range == j_range);
    }

    pub fn compute_next_block(
        &mut self,
        i_range: IRange,
        mut j_range: JRange,
        viz: &mut impl VisualizerInstance,
    ) {
        // Ensure that the j_range only grows.
        if let Some(next_block) = self.blocks.get(self.last_block_idx + 1) {
            j_range = JRange(
                min(j_range.0, next_block.j_range.0),
                max(j_range.1, next_block.j_range.1),
            );

            self.unique_rows -= next_block.j_range.exclusive_len() as usize / W;
        }
        let j_range = j_range.round_out();

        if self.trace && !self.params.sparse {
            // This is extracted to a separate function for reuse during traceback.
            return self.fill_block(i_range, j_range, viz);
        }

        assert_eq!(i_range.0, self.i_range.1);
        self.i_range.1 = i_range.1;

        if DEBUG {
            eprintln!("Compute block {:?} {:?}", i_range, j_range);
        }
        let v_range = j_range.0 as usize / W..j_range.1 as usize / W;
        self.unique_rows += v_range.len();
        // Get top/bot values in the previous column for the new j_range.
        let block = &mut self.blocks[self.last_block_idx];
        let mut top_val = block.index(j_range.0);
        let mut bot_val = block.index(j_range.1);

        if !self.trace && !self.params.incremental_doubling {
            // Update the existing `v` vector in the single block.
            top_val += i_range.len();
            // Ugly rust workaround: have to take out the block and put it back it.
            let mut v = std::mem::take(&mut block.v);
            bot_val += compute_columns(
                self.params,
                &self.a,
                &self.b,
                i_range,
                v_range.clone(),
                &mut v[v_range.clone().clone()],
                &mut self.h,
                &mut self.computed_rows,
                HMode::None,
                viz,
            );
            let next_block = &mut self.blocks[self.last_block_idx];
            next_block.v = v;
            next_block.i = i_range.1;
            next_block.j_range = j_range;
            next_block.top_val = top_val;
            next_block.bot_val = bot_val;
            next_block.check_top_bot_val();
            return;
        }

        // Compute the new `v` at the end of the `i_range` and push a new block.
        assert!(self.params.sparse);

        // Reuse memory from an existing block if possible.
        // Otherwise, push a new block.
        if self.last_block_idx + 1 == self.blocks.len() {
            self.blocks.push(Block::default());
        } else {
            let next_block = &mut self.blocks[self.last_block_idx + 1];
            assert_eq!(
                next_block.i, i_range.1,
                "Reused block for {} actually used to be for {}",
                next_block.i, i_range.1
            );
        };

        // Some trickery two access two elements at the same time.
        let [prev_block, next_block] = &mut self.blocks[self.last_block_idx..].split_array_mut().0;

        // Update the block properties.
        next_block.i = i_range.1;
        next_block.bot_val = bot_val;
        next_block.top_val = top_val + i_range.len();

        let mut v = std::mem::take(&mut next_block.v);
        // If no fixed_j_range was set, just compute everything.
        // TODO: Also just compute everything if the range is small anyway.
        // Fragmenting into smaller slices breaks SIMD and is slower.
        let bottom_delta = if self.params.incremental_doubling
            // New fixed range of previous block.
            && let Some(prev_fixed) = prev_block.fixed_j_range
            // Old fixed range of next block.
            && let Some(next_fixed) = next_block.fixed_j_range
        {
            let prev_fixed = prev_fixed;
            let next_fixed = next_fixed;
            // New range of next block.
            let new_range = j_range;
            // New j_h.
            // TODO: This is mutable and can be modified below to ensure
            // that the j_range before new_j_h has size a multiple of 8*W.
            let new_j_h = prev_fixed.1;

            let offset = new_range.0 as usize / W;

            // If there is already a fixed range here, a corresponding j_h, and the ranges before/after the fixed part do not overlap, then do a 3-range split:
            // range 0: everything before the fixed part.  h not used.
            // range 1: from previous j_h to new j_h.      h is updated.
            // range 2: from new j_h to end.               h is input.
            //
            // Otherwise, do a 2-range split:
            // range 01: everything before the new j_h.    h is output.
            // range  2: from new j_h to end.              h is output.
            let bottom_delta = if next_block.fixed_j_range.is_some()
                    && let Some(old_j_h) = next_block.j_h
                    && next_fixed.0 < old_j_h {
                resize_v_with_fixed(prev_block, next_block, j_range, &mut v);

                assert!(new_range.0 <= next_fixed.0);
                let v_range_0 = new_range.0 as usize / W..next_fixed.0 as usize / W;
                compute_columns(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range_0.clone(),
                    &mut v[v_range_0.start - offset..v_range_0.end - offset],
                    &mut self.h,
                    &mut self.computed_rows,
                    HMode::None,
                        viz,
                );

                assert!(old_j_h <= new_j_h, "j_h may only increase! i {i_range:?} old_j_h: {}, new_j_h: {}", old_j_h, new_j_h);
                //new_j_h = old_j_h + (new_j_h - old_j_h) / (8*WI) * (8*WI);
                let v_range_1 = old_j_h as usize / W..new_j_h as usize / W;
                compute_columns(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range_1.clone(),
                    &mut v[v_range_1.start - offset..v_range_1.end - offset],
                    &mut self.h,
                    &mut self.computed_rows,
                    HMode::Update,
                    viz
                );

                assert!(new_j_h <= new_range.1);
                let v_range_2 = new_j_h as usize / W..new_range.1 as usize / W;
                compute_columns(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range_2.clone(),
                    &mut v[v_range_2.start - offset..v_range_2.end - offset],
                    &mut self.h,
                    &mut self.computed_rows,
                    HMode::Input,
                    viz
                )
            } else {
                initialize_next_v(prev_block, j_range, &mut v);
                assert!(new_range.0 <= new_j_h);
                // Round new_j_h down to a multiple of 8*WI for better SIMD.
                //new_j_h = new_range.0 + (new_j_h - new_range.0) / (8*WI) * (8*WI);
                let v_range_01 = new_range.0 as usize / W..new_j_h as usize / W;
                compute_columns(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range_01.clone(),
                    &mut v[v_range_01.start - offset..v_range_01.end - offset],
                    &mut self.h,
                    &mut self.computed_rows,
                    HMode::Output,
                    viz
                );

                assert!(new_j_h <= new_range.1);
                let v_range_2 = new_j_h as usize / W..new_range.1 as usize / W;
                compute_columns(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range_2.clone(),
                    &mut v[v_range_2.start - offset..v_range_2.end - offset],
                    &mut self.h,
                    &mut self.computed_rows,
                    HMode::Input,
                    viz
                )
            };
            next_block.j_h = Some(new_j_h);
            next_block.offset = new_range.0;

            if cfg!(test) || DEBUG {
                // Redo the computation without the fixed range and test if they give the same results.
                let mut v2 = Vec::default();
                initialize_next_v(prev_block, j_range, &mut v2);
                let bottom_delta_2 = compute_columns(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range.clone(),
                    &mut v2,
                    &mut self.h,
                    &mut self.computed_rows,
                    HMode::None,
                    viz
                );
                assert_eq!(bottom_delta, bottom_delta_2);
                assert_eq!(v.len(), v2.len());
                if v != v2 {
                    for (i, (a, b)) in izip!(&v, &v2).enumerate() {
                        if a != b {
                            println!("{}+{}={}: {:?} != {:?}", i, offset, i+offset, a, b);
                        }
                    }

                    panic!("Vectors differ!");
                }
                assert_eq!(v, v2);
            }

            bottom_delta
        } else {
            // Incremental doubling disabled; just compute the entire `j_range`.
            initialize_next_v(prev_block, j_range, &mut v);
            let bottom_delta = compute_columns(
                self.params,
                &self.a,
                &self.b,
                i_range,
                v_range.clone(),
                &mut v,
                &mut self.h,
                &mut self.computed_rows,
                HMode::None,
                viz
            );
            next_block.offset = j_range.0;
            bottom_delta
        };
        self.last_block_idx += 1;
        let next_block = &mut self.blocks[self.last_block_idx];
        next_block.v = v;
        next_block.bot_val += bottom_delta;
        next_block.j_range = j_range;
        next_block.check_top_bot_val();
        // Will be updated later.
        //next_block.fixed_j_range = None;
    }

    pub fn last_i(&self) -> I {
        self.i_range.1
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
        assert!(self.blocks.last().unwrap().i == to.0);
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
            while self.last_block_idx > 0 && self.blocks[self.last_block_idx - 1].i >= to.0 {
                if DEBUG {
                    eprintln!(
                        "to {to:?} Pop block at i={}",
                        self.blocks[self.last_block_idx].i
                    );
                }
                self.pop_last_block();
            }

            // Try a Diagonal Transition based traceback first which should be faster for small distances.
            if self.params.dt_trace && to.0 > 0 {
                let prev_block = &self.blocks[self.last_block_idx - 1];
                if prev_block.i < to.0 - 1 {
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
                assert!(prev_block.i < to.0 && to.0 <= block.i);
                // If the previous block is the correct one, no need for further recomputation.
                if prev_block.i < to.0 - 1 || block.i > to.0 {
                    if DEBUG {
                        eprintln!("Expand previous block from {} to {}", prev_block.i, to.0);
                    }
                    let i_range = IRange(prev_block.i, to.0);
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
                        self.fill_block(i_range, j_range, viz);
                        if self.blocks[self.last_block_idx].index(to.1) == g {
                            break;
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
            // eprintln!("Update fixed_j_range from {:?}", self.blocks[self.last_block_idx].fixed_j_range);
            self.blocks[self.last_block_idx].fixed_j_range = Some(JRange(
                min(old.0, new.0),
                max(old.1, new.1),
            ).round_in());
            // eprintln!("Update fixed_j_range to {:?}", self.blocks[self.last_block_idx].fixed_j_range);
        } else {
            self.blocks[self.last_block_idx].fixed_j_range = fixed_j_range.map(|r| r.round_in());
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
            block.i == st.0,
            "Parent of state {st:?} but block.i is {}",
            block.i
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
        assert!(prev_block.i == st.0 - 1);

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
        let block_start = prev_block.i;
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
            elem.ext += extend_left_simd(&mut elem.i, prev_block.i, &mut j, a, b);
            *(&mut elem.i) == prev_block.i && prev_block.get(j) == Some(target_g)
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

    /// Iterate over columns `i_range` for `j_range`, storing a block per column.
    fn fill_block(
        &mut self,
        i_range: IRange,
        j_range: RoundedOutJRange,
        viz: &mut impl VisualizerInstance,
    ) {
        assert_eq!(
            i_range.0, self.i_range.1,
            "Current blocks range is {:?}. Computed range {i_range:?} does not fit!",
            self.i_range
        );
        self.i_range.1 = i_range.1;

        let j_range_rounded = j_range;
        let v_range = j_range_rounded.0 as usize / W..j_range_rounded.1 as usize / W;

        // Get top/bot values in the previous column for the new j_range_rounded.
        let prev_block = &self.blocks[self.last_block_idx];
        assert!(prev_block.i == i_range.0);

        let mut v = Vec::default();
        initialize_next_v(prev_block, j_range_rounded, &mut v);

        // 1. Push blocks for all upcoming columns.
        // 2. Take the vectors.
        // 3. Fill
        // 4. Put the vectors back.
        // 5. Compute bot values.

        let mut next_block = Block {
            // Will be resized in fill().
            v: vec![],
            i: i_range.0,
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
            next_block.i = i + 1;
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

fn compute_columns(
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

    // Do not count computed rows during traceback.
    if i_range.len() > 1 {
        if !(v_range.len() < computed_rows.len()) {
            computed_rows.resize(v_range.len() + 1, 0);
        }
        computed_rows[v_range.len()] += 1;
    }

    if i_range.len() > 1 && (cfg!(test) || DEBUG) {
        eprintln!("Compute i {i_range:?} x j {v_range:?} in mode {mode:?}");
    }

    let run = |h, exact_end| {
        if params.simd {
            // FIXME: Choose the optimal scalar function to use here.
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

/// Initialize the input vertical deltas for the given new range, by copying the overlap from the previous block.
/// Takes `v` as a mutable reference, so memory can be reused.
fn initialize_next_v(prev_block: &Block, j_range: RoundedOutJRange, v: &mut Vec<V>) {
    v.clear();
    // Make a 'working vector' with the correct range.
    v.resize(j_range.exclusive_len() as usize / W, V::one());
    // Copy the overlap from the last block.
    for target_idx in
        (max(j_range.0, prev_block.j_range.0)..min(j_range.1, prev_block.j_range.1)).step_by(W)
    {
        v[(target_idx - j_range.0) as usize / W] =
            prev_block.v[(target_idx - prev_block.offset) as usize / W];
    }
    assert_eq!(v.len(), j_range.exclusive_len() as usize / W);
}

/// Resize the `v` array to the `new_j_range_rounded`.
/// - Keep `new_block.fixed_j_range` intact.
/// - Copy over the rest from the previous block.
fn resize_v_with_fixed(
    prev_block: &Block,
    next_block: &mut Block,
    new_j_range: RoundedOutJRange,
    v: &mut Vec<V>,
) {
    let fixed = next_block.fixed_j_range.unwrap();
    assert!(
        new_j_range.0 <= next_block.j_range.0 && next_block.j_range.1 <= new_j_range.1,
        "New range must contain old range. old: {:?} new: {:?}",
        next_block.j_range,
        new_j_range
    );
    // 1. Resize the v array.
    v.resize(new_j_range.exclusive_len() as usize / W, V::one());
    let old_offset = next_block.offset;
    let new_offset = new_j_range.0;

    // 2. Move the fixed range for `next_block` to the right place.
    // NOTE: ALG:
    // It can happen that stored_h is larger than fixed_rounded.1,
    // meaning that the loop below will copy beyond the end of the fixed range.
    // That's OK though, since in this case, the end of the fixed range has
    // shrunk from the previous block. While that means some values there have f(u) > f_max,
    // these values are still guaranteed to be correct.
    let stored_h = next_block.j_h.unwrap();
    assert!(new_offset <= old_offset);
    assert!(fixed.0 <= stored_h);
    // NOTE: Moving existing fixed values is done before overwriting the prefix and suffix with 1.
    if new_offset < old_offset {
        // eprintln!(
        //     "Copy over fixed range from {} to {}",
        //     fixed_rounded.0 / WI,
        //     stored_h / WI
        // );
        for j in (fixed.0..stored_h).step_by(W).rev() {
            v[(j - new_offset) as usize / W] = v[(j - old_offset) as usize / W];
        }
    }

    // 3. Initialize the prefix and suffix with values from `prev_block`.
    // prefix: new.0..fixed.0
    for j in (new_j_range.0..fixed.0).step_by(W) {
        v[(j - new_offset) as usize / W] = prev_block.v[(j - prev_block.offset) as usize / W];
    }
    // suffix: from old j_h to the end.
    for j in (stored_h..new_j_range.1).step_by(W) {
        v[(j - new_offset) as usize / W] = prev_block
            .v
            .get((j - prev_block.offset) as usize / W)
            .copied()
            .unwrap_or(V::one());
    }
}
