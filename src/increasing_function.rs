use std::cmp::{Ord, Reverse};
use std::collections::BTreeMap;
use std::hash;
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeFull;

use crate::seeds::Match;
use crate::util::*;

pub struct IncreasingFunction<K, V> {
    pub m: BTreeMap<K, V>,
}

impl<K: Ord + Copy + std::fmt::Debug, V: Ord + Copy + std::fmt::Debug> IncreasingFunction<K, V> {
    pub fn new() -> Self {
        IncreasingFunction { m: BTreeMap::new() }
    }

    /// Set f(x) = y.
    /// Only inserts if y is larger than the current value at x.
    /// Returns whether insertion took place.
    pub fn set(&mut self, x: K, y: V) -> bool {
        //println!("Set {:?} to {:?}", x, y);
        let cur_val = self.get(x);
        if cur_val.map_or(false, |c| y <= c) {
            //println!("Set {:?} to {:?} -> SKIP", x, y);
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
        let v = self
            .m
            .range((Unbounded, Included(x)))
            .next_back()
            .map(|(_, y)| *y);
        //println!("Get {:?} = {:?}", x, v);
        v
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

pub type NodeIndex = usize;

// We guarantee that the function always contains (0,0), so lookup will always succeed.
#[derive(Default)]
pub struct IncreasingFunction2D<T: Copy + hash::Hash + Eq> {
    nodes: Vec<Node<T>>,
    root: NodeIndex,
    leftover_at_end: bool,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Node<T: Copy + hash::Hash + Eq> {
    pub pos: Pos,
    pub val: T,
    parent: Option<NodeIndex>,
    prev: Option<NodeIndex>,
    next: Option<NodeIndex>,
    child: Option<NodeIndex>,
}

// (value, nodeindex). Orders only by increasing value.
#[derive(Clone, Copy, Debug, Eq, Ord)]
struct Value(usize, usize);
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// (Coordinate, value at coordinate)
// This allows having multiple values in each coordinate, which is useful to
// keep the pareto fronts clean.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ValuedPos(usize, usize);
impl PartialOrd for ValuedPos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ValuedPos {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (Reverse(self.0), self.1).cmp(&(Reverse(other.0), other.1))
    }
}

impl IncreasingFunction2D<usize> {
    pub fn val(&self, idx: NodeIndex) -> usize {
        self.nodes[idx].val
    }

    // TODO: Support max_match_cost > 0
    /// Build the increasing function over the given points. `l` must be at least 1.
    /// `ps` must be sorted increasing by (x,y), first on x and then on y.
    pub fn new(target: Pos, max_match_cost: usize, leftover_at_end: bool, ps: Vec<Match>) -> Self {
        let mut s = Self {
            nodes: Vec::new(),
            // Placeholder until properly set in build.
            root: 0,
            leftover_at_end,
        };
        s.build(target, max_match_cost, ps);
        assert!(s.nodes.len() > 0);
        s
    }

    fn build<'a>(&'a mut self, target: Pos, max_match_cost: usize, ps: Vec<Match>) {
        // ValuedPos(j, value) -> Value(max walk to target, parent idx).
        type F = IncreasingFunction<ValuedPos, Value>;
        let mut front = F::new();
        let root = Pos(usize::MAX, usize::MAX);

        let push_node = |start: Pos, val: usize, front: &mut F, nodes: &mut Vec<Node<usize>>| {
            //println!("Bump {:?} to {}", start, val);
            // 1. Check if the value is still large enough.
            // 1b. If not, continue.
            let (next_val, mut parent) = front
                .get(ValuedPos(start.1, usize::MAX))
                .map_or((0, None), |Value(current_val, parent_idx)| {
                    (current_val + 1, Some(parent_idx))
                });
            if val < next_val {
                return;
            }

            // The value shouldn't grow much when using 1 extra seed. This makes
            // sure we add at most max_match_distance epsilon nodes.
            // TODO: Replace 1 by match distance
            assert!(
                val <= next_val + max_match_cost,
                "{} <= {}",
                val,
                next_val + max_match_cost
            );

            // 2. Insert nodes for all values up to the current value, to have consistent pareto fronts.
            for val in next_val..=val {
                // The id of the node we're adding here.
                let id = nodes.len();

                // 3. Find `next`: The index of the node with this value, if present.
                // This should just be the first incremental value after the current position.
                let next = front.get_larger(ValuedPos(start.1, val)).and_then(
                    |Value(nextval, next_idx)| {
                        // Since we keep clean pareto fronts, this must always exist.
                        // We can never skip a value.
                        assert_eq!(nextval, val);
                        // Prev/Next nodes are in direct correspondence.
                        assert!(nodes[next_idx].prev.is_none());
                        nodes[next_idx].prev = Some(id);
                        Some(next_idx)
                    },
                );

                //println!(
                //"Push id {}: {:?} => {}, parent {:?} next {:?}",
                //id, start, val, parent, next
                //);
                nodes.push(Node {
                    pos: start,
                    val,
                    parent,
                    prev: None,
                    next,
                    child: None,
                });
                // Update the child for our parent.
                if let Some(parent) = parent {
                    nodes[parent].child = Some(id);
                }

                assert!(front.set(ValuedPos(start.1, val), Value(val, id)));

                parent = Some(id);
            }
        };

        // Push the global root.
        push_node(root, 0, &mut front, &mut self.nodes);
        //dbg!(&front.m);
        //dbg!(&self.nodes);

        // Sort by start, the order in which we will add them to the front.
        let ps_by_start = {
            let mut ps = ps;
            ps.sort_by_key(|Match { start, end, .. }| Reverse((start.0, start.1, end.0, end.1)));
            ps
        };
        //let mut best_per_pos = HashMap::new();
        for m in ps_by_start {
            //println!("Match: {:?}", m);

            // Find the parent for the end of this match.
            let parent_idx = front
                .get(ValuedPos(m.end.1, usize::MAX))
                .map(|Value(_, hint)| self.incremental_forward(m.end, hint))
                .unwrap();
            //println!("Parent: {:?}", parent_idx);
            let val = match self.nodes[parent_idx] {
                // For matches to the end, take into account the gap penalty.
                // NOTE: This assumes that the global root is at index 0.
                Node { pos, .. } if pos == root => {
                    ((max_match_cost + 1) - m.match_cost).saturating_sub({
                        // gap cost between `end` and `target`
                        // This will only have effect when leftover_at_end is true
                        let di = target.0 - m.end.0;
                        let dj = target.1 - m.end.1;
                        let pot = (di + dj) / 2
                            - (if self.leftover_at_end {
                                max_match_cost + 1
                            } else {
                                0
                            });
                        let g = abs_diff(di, dj) / 2;
                        // println!(
                        //     "{:?} {:?} -> {} {} -> subtract: ({} - {} = {}) ({})",
                        //     m.end,
                        //     target,
                        //     di,
                        //     dj,
                        //     g,
                        //     pot,
                        //     g.saturating_sub(pot),
                        //     self.leftover_at_end
                        // );
                        g.saturating_sub(pot)
                    })
                }
                // The distance to the parent
                n => n.val + (max_match_cost + 1) - m.match_cost,
            };

            push_node(m.start, val, &mut front, &mut self.nodes);
        }
        //for n in &self.nodes {
        //println!("{:?}", n);
        //}
        //for n in &front.m {
        //println!("{:?}", n);
        //}
        // The root is the now largest value in the front.
        let Value(_, mut layer) = front.max().unwrap();
        self.root = layer;

        // Fill children pointers, layer by layer.
        while let Some(u) = self.nodes[layer].parent {
            // Since u is the parent of some node, it is guaranteed that is has a child.
            // Move left of u and copy over the parent.
            {
                let mut u = u;
                while let Some(v) = self.nodes[u].prev {
                    let c = self.nodes[u].child.unwrap();
                    self.nodes[v].child.get_or_insert(c);
                    u = v;
                }
            }
            // Move right of u and copy over the parent.
            {
                let mut u = u;
                while let Some(v) = self.nodes[u].next {
                    let c = self.nodes[u].child.unwrap();
                    self.nodes[v].child.get_or_insert(c);
                    u = v;
                }
            }
            layer = u;
        }
    }

    pub fn root<'a>(&'a self) -> NodeIndex {
        self.root
    }

    /// NOTE: This only works if pos is right-below (larger) than the position where hint_idx was obtained.
    /// Use `incremental` below otherwise.
    /// Moves to the next/prev neighbour as long as needed, and then goes to parents.
    pub fn incremental_forward<'a>(
        &'a self,
        pos @ Pos(i, j): Pos,
        mut hint_idx: NodeIndex,
    ) -> NodeIndex {
        //println!("GET JUMP {:?} {}", pos, hint_idx);
        loop {
            //println!("HINT: {}", hint_idx);
            let hint = &self.nodes[hint_idx];
            if pos <= hint.pos {
                //println!("GET JUMP {:?} {:?}", pos, Some(hint_idx));
                return hint_idx;
            }
            if let Some(next_idx) = hint.next {
                if j <= self.nodes[next_idx].pos.1 {
                    hint_idx = next_idx;
                    continue;
                }
            }
            if let Some(prev_idx) = hint.prev {
                if i <= self.nodes[prev_idx].pos.0 {
                    hint_idx = prev_idx;
                    continue;
                }
            }
            if let Some(prev_idx) = hint.parent {
                hint_idx = prev_idx;
                continue;
            }
            //println!("GET JUMP {:?} {:?}", pos, None::<()>);
            unreachable!("Pos {:?} is not covered by botright maximum", pos);
        }
    }

    // This also handles steps in the (1,-1) and (-1,1) quadrants.
    pub fn incremental<'a>(
        &'a self,
        pos @ Pos(i, j): Pos,
        mut hint_idx: NodeIndex,
        // TODO: Use this.
        _hint_pos: Pos,
    ) -> NodeIndex {
        if self.nodes.is_empty() {
            return hint_idx;
        }
        // TODO: This is ugly, but it should work for now as backward steps are small.
        if let Some(x) = self.nodes[hint_idx].child {
            hint_idx = x;
        }
        if let Some(x) = self.nodes[hint_idx].child {
            hint_idx = x;
        }

        //println!("GET JUMP {:?} {}", pos, hint_idx);
        loop {
            //println!("HINT: {}", hint_idx);
            let hint = &self.nodes[hint_idx];
            if pos <= hint.pos {
                //println!("GET JUMP {:?} {:?}", pos, Some(hint_idx));
                return hint_idx;
            }
            if let Some(next_idx) = hint.next {
                if j <= self.nodes[next_idx].pos.1 {
                    hint_idx = next_idx;
                    continue;
                }
            }
            if let Some(prev_idx) = hint.prev {
                if i <= self.nodes[prev_idx].pos.0 {
                    hint_idx = prev_idx;
                    continue;
                }
            }
            if let Some(prev_idx) = hint.parent {
                hint_idx = prev_idx;
                continue;
            }
            //println!("GET JUMP {:?} {:?}", pos, None::<()>);
            unreachable!("Pos {:?} is not covered by botright maximum", pos);
        }
    }

    pub fn to_map(&self) -> HashMap<Pos, usize> {
        self.nodes
            .iter()
            .filter_map(|&Node { pos, val, .. }| match pos {
                Pos(usize::MAX, usize::MAX) => None,
                _ => Some((pos, val)),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let f = IncreasingFunction2D::new(Pos(10, 10), 1, false, vec![]);
        assert_eq!(f.nodes.len(), 1);
    }

    fn to_map(f: &IncreasingFunction2D<usize>) -> HashMap<Pos, Node<usize>> {
        f.nodes
            .iter()
            .copied()
            .map(|n @ Node { pos, .. }| (pos, n))
            .collect()
    }

    #[test]
    fn test_cross() {
        for start_x in [7, 6] {
            println!("\n\nRUN: {}", start_x);
            let f = IncreasingFunction2D::new(
                Pos(10, 10),
                1,
                false,
                vec![
                    Match {
                        start: Pos(start_x, 9),
                        end: Pos(10, 10),
                        match_cost: 1,
                    },
                    Match {
                        start: Pos(4, 4),
                        end: Pos(6, 6),
                        match_cost: 0,
                    },
                    Match {
                        start: Pos(4, 4),
                        end: Pos(5, 7),
                        match_cost: 1,
                    },
                    Match {
                        start: Pos(4, 4),
                        end: Pos(7, 5),
                        match_cost: 1,
                    },
                    Match {
                        start: Pos(3, 5),
                        end: Pos(6, 6),
                        match_cost: 1,
                    },
                    Match {
                        start: Pos(5, 3),
                        end: Pos(6, 6),
                        match_cost: 1,
                    },
                ],
            );
            for x in &f.nodes {
                println!("{:?}", x);
            }
            let m = to_map(&f);
            assert!(m[&Pos(4, 4)].val == m[&Pos(3, 5)].val + 1);
            assert!(m[&Pos(4, 4)].val == m[&Pos(5, 3)].val + 1);
        }
    }

    #[test]
    fn broken_pareto_front() {
        let f = IncreasingFunction2D::new(
            Pos(10, 10),
            1,
            false,
            vec![
                Match {
                    start: Pos(3, 9),
                    end: Pos(10, 10),
                    match_cost: 1,
                },
                Match {
                    start: Pos(4, 8),
                    end: Pos(10, 10),
                    match_cost: 0,
                },
                Match {
                    start: Pos(5, 7),
                    end: Pos(10, 10),
                    match_cost: 1,
                },
                Match {
                    start: Pos(6, 6),
                    end: Pos(10, 10),
                    match_cost: 0,
                },
                Match {
                    start: Pos(7, 5),
                    end: Pos(10, 10),
                    match_cost: 1,
                },
                Match {
                    start: Pos(8, 4),
                    end: Pos(10, 10),
                    match_cost: 0,
                },
            ],
        );
        println!("\n\nRUN:");
        for x in &f.nodes {
            println!("{:?}", x);
        }
        assert_eq!(f.nodes.len(), 10);

        // Test Jump
        assert_eq!(f.incremental_forward(Pos(4, 9), 1), 0);
        assert_eq!(f.incremental_forward(Pos(7, 5), 5), 3);
        assert_eq!(f.incremental_forward(Pos(3, 9), 2), 9);
        assert_eq!(f.incremental_forward(Pos(3, 7), 2), 8);
    }
}
