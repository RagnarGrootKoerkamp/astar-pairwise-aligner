use crate::{cost_model::Cost, prelude::Pos};

use super::{BucketQueue, QueueElement};

pub trait ToPos {
    fn to_pos(&self) -> Pos;
}

/// A queue that supports increasing all elements below a position by a given
/// amount.  Keeps an internal offset of the shift to apply to each element.
///
/// To handle cases where most but not all elements are shifted, a small buffer
/// of the bottommost and rightmost elements of the queue is kept separately.
pub struct ShiftQueue<T> {
    queue: BucketQueue<T>,
    /// The amount added to each element in the queue.
    /// Initialized to h(0), and goes down over time.
    /// That way, queue elements become larger.
    shift: Cost,
}

impl<T> ShiftQueue<T> {
    pub fn new(max_shift: Cost) -> Self {
        ShiftQueue {
            queue: BucketQueue::default(),
            shift: max_shift,
        }
    }
    pub fn push(&mut self, mut element: QueueElement<T>) {
        element.f += self.shift;
        self.queue.push(element)
    }
    pub fn pop(&mut self) -> Option<QueueElement<T>> {
        let mut e = self.queue.pop();
        if let Some(e) = e.as_mut() {
            e.f -= self.shift;
        }
        e
    }
    pub fn shift(&mut self, shift: Cost) {
        assert!(shift <= self.shift);
        self.shift -= shift;
    }
}
