use crate::config::USE_TIP_BUFFER;
use pa_heuristic::PosOrderT;
use pa_types::Cost;
use std::cmp::{max, min};

#[derive(Copy, Clone, Debug)]
pub struct QueueElement<T> {
    pub f: Cost,
    pub data: T,
}

/// A heap where values are sorted by bucket sort.
#[derive(Debug)]
pub struct BucketQueue<T> {
    layers: Vec<Vec<T>>,
    /// The first layer with an element is at least `next`.
    next: usize,
    last: usize,
    /// Layers far lower than the current minimum are shrunk when the minimum f
    /// has increased sufficiently beyond them.
    next_clear: usize,
    size: usize,
}

const CLEAR_DELAY: usize = 10;

impl<T> BucketQueue<T> {
    pub fn push(&mut self, QueueElement { f, data }: QueueElement<T>) {
        if self.layers.len() <= f as usize {
            self.layers.resize_with(f as usize + 1, Vec::default);
        }
        self.next = min(self.next, f as usize);
        self.last = max(self.last, f as usize + 1);
        self.layers[f as usize].push(data);
        self.size += 1;
    }

    pub fn peek(&mut self) -> Option<Cost> {
        if self.is_empty() {
            return None;
        }
        loop {
            if !self.layers[self.next as usize].is_empty() {
                return Some(self.next as Cost);
            }
            self.next += 1;
            // Releasing memory 10 layers back.
            // The value of f shouldn't go down more than the maximum match
            // distance of 1 or 2, so this should be plenty.
            // TODO: Figure out if we can reuse this memory, possibly by moving it to the end of the layers vector?
            // NOTE: This needs to be a while loop since `next` can go up in jumps after being empty.
            while self.next_clear + CLEAR_DELAY < self.next {
                assert!(self.layers[self.next_clear as usize].is_empty());
                self.layers[self.next_clear as usize].shrink_to_fit();
                self.next_clear += 1;
            }
        }
    }

    pub fn pop(&mut self) -> Option<QueueElement<T>> {
        let Some(f) = self.peek() else {
            return None;
        };
        let qe = QueueElement {
            f,
            data: self.layers[f as usize].pop().unwrap(),
        };
        assert!(self.size > 0);
        self.size -= 1;
        if self.size == 0 {
            self.next = usize::MAX;
            self.last = 0;
        }
        Some(qe)
    }

    #[allow(unused)]
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl<T> Default for BucketQueue<T> {
    fn default() -> Self {
        Self {
            layers: Default::default(),
            next: usize::MAX,
            last: 0,
            next_clear: 0,
            size: 0,
        }
    }
}

pub trait ShiftOrderT<T>: PosOrderT + Default + Copy {
    fn from_t(t: &T) -> Self;
}

// NOTE: 0 for paper
const TIP_SIZE: usize = 0;

/// A queue that supports increasing all elements below a position by a given
/// amount.
/// To support this efficiently, all elements within ~2 seeds before the last shift are kept in a separate queue.
/// After each shift, the threshold is updated and larger elements are pushed to the 2nd queue.
#[derive(Debug)]
pub struct ShiftQueue<T, O> {
    queue: BucketQueue<T>,
    tip_queue: BucketQueue<T>,

    /// Elements at most `tip_start` go in the main `queue`. Updated after each shift.
    /// When USE_TIP_QUEUE is false, this is the maximum inserted element in the
    /// main queue.
    tip_start: O,

    /// The amount added to each element in the queues.
    /// Initialized to h(0), and goes down over time.
    /// That way, queue elements become larger.
    down_shift: Cost,

    missed: usize,
}

impl<T, O: ShiftOrderT<T>> ShiftQueue<T, O>
where
    T: std::fmt::Debug,
    O: std::fmt::Debug,
{
    pub fn new(max_shift: Cost) -> Self {
        ShiftQueue {
            queue: BucketQueue::default(),
            tip_queue: BucketQueue::default(),
            tip_start: O::default(),
            down_shift: max_shift,
            missed: 0,
        }
    }
    pub fn push(&mut self, mut element: QueueElement<T>)
    where
        T: Clone + std::fmt::Debug,
    {
        element.f += self.down_shift;
        if !USE_TIP_BUFFER {
            self.tip_start = O::max(self.tip_start, O::from_t(&element.data));
            self.queue.push(element);
        } else {
            if O::from_t(&element.data) <= self.tip_start {
                self.queue.push(element);
            } else {
                self.tip_queue.push(element);
            }
        }
    }
    pub fn pop(&mut self) -> Option<QueueElement<T>> {
        if !USE_TIP_BUFFER {
            let mut e = self.queue.pop();
            if let Some(e) = e.as_mut() {
                e.f -= self.down_shift;
            }
            e
        } else {
            let tf = self.tip_queue.peek();
            let qf = self.queue.peek();
            let mut e = if let Some(tf) = tf && qf.map_or(true, |qf| tf <= qf) {
                self.tip_queue.pop()
            } else {
                self.queue.pop()
            };
            if let Some(e) = e.as_mut() {
                e.f -= self.down_shift;
            }
            e
        }
    }
    pub fn shift(&mut self, shift: Cost, below: O) -> Cost {
        if shift == 0 {
            return 0;
        }
        if !(self.tip_start <= below) {
            self.missed += shift as usize;
            return 0;
        }

        assert!(shift <= self.down_shift);
        self.down_shift -= shift;

        if !USE_TIP_BUFFER {
            return shift;
        }

        // Any elements in the tip not smaller than `below` are shifted down, to correct for the global down_shift offset.
        let Some(f) = self.tip_queue.peek() else { return shift; };

        for f in f as usize..self.tip_queue.last {
            // Extract draining layer so we can modify it together with the target layer.
            let mut to_drain = std::mem::take(&mut self.tip_queue.layers[f]);
            //for data in to_drain.extract_if(|data| !(O::from_t(data) <= below)) {
            for data in to_drain.extract_if(|data| !(O::from_t(data) <= below)) {
                self.tip_queue.push(QueueElement {
                    f: f as Cost - shift,
                    data,
                });
            }
            self.tip_queue.layers[f] = to_drain;
        }

        // Any elements in the tip less than `new_tip_start` are moved to the main queue.
        self.tip_start = O::max(self.tip_start, O::tip_start(TIP_SIZE, below));
        for f in self.tip_queue.peek().unwrap() as usize..self.tip_queue.layers.len() {
            //for data in to_drain.extract_if(|data| !(O::from_t(data) <= below)) {
            for data in
                self.tip_queue.layers[f].extract_if(|data| O::from_t(data) <= self.tip_start)
            {
                self.tip_queue.size -= 1;
                self.queue.push(QueueElement { f: f as Cost, data });
            }
        }

        shift
    }
}
