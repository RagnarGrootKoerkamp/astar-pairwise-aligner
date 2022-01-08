use crate::diagonal_map::{DiagonalMapTrait, InsertIfSmallerResult};
use crate::prelude::*;
use crate::scored::MinScored;

// h: heuristic = lower bound on cost from node to end
// g: computed cost to reach node from the start
// f: g+h
pub fn astar<G, H, ExpandFn, ExploreFn>(
    graph: &G,
    start: NodeG<G>,
    target: G::Pos,
    mut h: H,
    retry_outdated: bool,
    // Some callbacks
    mut on_expand: ExpandFn,
    mut on_explore: ExploreFn,
    // Counters
    double_expands: &mut usize,
    retries: &mut usize,
) -> Option<(usize, Vec<G::Pos>)>
where
    G: ImplicitGraph,
    H: FnMut(NodeG<G>) -> usize,
    ExpandFn: FnMut(NodeG<G>),
    ExploreFn: FnMut(NodeG<G>),
{
    let mut visit_next = heap::Heap::<G>::default(); // f
    let mut scores = G::DiagonalMap::new(target); // g
    let mut estimate_scores = G::DiagonalMap::new(target); // f
    let mut path_tracker = PathTracker::<G>::new(target);

    let zero_score = 0;
    scores.insert(start.to_pos(), zero_score);
    visit_next.push(MinScored(h(start), start));

    while let Some(MinScored(estimate_score, node)) = visit_next.pop() {
        // This lookup can be unwrapped without fear of panic since the node was necessarily scored
        // before adding it to `visit_next`.
        let node_score = scores[node.to_pos()];

        if node.to_pos() == target {
            let path = path_tracker.reconstruct_path_to(node.to_pos());
            return Some((node_score, path));
        }

        // If the heuristic value is outdated, skip the node and re-push it with the updated value.
        if retry_outdated {
            let current_estimate_score = node_score + h(node);
            if current_estimate_score > estimate_score {
                *retries += 1;
                visit_next.push(MinScored(current_estimate_score, node));
                continue;
            }
        }

        match estimate_scores.insert_if_smaller(node.to_pos(), estimate_score) {
            InsertIfSmallerResult::New => {}
            InsertIfSmallerResult::Smaller => *double_expands += 1,
            InsertIfSmallerResult::Larger => continue,
        }

        // Number of times we compute the neighbours of a node.
        on_expand(node);

        graph.iterate_outgoing_edges(node, |next, cost| {
            let next_score = node_score + cost;

            if let InsertIfSmallerResult::Larger =
                scores.insert_if_smaller(next.to_pos(), next_score)
            {
                return;
            }

            // Number of pushes on the stack.
            on_explore(next);

            path_tracker.set_predecessor(next.to_pos(), node.to_pos());
            let next_estimate_score = next_score + h(next);
            visit_next.push(MinScored(next_estimate_score, next));
        });
    }

    None
}

struct PathTracker<G: ImplicitGraph> {
    came_from: G::DiagonalMap<G::Pos>,
}

impl<G: ImplicitGraph> PathTracker<G> {
    fn new(target: G::Pos) -> PathTracker<G> {
        PathTracker {
            came_from: G::DiagonalMap::new(target),
        }
    }

    fn set_predecessor(&mut self, node: G::Pos, previous: G::Pos) {
        self.came_from.insert(node, previous);
    }

    fn reconstruct_path_to(&self, last: G::Pos) -> Vec<G::Pos> {
        let mut path = vec![last];

        let mut current = last;
        while let Some(&previous) = self.came_from.get(&current) {
            path.push(previous);
            current = previous;
        }

        path.reverse();

        path
    }
}
