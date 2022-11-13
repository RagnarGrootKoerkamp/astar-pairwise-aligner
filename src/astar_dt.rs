use crate::{aligners::cigar::Cigar, prelude::*, visualizer::VisualizerT};
use astar::*;

#[derive(Clone, Copy, Debug)]
struct State<Hint> {
    fr: I,
    hint: Hint,
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

pub fn astar_dt<'a, H>(
    graph: &EditGraph,
    h: &mut H,
    v: &mut impl VisualizerT,
) -> (Option<(Cost, Vec<Pos>)>, AStarStats)
where
    H: HeuristicInstance<'a>,
{
    const D: bool = false;

    let mut stats = AStarStats::default();

    // f -> (DtPos(diagonal, g), fr)
    let mut queue = ShiftQueue::<(DtPos, I), H::Order>::new(if REDUCE_RETRIES {
        h.root_potential()
    } else {
        0
    });

    //let mut states = DiagonalMap::<State<H::Hint>>::new(graph.target());
    let mut states = HashMap::<DtPos, State<H::Hint>>::default();

    // Initialization with the root state.
    let mut max_f = 0;
    {
        let start = Pos(0, 0);
        let (hroot, hint) = h.h_with_hint(start, H::Hint::default());
        queue.push(QueueElement {
            f: hroot,
            data: (DtPos::from_pos(start, 0), DtPos::fr(start)),
        });
        stats.explored += 1;
        states.insert(DtPos::from_pos(start, 0), State { fr: 0, hint });
        v.new_layer_with_h(Some(h));
    }

    let mut retry_cnt = 0;

    let mut dist = None;
    'outer: while let Some(QueueElement {
        f: queue_f,
        data: (dt_pos, queue_fr),
    }) = queue.pop()
    {
        const RETRY_COUNT_EACH: i32 = 64;
        let expand_start = if retry_cnt % RETRY_COUNT_EACH == 0 {
            Some(instant::Instant::now())
        } else {
            None
        };

        let queue_g = dt_pos.g;
        let queue_f = queue_f;
        // This lookup can be unwrapped without fear of panic since the node was necessarily scored
        // before adding it to `visit_next`.
        //let g = gs[pos];
        let state = &mut states[dt_pos];
        let mut pos = dt_pos.to_pos(state.fr);

        // Skip non-furthest reaching states.
        if queue_fr < state.fr {
            continue;
        }

        assert!(queue_fr == state.fr);

        // Whenever A* pops a position, if the value of h and f is outdated, the point is pushed and not expanded.
        {
            let (current_h, new_hint) = h.h_with_hint(pos, state.hint);
            state.hint = new_hint;
            let current_f = queue_g + current_h;
            assert!(
                current_f >= queue_f,
                "Retry {pos} Current_f {current_f} smaller than queue_f {queue_f}! queue_g={} queue_h={} current_h={}", queue_g, queue_f-queue_g, current_h
            );
            if current_f > queue_f {
                stats.retries += 1;
                queue.push(QueueElement {
                    f: current_f,
                    data: (dt_pos, queue_fr),
                });
                retry_cnt += 1;
                if let Some(expand_start) = expand_start {
                    stats.retries_duration +=
                        RETRY_COUNT_EACH as f64 * expand_start.elapsed().as_secs_f64();
                }
                continue;
            }
            assert!(current_f == queue_f);
            if D && false {
                eprintln!(
                    "Expand {dt_pos} [{pos}] at \tg={queue_g} \tf={queue_f} \th={current_h}\tqueue_h={}",
                    queue_f - queue_g
                );
            }
        }

        stats.expanded += 1;

        if D {
            eprintln!("Expand {dt_pos} {queue_fr} => {pos}");
        }

        if queue_f > max_f {
            max_f = queue_f;
            v.new_layer_with_h(Some(h));
        }

        let mut prune = |pos| {
            v.expand_with_h(pos, queue_g, queue_f, Some(h));
            if !h.is_seed_start_or_end(pos) {
                return;
            }

            let (shift, pos) = h.prune(pos, state.hint);
            if REDUCE_RETRIES {
                stats.pq_shifts += queue.shift(shift, pos) as usize;
            }
        };

        while let Some(n) = graph.is_match(pos) {
            prune(pos);

            // Explore & expand `n`
            stats.explored += 1;
            stats.expanded += 1;
            stats.greedy_expanded += 1;
            if D {
                eprintln!("Greedy expand {dt_pos} {queue_fr} => {n}");
            }

            // Move to the pos state.
            pos = n;
            state.fr += 1;
        }

        // Check if prune is needed for the final pos.
        prune(pos);

        // Copy for local usage.
        let state = *state;

        // Retrace path to root and return.
        if pos == graph.target() {
            DiagonalMapTrait::insert(&mut states, dt_pos, state);
            if D {
                eprintln!("Reached target {pos} with state {state:?}");
            }
            dist = Some(queue_g);
            break 'outer;
        }

        graph.iterate_outgoing_edges(pos, |next, edge| {
            // Explore next
            let next_g = queue_g + edge.cost() as Cost;
            let dt_next = DtPos::from_pos(next, next_g);
            let cur_next = DiagonalMapTrait::get_mut(&mut states, dt_next);
            let next_fr = DtPos::fr(next);
            if D {
                eprintln!("Explore? {dt_next} at {next_fr}. currently at {cur_next:?}");
            }
            // If the next state was already visited with larger FR point, skip exploring again.
            if cur_next.fr >= next_fr {
                if D {
                    eprintln!("Skip");
                }
                return;
            };

            cur_next.fr = next_fr;

            let (next_h, next_hint) = h.h_with_hint(next, state.hint);
            cur_next.hint = next_hint;
            let next_f = next_g + next_h;

            if D {
                eprintln!("Explore {dt_next} {next_fr} => {next} at f={next_f}");
            }

            queue.push(QueueElement {
                f: next_f,
                data: (dt_next, next_fr),
            });

            h.explore(next);
            stats.explored += 1;
            v.explore_with_h(next, next_g, next_f, Some(h));
        });
    }

    let Some(dist) = dist else {  return (None, stats); };
    if D {
        eprintln!("DIST: {dist}");
    }

    stats.diagonalmap_capacity = states.dm_capacity();
    let traceback_start = instant::Instant::now();
    let path = traceback::<H>(&states, DtPos::from_pos(graph.target(), dist));
    stats.traceback_duration = traceback_start.elapsed().as_secs_f32();
    v.last_frame_with_h(
        path.as_ref()
            .map(|(_, path)| Cigar::from_path(graph.a, graph.b, path))
            .as_ref(),
        None,
        Some(h),
    );
    (path, stats)
}

fn dt_parent<'a, H>(states: &HashMap<DtPos, State<H::Hint>>, dt_pos: DtPos) -> (I, Edge)
where
    H: HeuristicInstance<'a>,
{
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

fn traceback<'a, H>(
    states: &HashMap<DtPos, State<H::Hint>>,
    target_dt: DtPos,
) -> Option<(u32, Vec<Pos>)>
where
    H: HeuristicInstance<'a>,
{
    // Traceback algorithm from Ukkonen'85.
    if let Some(state) = DiagonalMapTrait::get(states, target_dt) {
        let g = target_dt.g;
        let mut cost = 0;
        let mut cur_pos = target_dt.to_pos(state.fr);
        let mut path = vec![cur_pos];
        let mut cur_dt = target_dt;
        // If the state is not in the map, it was found via a match.
        while cur_dt != (DtPos { diagonal: 0, g: 0 }) {
            let (parent_fr, edge) = dt_parent::<H>(states, cur_dt);
            cost += edge.cost();
            let next_dt = edge
                .dt_back(&cur_dt)
                .expect("No parent found for position!");
            let next_pos = next_dt.to_pos(parent_fr);
            // Add as many matches as needed to end exactly in next_pos.
            // NOTE: We need the > here (!= won't do), since next_pos may actually be larger
            // than cur_pos, resulting in a possible infinite loop.
            while edge.back(&cur_pos).unwrap() > next_pos {
                cur_pos = Edge::Match.back(&cur_pos).unwrap();
                path.push(cur_pos);
            }
            cur_pos = edge.back(&cur_pos).unwrap();
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
        Some((g, path))
    } else {
        None
    }
}
