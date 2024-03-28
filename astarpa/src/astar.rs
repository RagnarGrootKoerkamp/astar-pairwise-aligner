use crate::{
    alignment_graph::*,
    bucket_queue::{QueueElement, ShiftOrderT, ShiftQueue},
    prelude::*,
    stats::AstarStats,
};
use pa_heuristic::{util::Timer, *};
use pa_vis::{VisualizerInstance, VisualizerT};

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

/// Align two sequences using the given heuristic and visualizer.
pub fn astar<'a, H: Heuristic>(
    a: Seq<'a>,
    b: Seq<'a>,
    h: &H,
    v: &impl VisualizerT,
) -> ((Cost, Cigar), AstarStats) {
    let mut v = v.build(a, b);
    astar_with_vis(a, b, h, &mut v)
}

/// Helper function to modify the visualizer state.
pub fn astar_with_vis<'a, H: Heuristic>(
    a: Seq<'a>,
    b: Seq<'a>,
    h: &H,
    v: &mut impl VisualizerInstance,
) -> ((Cost, Cigar), AstarStats) {
    let mut stats = AstarStats::init(a, b);

    let start = instant::Instant::now();
    let ref graph = EditGraph::new(a, b, true);
    let ref mut h = h.build(a, b);
    stats.timing.precomp = start.elapsed().as_secs_f64();

    // f -> (pos, g)
    let mut queue = ShiftQueue::<(Pos, Cost), <H::Instance<'a> as HeuristicInstance>::Order>::new(
        if REDUCE_REORDERING {
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
        let (hroot, hint) = h.h_with_hint_timed(start, Default::default()).0;
        queue.push(QueueElement {
            f: hroot,
            data: (start, 0),
        });
        stats.explored += 1;
        states.insert(start, State { g: 0, hint });
    }

    // Computation of h that turned out to be retry is double counted.
    // We track them and in the end subtract it from h time.
    let mut double_timed = 0.0;
    let mut retry_cnt = 0;

    let _dist = loop {
        let reorder_timer = Timer::new(&mut retry_cnt);
        let Some(QueueElement {f: queue_f, data: (pos, queue_g),}) = queue.pop() else {
                panic!("priority queue is empty before the end is reached.");
            };

        let state = states.entry(pos).or_default();

        if queue_g > state.g {
            continue;
        }

        assert!(queue_g == state.g);

        // Whenever A* pops a position, if the value of h and f is outdated, the point is pushed and not expanded.
        // Must be true for correctness.
        {
            let ((current_h, new_hint), hint_t) = h.h_with_hint_timed(pos, state.hint);

            state.hint = new_hint;
            let current_f = state.g + current_h;
            assert!(
                    current_f >= queue_f && current_h >= queue_f - queue_g,
                    "Retry {pos} Current_f {current_f} smaller than queue_f {queue_f}! state.g={} queue_g={} queue_h={} current_h={}", state.g, queue_g, queue_f-queue_g, current_h
                );
            if current_f > queue_f {
                stats.reordered += 1;
                queue.push(QueueElement {
                    f: current_f,
                    data: (pos, queue_g),
                });
                reorder_timer.end(&mut stats.timing.reordering);
                // Remove the double counted part.
                double_timed += hint_t;
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
            if REDUCE_REORDERING {
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

            let (next_h, next_hint) = h.h_with_hint_timed(next, state.hint).0;
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
    let end = instant::Instant::now();

    stats.h = h.stats();
    stats.h.h_duration -= double_timed;
    stats.timing.total = (end - start).as_secs_f64();
    stats.timing.traceback = (end - traceback_start).as_secs_f64();
    stats.timing.astar = (traceback_start - start).as_secs_f64()
        - stats.timing.precomp
        - stats.h.h_duration
        - stats.h.prune_duration
        - stats.h.contours_duration
        - stats.timing.reordering;

    v.last_frame(Some(&(&cigar).into()), None, Some(h));
    stats.h = h.stats();
    assert!(
        stats.h.h0 <= d,
        "Heuristic at start is {} but the distance is only {d}!",
        stats.h.h0
    );
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
