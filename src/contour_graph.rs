use std::cmp::{Ord, Ordering, Reverse};
use std::hash;

use itertools::Itertools;

use crate::prelude::*;
use crate::seeds::Match;
use crate::thresholds::vec::Thresholds;

// A private wrapper type. Indices should be hidden to the outside world.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct NodeIndex(usize);

// We guarantee that the function always contains (0,0), so lookup will always succeed.
#[derive(Default)]
pub struct ContourGraph<T: Copy + hash::Hash + Eq> {
    nodes: Vec<Node<T>>,
    // val=0
    bot: NodeIndex,
    // val=max
    max: NodeIndex,
    // FIXME
    #[allow(dead_code)]
    leftover_at_end: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Node<T: Copy + hash::Hash + Eq> {
    pub pos: Pos,
    pub val: T,
    parent: Option<NodeIndex>,
    prev: Option<NodeIndex>,
    next: Option<NodeIndex>,
    child: Option<NodeIndex>,
}

impl<T: Copy + hash::Hash + Eq> std::ops::Index<NodeIndex> for Vec<Node<T>> {
    type Output = Node<T>;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self[index.0 as usize]
    }
}
impl<T: Copy + hash::Hash + Eq> std::ops::IndexMut<NodeIndex> for Vec<Node<T>> {
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        &mut self[index.0 as usize]
    }
}

// (Coordinate, value at coordinate)
// This allows having multiple values in each coordinate, which is useful to
// keep the pareto fronts clean.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ValuedPos(usize, usize);
impl PartialOrd for ValuedPos {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ValuedPos {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        (Reverse(self.0), self.1).cmp(&(Reverse(other.0), other.1))
    }
}

impl ContourGraph<usize> {
    #[inline]
    pub fn val(&self, idx: NodeIndex) -> usize {
        self.nodes[idx].val
    }

    /// Build the increasing function over the given points. `l` must be at least 1.
    /// `ps` must be sorted increasing by (x,y), first on x and then on y.
    pub fn new(target: Pos, leftover_at_end: bool, ps: Vec<Match>) -> Self {
        let mut s = Self {
            nodes: Vec::new(),
            bot: NodeIndex(0),
            // Placeholder until properly set in build.
            max: NodeIndex(0),
            leftover_at_end,
        };
        s.build(target, ps);
        assert!(!s.nodes.is_empty());
        s
    }

    fn build(&mut self, target: Pos, ps: Vec<Match>) {
        // ValuedPos(j, value) -> (layer, parent idx).
        type F = Thresholds<ValuedPos, NodeIndex>;
        let mut front = F::default();
        let root = Pos(usize::MAX, usize::MAX);

        let push_node = |start: Pos, val: usize, front: &mut F, nodes: &mut Vec<Node<usize>>| {
            // 1. Check if the value is still large enough.
            // 1b. If not, continue.
            let (next_val, mut parent) = front
                .get(ValuedPos(start.1, usize::MAX))
                .map_or((0, None), |(current_val, parent_idx)| {
                    (current_val + 1, Some(parent_idx))
                });
            if val < next_val {
                return;
            }

            // 2. Insert nodes for all values up to the current value, to have consistent pareto fronts.
            for val in next_val..=val {
                // The id of the node we're adding here.
                let id = NodeIndex(nodes.len());

                // 3. Find `next`: The index of the node with this value, if present.
                // This should just be the first incremental value after the current position.
                let next = front
                    .get_larger(ValuedPos(start.1, val))
                    .map(|(nextval, next_idx)| {
                        // Since we keep clean pareto fronts, this must always exist.
                        // We can never skip a value.
                        assert_eq!(nextval, val);
                        // Prev/Next nodes are in direct correspondence.
                        assert!(nodes[next_idx].prev.is_none());
                        nodes[next_idx].prev = Some(id);
                        next_idx
                    });

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

                assert!(front.set(ValuedPos(start.1, val), (val, id)));

                parent = Some(id);
            }
        };

        // Push the global root.
        push_node(root, 0, &mut front, &mut self.nodes);

        // Sort reversed by start, the order in which we will add them to the front.
        let mut ps_by_start = ps;
        ps_by_start.sort_unstable_by_key(|&Match { start, .. }| Reverse(LexPos(start)));
        let ps_by_start = ps_by_start;

        // Same order as ps_by_start.
        let mut match_values = vec![0; ps_by_start.len()];

        // Sort reversed by end, the order in which we evaluate their values.
        let mut ps_by_end: Vec<usize> = (0..ps_by_start.len()).collect_vec();
        ps_by_end.sort_unstable_by_key(|&idx| Reverse(LexPos(ps_by_start[idx].end)));
        let mut ps_by_end = ps_by_end.into_iter().peekable();

        for (idx, m) in ps_by_start.iter().enumerate() {
            while let Some(&idx) = ps_by_end.peek() {
                if LexPos(ps_by_start[idx].end) <= LexPos(m.start) {
                    break;
                }
                let m = &ps_by_start[idx];
                ps_by_end.next();

                // Process all ends after this start.
                // Find the parent for the end of this match.
                let parent_idx = front.get(ValuedPos(m.end.1, usize::MAX)).unwrap().1;
                let parent = self.nodes[parent_idx];
                let val = if parent.pos != root {
                    // The distance to the parent
                    parent.val + m.seed_potential - m.match_cost
                } else {
                    // For matches to the end, take into account the gap penalty.
                    // NOTE: This assumes that the global root is at index 0.
                    (m.seed_potential - m.match_cost).saturating_sub({
                        // gap cost between `end` and `target`
                        // This will only have effect when leftover_at_end is true
                        let di = target.0 - m.end.0;
                        let dj = target.1 - m.end.1;
                        // TODO: Fix edge case when there are characters at the end not covered by a seed.
                        let pot = (di + dj) / 2;
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
                };
                assert!(match_values[idx] == 0);
                match_values[idx] = val;
            }

            push_node(m.start, match_values[idx], &mut front, &mut self.nodes);
        }

        // The root is the now largest value in the front.
        let (_, mut layer) = front.max().unwrap();
        self.max = layer;

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

        // Sorting the nodes improves cache locality.
        self.sort_nodes();
    }

    // Sort the nodes by (layer, position) and update all pointers.
    // This should improve cache locality.
    fn sort_nodes(&mut self) {
        let mut perm = (0..self.nodes.len()).collect_vec();
        perm.sort_by_key(|i| (Reverse(self.nodes[*i].val), (self.nodes[*i].pos.0)));
        let mut inv = vec![0; self.nodes.len()];
        for (i, &x) in perm.iter().enumerate() {
            inv[x] = i;
        }

        // Update pointers.
        self.nodes.iter_mut().for_each(|node| {
            node.child.as_mut().map(|c| c.0 = inv[c.0]);
            node.parent.as_mut().map(|c| c.0 = inv[c.0]);
            node.next.as_mut().map(|c| c.0 = inv[c.0]);
            node.prev.as_mut().map(|c| c.0 = inv[c.0]);
        });
        self.bot.0 = inv[self.bot.0];
        self.max.0 = inv[self.max.0];

        // Reorder elements.
        self.nodes = perm.into_iter().map(|idx| self.nodes[idx]).collect_vec();
    }

    #[inline]
    pub fn bot(&self) -> NodeIndex {
        self.bot
    }

    #[inline]
    pub fn max(&self) -> NodeIndex {
        self.max
    }

    /// NOTE: This only works if pos is right-below (larger) than the position where hint_idx was obtained.
    /// Use `incremental` below otherwise.
    /// Moves to the next/prev neighbour as long as needed, and then goes to parents.
    #[inline]
    pub fn incremental_forward(&self, pos @ Pos(i, j): Pos, mut hint_idx: NodeIndex) -> NodeIndex {
        loop {
            let hint = &self.nodes[hint_idx];
            if pos <= hint.pos {
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
            unreachable!("Pos {:?} is not covered by botright maximum", pos);
        }
    }

    // This also handles steps in the (1,-1) and (-1,1) quadrants.
    #[inline]
    pub fn incremental(
        &self,
        pos @ Pos(i, j): Pos,
        mut hint_idx: NodeIndex,
        // TODO: Use this.
        hint_pos: Pos,
    ) -> NodeIndex {
        if self.nodes.is_empty() {
            return hint_idx;
        }
        // Small optimization, since this is the common case after transformation.
        if pos == hint_pos {
            return hint_idx;
        }
        // TODO: This is ugly, but it should work for now as backward steps are small.
        // TODO: Add an assertion to make sure we've walked backwards far enough.
        if !(pos >= hint_pos) {
            if let Some(x) = self.nodes[hint_idx].child {
                hint_idx = x;
            }
            if let Some(x) = self.nodes[hint_idx].child {
                //if self.nodes[hint_idx].pos == self.nodes[x].pos {
                hint_idx = x;
                //}
            }
        }

        loop {
            let hint = &self.nodes[hint_idx];
            if pos <= hint.pos {
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
            unreachable!("Pos {:?} is not covered by botright maximum", pos);
        }
    }

    pub fn to_map(&self) -> HashMap<Pos, usize> {
        // There can be multiple nodes at the same position. In that case we take the maximum value.
        let mut m = HashMap::default();
        for &Node { pos, val, .. } in &self.nodes {
            if pos == Pos(usize::MAX, usize::MAX) {
                continue;
            }

            match m.entry(pos) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(val);
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    if *entry.get() < val {
                        entry.insert(val);
                    }
                }
            }
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let f = ContourGraph::new(Pos(10, 10), false, vec![]);
        assert_eq!(f.nodes.len(), 1);
    }

    #[test]
    fn test_cross() {
        for start_x in [7, 6] {
            println!("\n\nRUN: {}", start_x);
            let f = ContourGraph::new(
                Pos(10, 10),
                false,
                vec![
                    Match {
                        start: Pos(start_x, 9),
                        end: Pos(10, 10),
                        match_cost: 1,
                        seed_potential: 2,
                    },
                    Match {
                        start: Pos(4, 4),
                        end: Pos(6, 6),
                        match_cost: 0,
                        seed_potential: 2,
                    },
                    Match {
                        start: Pos(4, 4),
                        end: Pos(5, 7),
                        match_cost: 1,
                        seed_potential: 2,
                    },
                    Match {
                        start: Pos(4, 4),
                        end: Pos(7, 5),
                        match_cost: 1,
                        seed_potential: 2,
                    },
                    Match {
                        start: Pos(3, 5),
                        end: Pos(6, 6),
                        match_cost: 1,
                        seed_potential: 2,
                    },
                    Match {
                        start: Pos(5, 3),
                        end: Pos(6, 6),
                        match_cost: 1,
                        seed_potential: 2,
                    },
                ],
            );
            let m = f.to_map();
            assert!(m[&Pos(4, 4)] == m[&Pos(3, 5)] + 1);
            assert!(m[&Pos(4, 4)] == m[&Pos(5, 3)] + 1);
        }
    }

    #[test]
    fn broken_pareto_front() {
        let f = ContourGraph::new(
            Pos(10, 10),
            false,
            vec![
                Match {
                    start: Pos(3, 9),
                    end: Pos(10, 10),
                    match_cost: 1,
                    seed_potential: 2,
                },
                Match {
                    start: Pos(4, 8),
                    end: Pos(10, 10),
                    match_cost: 0,
                    seed_potential: 2,
                },
                Match {
                    start: Pos(5, 7),
                    end: Pos(10, 10),
                    match_cost: 1,
                    seed_potential: 2,
                },
                Match {
                    start: Pos(6, 6),
                    end: Pos(10, 10),
                    match_cost: 0,
                    seed_potential: 2,
                },
                Match {
                    start: Pos(7, 5),
                    end: Pos(10, 10),
                    match_cost: 1,
                    seed_potential: 2,
                },
                Match {
                    start: Pos(8, 4),
                    end: Pos(10, 10),
                    match_cost: 0,
                    seed_potential: 2,
                },
            ],
        );
        assert_eq!(f.nodes.len(), 10);

        // Test Jump
        assert_eq!(f.incremental_forward(Pos(4, 9), NodeIndex(2)), NodeIndex(9));
        assert_eq!(f.incremental_forward(Pos(7, 5), NodeIndex(1)), NodeIndex(7));
        assert_eq!(f.incremental_forward(Pos(3, 9), NodeIndex(2)), NodeIndex(3));
        assert_eq!(f.incremental_forward(Pos(3, 7), NodeIndex(2)), NodeIndex(0));
    }
}
