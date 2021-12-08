use std::cmp::Ord;
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeFull;

use crate::util::*;

pub struct IncreasingFunction<K, V> {
    pub m: BTreeMap<K, V>,
}

impl<K: Ord + Copy, V: Ord + Copy> IncreasingFunction<K, V> {
    pub fn new() -> Self {
        IncreasingFunction { m: BTreeMap::new() }
    }

    /// Set f(x) = y.
    /// Only inserts if y is larger than the current value at x.
    /// Returns whether insertion took place.
    pub fn set(&mut self, x: K, y: V) -> bool {
        let cur_val = self.get(x);
        if cur_val.map_or(false, |c| y <= c) {
            return false;
        }
        // Delete elements right of x at most y.
        let to_remove = self
            .m
            .range((Excluded(x), Unbounded))
            .take_while(|&(_, &value)| value <= y)
            .map(|(&key, _)| key)
            .collect::<Vec<_>>();
        for key in to_remove {
            self.m.remove(&key);
        }
        self.m.insert(x, y);
        true
    }

    /// Get the largest value in the map.
    pub fn max(&self) -> Option<V> {
        self.m.range(RangeFull).next_back().map(|(_, y)| *y)
    }

    /// Get f(x): the y for the largest key <= x inserted into the map.
    pub fn get(&self, x: K) -> Option<V> {
        self.m
            .range((Unbounded, Included(x)))
            .next_back()
            .map(|(_, y)| *y)
    }

    /// f(x') for the largest x' < x inserted into the map.
    pub fn get_smaller(&self, x: K) -> Option<V> {
        self.m
            .range((Unbounded, Excluded(x)))
            .next_back()
            .map(|(_, y)| *y)
    }

    /// f(x') for the smallest x' > x inserted into the map.
    pub fn get_larger(&self, x: K) -> Option<V> {
        self.m
            .range((Excluded(x), Unbounded))
            .next()
            .map(|(_, y)| *y)
    }
}

pub struct IncreasingFunction2D<T> {
    nodes: Vec<Node<T>>,
    root: Option<usize>,
}

type NodeIndex = usize;

pub struct Node<T> {
    pub pos: Pos,
    pub val: T,
    parent: Option<NodeIndex>,
    prev: Option<NodeIndex>,
    next: Option<NodeIndex>,
}

impl IncreasingFunction2D<usize> {
    fn get_idx<'a>(&'a self, idx: Option<usize>) -> Option<&'a Node<usize>> {
        Some(&self.nodes[idx?])
    }
    fn next<'a>(&'a self, node: &'a Node<usize>) -> Option<&'a Node<usize>> {
        Some(&self.nodes[node.next?])
    }
    fn prev<'a>(&'a self, node: &'a Node<usize>) -> Option<&'a Node<usize>> {
        Some(&self.nodes[node.prev?])
    }
    fn parent<'a>(&'a self, node: &'a Node<usize>) -> Option<&'a Node<usize>> {
        Some(&self.nodes[node.parent?])
    }

    /// Build the increasing function over the given points. `l` must be at least 1.
    /// `ps` must be sorted increasing by (x,y), first on x and then on y.
    pub fn new(ps: impl IntoIterator<Item = Pos>, l: usize) -> Self {
        let mut s = Self {
            nodes: vec![Node {
                pos: Pos(0, 0),
                val: 0,
                parent: None,
                prev: None,
                next: None,
            }],
            root: None,
        };
        s.build(ps, l);
        s
    }

    fn build<'a>(&'a mut self, ps: impl IntoIterator<Item = Pos>, l: usize) {
        assert!(l >= 1);
        let mut front = IncreasingFunction::new();
        let mut lagging_front = IncreasingFunction::new();

        // Index into self.nodes.
        let mut lagging_index = 0;

        for pos in ps {
            let Pos(i, j) = pos;
            // Update lagging front.
            while lagging_index < self.nodes.len() {
                let node = &self.nodes[lagging_index];
                if node.pos <= Pos(i - l, j - l) {
                    lagging_front.set(node.pos.1, (node.val, lagging_index));
                    lagging_index += 1;
                } else {
                    break;
                }
            }

            // Get the value for the position.
            let (val, parent) = match lagging_front.get(j - l) {
                None => (0, None),
                Some((val, p)) => (val + 1, Some(p)),
            };

            let id = self.nodes.len();

            let next = front.get_larger(j).and_then(
                |(nextval, idx)| {
                    if nextval != val {
                        None
                    } else {
                        Some(idx)
                    }
                },
            );

            // Only continue if the value is larger than existing.
            if !front.set(j, (val, id)) {
                continue;
            }

            if let Some(next_idx) = next {
                self.nodes[next_idx].prev = Some(id);
            }

            self.nodes.push(Node {
                pos,
                val,
                parent,
                prev: None,
                next,
            });
        }
        // The root is the now largest value in the front.
        self.root = front.max().map(|x| x.1);
    }

    pub fn root<'a>(&'a self) -> Option<&'a Node<usize>> {
        self.get_idx(self.root)
    }

    /// hint: Node index of the heuristic value at the direct predecessor of `pos`.
    /// If Pos(x,y) is where hint was obtained, pos should be one of Pos(x-1,y-1), Pos(x-1,y), Pos(x,y-1).
    /// Use `get_jump` if `pos` is _somewhere_ below the previous position.
    pub fn get<'a>(&'a self, pos: Pos, hint: &'a Node<usize>) -> Option<&'a Node<usize>> {
        if pos >= hint.pos {
            return Some(hint);
        } else {
            // Maybe we can keep the same hint value, by moving to hint.next or hint.prev.
            for node in [hint.next, hint.prev, hint.parent]
                .iter()
                .filter_map(|idx| self.get_idx(*idx))
            {
                if pos >= node.pos {
                    return Some(node);
                }
            }
            None
        }
    }

    /// Same as get, but can handle larger jumps of position.
    /// Moves to the next/prev neighbour as long as needed, and then goes to parents.
    pub fn get_jump<'a>(
        &'a self,
        pos @ Pos(i, j): Pos,
        mut hint: &'a Node<usize>,
    ) -> Option<&'a Node<usize>> {
        loop {
            if pos >= hint.pos {
                return Some(hint);
            }
            if let Some(next) = self.next(hint) {
                if j >= next.pos.1 {
                    hint = next;
                }
            } else if let Some(prev) = self.prev(hint) {
                if i >= prev.pos.0 {
                    hint = prev;
                }
            } else if let Some(next) = self.parent(hint) {
                hint = next;
            } else {
                return None;
            }
        }
    }
}
