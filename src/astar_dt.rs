use crate::{
    aligners::cigar::Cigar,
    prelude::*,
    stats::*,
    visualizer_trait::{Visualizer, VisualizerInstance},
};

const D: bool = false;

#[derive(Clone, Copy, Debug)]
pub struct State<Hint> {
    pub fr: I,
    pub hint: Hint,
}

impl<Hint: Default> Default for State<Hint> {
    fn default() -> Self {
        Self {
            fr: 0,
            hint: Hint::default(),
        }
    }
}

impl<P: PosOrderT> ShiftOrderT<(DtPos, I)> for P {
    fn from_t(t: &(DtPos, I)) -> Self {
        P::from_pos(t.0.to_pos(t.1))
    }
}

pub fn astar_dt<'a, H: Heuristic>(
    a: Seq<'a>,
    b: Seq<'a>,
    h: &H,
    v: &impl Visualizer,
) -> ((Cost, Cigar), AstarStats) {
    let start = instant::Instant::now();
    let ref graph = EditGraph::new(a, b, true);
    let ref mut h = h.build(a, b);
    let precomp = start.elapsed().as_secs_f32();
    let ref mut v = v.build(a, b);

    let mut stats = AstarStats::default();

    // f -> (pos, g)
    let mut queue = ShiftQueue::<(Pos, Cost), <H::Instance<'a> as HeuristicInstance>::Order>::new(
        if REDUCE_RETRIES {
            h.root_potential()
        } else {
            0
        },
    );

    //let mut states = DiagonalMap::<State<H::Hint>>::new(graph.target());
    let mut states =
        HashMap::<DtPos, State<<H::Instance<'a> as HeuristicInstance>::Hint>>::default();

    let mut max_f = 0;
    v.new_layer_with_h(Some(h));

    // Initialization with the root state.
    {
        let start = Pos(0, 0);
        let (hroot, hint) = h.h_with_hint(
            start,
            <H::Instance<'a> as HeuristicInstance>::Hint::default(),
        );
        queue.push(QueueElement {
            f: hroot,
            data: (start, 0),
        });
        stats.explored += 1;
        states.insert(DtPos::from_pos(start, 0), State { fr: 0, hint });
    }

    let mut retry_cnt = 0;

    let dist = loop {
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

        let dt_pos = DtPos::from_pos(pos, queue_g);
        let queue_g = dt_pos.g;
        let queue_f = queue_f;
        let queue_fr = DtPos::fr(pos);

        let state = &mut states[dt_pos];

        if queue_fr < state.fr {
            continue;
        }

        assert!(
            queue_fr == state.fr,
            "\nBad FR value in queue when popping {pos}. Queue: {queue_fr}, map: {}\n",
            state.fr
        );

        // Whenever A* pops a position, if the value of h and f is outdated, the point is pushed and not expanded.
        // Must be true for correctness.
        {
            let (current_h, new_hint) = h.h_with_hint(pos, state.hint);
            state.hint = new_hint;
            let current_f = queue_g + current_h;
            assert!(
                current_f >= queue_f && current_h >= queue_f - queue_g,
                "Retry {pos} Current_f {current_f} smaller than queue_f {queue_f}! state.fr={} queue_fr={} queue_h={} current_h={}", state.fr, queue_fr, queue_f-queue_g, current_h
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
            println!("Expand {queue_f}: {pos} g={queue_g}");
        }

        stats.expanded += 1;
        v.expand_with_h(pos, queue_g, queue_f, Some(h));

        if queue_f > max_f {
            max_f = queue_f;
            v.new_layer_with_h(Some(h));
        }

        // Copy for local usage.
        let state = *state;

        // Retrace path to root and return.
        if pos == graph.target() {
            if D {
                println!("Reached target {pos} with state {state:?}");
            }
            break queue_g;
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
            let next_g = queue_g + edge.cost() as Cost;
            let dt_next = DtPos::from_pos(next, next_g);
            let next_fr = DtPos::fr(next);
            let cur_next = DiagonalMapTrait::get_mut(&mut states, dt_next);

            // If there is already a farther reaching state on this diagonal, no need for greedy matching.
            if cur_next.fr >= next_fr {
                return;
            };

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
                    stats.extended += 1;
                    v.explore_with_h(next, queue_g, queue_f, Some(h));
                    v.expand_with_h(next, queue_g, queue_f, Some(h));
                    if D {
                        println!("Greedy {queue_f}: {next} g={queue_g} from {pos}");
                    }

                    // Move to the next state.
                    next = n;
                }
            }
            // Update the value after greedy extension.
            let next_fr = DtPos::fr(next);

            cur_next.fr = next_fr;

            let (next_h, next_hint) = h.h_with_hint(next, state.hint);
            cur_next.hint = next_hint;
            let next_f = next_g + next_h;

            // Open next
            if D {
                println!(
                    "Open   {next_f}: {next} g={next_g} fr={next_fr} cur_fr={} from {pos}",
                    cur_next.fr
                );
            }

            queue.push(QueueElement {
                f: next_f,
                data: (next, next_g),
            });

            h.explore(next);
            stats.explored += 1;
            v.explore_with_h(next, next_g, next_f, Some(h));
        });
    };

    stats.diagonalmap_capacity = states.dm_capacity();
    let traceback_start = instant::Instant::now();
    let (d, path) = traceback(&states, graph.target(), dist);
    let cigar = Cigar::from_path(graph.a, graph.b, &path);
    stats.timing.traceback = traceback_start.elapsed().as_secs_f32();
    v.last_frame_with_h(Some(&cigar), None, Some(h));
    stats.h = h.stats();
    assert!(stats.h.h0 <= d);

    let total = start.elapsed().as_secs_f32();
    stats.timing.total = total;
    stats.timing.precomp = precomp;
    stats.timing.astar = total - precomp;
    ((d, cigar), stats)
}

pub fn dt_parent<'a, Hint: Default>(
    states: &HashMap<DtPos, State<Hint>>,
    dt_pos: DtPos,
) -> (I, Edge) {
    let mut max_fr = (0, Edge::None);
    for edge in [Edge::Right, Edge::Down, Edge::Substitution] {
        if let Some(p) = edge.dt_back(&dt_pos) {
            if let Some(state) = DiagonalMapTrait::get(states, p) {
                if state.fr + edge.to_f() >= max_fr.0 + max_fr.1.to_f() {
                    max_fr = (state.fr, edge);
                }
            }
        }
    }
    max_fr
}

pub fn traceback<'a, Hint: Default>(
    states: &HashMap<DtPos, State<Hint>>,
    target: Pos,
    g: Cost,
) -> (Cost, Vec<Pos>) {
    let target_dt = DtPos::from_pos(target, g);
    // Traceback algorithm from Ukkonen'85.
    let mut cost = 0;
    let mut cost_from_start = g;
    let mut cur_pos = target;
    let mut path = vec![cur_pos];
    let mut cur_dt = target_dt;
    // If the state is not in the map, it was found via a match.
    while cur_dt != (DtPos { diagonal: 0, g: 0 }) {
        let (parent_fr, edge) = dt_parent(states, cur_dt);
        cost += edge.cost();
        let next_dt = edge
            .dt_back(&cur_dt)
            .expect("No parent found for position!");
        let next_pos = next_dt.to_pos(parent_fr);
        if D {
            eprintln!("Current pos {cost_from_start}\t / {cur_pos}\t at cost {cost} edge {edge:?}");
            eprintln!("Target  pos {cost_from_start}\t / {next_pos} at fr {parent_fr}");
        }
        // Add as many matches as needed to end exactly in next_pos.
        // NOTE: We need the > here (!= won't do), since next_pos may actually be larger
        // than cur_pos, resulting in a possible infinite loop.
        while edge.back(&cur_pos).unwrap() > next_pos {
            if D {
                eprintln!(
                    "Push {} @ {cost_from_start}, since {} > {next_pos}",
                    Edge::Match.back(&cur_pos).unwrap(),
                    edge.back(&cur_pos).unwrap()
                );
            }
            cur_pos = Edge::Match.back(&cur_pos).unwrap();
            path.push(cur_pos);
        }
        cur_pos = edge.back(&cur_pos).unwrap();
        cost_from_start -= edge.cost();
        if D {
            eprintln!("Push {cur_pos} @ {cost_from_start}");
        }
        path.push(cur_pos);
        cur_dt = next_dt;
    }
    while cur_pos != Pos(0, 0) {
        cur_pos = Edge::Match.back(&cur_pos).unwrap();
        path.push(cur_pos);
    }

    path.reverse();
    assert_eq!(
        cost, g,
        "Traceback cost {cost} does not equal distance to end {g}!"
    );
    assert_eq!(cost_from_start, 0);
    (g, path)
}
