use std::cmp::min;

use crate::scored::MinScored;

/// A heap where values are sorted by bucket sort.
// TODO: Investigate whether to enable sorting.
// Can be disabled by initializing next_sort to 0.
pub struct BucketHeap<T> {
    layers: Vec<Vec<T>>,
    next: usize,
    next_sort: usize,
}

impl<T: std::cmp::Ord> BucketHeap<T> {
    pub fn push(&mut self, MinScored(k, v): MinScored<usize, T>) {
        if self.layers.len() <= k {
            self.layers.resize_with(k + 1, || Vec::default());
        }
        self.next = min(self.next, k);
        self.layers[k].push(v);
    }
    pub fn pop(&mut self) -> Option<MinScored<usize, T>> {
        while let Some(layer) = self.layers.get_mut(self.next) {
            if let Some(back) = layer.pop() {
                return Some(MinScored(self.next, back));
            }
            // TODO: Sort the keys in the next layer.
            self.next += 1;
            if self.next == self.next_sort {
                if let Some(layer) = self.layers.get_mut(self.next_sort) {
                    layer.sort();
                }
                self.next_sort += 1;
            }
        }
        None
    }
}

impl<T> Default for BucketHeap<T> {
    fn default() -> Self {
        Self {
            layers: Default::default(),
            next: 0,
            next_sort: 1,
        }
    }
}
