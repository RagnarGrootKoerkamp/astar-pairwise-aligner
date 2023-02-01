use crate::{
    alignment_graph::*,
    bucket_queue::{QueueElement, ShiftOrderT, ShiftQueue},
    prelude::*,
    stats::AstarStats,
};
use pa_heuristic::*;
use pa_vis_types::{VisualizerInstance, VisualizerT};

const D: bool = false;

#[derive(Clone, Copy, Debug)]
struct State<Hint> {
    g: Cost,
    /// NOTE: `hint` could also be passed via the priority queue.
    hint: Hint,
}

impl<Hint: Default> Default for State<Hint> {
    fn default() -> Self {
        Self {
            g: Cost::MAX,
            hint: Hint::default(),
        }
    }
}

impl<P: PosOrderT> ShiftOrderT<(Pos, Cost)> for P {
    fn from_t(t: &(Pos, Cost)) -> Self {
        P::from_pos(t.0)
    }
}

pub fn astar<'a, H: Heuristic>(
    a: Seq<'a>,
    b: Seq<'a>,
    h: &H,
    v: &impl VisualizerT,
) -> ((Cost, Cigar), AstarStats) {
    let start = instant::Instant::now();
    let ref graph = EditGraph::new(a, b, true);
    let ref mut h = h.build(a, b);
    let precomp = start.elapsed().as_secs_f32();
    let ref mut v = v.build(a, b);

    let mut stats = AstarStats::init(a, b);

    // f -> (pos, g)
    let mut queue = ShiftQueue::<(Pos, Cost), <H::Instance<'a> as HeuristicInstance>::Order>::new(
        if REDUCE_RETRIES {
            h.root_potential()
        } else {
            0
        },
    );

    let mut states =
        HashMap::<Pos, State<<H::Instance<'a> as HeuristicInstance<'a>>::Hint>>::default();

    let mut max_f = 0;
    v.new_layer(Some(h));

    // Initialization with the root state.
    {
        let start = Pos(0, 0);
        let (hroot, hint) = h.h_with_hint(start, Default::default());
        queue.push(QueueElement {
            f: hroot,
            data: (start, 0),
        });
        stats.explored += 1;
        states.insert(start, State { g: 0, hint });
    }

    let mut retry_cnt = 0;

    let _dist = loop {
        let Some(QueueElement {f: queue_f, data: (pos, queue_g),}) = queue.pop() else {
                panic!("priority queue is empty before the end is reached.");
            };

        // Time the duration of retrying once in this many iterations.
        const TIME_EACH: i32 = 64;
        let expand_start = if retry_cnt % TIME_EACH == 0 {
            Some(instant::Instant::now())
        } else {
            None
        };

        let state = states.entry(pos).or_default();

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
                    f: current_f,
                    data: (pos, queue_g),
                });
                retry_cnt += 1;
                if let Some(expand_start) = expand_start {
                    stats.timing.retries += TIME_EACH as f32 * expand_start.elapsed().as_secs_f32();
                }
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

        // Expand u
        if D {
            println!("Expand {pos} {}", state.g);
        }

        stats.expanded += 1;
        v.expand(pos, queue_g, queue_f, Some(h));

        if queue_f > max_f {
            max_f = queue_f;
            v.new_layer(Some(h));
        }

        // Copy for local usage.
        let state = *state;

        // Retrace path to root and return.
        if pos == graph.target() {
            if D {
                println!("Reached target {pos} with state {state:?}");
            }
            break state.g;
        }

        // Prune is needed
        if h.is_seed_start_or_end(pos) {
            let (shift, pos) = h.prune(pos, state.hint);
            if REDUCE_RETRIES {
                stats.pq_shifts += queue.shift(shift, pos) as usize;
            }
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
                    // stats.explored += 1;
                    // stats.expanded += 1;
                    stats.extended += 1;
                    v.explore(next, queue_g, queue_f, Some(h));
                    v.expand(next, queue_g, queue_f, Some(h));
                    if D {
                        println!("Greedy expand {next} {}", state.g);
                    }

                    // Move to the next state.
                    next = n;
                }
            }

            let cur_next = states.entry(next).or_default();

            // If the next state was already visited with smaller g, skip exploring again.
            if cur_next.g <= next_g {
                return;
            };

            // Open next
            if D {
                println!("Open {next} from {pos} g {next_g}");
            }

            cur_next.g = next_g;

            let (next_h, next_hint) = h.h_with_hint(next, state.hint);
            cur_next.hint = next_hint;
            let next_f = next_g + next_h;

            queue.push(QueueElement {
                f: next_f,
                data: (next, next_g),
            });

            h.explore(next);
            stats.explored += 1;
            v.explore(next, next_g, next_f, Some(h));
        });
    };

    stats.hashmap_capacity = states.capacity();
    let traceback_start = instant::Instant::now();
    let (d, path) = traceback(&states, graph.target());
    let cigar = Cigar::from_path(graph.a, graph.b, &path);
    stats.timing.traceback = traceback_start.elapsed().as_secs_f32();
    v.last_frame(Some(&(&cigar).into()), None, Some(h));
    stats.h = h.stats();
    assert!(
        stats.h.h0 <= d,
        "Heuristic at start is {} but the distance is only {d}!",
        stats.h.h0
    );

    let total = start.elapsed().as_secs_f32();
    stats.timing.total = total;
    stats.timing.precomp = precomp;
    stats.timing.astar = total - precomp;
    stats.distance = d;
    ((d, cigar), stats)
}

fn parent<'a, Hint: Default>(states: &HashMap<Pos, State<Hint>>, pos: Pos, g: Cost) -> Edge {
    for edge in [Edge::Substitution, Edge::Right, Edge::Down] {
        if let Some(p) = edge.back(&pos) {
            if let Some(state) = states.get(&p) {
                if state.g + edge.cost() == g {
                    return edge;
                }
            }
        }
    }
    Edge::Match
}

// TODO: Make this return Cigar instead.
fn traceback<'a, Hint: Default>(
    states: &HashMap<Pos, State<Hint>>,
    target: Pos,
) -> (Cost, Vec<Pos>) {
    let Some(state) = states.get(&target) else {
        panic!();
    };
    let g = state.g;
    let mut path = vec![target];
    let mut cost = 0;
    let mut current = target;
    // If the state is not in the map, it was found via a match.
    while current != Pos(0, 0) {
        let e = parent(states, current, g - cost);
        cost += e.cost();
        current = e.back(&current).expect("No parent found for position!");
        path.push(current);
    }
    path.reverse();
    assert_eq!(
        cost, g,
        "Traceback cost {cost} does not equal distance to end {g}!"
    );
    (g, path)
}
