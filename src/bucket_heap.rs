use std::cmp::min;

use crate::{
    graph::{ImplicitGraph, NodeG},
    scored::MinScored,
};

/// A heap where values are sorted by bucket sort.
// TODO: Investigate whether to enable sorting.
// Can be disabled by initializing next_sort to 0.
// TODO: Could be generatlized to take arbitrary T instead of NodeG<G>.
pub struct BucketHeap<G: ImplicitGraph> {
    layers: Vec<Vec<NodeG<G>>>,
    next: usize,
    next_sort: usize,
}

impl<G: ImplicitGraph> BucketHeap<G> {
    #[inline]
    pub fn push(&mut self, MinScored(k, v): MinScored<usize, G>) {
        if self.layers.len() <= k {
            self.layers.resize_with(k + 1, Vec::default);
        }
        self.next = min(self.next, k);
        self.layers[k].push(v);
    }
    pub fn pop(&mut self) -> Option<MinScored<usize, G>> {
        while let Some(layer) = self.layers.get_mut(self.next) {
            if let Some(back) = layer.pop() {
                return Some(MinScored(self.next, back));
            }
            self.next += 1;
            if self.next == self.next_sort {
                if let Some(layer) = self.layers.get_mut(self.next_sort) {
                    layer.sort_unstable();
                }
                self.next_sort += 1;
            }
        }
        None
    }
}

impl<G: ImplicitGraph> Default for BucketHeap<G> {
    fn default() -> Self {
        Self {
            layers: Default::default(),
            next: 0,
            next_sort: 1,
        }
    }
}
