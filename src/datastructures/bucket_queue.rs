use std::cmp::min;

use crate::{config::SORT_QUEUE_ELEMENTS, prelude::Cost};

///
#[derive(Copy, Clone)]
pub struct QueueElement<Score, T> {
    pub f: Score,
    pub data: T,
}

/// A heap where values are sorted by bucket sort.
// TODO: Investigate whether to enable sorting.
// Can be disabled by initializing next_sort to 0.
pub struct BucketQueue<T> {
    layers: Vec<Vec<T>>,
    next: Cost,
    next_sort: Cost,
    next_clear: Cost,
}

pub trait QueueOrder {
    type O: Ord;
    fn key(&self) -> Self::O;
}

impl<T> BucketQueue<T>
where
    T: QueueOrder,
{
    #[inline]
    pub fn push(&mut self, QueueElement { f, data }: QueueElement<Cost, T>) {
        if self.layers.len() <= f as usize {
            self.layers.resize_with(f as usize + 1, Vec::default);
        }
        self.next = min(self.next, f);
        self.layers[f as usize].push(data);
    }
    pub fn pop(&mut self) -> Option<QueueElement<Cost, T>> {
        while let Some(layer) = self.layers.get_mut(self.next as usize) {
            if let Some(data) = layer.pop() {
                return Some(QueueElement { f: self.next, data });
            }
            self.next += 1;
            if SORT_QUEUE_ELEMENTS {
                if self.next == self.next_sort {
                    if let Some(layer) = self.layers.get_mut(self.next_sort as usize) {
                        layer.sort_unstable_by_key(|pd| QueueOrder::key(pd));
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

impl<T> Default for BucketQueue<T> {
    fn default() -> Self {
        Self {
            layers: Default::default(),
            next: 0,
            next_sort: 1,
            next_clear: 0,
        }
    }
}
