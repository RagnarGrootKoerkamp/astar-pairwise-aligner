use std::cmp::min;

use crate::{prelude::PosOrder, scored::MinScored};

/// A heap where values are sorted by bucket sort.
// TODO: Investigate whether to enable sorting.
// Can be disabled by initializing next_sort to 0.
// TODO: Could be generalized to take arbitrary T instead of NodeG<G>.
pub struct BucketQueue<Pos> {
    layers: Vec<Vec<Pos>>,
    next: usize,
    next_sort: usize,
    next_clear: usize,
}

impl<Pos: PosOrder> BucketQueue<Pos> {
    #[inline]
    pub fn push(&mut self, MinScored(k, v): MinScored<usize, Pos>) {
        if self.layers.len() <= k {
            self.layers.resize_with(k + 1, Vec::default);
        }
        self.next = min(self.next, k);
        self.layers[k].push(v);
    }
    pub fn pop(&mut self) -> Option<MinScored<usize, Pos>> {
        while let Some(layer) = self.layers.get_mut(self.next) {
            if let Some(back) = layer.pop() {
                return Some(MinScored(self.next, back));
            }
            self.next += 1;
            if self.next == self.next_sort {
                if let Some(layer) = self.layers.get_mut(self.next_sort) {
                    layer.sort_unstable_by_key(|pos| <Pos as PosOrder>::key(pos));
                }
                self.next_sort += 1;
            }
            // Start clearing memory 10 layers back.
            if self.next_clear + 10 < self.next {
                self.layers[self.next_clear].shrink_to_fit();
                self.next_clear += 1;
            }
        }
        None
    }
}

impl<Pos> Default for BucketQueue<Pos> {
    fn default() -> Self {
        Self {
            layers: Default::default(),
            next: 0,
            next_sort: 1,
            next_clear: 0,
        }
    }
}
