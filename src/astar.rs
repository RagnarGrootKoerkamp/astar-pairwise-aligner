use std::time;

use crate::{prelude::*, visualizer::VisualizerT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Unvisited,
    Explored,
    Expanded,
}
use Status::*;

#[derive(Clone, Copy, Debug)]
struct State<Hint> {
    /// TODO: `status` is only used for double-expand checks.
    /// The field should be removed at some point or only used in debug mode.
    status: Status,
    g: Cost,
    /// NOTE: `hint` could also be passed via the priority queue.
    hint: Hint,
}

impl<Hint: Default> Default for State<Hint> {
    fn default() -> Self {
        Self {
            status: Unvisited,
            g: Cost::MAX,
            hint: Hint::default(),
        }
    }
}

impl QueueOrder for (Pos, Cost) {
    type O = I;

    // Sort by Pos.i.
    fn key(&self) -> Self::O {
        self.0 .0
    }
}

#[derive(Default, Clone)]
pub struct AStarStats {
    pub expanded: usize,
    pub explored: usize,
    pub greedy_expanded: usize,
    /// Number of times an already expanded node was expanded again with a lower value of f.
    pub double_expanded: usize,
    /// Number of times a node was popped and found to have an outdated value of h after pruning.
    pub retries: usize,
    /// Total priority queue shift after pruning.
    pub pq_shifts: usize,
    /// Number of states allocated in the DiagonalMap
    pub diagonalmap_capacity: usize,

    pub traceback_duration: f32,
}

// h: heuristic = lower bound on cost from node to end
// g: computed cost to reach node from the start
// f: g+h
// TODO: Inline on_expand and on_explore functions by direct calls to h.
pub fn astar<'a, H>(
    graph: &EditGraph,
    h: &mut H,
    v: &mut impl VisualizerT,
) -> (Option<(Cost, Vec<Pos>)>, AStarStats)
where
    H: HeuristicInstance<'a>,
{
    const D: bool = false;

    let mut stats = AStarStats::default();

    // f -> (pos, g)
    let mut queue = BucketQueue::<(Pos, Cost)>::default();
    // When > 0, queue[x] corresponds to f=x+offset.
    // Increasing the offset implicitly shifts all elements of the queue up.
    let mut queue_offset: Cost = 0;
    // An upper bound on the queue_offset, to make sure indices never become negative.
    let max_queue_offset = if REDUCE_RETRIES {
        h.root_potential()
    } else {
        0
    };

    //let mut states = DiagonalMap::<State<H::Hint>>::new(graph.target());
    let mut states = HashMap::<Pos, State<H::Hint>>::default();

    let mut max_f = 0;
    v.new_layer_with_h(Some(h));

    // Initialization with the root state.
    {
        let start = Pos(0, 0);
        let (hroot, hint) = h.h_with_hint(start, H::Hint::default());
        queue.push(QueueElement {
            f: hroot + (max_queue_offset - queue_offset),
            data: (start, 0),
        });
        stats.explored += 1;
        states.insert(
            start,
            State {
                status: Explored,
                g: 0,
                hint,
            },
        );
    }

    'outer: while let Some(QueueElement {
        f: queue_f,
        data: (pos, queue_g),
    }) = queue.pop()
    {
        let queue_f = queue_f + queue_offset - max_queue_offset;
        // This lookup can be unwrapped without fear of panic since the node was necessarily scored
        // before adding it to `visit_next`.
        //let g = gs[pos];
        let state = &mut states[pos];

        if queue_g > state.g {
            continue;
        }

        assert!(queue_g == state.g);

        // Whenever A* pops a position, if the value of h and f is outdated, the point is pushed and not expanded.
        // Must be true for correctness.
        {
            let (current_h, new_hint) = h.h_with_hint(pos, state.hint);
            state.hint = new_hint;
            let current_f = state.g + current_h;
            assert!(
                current_f >= queue_f && current_h >= queue_f - queue_g,
                "Retry {pos} Current_f {current_f} smaller than queue_f {queue_f}! state.g={} queue_g={} queue_h={} current_h={}", state.g, queue_g, queue_f-queue_g, current_h
            );
            if current_f > queue_f {
                stats.retries += 1;
                queue.push(QueueElement {
                    f: current_f + (max_queue_offset - queue_offset),
                    data: (pos, queue_g),
                });
                continue;
            }
            assert!(current_f == queue_f);
            if D {
                println!(
                    "Expand {pos} at \tg={queue_g} \tf={queue_f} \th={current_h}\tqueue_h={}",
                    queue_f - queue_g
                );
            }
        }

        if queue_f > max_f {
            max_f = queue_f;
            v.new_layer_with_h(Some(h));
        }

        // Expand the state.
        match state.status {
            Unvisited => {
                unreachable!("Cannot explore an unvisited node")
            }
            // Expand the currently explored state.
            Explored => {
                state.status = Expanded;
            }
            Expanded => {
                stats.double_expanded += 1;
                assert!(
                    !h.is_seed_start_or_end(pos),
                    "Double expanded start of seed {:?}",
                    pos
                );
            }
        };

        // Copy for local usage.
        let state = *state;

        stats.expanded += 1;
        v.expand_with_h(pos, Some(h));

        // Prune is needed
        if h.is_seed_start_or_end(pos) {
            let pq_shift = h.prune(pos, state.hint);
            if REDUCE_RETRIES && pq_shift > 0 {
                stats.pq_shifts += pq_shift as usize;
                queue_offset += pq_shift;
            }
        }

        if D {
            println!("Expand {pos} {}", state.g);
        }

        // Retrace path to root and return.
        if pos == graph.target() {
            DiagonalMapTrait::insert(&mut states, pos, state);
            if D {
                println!("Reached target {pos} with state {state:?}");
            }
            break 'outer;
        }

        graph.iterate_outgoing_edges(pos, |mut next, edge| {
            // Explore next
            let next_g = state.g + edge.cost() as Cost;

            // Do greedy matching within the current seed.
            if graph.greedy_matching {
                while let Some(n) = graph.is_match(next) {
                    // Never greedy expand the start of a seed.
                    // Doing so may cause problems when h is not consistent and is
                    // larger at the start of seed than at the position where the
                    // greedy run started.
                    if h.is_seed_start_or_end(next) {
                        break;
                    }

                    // Explore & expand `next`
                    stats.explored += 1;
                    stats.expanded += 1;
                    stats.greedy_expanded += 1;
                    v.explore_with_h(next, Some(h));
                    v.expand_with_h(next, Some(h));
                    if D {
                        println!("Greedy expand {next} {}", state.g);
                    }

                    // Move to the next state.
                    next = n;
                }
            }

            let cur_next = DiagonalMapTrait::get_mut(&mut states, next);

            // If the next state was already visited with smaller g, skip exploring again.
            if cur_next.g <= next_g {
                return;
            };
            cur_next.g = next_g;
            if cur_next.status == Unvisited {
                cur_next.status = Explored;
            }

            let (next_h, next_hint) = h.h_with_hint(next, state.hint);
            cur_next.hint = next_hint;
            let next_f = next_g + next_h;

            queue.push(QueueElement {
                f: next_f + (max_queue_offset - queue_offset),
                data: (next, next_g),
            });
            if D {
                println!("Explore {next} from {pos} g {next_g}");
            }

            h.explore(next);
            stats.explored += 1;
            v.explore_with_h(next, Some(h));
        });
    }

    stats.diagonalmap_capacity = states.dm_capacity();
    let traceback_start = time::Instant::now();
    let path = traceback::<H>(&states, graph.target());
    stats.traceback_duration = traceback_start.elapsed().as_secs_f32();
    if let Some((_, actual_path)) = path.as_ref() {
        v.last_frame_with_h(Some(actual_path), Some(h));
    } else {
        v.last_frame_with_h(None, Some(h));
    }
    (path, stats)
}

fn parent<'a, H>(states: &HashMap<Pos, State<H::Hint>>, pos: Pos, g: Cost) -> Edge
where
    H: HeuristicInstance<'a>,
{
    for edge in [Edge::Substitution, Edge::Right, Edge::Down] {
        if let Some(p) = edge.back(&pos) {
            if let Some(state) = DiagonalMapTrait::get(states, p) {
                if state.g + edge.cost() == g {
                    return edge;
                }
            }
        }
    }
    Edge::Match
}

fn traceback<'a, H>(states: &HashMap<Pos, State<H::Hint>>, target: Pos) -> Option<(u32, Vec<Pos>)>
where
    H: HeuristicInstance<'a>,
{
    if let Some(state) = DiagonalMapTrait::get(states, target) {
        let g = state.g;
        assert_eq!(state.status, Expanded);
        let mut path = vec![target];
        let mut cost = 0;
        let mut current = target;
        // If the state is not in the map, it was found via a match.
        while current != Pos(0, 0) {
            let e = parent::<H>(states, current, g - cost);
            cost += e.cost();
            current = e.back(&current).expect("No parent found for position!");
            path.push(current);
        }
        path.reverse();
        assert_eq!(
            cost, g,
            "Traceback cost {cost} does not equal distance to end {g}!"
        );
        Some((g, path))
    } else {
        None
    }
}
