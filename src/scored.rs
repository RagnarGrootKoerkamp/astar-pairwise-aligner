use std::cmp::Ordering;

use crate::graph::{ImplicitGraph, NodeG};

/// `MinScored<K, T>` holds a score `K` and a scored object `T` in
/// a pair for use with a `BinaryHeap`.
///
/// `MinScored` compares in reverse order by the score, so that we can
/// use `BinaryHeap` as a min-heap to extract the score-value pair with the
/// least score.
#[derive(Copy, Clone)]
pub struct MinScored<V, G: ImplicitGraph>(pub V, pub NodeG<G>);

impl<V: Eq, G: ImplicitGraph> PartialEq for MinScored<V, G> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1.to_pos() == other.1.to_pos()
    }
}

impl<V: Eq, G: ImplicitGraph> Eq for MinScored<V, G> {}

impl<V: Ord, G: ImplicitGraph> PartialOrd for MinScored<V, G> {
    #[inline]
    fn partial_cmp(&self, other: &MinScored<V, G>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<V: Ord, G: ImplicitGraph> Ord for MinScored<V, G> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.cmp(&other.0) {
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => self.1.cmp(&other.1),
            Ordering::Greater => Ordering::Less,
        }
    }
}
