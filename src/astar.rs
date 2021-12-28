use std::hash::Hash;

use petgraph::visit::{EdgeRef, GraphBase, IntoEdges};

use crate::diagonal_map::ToPos;
use crate::implicit_graph::IterateEdgesDirected;
use crate::prelude::*;
use crate::scored::MinScored;

/// \[Generic\] A* shortest path algorithm.
///
/// Computes the shortest path from `start` to `finish`, including the total path cost.
///
/// `finish` is implicitly given via the `is_goal` callback, which should return `true` if the
/// given node is the finish node.
///
/// The function `edge_cost` should return the cost for a particular edge. Edge costs must be
/// non-negative.
///
/// The function `estimate_cost` should return the estimated cost to the finish for a particular
/// node. For the algorithm to find the actual shortest path, it should be admissible, meaning that
/// it should never overestimate the actual cost to get to the nearest goal node. Estimate costs
/// must also be non-negative.
///
/// The graph should be `Visitable` and implement `IntoEdges`.
///
/// # Example
/// ```
/// use petgraph::Graph;
/// use petgraph::algo::astar;
///
/// let mut g = Graph::new();
/// let a = g.add_node((0., 0.));
/// let b = g.add_node((2., 0.));
/// let c = g.add_node((1., 1.));
/// let d = g.add_node((0., 2.));
/// let e = g.add_node((3., 3.));
/// let f = g.add_node((4., 2.));
/// g.extend_with_edges(&[
///     (a, b, 2),
///     (a, d, 4),
///     (b, c, 1),
///     (b, f, 7),
///     (c, e, 5),
///     (e, f, 1),
///     (d, e, 1),
/// ]);
///
/// // Graph represented with the weight of each edge
/// // Edges with '*' are part of the optimal path.
/// //
/// //     2       1
/// // a ----- b ----- c
/// // | 4*    | 7     |
/// // d       f       | 5
/// // | 1*    | 1*    |
/// // \------ e ------/
///
/// let path = astar(&g, a, |finish| finish == f, |e| *e.weight(), |_| 0);
/// assert_eq!(path, Some((6, vec![a, d, e, f])));
/// ```
///
/// Returns the total cost + the path of subsequent `NodeId` from start to finish, if one was
/// found.
pub fn astar<G, F, H, ExpandFn, ExploreFn>(
    target: Pos,
    graph: G,
    start: G::NodeId,
    mut edge_cost: F,
    mut estimate_cost: H,
    mut on_expand: ExpandFn,
    mut on_explore: ExploreFn,
    retry_outdated: bool,
    double_expands: &mut usize,
    retries: &mut usize,
) -> Option<(usize, Vec<Pos>)>
where
    G: IntoEdges + IterateEdgesDirected,
    G::NodeId: Eq + Hash + Ord + ToPos,
    F: FnMut(G::EdgeRef) -> usize,
    H: FnMut(G::NodeId) -> usize,
    ExpandFn: FnMut(G::NodeId),
    ExploreFn: FnMut(G::NodeId),
    <G as GraphBase>::NodeId: std::fmt::Debug,
{
    let mut visit_next = heap::Heap::default(); // f-values, cost to reach + estimate cost to goal, and the node itself
    let mut scores = diagonal_map::DiagonalMap::new(target); // g-values, cost to reach the node
    let mut estimate_scores = diagonal_map::DiagonalMap::new(target); // f-values, cost to reach + estimate cost to goal
    let mut path_tracker = PathTracker::new(target);

    let zero_score = 0usize;
    scores.insert(start.to_pos(), zero_score);
    visit_next.push(MinScored(estimate_cost(start), start));

    while let Some(MinScored(estimate_score, node)) = visit_next.pop() {
        // This lookup can be unwrapped without fear of panic since the node was necessarily scored
        // before adding it to `visit_next`.
        let node_score = scores[&node.to_pos()];

        if node.to_pos() == target {
            let path = path_tracker.reconstruct_path_to(node.to_pos());
            return Some((node_score, path));
        }

        // If the heuristic value is outdated, skip the node and re-push it with the updated value.
        if retry_outdated {
            let current_estimate_score = node_score + estimate_cost(node);
            if current_estimate_score > estimate_score {
                *retries += 1;
                visit_next.push(MinScored(current_estimate_score, node));
                continue;
            }
        }

        match estimate_scores.entry(node.to_pos()) {
            diagonal_map::Entry::Occupied(mut entry) => {
                // If the node has already been visited with an equal or lower score than now, then
                // we do not need to re-visit it.
                if *entry.get() <= estimate_score {
                    continue;
                }
                *double_expands += 1;
                // assert!(
                //     false,
                //     "Double expand of {:?} first {} now {}",
                //     node,
                //     *entry.get(),
                //     estimate_score
                // );
                entry.insert(estimate_score);
            }
            diagonal_map::Entry::Vacant(entry) => {
                entry.insert(estimate_score);
            }
        }

        // Number of times we compute the neighbours of a node.
        on_expand(node);

        graph.iterate_edges_directed(node, petgraph::EdgeDirection::Outgoing, |edge| {
            let next = edge.target();
            let next_score = node_score + edge_cost(edge);

            match scores.entry(next.to_pos()) {
                diagonal_map::Entry::Occupied(mut entry) => {
                    // No need to add neighbours that we have already reached through a shorter path
                    // than now.
                    if *entry.get() <= next_score {
                        return;
                    }
                    entry.insert(next_score);
                }
                diagonal_map::Entry::Vacant(entry) => {
                    entry.insert(next_score);
                }
            }

            // Number of pushes on the stack.
            on_explore(next);

            path_tracker.set_predecessor(next.to_pos(), node.to_pos());
            let next_estimate_score = next_score + estimate_cost(next);
            // FIXME: Enable this assert
            // assert!(
            //     estimate_score <= next_estimate_score,
            //     "Heuristic is not path consistent. {:?}: {}+{} -> {:?}: {}+{}",
            //     node,
            //     node_score,
            //     estimate_score - node_score,
            //     next,
            //     next_score,
            //     next_estimate_score - next_score
            // );
            visit_next.push(MinScored(next_estimate_score, next));
        });
    }

    None
}

struct PathTracker {
    came_from: diagonal_map::DiagonalMap<Pos>,
}

impl PathTracker {
    fn new(target: Pos) -> PathTracker {
        PathTracker {
            came_from: diagonal_map::DiagonalMap::new(target),
        }
    }

    fn set_predecessor(&mut self, node: Pos, previous: Pos) {
        self.came_from.insert(node, previous);
    }

    fn reconstruct_path_to(&self, last: Pos) -> Vec<Pos> {
        let mut path = vec![last];

        let mut current = last;
        while let Some(&previous) = self.came_from.get(&current.to_pos()) {
            path.push(previous);
            current = previous;
        }

        path.reverse();

        path
    }
}
