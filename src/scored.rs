use std::cmp::{Ordering, Reverse};

use crate::prelude::PosOrder;

/// `MinScored<K, T>` holds a score `K` and a scored object `T` in
/// a pair for use with a `BinaryHeap`.
///
/// `MinScored` compares in reverse order by the score, so that we can
/// use `BinaryHeap` as a min-heap to extract the score-value pair with the
/// least score.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct MinScored<V, P>(pub V, pub P);

impl<V: Ord, P: PosOrder + Eq> PartialOrd for MinScored<V, P> {
    #[inline]
    fn partial_cmp(&self, other: &MinScored<V, P>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V: Ord, P: PosOrder + std::cmp::Eq> Ord for MinScored<V, P> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        (Reverse(&self.0), self.1.key()).cmp(&(Reverse(&other.0), <P as PosOrder>::key(&other.1)))
    }
}
