use super::*;

impl Blocks {
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

        // Collect some statistics.
        let mut dt_trace_tries = 0;
        let mut dt_trace_success = 0;
        let mut dt_trace_fallback = 0;

        // Some allocated memory that can be reused.
        let dt_cache = &mut vec![BlockElem::default(); (self.params.max_g + 1).pow(2) as usize];

        while to != from {
            // Remove blocks to the right of `to`.
            while self.last_block_idx > 0 && self.blocks[self.last_block_idx].i_range.0 >= to.0 {
                self.pop_last_block();
            }

            // Try a Diagonal Transition based traceback first which should be faster for small distances.
            if self.params.dt_trace && to.0 > 0 {
                let prev_block = &self.blocks[self.last_block_idx - 1];
                if prev_block.i_range.1 < to.0 - 1 {
                    dt_trace_tries += 1;
                    if let Some(new_to) =
                        self.dt_trace_block(a, b, to, &mut g, prev_block, &mut cigar, dt_cache)
                    {
                        dt_trace_success += 1;
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

            // Shrink diagonals more than `x_drop` behind.
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
    // TODO: Compare performance.
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
