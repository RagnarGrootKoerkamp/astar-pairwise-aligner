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
struct State<Parent, Hint> {
    status: Status,
    // TODO: Do we really need f?
    f: Cost,
    g: Cost,
    parent: Parent,
    hint: Hint,
}

impl<Parent: Default, Hint: Default> Default for State<Parent, Hint> {
    fn default() -> Self {
        Self {
            status: Unvisited,
            f: Cost::MAX,
            g: Cost::MAX,
            parent: Parent::default(),
            hint: Hint::default(),
        }
    }
}

// h: heuristic = lower bound on cost from node to end
// g: computed cost to reach node from the start
// f: g+h
// TODO: Inline on_expand and on_explore functions by direct calls to h.
pub fn astar<G, H, ExpandFn, ExploreFn, Hint: Default + Copy>(
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
    H: FnMut(G::Pos, Hint) -> (Cost, Hint),
    ExpandFn: FnMut(G::Pos),
    ExploreFn: FnMut(G::Pos),
{
    let mut queue = heap::Heap::<G::Pos>::default(); // f
    let mut states = G::DiagonalMap::<State<G::Parent, Hint>>::new(target);

    {
        let (hroot, hint) = h(start, Hint::default());
        queue.push(MinScored(hroot, start));
        states.insert(
            start,
            State {
                status: Explored,
                f: 0,
                g: 0,
                parent: Default::default(),
                hint,
            },
        );
    }

    while let Some(MinScored(f, pos)) = queue.pop() {
        // This lookup can be unwrapped without fear of panic since the node was necessarily scored
        // before adding it to `visit_next`.
        //let g = gs[pos];
        let state = &mut states[pos];
        let g = state.g;
        let hint = state.hint;

        // If the heuristic value is outdated, skip the node and re-push it with the updated value.
        if retry_outdated {
            let (current_h, new_hint) = h(pos, state.hint);
            state.hint = new_hint;
            let current_f = g + current_h;
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

            let (next_h, next_hint) = h(next, hint);
            next_state.hint = next_hint;
            let next_f = next_g + next_h;
            queue.push(MinScored(next_f, next));
        });
    }

    None
}
