use std::cmp::min;

use crate::{
    prelude::{Cost, PosTrait, SORT_QUEUE_ELEMENTS},
    scored::MinScored,
};

/// A heap where values are sorted by bucket sort.
// TODO: Investigate whether to enable sorting.
// Can be disabled by initializing next_sort to 0.
// TODO: Could be generalized to take arbitrary T instead of NodeG<G>.
pub struct BucketQueue<Pos, D> {
    layers: Vec<Vec<(Pos, D)>>,
    next: Cost,
    next_sort: Cost,
    next_clear: Cost,
}

impl<Pos: PosTrait, D> BucketQueue<Pos, D> {
    #[inline]
    pub fn push(&mut self, MinScored(k, v, d): MinScored<Cost, Pos, D>) {
        if self.layers.len() <= k as usize {
            self.layers.resize_with(k as usize + 1, Vec::default);
        }
        self.next = min(self.next, k);
        self.layers[k as usize].push((v, d));
    }
    pub fn pop(&mut self) -> Option<MinScored<Cost, Pos, D>> {
        while let Some(layer) = self.layers.get_mut(self.next as usize) {
            if let Some((back, d)) = layer.pop() {
                return Some(MinScored(self.next, back, d));
            }
            self.next += 1;
            if SORT_QUEUE_ELEMENTS {
                if self.next == self.next_sort {
                    if let Some(layer) = self.layers.get_mut(self.next_sort as usize) {
                        layer.sort_unstable_by_key(|(pos, _)| <Pos as PosTrait>::key(pos));
                    }
                    self.next_sort += 1;
                }
            }
            // Clearing memory 10 layers back.
            // The value of f shouldn't go down more than the maximum match
            // distance of 1 or 2, so this should be plenty.
            if self.next_clear + 10 < self.next {
                self.layers[self.next_clear as usize].shrink_to_fit();
                self.next_clear += 1;
            }
        }
        None
    }
}

impl<Pos, D> Default for BucketQueue<Pos, D> {
    fn default() -> Self {
        Self {
            layers: Default::default(),
            next: 0,
            next_sort: 1,
            next_clear: 0,
        }
    }
}
