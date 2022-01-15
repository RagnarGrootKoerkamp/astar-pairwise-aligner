use crate::diagonal_map::DiagonalMapTrait;
use crate::prelude::*;
use crate::scored::MinScored;

#[derive(Clone, Copy)]
struct ExploredState<Parent> {
    g: usize,
    parent: Parent,
}

#[derive(Clone, Copy)]
struct ExpandedState<Parent> {
    g: usize,
    // TODO: Can we not just use g instead?
    f: usize,
    parent: Parent,
}

enum State<Parent> {
    Unvisited,
    Explored(ExploredState<Parent>),
    Expanded(ExpandedState<Parent>),
}
use State::*;

impl<Parent> State<Parent> {
    fn expanded(&self) -> &ExpandedState<Parent> {
        match self {
            Expanded(state) => state,
            _ => unreachable!("Not an explored state"),
        }
    }
    fn g(&self) -> usize {
        match self {
            Unvisited => unreachable!("Not a visited state"),
            Explored(state) => state.g,
            Expanded(state) => state.g,
        }
    }
}

impl<Parent> Default for State<Parent> {
    fn default() -> Self {
        State::Unvisited
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
) -> Option<(usize, Vec<G::Pos>)>
where
    G: ImplicitGraph,
    H: FnMut(G::Pos) -> usize,
    ExpandFn: FnMut(G::Pos),
    ExploreFn: FnMut(G::Pos),
{
    let mut queue = heap::Heap::<G::Pos>::default(); // f

    // TODO: Merge these three DiagonalMaps?
    let mut states = G::DiagonalMap::<State<G::Parent>>::new(target);

    states.insert(
        start,
        Explored(ExploredState::<G::Parent> {
            g: 0,
            parent: G::Parent::default(),
        }),
    );
    queue.push(MinScored(h(start), start));

    while let Some(MinScored(f, pos)) = queue.pop() {
        // This lookup can be unwrapped without fear of panic since the node was necessarily scored
        // before adding it to `visit_next`.
        //let g = gs[pos];
        let state = &mut states[pos];
        let g = state.g();

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
        *state = match *state {
            Unvisited => {
                unreachable!("Cannot explore an unvisited node")
            }
            // Expand the currently explored state.
            Explored(ExploredState { g, parent }) => Expanded(ExpandedState { g, f, parent }),
            Expanded(mut s) => {
                if f < s.f {
                    s.f = f;
                    *double_expands += 1;
                    Expanded(s)
                } else {
                    // Skip if f is not better than the previous best f.
                    continue;
                }
            }
        };

        // Retrace path to root and return.
        if pos == target {
            let last = pos;
            let mut path = vec![last];

            let mut current = last;
            while let Some(previous) = states[current].expanded().parent.parent(&current) {
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
            match &mut states[next] {
                s @ Unvisited => *s = Explored(ExploredState { g: next_g, parent }),
                // Expand the currently explored state.
                Explored(s) => {
                    if next_g >= s.g {
                        return;
                    }
                    *s = ExploredState { g: next_g, parent };
                }
                Expanded(s) => {
                    if next_g >= s.g {
                        return;
                    }
                    *s = ExpandedState {
                        g: next_g,
                        f: s.f,
                        parent,
                    };
                }
            };

            // Number of pushes on the stack.
            on_explore(next);

            let next_f = next_g + h(next);
            queue.push(MinScored(next_f, next));
        });
    }

    None
}
