use crate::diagonal_map::DiagonalMapTrait;
use crate::prelude::*;
use crate::scored::MinScored;

#[derive(Clone, Copy)]
enum Status {
    Unvisited,
    Explored,
    Expanded,
}
use Status::*;

#[derive(Clone, Copy)]
struct State<Parent> {
    status: Status,
    // TODO: Do we really need f?
    f: Cost,
    g: Cost,
    parent: Parent,
}

impl<Parent: Default> Default for State<Parent> {
    fn default() -> Self {
        Self {
            status: Unvisited,
            f: Cost::MAX,
            g: Cost::MAX,
            parent: Parent::default(),
        }
    }
}

// h: heuristic = lower bound on cost from node to end
// g: computed cost to reach node from the start
// f: g+h
pub fn astar<G, H, ExpandFn, ExploreFn>(
    graph: &G,
    start: G::Pos,
    target: G::Pos,
    mut h: H,
    retry_outdated: bool,
    // Some callbacks
    mut on_expand: ExpandFn,
    mut on_explore: ExploreFn,
    // Counters
    double_expands: &mut usize,
    retries: &mut usize,
) -> Option<(Cost, Vec<G::Pos>)>
where
    G: ImplicitGraph,
    H: FnMut(G::Pos) -> Cost,
    ExpandFn: FnMut(G::Pos),
    ExploreFn: FnMut(G::Pos),
{
    let mut queue = heap::Heap::<G::Pos>::default(); // f
    let mut states = G::DiagonalMap::<State<G::Parent>>::new(target);

    states.insert(
        start,
        State {
            status: Explored,
            f: 0,
            g: 0,
            parent: Default::default(),
        },
    );
    queue.push(MinScored(h(start), start));

    while let Some(MinScored(f, pos)) = queue.pop() {
        // This lookup can be unwrapped without fear of panic since the node was necessarily scored
        // before adding it to `visit_next`.
        //let g = gs[pos];
        let state = &mut states[pos];
        let g = state.g;

        // If the heuristic value is outdated, skip the node and re-push it with the updated value.
        if retry_outdated {
            let current_f = g + h(pos);
            if current_f > f {
                *retries += 1;
                queue.push(MinScored(current_f, pos));
                continue;
            }
        }

        // Expand the state.
        match state.status {
            Unvisited => {
                unreachable!("Cannot explore an unvisited node")
            }
            // Expand the currently explored state.
            Explored => {
                state.status = Expanded;
                state.f = f;
            }
            Expanded => {
                if f < state.f {
                    state.f = f;
                    *double_expands += 1;
                } else {
                    // Skip if f is not better than the previous best f.
                    // FIXME: Does this skipping break consistency if f has
                    // jumped up from pruning in between the first and the
                    // second time visiting this node?
                    // Could be fixed by checking g instead.
                    continue;
                }
            }
        };

        // Retrace path to root and return.
        if pos == target {
            let last = pos;
            let mut path = vec![last];

            let mut current = last;
            while let Some(previous) = states[current].parent.parent(&current) {
                path.push(previous);
                current = previous;
            }

            path.reverse();
            return Some((g, path));
        }

        // Number of times we compute the neighbours of a node.
        on_expand(pos);

        graph.iterate_outgoing_edges(pos, |next, cost, parent| {
            let next_g = g + cost;

            // Expand next
            let next_state = &mut states[next];
            if let Unvisited = next_state.status {
                next_state.status = Explored;
            } else {
                if next_g >= next_state.g {
                    return;
                }
            };
            next_state.g = next_g;
            next_state.parent = parent;

            // Number of pushes on the stack.
            on_explore(next);

            let next_f = next_g + h(next);
            queue.push(MinScored(next_f, next));
        });
    }

    None
}
