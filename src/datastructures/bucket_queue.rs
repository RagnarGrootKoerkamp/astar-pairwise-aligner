use std::cmp::min;

use crate::prelude::Cost;

#[derive(Copy, Clone)]
pub struct QueueElement<T> {
    pub f: Cost,
    pub data: T,
}

/// A heap where values are sorted by bucket sort.
// TODO: Investigate whether to enable sorting.
// Can be disabled by initializing next_sort to 0.
pub struct BucketQueue<T> {
    layers: Vec<Vec<T>>,
    next: Cost,
    /// Layers far lower than the current minimum are shrunk when the minimum f
    /// has increased sufficiently beyond them.
    next_clear: Cost,
}

impl<T> BucketQueue<T> {
    pub fn push(&mut self, QueueElement { f, data }: QueueElement<T>) {
        if self.layers.len() <= f as usize {
            self.layers.resize_with(f as usize + 1, Vec::default);
        }
        self.next = min(self.next, f);
        self.layers[f as usize].push(data);
    }
    pub fn pop(&mut self) -> Option<QueueElement<T>> {
        while let Some(layer) = self.layers.get_mut(self.next as usize) {
            if let Some(data) = layer.pop() {
                return Some(QueueElement { f: self.next, data });
            }
            self.next += 1;
            // Releasing memory 10 layers back.
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

impl<T> Default for BucketQueue<T> {
    fn default() -> Self {
        Self {
            layers: Default::default(),
            next: 0,
            next_clear: 0,
        }
    }
}
