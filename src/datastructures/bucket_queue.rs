use std::cmp::min;

use crate::prelude::{Cost, Pos, SORT_QUEUE_ELEMENTS};
use std::cmp::{Ordering, Reverse};

/// `MinScored<K, T>` holds a score `K` and a scored object `T` in
/// a pair for use with priority queue.
///
/// `MinScored` compares in reverse order by the score.
#[derive(Copy, Clone)]
pub struct MinScored<V, D>(pub V, pub Pos, pub D);

impl<V: Eq, D> PartialEq for MinScored<V, D> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl<V: Eq, D> Eq for MinScored<V, D> {}

impl<V: Ord, D> PartialOrd for MinScored<V, D> {
    #[inline]
    fn partial_cmp(&self, other: &MinScored<V, D>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V: Ord, D> Ord for MinScored<V, D> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        (Reverse(&self.0), self.1.key()).cmp(&(Reverse(&other.0), Pos::key(&other.1)))
    }
}

/// A heap where values are sorted by bucket sort.
// TODO: Investigate whether to enable sorting.
// Can be disabled by initializing next_sort to 0.
// TODO: Could be generalized to take arbitrary T instead of NodeG<G>.
pub struct BucketQueue<D> {
    layers: Vec<Vec<(Pos, D)>>,
    next: Cost,
    next_sort: Cost,
    next_clear: Cost,
}

impl<D> BucketQueue<D> {
    #[inline]
    pub fn push(&mut self, MinScored(k, v, d): MinScored<Cost, D>) {
        if self.layers.len() <= k as usize {
            self.layers.resize_with(k as usize + 1, Vec::default);
        }
        self.next = min(self.next, k);
        self.layers[k as usize].push((v, d));
    }
    pub fn pop(&mut self) -> Option<MinScored<Cost, D>> {
        while let Some(layer) = self.layers.get_mut(self.next as usize) {
            if let Some((back, d)) = layer.pop() {
                return Some(MinScored(self.next, back, d));
            }
            self.next += 1;
            if SORT_QUEUE_ELEMENTS {
                if self.next == self.next_sort {
                    if let Some(layer) = self.layers.get_mut(self.next_sort as usize) {
                        layer.sort_unstable_by_key(|(pos, _)| Pos::key(pos));
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

impl<D> Default for BucketQueue<D> {
    fn default() -> Self {
        Self {
            layers: Default::default(),
            next: 0,
            next_sort: 1,
            next_clear: 0,
        }
    }
}
