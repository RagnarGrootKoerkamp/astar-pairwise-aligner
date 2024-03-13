//!
//! TODO: [fill_block] use a single allocation for all fronts in the block. Takes up to 2% of time.
//! TODO: [fill_block] store horizontal deltas in blocks, so that `parent` is more
//!       efficient and doesn't have to use relatively slow `front.index` operations.
//!       (NOTE though that this doesn't actually seem that bad in practice.)
//! TODO: Separate strong types for row `I` and 'block-row' `I*64`.
use super::*;
use itertools::{izip, Itertools};
use pa_bitpacking::{BitProfile, HEncoding, Profile, B, V, W};
use std::ops::{Index, IndexMut};

const DEBUG: bool = false;

const WI: I = W as I;

type PA = <BitProfile as Profile>::A;
type PB = <BitProfile as Profile>::B;
type H = (B, B);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct BitFrontsTag {
    /// When true, `trace` mode only stores one front per block, instead of all columns.
    /// `cost` mode always stores only the last front.
    pub sparse: bool,
    #[serde(default)]
    pub simd: bool,
    #[serde(default)]
    pub incremental_doubling: bool,
    #[serde(default)]
    pub dt_trace: bool,
    /// Do traceback up to this g. 0 disables the limit.
    #[serde(default)]
    pub max_g: Cost,
    #[serde(default)]
    pub drop: I,
}

impl Default for BitFrontsTag {
    fn default() -> Self {
        Self {
            sparse: true,
            simd: true,
            incremental_doubling: true,
            dt_trace: false,
            max_g: 40,
            drop: 20,
        }
    }
}

pub struct BitFronts {
    // Input/parameters.
    params: BitFrontsTag,
    trace: bool,
    a: Vec<PA>,
    b: Vec<PA>,
    cm: AffineCost<0>,

    // State.
    /// The list of fronts.
    /// NOTE: When using sparse traceback fronts, indices do not correspond to `i`!
    fronts: Vec<BitFront>,
    last_front_idx: usize,
    i_range: IRange,

    /// Store horizontal differences for row `j_h`.
    /// This allows for incremental band doubling.
    h: Vec<H>,

    /// The distribution of number of rows in `compute` calls.
    computed_rows: Vec<usize>,
    unique_rows: usize,
}

pub struct BitFront {
    /// The vertical differences at the end of front.
    v: Vec<V>,
    /// The column of this front.
    i: I,
    /// The 'input' range, that is rounded to `W=64` bits in practice.
    j_range: JRange,
    /// Helper for `NW`: the range of rows in this column with `f(u) <= f_max`.
    fixed_j_range: Option<JRange>,

    /// The `j` of the first element of `v`.
    /// Can be different from `j_range.0` when only a slice of the array corresponds to the `j_range`.
    offset: I,
    /// The value at the top of the rounded range, set on construction.
    top_val: Cost,
    /// The value at the bottom of the rounded range, computed after the range itself.
    bot_val: Cost,

    /// Store horizontal differences for row `j_h`.
    j_h: Option<I>,
}

/// Custom Clone implementation so we can `clone_from` `v`.
impl Clone for BitFront {
    fn clone(&self) -> Self {
        Self {
            v: self.v.clone(),
            i: self.i,
            j_range: self.j_range,
            fixed_j_range: self.fixed_j_range,
            offset: self.offset,
            top_val: self.top_val,
            bot_val: self.bot_val,
            j_h: None,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.v.clone_from(&source.v);
        self.i = source.i;
        self.j_range = source.j_range;
        self.fixed_j_range = source.fixed_j_range;
        self.offset = source.offset;
        self.top_val = source.top_val;
        self.bot_val = source.bot_val;
    }
}

impl Default for BitFront {
    fn default() -> Self {
        Self {
            v: vec![],
            i: 0,
            j_range: JRange(-1, -1),
            fixed_j_range: None,
            offset: 0,
            top_val: Cost::MAX,
            bot_val: Cost::MAX,
            j_h: None,
        }
    }
}

impl NwFront for BitFront {
    fn j_range(&self) -> JRange {
        self.j_range
    }
    fn j_range_rounded(&self) -> JRange {
        round(self.j_range)
    }
    fn fixed_j_range(&self) -> Option<JRange> {
        self.fixed_j_range
    }

    /// Get the value at the given index, by counting bits from the top or bottom.
    /// For `j` larger than the range, vertical deltas of `1` are assumed.
    fn index(&self, j: I) -> Cost {
        let rounded = round(self.j_range);
        assert!(
            rounded.0 <= j,
            "Cannot index front {} with range {:?} by {}",
            self.i,
            rounded,
            j
        );
        // All of rounded must be indexable.
        assert!(
            rounded.0 - self.offset >= 0,
            "Offset too large: {} - {} = {}, jrange {:?}",
            rounded.0,
            self.offset,
            rounded.0 - self.offset,
            self.j_range
        );
        assert!(
            rounded.1 - self.offset <= self.v.len() as I * WI,
            "v not long enough: {} - {} = {}, v len {}, jrange {:?}",
            rounded.1,
            self.offset,
            rounded.1 - self.offset,
            self.v.len() * W,
            self.j_range
        );

        if j > rounded.1 {
            return self.bot_val + (j - rounded.1) as Cost;
        }
        if j - rounded.0 < rounded.1 - j {
            // go from top
            let mut val = self.top_val;
            let mut j0 = rounded.0;
            while j0 + WI <= j {
                val += self.v[(j0 - self.offset) as usize / W].value() as Cost;
                j0 += WI;
            }
            val + self.v[(j0 - self.offset) as usize / W].value_of_prefix(j - j0) as Cost
        } else {
            // go from bottom
            let mut val = self.bot_val;
            let mut j1 = rounded.1;
            while j1 - WI > j {
                val -= self.v[(j1 - WI - self.offset) as usize / W].value() as Cost;
                j1 -= WI;
            }
            if j1 > j {
                val -= self.v[(j1 - WI - self.offset) as usize / W].value_of_suffix(j1 - j) as Cost
            }
            val
        }
    }

    fn get(&self, j: I) -> Option<Cost> {
        let rounded = round(self.j_range);
        if j < rounded.0 || j > rounded.1 {
            return None;
        }
        Some(self.index(j))
    }
}

fn round(j_range: JRange) -> JRange {
    JRange(j_range.0 / WI * WI, j_range.1.next_multiple_of(WI))
}

fn round_inward(j_range: JRange) -> JRange {
    JRange(j_range.0.next_multiple_of(WI), j_range.1 / WI * WI)
}

impl BitFront {
    fn first_col(j_range: JRange) -> Self {
        assert!(j_range.0 == 0);
        let rounded = round(j_range);
        Self {
            v: vec![V::one(); rounded.exclusive_len() as usize / W],
            i: 0,
            j_range,
            // In the first col, all computed values are correct directly.
            fixed_j_range: Some(j_range),
            offset: 0,
            top_val: 0,
            bot_val: rounded.exclusive_len(),
            j_h: None,
        }
    }

    /// Assert that the vertical difference between the top and bottom values is correct.
    fn check_top_bot_val(&self) {
        if !DEBUG {
            return;
        }
        let mut val = self.top_val;
        let rounded = round(self.j_range);
        for v in
            &self.v[(rounded.0 - self.offset) as usize / W..(rounded.1 - self.offset) as usize / W]
        {
            val += v.value();
        }
        assert_eq!(val, self.bot_val);
    }

    /// Get the difference from row `j` to `j+1`.
    fn get_diff(&self, j: I) -> Option<Cost> {
        if j < self.offset {
            return None;
        }
        let idx = (j - self.offset) as usize / W;
        if idx >= self.v.len() {
            return None;
        }
        let bit = (j - self.offset) as usize % W;

        Some(((self.v[idx].p() >> bit) & 1) as Cost - ((self.v[idx].m() >> bit) & 1) as Cost)
    }
}

impl NwFrontsTag<0usize> for BitFrontsTag {
    type Fronts<'a> = BitFronts;
    const BLOCKSIZE: I = 64;
    fn new<'a>(
        &self,
        trace: bool,
        a: Seq<'a>,
        b: Seq<'a>,
        cm: &'a AffineCost<0>,
    ) -> Self::Fronts<'a> {
        assert_eq!(*cm, AffineCost::unit());
        let (a, b) = BitProfile::build(a, b);
        BitFronts {
            params: *self,
            fronts: vec![],
            trace,
            cm: *cm,
            i_range: IRange(-1, 0),
            last_front_idx: 0,
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

impl Drop for BitFronts {
    fn drop(&mut self) {
        if !PRINT {
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

impl IndexMut<usize> for BitFronts {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.fronts[index]
    }
}

impl Index<usize> for BitFronts {
    type Output = BitFront;

    fn index(&self, index: usize) -> &Self::Output {
        &self.fronts[index]
    }
}

impl NwFronts<0usize> for BitFronts {
    type Front = BitFront;

    fn init(&mut self, mut initial_j_range: JRange) {
        assert!(initial_j_range.0 == 0);
        self.last_front_idx = 0;
        self.i_range = IRange(-1, 0);

        // eprintln!("Init first front for {:?}", initial_j_range);
        if let Some(front) = self.fronts.get(0) {
            initial_j_range = JRange(
                min(front.j_range.0, initial_j_range.0),
                max(front.j_range.1, initial_j_range.1),
            );
            // eprintln!("Upated initial range to {:?}", initial_j_range);
        }

        let front = if self.trace {
            // First column front, with more fronts pushed after.
            BitFront::first_col(initial_j_range)
        } else {
            // Front spanning the entire first column.
            BitFront {
                v: vec![V::one(); self.b.len()],
                i: 0,
                j_range: initial_j_range,
                fixed_j_range: Some(initial_j_range),
                offset: 0,
                top_val: 0,
                bot_val: round(initial_j_range).1,
                j_h: None,
            }
        };
        if self.fronts.is_empty() {
            self.fronts.push(front);
        } else {
            self.fronts[0] = front;
        }
        //self.computed_rows.fill(0);
    }

    // TODO: Maybe we should at some point drop the unused fronts?
    fn pop_last_front(&mut self) {
        assert!(self.i_range.1 == self.fronts[self.last_front_idx].i);
        self.last_front_idx -= 1;
        self.i_range.1 = self.fronts.get(self.last_front_idx).map_or(-1, |f| f.i);
    }

    fn reuse_next_block(&mut self, i_range: IRange, j_range: JRange) {
        assert_eq!(self.i_range.1, i_range.0);
        self.i_range.1 = i_range.1;

        self.last_front_idx += 1;
        assert!(self.last_front_idx < self.fronts.len());
        let front = &mut self.fronts[self.last_front_idx];
        assert!(front.i == i_range.1);
        assert!(front.j_range == j_range);
    }

    fn compute_next_block(
        &mut self,
        i_range: IRange,
        mut j_range: JRange,
        viz: &mut impl VisualizerInstance,
    ) {
        // Ensure that the j_range only grows.
        if let Some(next_front) = self.fronts.get(self.last_front_idx + 1) {
            j_range = JRange(
                min(j_range.0, next_front.j_range.0),
                max(j_range.1, next_front.j_range.1),
            );

            self.unique_rows -= (round(next_front.j_range).len() - 1) as usize / W;
        }

        if self.trace && !self.params.sparse {
            // This is extracted to a separate function for reuse during traceback.
            return self.fill_block(i_range, j_range, viz);
        }

        assert_eq!(i_range.0, self.i_range.1);
        self.i_range.1 = i_range.1;

        let j_range_rounded = round(j_range);
        if PRINT {
            // eprintln!("Compute block {:?} {:?}", i_range, j_range);
        }
        let v_range = j_range_rounded.0 as usize / W..j_range_rounded.1 as usize / W;
        self.unique_rows += v_range.len();
        // Get top/bot values in the previous column for the new j_range_rounded.
        let front = &mut self.fronts[self.last_front_idx];
        let mut top_val = front.index(j_range_rounded.0);
        let mut bot_val = front.index(j_range_rounded.1);

        if self.trace {
            // Compute the new `v` at the end of the `i_range` and push a new front.
            assert!(self.params.sparse);

            // Reuse memory from an existing front if possible.
            // Otherwise, push a new front.
            if self.last_front_idx + 1 == self.fronts.len() {
                self.fronts.push(BitFront::default());
            } else {
                let next_front = &mut self.fronts[self.last_front_idx + 1];
                assert_eq!(
                    next_front.i, i_range.1,
                    "Reused front for {} actually used to be for {}",
                    next_front.i, i_range.1
                );
            };

            // Some trickery two access two elements at the same time.
            let [prev_front, next_front] = self.fronts[self.last_front_idx..]
                .first_chunk_mut()
                .unwrap();

            // Update the front properties.
            next_front.i = i_range.1;
            next_front.bot_val = bot_val;
            next_front.top_val = top_val + i_range.len();

            let mut v = std::mem::take(&mut next_front.v);
            // If no fixed_j_range was set, just compute everything.
            // TODO: Also just compute everything if the range is small anyway.
            // Fragmenting into smaller slices breaks SIMD and is slower.
            let bottom_delta = if self.params.incremental_doubling
                // New fixed range of previous front.
                && let Some(prev_fixed) = prev_front.fixed_j_range
                // Old fixed range of next front.
                && let Some(next_fixed) = next_front.fixed_j_range
            {
                let prev_fixed = round_inward(prev_fixed);
                let next_fixed = round_inward(next_fixed);
                // New range of next front.
                let new_range = round(j_range);
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
                let bottom_delta = if next_front.fixed_j_range.is_some()
                    && let Some(old_j_h) = next_front.j_h
                    && next_fixed.0 < old_j_h
                {
                    resize_v_with_fixed(prev_front, next_front, j_range, &mut v);

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

                    assert!(
                        old_j_h <= new_j_h,
                        "j_h may only increase! i {i_range:?} old_j_h: {}, new_j_h: {}",
                        old_j_h,
                        new_j_h
                    );
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
                        viz,
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
                        viz,
                    )
                } else {
                    initialize_next_v(prev_front, j_range_rounded, &mut v);
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
                        viz,
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
                        viz,
                    )
                };
                next_front.j_h = Some(new_j_h);
                next_front.offset = new_range.0;

                if cfg!(test) || DEBUG {
                    // Redo the computation without the fixed range and test if they give the same results.
                    let mut v2 = Vec::default();
                    initialize_next_v(prev_front, j_range_rounded, &mut v2);
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
                        viz,
                    );
                    assert_eq!(bottom_delta, bottom_delta_2);
                    assert_eq!(v.len(), v2.len());
                    if v != v2 {
                        for (i, (a, b)) in izip!(&v, &v2).enumerate() {
                            if a != b {
                                println!("{}+{}={}: {:?} != {:?}", i, offset, i + offset, a, b);
                            }
                        }

                        panic!("Vectors differ!");
                    }
                    assert_eq!(v, v2);
                }

                bottom_delta
            } else {
                // Incremental doubling disabled; just compute the entire `j_range`.
                initialize_next_v(prev_front, j_range_rounded, &mut v);
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
                    viz,
                );
                next_front.offset = j_range_rounded.0;
                bottom_delta
            };
            self.last_front_idx += 1;
            let next_front = &mut self.fronts[self.last_front_idx];
            next_front.v = v;
            next_front.bot_val += bottom_delta;
            next_front.j_range = j_range;
            next_front.check_top_bot_val();
            // Will be updated later.
            //next_front.fixed_j_range = None;
        } else {
            // Update the existing `v` vector in the single front.
            top_val += i_range.len();
            // Ugly rust workaround: have to take out the front and put it back it.
            let mut v = std::mem::take(&mut front.v);
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
            let next_front = &mut self.fronts[self.last_front_idx];
            next_front.v = v;
            next_front.i = i_range.1;
            next_front.j_range = j_range;
            next_front.top_val = top_val;
            next_front.bot_val = bot_val;
            next_front.check_top_bot_val();
        }
    }

    fn cm(&self) -> &AffineCost<0> {
        &self.cm
    }

    fn last_i(&self) -> I {
        self.i_range.1
    }

    fn last_front(&self) -> &Self::Front {
        &self.fronts[self.last_front_idx]
    }

    fn next_front_j_range(&self) -> Option<JRange> {
        self.fronts.get(self.last_front_idx + 1).map(|f| f.j_range)
    }

    /// Traceback the back from `from` to `to`.
    ///
    /// This requires `self.trace` to be `true`. In case of sparse fronts, this
    /// recomputes fronts as needed.
    fn trace(
        &mut self,
        a: Seq,
        b: Seq,
        from: State,
        mut to: State,
        viz: &mut impl VisualizerInstance,
    ) -> AffineCigar {
        assert!(self.trace);
        assert!(self.fronts.last().unwrap().i == to.i);
        let mut cigar = AffineCigar::default();
        let mut g = self.fronts[self.last_front_idx].index(to.j);

        if PRINT {
            eprintln!("Trace from distance {g}");
        }

        let mut dt_trace_tries = 0;
        let mut dt_trace_success = 0;
        let mut dt_trace_fallback = 0;

        let cached_dt_fronts =
            &mut vec![FrontElem::default(); (self.params.max_g + 1).pow(2) as usize];

        while to != from {
            // Remove fronts to the right of `to`.
            while self.last_front_idx > 0 && self.fronts[self.last_front_idx - 1].i >= to.i {
                if PRINT {
                    eprintln!(
                        "to {to:?} Pop front at i={}",
                        self.fronts[self.last_front_idx].i
                    );
                }
                self.pop_last_front();
            }

            // Try a Diagonal Transition based traceback first which should be faster for small distances.
            if self.params.dt_trace && to.i > 0 {
                let prev_front = &self.fronts[self.last_front_idx - 1];
                if prev_front.i < to.i - 1 {
                    dt_trace_tries += 1;
                    if let Some(new_to) = self.dt_trace_block(
                        a,
                        b,
                        to,
                        &mut g,
                        prev_front,
                        &mut cigar,
                        cached_dt_fronts,
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

            // In case of sparse fronts, fill missing columns by recomputing the
            // block and storing all columns.
            if self.params.sparse && to.i > 0 {
                let front = &self.fronts[self.last_front_idx];
                let prev_front = &self.fronts[self.last_front_idx - 1];
                assert!(prev_front.i < to.i && to.i <= front.i);
                // If the previous front is the correct one, no need for further recomputation.
                if prev_front.i < to.i - 1 || front.i > to.i {
                    if PRINT {
                        eprintln!("Expand previous front from {} to {}", prev_front.i, to.i);
                    }
                    let i_range = IRange(prev_front.i, to.i);
                    let j_range = JRange(front.j_range.0, to.j);
                    self.pop_last_front();
                    // NOTE: It's unlikely the full (large) `j_range` is needed to trace back through the current block.
                    // 1. We don't need states with `j > to.j`, because the path (in reverse direction) can never go down.
                    // 2. It's unlikely we'll need all states starting at the (possibly much smaller) `j_range.0`.
                    //    Instead, we do an exponential search for the start of the `j_range`, starting at `to.j-2*i_range.len()`.
                    //    The block is high enough once the cost to `to` equals `g`.
                    let mut height = max(j_range.exclusive_len(), i_range.len() * 5 / 4);
                    loop {
                        let j_range = JRange(max(j_range.1 - height, 0), j_range.1);
                        if PRINT {
                            eprintln!("Fill block {:?} {:?}", i_range, j_range);
                        }
                        self.fill_block(i_range, j_range, viz);
                        if self.fronts[self.last_front_idx].index(to.j) == g {
                            break;
                        }
                        // Pop all the computed fronts.
                        for _i in i_range.0..i_range.1 {
                            self.pop_last_front();
                        }
                        // Try again with a larger height.
                        height *= 2;
                    }
                }
            }

            if PRINT && to.i % 256 == 0 {
                eprintln!(
                    "Parent of {to:?} at distance {g} with range {:?}",
                    self.fronts[self.last_front_idx].j_range
                );
            }
            let (parent, cigar_elem) = self.parent(to, &mut g);
            to = parent;
            cigar.push_elem(cigar_elem);
        }
        if PRINT {
            eprintln!("dt_trace_tries:    {:>7}", dt_trace_tries);
            eprintln!("dt_trace_success:  {:>7}", dt_trace_success);
            eprintln!("dt_trace_fallback: {:>7}", dt_trace_fallback);
        }
        assert_eq!(g, 0);
        cigar.reverse();
        cigar
    }

    // Update the fixed range, and make sure it only grows.
    fn set_last_front_fixed_j_range(&mut self, fixed_j_range: Option<JRange>) {
        assert!(fixed_j_range.is_some());
        if let Some(old) = self.fronts[self.last_front_idx].fixed_j_range
            && let Some(new) = fixed_j_range
        {
            // eprintln!("Update fixed_j_range from {:?}", self.fronts[self.last_front_idx].fixed_j_range);
            self.fronts[self.last_front_idx].fixed_j_range =
                Some(JRange(min(old.0, new.0), max(old.1, new.1)));
            // eprintln!("Update fixed_j_range to {:?}", self.fronts[self.last_front_idx].fixed_j_range);
        } else {
            self.fronts[self.last_front_idx].fixed_j_range = fixed_j_range;
        }
    }
}

#[derive(Clone, Copy)]
struct FrontElem {
    /// The current column.
    i: I,
    /// The length of the extension to get here.
    ext: I,
    /// The diagonal of the parent relative to this one.
    parent_d: I,
}
impl Default for FrontElem {
    fn default() -> Self {
        FrontElem {
            i: I::MAX,
            ext: 0,
            parent_d: 0,
        }
    }
}
impl FrontElem {
    fn reset(&mut self) {
        *self = FrontElem::default();
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
            *(a[*i as usize - 8..].as_ptr() as *const usize)
                ^ *(b[*j as usize - 8..].as_ptr() as *const usize)
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

impl BitFronts {
    /// Find the parent of `st`.
    /// NOTE: This assumes that `st.i` is in the last front, and that the front before is for `st.i-1`.
    /// `g`: distance to `st`.
    /// `block_start`: the IRange.0 of the previous block.
    /// ALG: NOTE: Greedy backward matching is OK (it is guaranteed that all
    /// computed cells reached this way have the same score). But note that this
    /// may end up outside the computed area. In that case we use insertions or
    /// deletions as needed to get back.
    fn parent(&self, mut st: State, g: &mut Cost) -> (State, AffineCigarElem) {
        let front = &self.fronts[self.last_front_idx];
        assert!(
            front.i == st.i,
            "Parent of state {st:?} but front.i is {}",
            front.i
        );

        // Greedy matching.
        let mut cnt = 0;
        // TODO: SIMD using raw A and B.
        while st.i > 0 && st.j > 0 && BitProfile::is_match(&self.a, &self.b, st.i - 1, st.j - 1) {
            cnt += 1;
            st.i -= 1;
            st.j -= 1;
        }
        if cnt > 0 {
            return (
                st,
                AffineCigarElem {
                    op: AffineCigarOp::Match,
                    cnt,
                },
            );
        }

        // Vertical delta (insert).
        // (This is first since it only needs a single delta bit, instead of an index() call.)
        let vd = front.get_diff(st.j - 1);
        if vd == Some(1) {
            *g -= 1;
            return (
                State {
                    i: st.i,
                    j: st.j - 1,
                    layer: None,
                },
                AffineCigarElem {
                    op: AffineCigarOp::Ins,
                    cnt: 1,
                },
            );
        }

        let prev_front = &self.fronts[self.last_front_idx - 1];
        assert!(prev_front.i == st.i - 1);

        // Horizontal delta (delete).
        let hd = *g - prev_front.index(st.j);
        if hd == 1 {
            *g -= 1;
            return (
                State {
                    i: st.i - 1,
                    j: st.j,
                    layer: None,
                },
                AffineCigarElem {
                    op: AffineCigarOp::Del,
                    cnt: 1,
                },
            );
        }

        // Diagonal delta (substitution).
        // This edge case happens when entering the previous front exactly in
        // the bottom-most row, where no vertical delta is available.
        let dd = if st.j > prev_front.j_range.1 {
            assert_eq!(st.j, prev_front.j_range.1 + 1);
            1
        } else {
            prev_front.get_diff(st.j - 1).unwrap() + hd
        };
        if dd == 1 {
            *g -= 1;
            return (
                State {
                    i: st.i - 1,
                    j: st.j - 1,
                    layer: None,
                },
                AffineCigarElem {
                    op: AffineCigarOp::Sub,
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
        st: State,
        g_st: &mut Cost,
        prev_front: &BitFront,
        cigar: &mut AffineCigar,
        fronts: &mut Vec<FrontElem>,
    ) -> Option<State> {
        // eprintln!(
        //     "DT Trace from {st:?} with g={g_st} back to {}",
        //     prev_front.i
        // );
        let block_start = prev_front.i;
        // Returns true when `end_i` is reached.
        // The front stores the leftmost reachable column at distance g in diagonal d relative to st.
        // The element for (g,d) is at position g*g+g+d.
        fronts[0] = FrontElem {
            i: st.i,
            ext: 0,
            parent_d: 0,
        };

        fn index(g: Cost, d: I) -> usize {
            (g * g + g + d) as usize
        }
        fn get(fronts: &Vec<FrontElem>, g: Cost, d: I) -> FrontElem {
            fronts[index(g, d)]
        }
        fn get_mut(fronts: &mut Vec<FrontElem>, g: Cost, d: I) -> &mut FrontElem {
            &mut fronts[index(g, d)]
        }

        fn trace(
            fronts: &Vec<FrontElem>,
            mut g: Cost,
            mut d: I,
            st: State,
            g_st: &mut Cost,
            block_start: I,
            cigar: &mut AffineCigar,
        ) -> State {
            //eprintln!("TRACE");
            let new_st = State {
                i: block_start,
                j: st.j - (st.i - block_start) - d,
                layer: None,
            };
            *g_st -= g;
            let mut ops = vec![];
            loop {
                let fr = get(fronts, g, d);
                if fr.ext > 0 {
                    //eprintln!("Ext: {}", fr.ext);
                    ops.push(AffineCigarElem {
                        op: AffineCigarOp::Match,
                        cnt: fr.ext,
                    })
                }
                if g == 0 {
                    break;
                }
                g -= 1;
                d += fr.parent_d;
                let op = match fr.parent_d {
                    -1 => AffineCigarOp::Ins,
                    0 => AffineCigarOp::Sub,
                    1 => AffineCigarOp::Del,
                    _ => panic!(),
                };
                //eprintln!("Op: {:?}", op);
                ops.push(AffineCigarElem { op, cnt: 1 });
            }
            for e in ops.into_iter().rev() {
                cigar.push_elem(e);
            }
            new_st
        }

        let mut g = 0 as Cost;

        // Extend up to the start of the previous front and check if the distance is correct.
        let extend_left_simd_and_check = |elem: &mut FrontElem, mut j: I, target_g: Cost| -> bool {
            elem.ext += extend_left_simd(&mut elem.i, prev_front.i, &mut j, a, b);
            *(&mut elem.i) == prev_front.i && prev_front.get(j) == Some(target_g)
        };

        if extend_left_simd_and_check(&mut fronts[0], st.j, 0) {
            return Some(trace(&fronts, 0, 0, st, g_st, block_start, cigar));
        }
        //eprintln!("extend d=0 from {:?} to {}", st, fronts[0][0].i);

        let mut d_range = (0, 0);
        loop {
            let ng = g + 1;
            // Init next front

            let end_idx = index(ng, d_range.1 + 1);
            if fronts.len() <= end_idx {
                fronts.resize(end_idx + 1, FrontElem::default());
            }
            for fe in &mut fronts[index(ng, d_range.0 - 1)..=end_idx] {
                fe.reset();
            }

            // EXPAND.
            //eprintln!("expand");
            for d in d_range.0..=d_range.1 {
                let fr = get(fronts, g, d);
                //eprintln!("Expand g={} d={} i={}", g, d, fr.i);
                fn update(x: &mut FrontElem, y: I, d: I) {
                    if y < x.i {
                        //eprintln!("update d={d} from {} to {}", x.i, y);
                        x.i = y;
                        x.parent_d = d;
                    }
                }
                update(&mut get_mut(fronts, ng, d - 1), fr.i - 1, 1);
                update(&mut get_mut(fronts, ng, d), fr.i - 1, 0);
                update(&mut get_mut(fronts, ng, d + 1), fr.i, -1);
            }
            g += 1;
            d_range.0 -= 1;
            d_range.1 += 1;

            // Extend.
            let mut min_i = I::MAX;
            for d in d_range.0..=d_range.1 {
                let fr = get_mut(fronts, g, d);
                if fr.i == I::MAX {
                    continue;
                }
                let j = st.j - (st.i - fr.i) - d;
                // let old_i = fr.i;
                if extend_left_simd_and_check(fr, j, *g_st - g) {
                    return Some(trace(&fronts, g, d, st, g_st, block_start, cigar));
                }
                // eprintln!("extend d={d} from {} to {}", Pos(old_i, j), fr.i);
                min_i = min(min_i, fr.i);
            }

            if g == self.params.max_g {
                return None;
            }

            // Shrink diagonals more than 5 behind.
            if self.params.drop > 0 {
                while d_range.0 < d_range.1
                    && get(fronts, g, d_range.0).i > min_i + self.params.drop
                {
                    d_range.0 += 1;
                }
                while d_range.0 < d_range.1
                    && get(fronts, g, d_range.1).i > min_i + self.params.drop
                {
                    d_range.1 -= 1;
                }
            }
        }
    }

    /// Iterate over columns `i_range` for `j_range`, storing a front per column.
    fn fill_block(&mut self, i_range: IRange, j_range: JRange, viz: &mut impl VisualizerInstance) {
        assert_eq!(
            i_range.0, self.i_range.1,
            "Current fronts range is {:?}. Computed range {i_range:?} does not fit!",
            self.i_range
        );
        self.i_range.1 = i_range.1;

        let j_range_rounded = round(j_range);
        let v_range = j_range_rounded.0 as usize / W..j_range_rounded.1 as usize / W;

        // Get top/bot values in the previous column for the new j_range_rounded.
        let prev_front = &self.fronts[self.last_front_idx];
        assert!(prev_front.i == i_range.0);

        let mut v = Vec::default();
        initialize_next_v(prev_front, j_range_rounded, &mut v);

        // 1. Push fronts for all upcoming columns.
        // 2. Take the vectors.
        // 3. Fill
        // 4. Put the vectors back.
        // 5. Compute bot values.

        let mut next_front = BitFront {
            // Will be resized in fill().
            v: vec![],
            i: i_range.0,
            j_range,
            offset: j_range_rounded.0,
            fixed_j_range: None,
            top_val: prev_front.index(j_range_rounded.0),
            // Will be set later.
            bot_val: 0,
            // bot_val: prev_front.index(j_range_rounded.1),
            // During traceback, we ignore any stored horizontal deltas.
            j_h: None,
        };

        // 1.
        for i in i_range.0..i_range.1 {
            // Along the top row, horizontal deltas are 1.
            next_front.i = i + 1;
            next_front.top_val += 1;
            self.last_front_idx += 1;
            if self.last_front_idx == self.fronts.len() {
                self.fronts.push(next_front.clone());
            } else {
                self.fronts[self.last_front_idx].clone_from(&next_front);
            }
        }

        // 2.
        let mut values = vec![vec![]; i_range.len() as usize];
        for (front, vv) in izip!(
            &mut self.fronts
                [self.last_front_idx + 1 - i_range.len() as usize..=self.last_front_idx],
            values.iter_mut()
        ) {
            *vv = std::mem::take(&mut front.v);
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
            self.fronts[self.last_front_idx - i_range.len() as usize].index(j_range_rounded.1);
        for (front, vv, h) in izip!(
            &mut self.fronts
                [self.last_front_idx + 1 - i_range.len() as usize..=self.last_front_idx],
            values.into_iter(),
            h.iter(),
        ) {
            front.v = vv;
            bot_val += h.value();
            front.bot_val = bot_val;
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
    params: BitFrontsTag,
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

/// Initialize the input vertical deltas for the given new range, by copying the overlap from the previous front.
/// Takes `v` as a mutable reference, so memory can be reused.
fn initialize_next_v(prev_front: &BitFront, j_range_rounded: JRange, v: &mut Vec<V>) {
    v.clear();
    // Make a 'working vector' with the correct range.
    v.resize(j_range_rounded.exclusive_len() as usize / W, V::one());
    // Copy the overlap from the last front.
    let prev_rounded = round(prev_front.j_range);
    for target_idx in
        (max(j_range_rounded.0, prev_rounded.0)..min(j_range_rounded.1, prev_rounded.1)).step_by(W)
    {
        v[(target_idx - j_range_rounded.0) as usize / W] =
            prev_front.v[(target_idx - prev_front.offset) as usize / W];
    }
    assert_eq!(v.len(), j_range_rounded.exclusive_len() as usize / W);
}

/// Resize the `v` array to the `new_j_range_rounded`.
/// - Keep `new_front.fixed_j_range` intact.
/// - Copy over the rest from the previous front.
fn resize_v_with_fixed(
    prev_front: &BitFront,
    next_front: &mut BitFront,
    new_j_range: JRange,
    v: &mut Vec<V>,
) {
    // Simplifying assumption.
    let new_rounded = round(new_j_range);
    let fixed_rounded = round_inward(next_front.fixed_j_range.unwrap());
    assert!(
        new_rounded.0 <= next_front.j_range.0 && next_front.j_range.1 <= new_rounded.1,
        "New range must contain old range. old: {:?} new: {:?}",
        next_front.j_range,
        new_rounded
    );
    // 1. Resize the v array.
    v.resize(new_rounded.exclusive_len() as usize / W, V::one());
    let old_offset = next_front.offset;
    let new_offset = new_rounded.0;

    // 2. Move the fixed range for `next_front` to the right place.
    // NOTE: ALG:
    // It can happen that stored_h is larger than fixed_rounded.1,
    // meaning that the loop below will copy beyond the end of the fixed range.
    // That's OK though, since in this case, the end of the fixed range has
    // shrunk from the previous front. While that means some values there have f(u) > f_max,
    // these values are still guaranteed to be correct.
    let stored_h = next_front.j_h.unwrap();
    assert!(new_offset <= old_offset);
    assert!(fixed_rounded.0 <= stored_h);
    // NOTE: Moving existing fixed values is done before overwriting the prefix and suffix with 1.
    if new_offset < old_offset {
        // eprintln!(
        //     "Copy over fixed range from {} to {}",
        //     fixed_rounded.0 / WI,
        //     stored_h / WI
        // );
        for j in (fixed_rounded.0..stored_h).step_by(W).rev() {
            v[(j - new_offset) as usize / W] = v[(j - old_offset) as usize / W];
        }
    }

    // 3. Initialize the prefix and suffix with values from `prev_front`.
    // prefix: new.0..fixed.0
    for j in (new_rounded.0..fixed_rounded.0).step_by(W) {
        v[(j - new_offset) as usize / W] = prev_front.v[(j - prev_front.offset) as usize / W];
    }
    // suffix: from old j_h to the end.
    for j in (stored_h..new_rounded.1).step_by(W) {
        v[(j - new_offset) as usize / W] = prev_front
            .v
            .get((j - prev_front.offset) as usize / W)
            .copied()
            .unwrap_or(V::one());
    }
}
