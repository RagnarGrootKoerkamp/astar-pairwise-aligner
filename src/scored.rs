use std::cmp::{Ordering, Reverse};

use crate::prelude::PosTrait;

/// `MinScored<K, T>` holds a score `K` and a scored object `T` in
/// a pair for use with a `BinaryHeap`.
///
/// `MinScored` compares in reverse order by the score, so that we can
/// use `BinaryHeap` as a min-heap to extract the score-value pair with the
/// least score.
#[derive(Copy, Clone)]
pub struct MinScored<V, P, D>(pub V, pub P, pub D);

impl<V: Eq, P: Eq, D> PartialEq for MinScored<V, P, D> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl<V: Eq, P: Eq, D> Eq for MinScored<V, P, D> {}

impl<V: Ord, P: PosTrait + Eq, D> PartialOrd for MinScored<V, P, D> {
    #[inline]
    fn partial_cmp(&self, other: &MinScored<V, P, D>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V: Ord, P: PosTrait + std::cmp::Eq, D> Ord for MinScored<V, P, D> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        (Reverse(&self.0), self.1.key()).cmp(&(Reverse(&other.0), <P as PosTrait>::key(&other.1)))
    }
}
