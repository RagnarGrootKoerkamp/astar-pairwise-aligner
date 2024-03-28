//!
//! TODO: [fill_block] use a single allocation for all blocks in the block. Takes up to 2% of time.
//! TODO: [fill_block] store horizontal deltas in blocks, so that `parent` is more
//!       efficient and doesn't have to use relatively slow `block.index` operations.
//!       (NOTE though that this doesn't actually seem that bad in practice.)
//! TODO: Separate strong types for row `I` and 'block-row' `I*64`.

pub mod trace;

use std::{
    cmp::{max, min},
    ops::{Index, IndexMut},
    ptr::read_unaligned,
    time::Duration,
};

use itertools::{izip, Itertools};
use pa_bitpacking::{BitProfile, HEncoding, Profile, B, V};
use pa_types::*;
use pa_vis::VisualizerInstance;
use serde::{Deserialize, Serialize};

use super::*;
use crate::block::*;

type PA = <BitProfile as Profile>::A;
type PB = <BitProfile as Profile>::B;
type H = (B, B);

/// Parameters for BitBlock.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BlockParams {
    /// When true, `trace` mode only stores the last column of each block, instead of all columns.
    /// `cost` mode always stores only the last column.
    /// FIXME: REMOVE AND ALWAYS SET TO TRUE?
    pub sparse: bool,

    #[serde(default)]
    pub simd: bool,

    /// Disable instruction-level-parallelism and only run a single SIMD vector at a time.
    #[serde(default)]
    pub no_ilp: bool,

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
    pub fr_drop: I,
}

impl Default for BlockParams {
    fn default() -> Self {
        Self {
            sparse: true,
            simd: true,
            no_ilp: false,
            incremental_doubling: true,
            dt_trace: false,
            max_g: 40,
            fr_drop: 20,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BlockStats {
    pub num_blocks: usize,
    pub num_incremental_blocks: usize,
    pub computed_lanes: usize,
    pub unique_lanes: usize,

    pub t_compute: Duration,
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

    pub stats: BlockStats,
}

impl BlockParams {
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
            stats: BlockStats::default(),
        }
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
        assert_eq!(initial_j_range.0, 0);
        self.last_block_idx = 0;
        self.i_range = IRange(-1, 0);

        let fixed_j_range = initial_j_range;
        if let Some(block) = self.blocks.get(0) {
            initial_j_range = initial_j_range.union(*block.j_range);
        }
        let initial_j_range = initial_j_range.round_out();

        let block = if self.trace {
            // First column block, with more blocks pushed after.
            Block::first_col(fixed_j_range, initial_j_range)
        } else {
            // Block spanning the entire first column.
            Block {
                v: vec![V::one(); self.b.len()],
                i_range: IRange(-1, 0),
                original_j_range: fixed_j_range,
                j_range: initial_j_range,
                fixed_j_range: Some(fixed_j_range),
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
        self.stats.num_blocks += 1;
        let start = std::time::Instant::now();

        let original_j_range = j_range;
        let j_range = j_range.round_out();

        let v_range = j_range.v_range();
        self.stats.unique_lanes += v_range.len();

        if let Some(next_block) = self.blocks.get(self.last_block_idx + 1) {
            assert!(
                j_range.contains_range(*next_block.j_range),
                "j_range must grow"
            );
            self.stats.unique_lanes -= next_block.j_range.exclusive_len() as usize / W;
        }

        if DEBUG {
            eprintln!("Compute block {:?} {:?}", i_range, j_range);
        }

        if self.trace && !self.params.sparse {
            // This is extracted to a separate function for reuse during traceback.
            self.fill_with_blocks(i_range, original_j_range);
            viz.expand_block_simple(
                Pos(i_range.0, j_range.0),
                Pos(i_range.len(), j_range.exclusive_len()),
            );
            self.stats.t_compute += start.elapsed();
            return;
        }

        self.i_range.push(i_range);

        // Get top/bot values in the previous column for the new j_range.
        let prev_top_val = self.last_block().index(j_range.0);
        let prev_bot_val = self.last_block().index(j_range.1);
        if DEBUG {
            eprintln!("Prev top/bot: {prev_top_val}/{prev_bot_val}");
        }

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
                    &mut self.stats,
                    HMode::None,
                    viz,
                );
            // In this case there is only a single reused block. Overwrite its range.
            let block = &mut self.blocks[self.last_block_idx];
            block.i_range = i_range;
            block.original_j_range = original_j_range;
            block.j_range = j_range;
            block.top_val = top_val;
            block.bot_val = bot_val;
            block.check_top_bot_val();
            self.stats.t_compute += start.elapsed();
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
        let [prev_block, next_block] = &mut self.blocks[self.last_block_idx..]
            .first_chunk_mut()
            .unwrap();
        self.last_block_idx += 1;

        // Copy settings, but not the vector.
        let old_block = Block {
            v: vec![],
            ..*next_block
        };
        // Overwrite the nxt
        *next_block = Block {
            v: std::mem::take(&mut next_block.v),
            i_range,
            original_j_range,
            j_range,
            fixed_j_range: next_block.fixed_j_range,
            offset: j_range.0,
            top_val: prev_top_val + i_range.len(),
            // This will be incremented with the horizontal bottom delta later.
            bot_val: prev_bot_val,
            // This will be set later based on whether incremental_doubling is enabled.
            j_h: None,
        };

        // If no incremental doubling or no fixed_j_range was set, just compute everything.
        // TODO: Also just compute everything if the range is small anyway.
        // Fragmenting into smaller slices breaks SIMD and is slower.
        if !self.params.incremental_doubling || prev_block.fixed_j_range.is_none() {
            // Incremental doubling disabled; just compute the entire `j_range`.
            init_v_with_overlap(prev_block, next_block);
            next_block.bot_val += compute_block(
                self.params,
                &self.a,
                &self.b,
                i_range,
                v_range.clone(),
                &mut next_block.v,
                &mut self.h,
                &mut self.stats,
                HMode::None,
                viz,
            );
            next_block.check_top_bot_val();
            self.stats.t_compute += start.elapsed();
            return;
        }

        // Do incremental doubling.

        let prev_fixed = prev_block.fixed_j_range.unwrap().round_in();
        let old_fixed = old_block.fixed_j_range;

        // New j_h.
        next_block.j_h = Some(prev_fixed.1);
        let new_j_h = prev_fixed.1;
        let offset = j_range.v_range().start;

        let i_slice = i_range.0 as usize..i_range.1 as usize;

        let old_h = if DEBUG {
            self.h[i_slice.clone()].to_vec()
        } else {
            vec![]
        };

        // If there is already a fixed range here, a corresponding j_h, and the ranges before/after the fixed part do not overlap, then do a 3-range split:
        // range 0: everything before the fixed part.  h not used.
        // range 1: from previous j_h to new j_h.      h is updated.
        // range 2: from new j_h to end.               h is input.
        //
        // Otherwise, do a 2-range split:
        // range 01: everything before the new j_h.    h is output.
        // range  2: from new j_h to end.              h is output.
        //
        // FIXME(new): Only split when j_range >> 256.
        if let Some(old_j_h) = old_block.j_h
            && let Some(old_fixed) = old_fixed
            && (old_fixed.0 - 1).next_multiple_of(WI) < old_j_h
        {
            init_v_with_overlap_preserve_fixed(prev_block, &old_block, next_block);

            let v_range_0 = JRange(j_range.0, old_fixed.0 - 1).round_out().v_range();
            assert!(v_range_0.start <= v_range_0.end);
            // The part between next_fixed.0 and old_j_h is fixed and skipped!
            let v_range_1 = JRange(old_j_h, new_j_h).assert_rounded().v_range();
            assert!(
                v_range_1.start <= v_range_1.end,
                "j_h may only increase! i {i_range:?} old_j_h: {}, new_j_h: {}",
                old_j_h,
                new_j_h
            );
            let v_range_2 = JRange(new_j_h, j_range.1).assert_rounded().v_range();
            assert!(v_range_2.start <= v_range_2.end);
            if DEBUG {
                eprintln!("INIT1: {:?}", next_block.v);
            }

            // Compute the part before the fixed range without using input/output horizontal deltas.
            compute_block(
                self.params,
                &self.a,
                &self.b,
                i_range,
                v_range_0.clone(),
                &mut next_block.v[v_range_0.start - offset..v_range_0.end - offset],
                &mut self.h,
                &mut self.stats,
                HMode::None,
                viz,
            );

            // Update the horizontal deltas from old_j_h to new_j_h.
            if !v_range_1.is_empty() {
                compute_block(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range_1.clone(),
                    &mut next_block.v[v_range_1.start - offset..v_range_1.end - offset],
                    &mut self.h,
                    &mut self.stats,
                    HMode::Update,
                    viz,
                );
            }

            // Compute the part below new_j_h using the horizontal deltas.
            next_block.bot_val += compute_block(
                self.params,
                &self.a,
                &self.b,
                i_range,
                v_range_2.clone(),
                &mut next_block.v[v_range_2.start - offset..v_range_2.end - offset],
                &mut self.h,
                &mut self.stats,
                HMode::Input,
                viz,
            );
        } else {
            init_v_with_overlap(prev_block, next_block);

            let v_range_01 = JRange(j_range.0, new_j_h).assert_rounded().v_range();
            assert!(v_range_01.start <= v_range_01.end);
            let v_range_2 = JRange(new_j_h, j_range.1).assert_rounded().v_range();
            assert!(v_range_2.start <= v_range_2.end);

            // Note: We cannot skip an empty output range, since we still need to set -1 deltas along it.
            compute_block(
                self.params,
                &self.a,
                &self.b,
                i_range,
                v_range_01.clone(),
                &mut next_block.v[v_range_01.start - offset..v_range_01.end - offset],
                &mut self.h,
                &mut self.stats,
                HMode::Output,
                viz,
            );

            next_block.bot_val += compute_block(
                self.params,
                &self.a,
                &self.b,
                i_range,
                v_range_2.clone(),
                &mut next_block.v[v_range_2.start - offset..v_range_2.end - offset],
                &mut self.h,
                &mut self.stats,
                HMode::Input,
                viz,
            );
        };

        // Test incremental doubling: Redo the computation without the
        // fixed range and test if they give the same results.
        if (cfg!(test) || DEBUG) && !cfg!(feature = "example") {
            if let Some(old_j_h) = old_block.j_h {
                // Check whether the fixed row has correct values.
                eprintln!("DEBUG MODE: RECOMPUTE OLD FIXED H");
                let next_block_2 = &mut next_block.clone();
                init_v_with_overlap(prev_block, next_block_2);
                let h2 = self.h[i_slice.clone()].to_vec();
                let v_range = JRange(j_range.0, old_j_h).assert_rounded().v_range();
                let offset = j_range.v_range().start;
                compute_block(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range.clone(),
                    &mut next_block_2.v[v_range.start - offset..v_range.end - offset],
                    &mut self.h,
                    &mut self.stats,
                    HMode::Output,
                    viz,
                );
                assert_eq!(old_h, self.h[i_slice.clone()]);
                self.h[i_slice.clone()].copy_from_slice(&h2);
            }
            {
                // Check whether the fixed row has correct values.
                eprintln!("DEBUG MODE: RECOMPUTE UPDATED FIXED H");
                let next_block_2 = &mut next_block.clone();
                init_v_with_overlap(prev_block, next_block_2);
                let h2 = self.h[i_slice.clone()].to_vec();
                let v_range = JRange(j_range.0, new_j_h).assert_rounded().v_range();
                let offset = j_range.v_range().start;
                compute_block(
                    self.params,
                    &self.a,
                    &self.b,
                    i_range,
                    v_range.clone(),
                    &mut next_block_2.v[v_range.start - offset..v_range.end - offset],
                    &mut self.h,
                    &mut self.stats,
                    HMode::Output,
                    viz,
                );
                assert_eq!(h2, self.h[i_slice]);
            }
            eprintln!("DEBUG MODE: Recompute without incremental doubling");
            let next_block_2 = &mut next_block.clone();
            init_v_with_overlap(prev_block, next_block_2);
            eprintln!("INIT2: {:?}", next_block_2.v);
            let bot_diff = compute_block(
                self.params,
                &self.a,
                &self.b,
                i_range,
                v_range.clone(),
                &mut next_block_2.v,
                &mut self.h,
                &mut self.stats,
                HMode::None,
                viz,
            );
            next_block_2.bot_val = prev_bot_val + bot_diff;
            eprintln!("Bot diff: {bot_diff}");
            assert_eq!(next_block.top_val, next_block_2.top_val);
            assert_eq!(next_block.v, next_block_2.v);
            assert_eq!(next_block.bot_val, next_block_2.bot_val);
            eprintln!("Check top bot val");
            next_block_2.check_top_bot_val();
            next_block.check_top_bot_val();
        }
        self.stats.t_compute += start.elapsed();
    }

    pub fn last_block(&self) -> &Block {
        &self.blocks[self.last_block_idx]
    }

    pub fn next_block_j_range(&self) -> Option<JRange> {
        self.blocks.get(self.last_block_idx + 1).map(|f| *f.j_range)
    }

    // Update the fixed range, and make sure it only grows.
    pub fn set_last_block_fixed_j_range(&mut self, fixed_j_range: Option<JRange>) {
        if let Some(old) = self.blocks[self.last_block_idx].fixed_j_range
            && let Some(new) = fixed_j_range
        {
            self.blocks[self.last_block_idx].fixed_j_range = Some(old.union(new));
        } else {
            self.blocks[self.last_block_idx].fixed_j_range = fixed_j_range;
        }

        if let Some(fixed_j_range) = fixed_j_range {
            let block = &self.blocks[self.last_block_idx];
            assert!(block.original_j_range.contains_range(fixed_j_range));
        }
    }

    /// Store a single block for each column in `i_range`.
    fn fill_with_blocks(&mut self, i_range: IRange, original_j_range: JRange) {
        let j_range = original_j_range.round_out();
        self.i_range.push(i_range);
        let v_range = j_range.v_range();

        // Get top/bot values in the previous column for the new j_range_rounded.
        let prev_block = &self.blocks[self.last_block_idx];
        assert!(IRange::consecutive(prev_block.i_range, i_range));

        // 1. Push blocks for all upcoming columns.
        // 2. Take the vectors.
        // 3. Fill
        // 4. Put the vectors back.
        // 5. Compute bot values.

        let mut next_block = Block {
            // Will be resized in fill().
            v: vec![],
            i_range: IRange(i_range.0, i_range.0),
            original_j_range,
            j_range,
            offset: j_range.0,
            fixed_j_range: None,
            top_val: prev_block.index(j_range.0),
            // Will be set later.
            bot_val: 0,
            // bot_val: prev_block.index(j_range_rounded.1),
            // During traceback, we ignore any stored horizontal deltas.
            j_h: None,
        };

        init_v_with_overlap(prev_block, &mut next_block);

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
        if self.params.simd {
            pa_bitpacking::simd::fill::<2, H, 4>(
                &self.a[i_range.0 as usize..i_range.1 as usize],
                &self.b[v_range],
                h,
                &mut next_block.v,
                true,
                &mut values[..],
            );
        } else {
            pa_bitpacking::scalar::fill::<BitProfile, H>(
                &self.a[i_range.0 as usize..i_range.1 as usize],
                &self.b[v_range],
                h,
                &mut next_block.v,
                &mut values[..],
            );
        }

        // 4. 5.
        let mut bot_val =
            self.blocks[self.last_block_idx - i_range.len() as usize].index(j_range.1);
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

#[derive(Debug, PartialEq)]
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
    stats: &mut BlockStats,
    mode: HMode,
    viz: &mut impl VisualizerInstance,
) -> i32 {
    viz.expand_block_simple(
        Pos(i_range.0, v_range.start as I * WI),
        Pos(i_range.len(), v_range.len() as I * WI),
    );

    // Keep statistics on how many rows are computed at a time.
    // Skipped during traceback.
    if i_range.len() > 1 {
        if DEBUG | cfg!(test) {
            eprintln!("Compute i {i_range:?} x j {v_range:?} in mode {mode:?}");
        }

        stats.computed_lanes += v_range.len();
        stats.num_incremental_blocks += 1;
    }

    let run = |h: &mut [H], exact_end| {
        let a = &a[i_range.0 as usize..i_range.1 as usize];
        let b = &b[v_range];
        if params.simd {
            if params.no_ilp {
                pa_bitpacking::simd::compute::<1, H, 4>(a, b, h, v, exact_end) as I
            } else {
                pa_bitpacking::simd::compute::<2, H, 4>(a, b, h, v, exact_end) as I
            }
        } else {
            pa_bitpacking::scalar::row::<BitProfile, H>(a, b, h, v) as I
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
fn init_v_with_overlap(prev_block: &Block, next_block: &mut Block) {
    assert_eq!(next_block.offset, next_block.j_range.0);
    assert_eq!(prev_block.offset, prev_block.j_range.0);
    let prev_v_range = prev_block.j_range.v_range();
    let v_range = next_block.j_range.v_range();

    next_block.v.clear();
    next_block.v.resize(v_range.len(), V::one());

    // Copy the overlap from the last block.
    let overlap = RoundedOutJRange::intersection(next_block.j_range, prev_block.j_range).v_range();
    next_block.v[overlap.start - v_range.start..overlap.end - v_range.start].copy_from_slice(
        &prev_block.v[overlap.start - prev_v_range.start..overlap.end - prev_v_range.start],
    );
}

/// This prepares the `v` vector of vertical differences for a new block.
///
/// It copies the overlap with the previous block, and fills the rest with +1.
///
/// Unlike `init_v_with_overlap`, this preserves the existing `fixed_j_range` of the block.
fn init_v_with_overlap_preserve_fixed(
    prev_block: &Block,
    old_block: &Block,
    next_block: &mut Block,
) {
    let v = &mut next_block.v;

    // Some simplifying assumptions.
    assert!(prev_block.offset == prev_block.j_range.0);
    assert!(old_block.offset == old_block.j_range.0);
    assert!(next_block.offset == next_block.j_range.0);
    assert!(next_block.j_range.contains_range(*old_block.j_range));

    let prev_v_range = prev_block.j_range.v_range();
    let old_v_range = old_block.j_range.v_range();
    let v_range = next_block.j_range.v_range();
    assert!(prev_v_range.start <= v_range.start);
    assert!(v_range.start <= old_v_range.start);
    let preserve = JRange(
        old_block.fixed_j_range.unwrap().0 - 1,
        old_block.j_h.unwrap(),
    )
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
