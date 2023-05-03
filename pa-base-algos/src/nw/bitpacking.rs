use std::cmp::min;

use pa_bitpacking::{compute_columns, profile, CompressedSequence, Profile, V, W};
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
    /// The `j` of the first element of `v`.
    /// Can be different from `j_range.0` when only a slice of the array corresponds to the `j_range`.
    offset: I,
    /// The value at the top of the rounded range, set on construction.
    top_val: Cost,
    /// The value at the bottom of the rounded range, computed after the range itself.
    bot_val: Cost,
}

pub struct BitFronts {
    params: BitFrontsTag,
    trace: bool,
    a: CompressedSequence,
    b: Profile,
    cm: AffineCost<0>,
    /// The list of fronts.
    /// NOTE: When using sparse traceback fronts, indices do not correspond to `i`!
    fronts: Vec<BitFront>,
    i_range: IRange,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct BitFrontsTag {
    /// When true, `trace` mode only stores one front per block, instead of all columns.
    /// `cost` most always stores only the last front.
    pub sparse: bool,
}

impl Default for BitFront {
    fn default() -> Self {
        Self {
            v: vec![],
            i: 0,
            j_range: JRange(0, 0),
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
        let front = &mut self.fronts.last_mut().unwrap();
        let mut top_val = front.index(j_range_rounded.0);
        let mut bot_val = front.index(j_range_rounded.1);

        if self.trace {
            // Compute the new `v` at the end of the `i_range` and push a new front.
            assert!(self.params.sparse);
            let mut v = initialize_next_v(front, j_range_rounded);

            top_val += i_range.len();
            bot_val += compute_columns(
                &self.a[i_range.0 as usize..i_range.1 as usize],
                &self.b[v_range.clone()],
                &mut v,
            ) as I;

            self.fronts.push(BitFront {
                v,
                i: i_range.1,
                j_range,
                offset: j_range_rounded.0,
                top_val,
                bot_val,
            });
        } else {
            // Update the existing `v` vector in the single front.
            top_val += i_range.len();
            bot_val += compute_columns(
                &self.a[i_range.0 as usize..i_range.1 as usize],
                &self.b[v_range.clone()],
                &mut front.v[v_range.clone()],
            ) as I;
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
        self.fronts.last().unwrap()
    }

    /// Find the parent of `st`.
    /// NOTE: This assumes that `st.i` is in the last front, and that the front before is for `st.i-1`.
    fn parent(&self, st: State) -> Option<(State, AffineCigarOps)> {
        let front = &self.fronts[self.fronts.len() - 1];
        assert!(front.i == st.i);
        let prev_front = if st.i > 0 {
            let prev_front = &self.fronts[self.fronts.len() - 2];
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
    ///
    /// TODO: Store horizontal deltas in blocks, so that `parent` is more
    /// efficient and doesn't have to use relatively slow `front.index`
    /// operations.
    fn trace(&mut self, from: State, mut to: State) -> AffineCigar {
        assert!(self.trace);
        let mut cigar = AffineCigar::default();

        while to != from {
            // Remove fronts to the right of `to`.
            while self.fronts.last().unwrap().i > to.i {
                self.fronts.pop();
                self.i_range.1 = self.fronts.last().unwrap().i;
            }

            // In case of sparse fronts, fill missing columns by recomputing the
            // block and storing all columns.
            // NOTE: We only compute up to the row of `st`. More isn't needed,
            // and this speeds up indexing by being close to the boundary.
            if self.params.sparse {
                let front = &self.fronts[self.fronts.len() - 1];
                let prev_front = &self.fronts[self.fronts.len() - 2];
                assert_eq!(front.i, to.i);
                let i_range = IRange(prev_front.i, front.i);
                assert!(front.j_range.0 <= to.j && to.j <= front.j_range.1);
                let j_range = JRange(front.j_range.0, to.j);
                // TODO: Reuse this memory
                self.fronts.pop();
                self.i_range.1 = self.fronts.last().unwrap().i;
                self.fill_block(i_range, j_range);
            }

            let (parent, cigar_ops) = self.parent(to).unwrap();
            to = parent;
            for op in cigar_ops {
                if let Some(op) = op {
                    cigar.push(op);
                }
            }
        }
        cigar.reverse();
        cigar
    }
}

impl BitFronts {
    /// Iterate over columns `i_range` for `j_range`, storing a front per column.
    fn fill_block(&mut self, i_range: IRange, j_range: JRange) {
        assert!(i_range.0 == self.i_range.1);
        self.i_range.1 = i_range.1;

        let j_range_rounded = round(j_range);
        let v_range = j_range_rounded.0 as usize / W..j_range_rounded.1 as usize / W;
        // Get top/bot values in the previous column for the new j_range_rounded.
        let front = &self.fronts.last_mut().unwrap();
        assert!(front.i == i_range.0);
        let mut top_val = front.index(j_range_rounded.0);
        let mut bot_val = front.index(j_range_rounded.1);

        let mut v = initialize_next_v(front, j_range_rounded);

        for i in i_range.0..i_range.1 {
            // Along the top row, horizontal deltas are 1.
            top_val += 1;
            bot_val += compute_columns(
                &self.a[i as usize..i as usize + 1],
                &self.b[v_range.clone()],
                &mut v,
            ) as I;

            self.fronts.push(BitFront {
                // Copy `v`, or take it if this is the last column.
                v: if i < i_range.1 - 1 {
                    v.clone()
                } else {
                    std::mem::take(&mut v)
                },
                i: i + 1,
                j_range,
                offset: j_range_rounded.0,
                top_val,
                bot_val,
            });
        }
    }
}

fn initialize_next_v(front: &BitFront, j_range_rounded: JRange) -> Vec<V> {
    // Make a 'working vector' with the correct range.
    let mut v = vec![V::one(); j_range_rounded.exclusive_len() as usize / W];
    // Copy the overlap from the last front.
    let prev_rounded = round(front.j_range);
    for jj in
        (max(j_range_rounded.0, prev_rounded.0)..min(j_range_rounded.1, prev_rounded.1)).step_by(W)
    {
        v[(jj - j_range_rounded.0) as usize / W] = front.v[(jj - front.offset) as usize / W];
    }
    v
}
