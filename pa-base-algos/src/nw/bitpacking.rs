//!
//! TODO: [fill_block] use a single allocation for all fronts in the block. Takes up to 2% of time.
//! TODO: SIMD for compute_block
//! TODO: [fill_block] store horizontal deltas in blocks, so that `parent` is more
//!       efficient and doesn't have to use relatively slow `front.index` operations.
//!       (NOTE though that this doesn't actually seem that bad in practice.)
use std::cmp::min;

use pa_bitpacking::{
    compute_columns, compute_columns_simd, profile, CompressedSequence, Profile, V, W,
};
use pa_types::{Cost, Seq, I};

use crate::edit_graph::AffineCigarOps;

use super::*;

const WI: I = W as I;

pub struct BitFront {
    /// The vertical differences in this front.
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

pub struct BitFronts {
    // Input/parameters.
    params: BitFrontsTag,
    trace: bool,
    a: CompressedSequence,
    b: Profile,
    cm: AffineCost<0>,

    // State.
    /// The list of fronts.
    /// NOTE: When using sparse traceback fronts, indices do not correspond to `i`!
    fronts: Vec<BitFront>,
    last_front_idx: usize,
    i_range: IRange,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct BitFrontsTag {
    /// When true, `trace` mode only stores one front per block, instead of all columns.
    /// `cost` most always stores only the last front.
    pub sparse: bool,
    #[serde(default)]
    pub simd: bool,
}

impl Default for BitFront {
    fn default() -> Self {
        Self {
            v: vec![],
            i: 0,
            j_range: JRange(-1, -1),
            fixed_j_range: Some(JRange(-1, -1)),
            offset: 0,
            top_val: Cost::MAX,
            bot_val: Cost::MAX,
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
        assert!(rounded.0 <= j);
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
        }
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
        initial_j_range: JRange,
    ) -> Self::Fronts<'a> {
        assert_eq!(*cm, AffineCost::unit());
        assert!(initial_j_range.0 == 0);
        let (a, b) = profile(a, b);
        BitFronts {
            params: *self,
            fronts: if trace {
                // First column front, with more fronts pushed after.
                vec![BitFront::first_col(initial_j_range)]
            } else {
                // Front spanning the entire first column.
                vec![BitFront {
                    v: vec![V::one(); b.len()],
                    i: 0,
                    j_range: initial_j_range,
                    fixed_j_range: None,
                    offset: 0,
                    top_val: 0,
                    bot_val: round(initial_j_range).1,
                }]
            },
            trace,
            cm: *cm,
            a,
            b,
            i_range: IRange(-1, 0),
            last_front_idx: 0,
        }
    }
}

impl NwFronts<0usize> for BitFronts {
    type Front = BitFront;

    fn compute_next_block(&mut self, i_range: IRange, j_range: JRange) {
        if self.trace && !self.params.sparse {
            // This is extracted to a separate function for reuse during traceback.
            return self.fill_block(i_range, j_range);
        }

        assert!(i_range.0 == self.i_range.1);
        self.i_range.1 = i_range.1;

        let j_range_rounded = round(j_range);
        let v_range = j_range_rounded.0 as usize / W..j_range_rounded.1 as usize / W;
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
            };

            // Some trickery two access two elements at the same time.
            let [front, new_front] = &mut self.fronts[self.last_front_idx..].split_array_mut().0;

            // Assign to new_front, reusing existing memory.
            new_front.clone_from(&BitFront {
                v: Vec::default(),
                i: i_range.1,
                j_range,
                fixed_j_range: None,
                offset: j_range_rounded.0,
                top_val: top_val + i_range.len(),
                bot_val,
            });

            initialize_next_v(front, j_range_rounded, &mut new_front.v);

            let mut v = std::mem::take(&mut new_front.v);
            let bottom_delta = self.compute_columns(i_range, v_range.clone(), &mut v);
            self.last_front_idx += 1;
            let new_front = &mut self.fronts[self.last_front_idx];
            new_front.v = v;
            new_front.bot_val += bottom_delta;
        } else {
            // Update the existing `v` vector in the single front.
            top_val += i_range.len();
            // Ugly rust workaround: have to take out the front and put it back it.
            let mut v = std::mem::take(&mut front.v);
            bot_val += self.compute_columns(i_range, v_range.clone(), &mut v[v_range.clone()]);
            let front = &mut self.fronts[self.last_front_idx];
            front.v = v;
            front.i = i_range.1;
            front.j_range = j_range;
            front.top_val = top_val;
            front.bot_val = bot_val;
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

    /// Find the parent of `st`.
    /// NOTE: This assumes that `st.i` is in the last front, and that the front before is for `st.i-1`.
    fn parent(&self, st: State) -> Option<(State, AffineCigarOps)> {
        let front = &self.fronts[self.last_front_idx];
        assert!(front.i == st.i);
        let prev_front = if st.i > 0 {
            let prev_front = &self.fronts[self.last_front_idx - 1];
            assert!(prev_front.i == st.i - 1);
            prev_front
        } else {
            front
        };

        let st_cost = front.index(st.j);
        let is_match = st.i > 0
            && st.j > 0
            && (self.b[(st.j - 1) as usize / W][self.a[st.i as usize - 1] as usize]
                & (1 << (st.j - 1) % WI))
                != 0;
        for (di, dj, edge, op) in [
            (-1, 0, 1, CigarOp::Del),
            (0, -1, 1, CigarOp::Ins),
            (
                -1,
                -1,
                if is_match { 0 } else { 1 },
                if is_match {
                    CigarOp::Match
                } else {
                    CigarOp::Sub
                },
            ),
        ] {
            if let Some(parent_cost) = (if di == 0 { front } else { prev_front }).get(st.j + dj) {
                if st_cost == parent_cost + edge {
                    return Some((
                        State {
                            i: st.i + di,
                            j: st.j + dj,
                            layer: None,
                        },
                        [Some(op.into()), None],
                    ));
                }
            }
        }
        None
    }

    /// Traceback the back from `from` to `to`.
    ///
    /// This requires `self.trace` to be `true`. In case of sparse fronts, this
    /// recomputes fronts as needed.
    fn trace(&mut self, from: State, mut to: State) -> AffineCigar {
        assert!(self.trace);
        assert!(self.fronts.last().unwrap().i == to.i);
        let mut cigar = AffineCigar::default();
        let mut g = self.fronts[self.last_front_idx].index(to.j);

        while to != from {
            // Remove fronts to the right of `to`.
            while self.fronts[self.last_front_idx].i > to.i {
                self.pop_last_front();
            }

            // In case of sparse fronts, fill missing columns by recomputing the
            // block and storing all columns.
            if self.params.sparse && to.i > 0 {
                let front = &self.fronts[self.last_front_idx];
                let prev_front = &self.fronts[self.last_front_idx - 1];
                assert_eq!(front.i, to.i);
                // If the previous front is the correct one, no need for further recomputation.
                if prev_front.i < to.i - 1 {
                    let i_range = IRange(prev_front.i, front.i);
                    assert!(front.j_range.0 <= to.j && to.j <= front.j_range.1);
                    let j_range = JRange(front.j_range.0, to.j);
                    self.pop_last_front();
                    // NOTE: It's unlikely the full (large) `j_range` is needed to trace back through the current block.
                    // 1. We don't need states with `j > to.j`, because the path (in reverse direction) can never go down.
                    // 2. It's unlikely we'll need all states starting at the (possibly much smaller) `j_range.0`.
                    //    Instead, we do an exponential search for the start of the `j_range`, starting at `to.j-2*i_range.len()`.
                    //    The block is high enough once the cost to `to` equals `g`.
                    let mut height = 2 * i_range.len();
                    loop {
                        let j_range = JRange(max(j_range.0, j_range.1 - height), j_range.1);
                        self.fill_block(i_range, j_range);
                        if self.fronts[self.last_front_idx].index(to.j) == g {
                            break;
                        }
                        // Pop all the computed fronts.
                        // TODO: This could be more efficient by merging the fronts for a block.
                        for _i in i_range.0..i_range.1 {
                            self.pop_last_front();
                        }
                        // Try again with a larger height.
                        height *= 2;
                    }

                    //self.fill_block(i_range, j_range);
                }
            }

            let (parent, cigar_ops) = self.parent(to).unwrap();
            to = parent;
            for op in cigar_ops {
                if let Some(op) = op {
                    cigar.push(op);

                    g -= match op {
                        AffineCigarOp::Match => 0,
                        _ => 1,
                    };
                }
            }
        }
        assert_eq!(g, 0);
        cigar.reverse();
        cigar
    }

    fn set_last_front_fixed_j_range(&mut self, fixed_j_range: Option<JRange>) {
        self.fronts[self.last_front_idx].fixed_j_range = fixed_j_range;
    }
}

impl BitFronts {
    /// Iterate over columns `i_range` for `j_range`, storing a front per column.
    fn fill_block(&mut self, i_range: IRange, j_range: JRange) {
        assert_eq!(
            i_range.0, self.i_range.1,
            "Current fronts range is {:?}. Computed range {i_range:?} does not fit!",
            self.i_range
        );
        self.i_range.1 = i_range.1;

        let j_range_rounded = round(j_range);
        let v_range = j_range_rounded.0 as usize / W..j_range_rounded.1 as usize / W;
        // Get top/bot values in the previous column for the new j_range_rounded.
        let front = &self.fronts[self.last_front_idx];
        assert!(front.i == i_range.0);

        let mut next_front = BitFront {
            v: Vec::default(),
            i: i_range.0,
            j_range,
            offset: j_range_rounded.0,
            fixed_j_range: None,
            top_val: front.index(j_range_rounded.0),
            bot_val: front.index(j_range_rounded.1),
        };
        // TODO: This allocation in `v` could possibly be removed.
        initialize_next_v(front, j_range_rounded, &mut next_front.v);

        for i in i_range.0..i_range.1 {
            // Along the top row, horizontal deltas are 1.
            next_front.i = i + 1;
            next_front.top_val += 1;
            next_front.bot_val +=
                self.compute_columns(IRange(i, i + 1), v_range.clone(), &mut next_front.v);

            self.last_front_idx += 1;
            if self.last_front_idx == self.fronts.len() {
                self.fronts.push(next_front.clone());
            } else {
                self.fronts[self.last_front_idx].clone_from(&next_front);
            }
        }
    }

    // TODO: Maybe we should at some point drop the unused fronts?
    fn pop_last_front(&mut self) {
        assert!(self.i_range.1 == self.fronts[self.last_front_idx].i);
        self.last_front_idx -= 1;
        self.i_range.1 = self.fronts[self.last_front_idx].i;
    }

    fn compute_columns(
        &self,
        i_range: IRange,
        v_range: std::ops::Range<usize>,
        v: &mut [V],
    ) -> i32 {
        if self.params.simd {
            compute_columns_simd(
                self.a.index(i_range.0 as usize..i_range.1 as usize),
                &self.b[v_range],
                v,
            ) as I
        } else {
            compute_columns(
                self.a.index(i_range.0 as usize..i_range.1 as usize),
                &self.b[v_range],
                v,
            ) as I
        }
    }
}

/// Initialize the input vertical deltas for the given new range, by copying the overlap from the previous front.
/// Takes `v` as a mutable reference, so memory can be reused.
fn initialize_next_v(front: &BitFront, j_range_rounded: JRange, v: &mut Vec<V>) {
    v.clear();
    // Make a 'working vector' with the correct range.
    v.resize(j_range_rounded.exclusive_len() as usize / W, V::one());
    // Copy the overlap from the last front.
    let prev_rounded = round(front.j_range);
    for jj in
        (max(j_range_rounded.0, prev_rounded.0)..min(j_range_rounded.1, prev_rounded.1)).step_by(W)
    {
        v[(jj - j_range_rounded.0) as usize / W] = front.v[(jj - front.offset) as usize / W];
    }
}
