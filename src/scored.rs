use std::cmp::{Ordering, Reverse};

use crate::prelude::Pos;

/// `MinScored<K, T>` holds a score `K` and a scored object `T` in
/// a pair for use with a `BinaryHeap`.
///
/// `MinScored` compares in reverse order by the score, so that we can
/// use `BinaryHeap` as a min-heap to extract the score-value pair with the
/// least score.
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
