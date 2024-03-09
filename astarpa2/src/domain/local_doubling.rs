use super::*;

impl<'a, V: VisualizerT, H: Heuristic> AstarPa2Instance<'a, V, H> {
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
                let next_fixed_j_range = self.fixed_j_range(
                    i_range.1,
                    f_max,
                    Some(prev_fixed_j_range),
                    blocks.last_block(),
                );
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
        let (cigar, _stats) = blocks.trace(
            self.a,
            self.b,
            Pos(0, 0),
            Pos::target(self.a, self.b),
            &mut self.v,
        );
        (dist, cigar)
    }
}
