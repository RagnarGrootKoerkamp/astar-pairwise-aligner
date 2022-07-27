use std::time;

use crate::prelude::*;
use astar::*;

#[derive(Clone, Copy, Debug)]
struct State<Hint> {
    fr: I,
    hint: Hint,
}

// Sort by furthest reaching.
impl QueueOrder for (DtPos, I) {
    type O = I;

    fn key(&self) -> Self::O {
        self.1
    }
}

impl<Hint: Default> Default for State<Hint> {
    fn default() -> Self {
        Self {
            fr: 0,
            hint: Hint::default(),
        }
    }
}

// h: heuristic = lower bound on cost from node to end
// g: computed cost to reach node from the start
// f: g+h
// TODO: Inline on_expand and on_explore functions by direct calls to h.
pub fn astar_dt<'a, H>(graph: &EditGraph, h: &mut H) -> (Option<(Cost, Vec<Pos>)>, AStarStats)
where
    H: HeuristicInstance<'a>,
{
    const D: bool = false;

    let mut stats = AStarStats::default();

    // f -> (DtPos(diagonal, g), fr)
    let mut queue = BucketQueue::<(DtPos, I)>::default();
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
    let mut states = HashMap::<DtPos, State<H::Hint>>::default();

    // Initialization with the root state.
    {
        let start = Pos(0, 0);
        let (hroot, hint) = h.h_with_hint(start, H::Hint::default());
        queue.push(QueueElement {
            f: hroot + (max_queue_offset - queue_offset),
            data: (DtPos::from_pos(start, 0), DtPos::fr(start)),
        });
        stats.explored += 1;
        if DEBUG {
            stats.explored_states.push(Pos(0, 0));
        }
        states.insert(DtPos::from_pos(start, 0), State { fr: 0, hint });
    }

    let mut dist = None;
    'outer: while let Some(QueueElement {
        f: queue_f,
        data: (dt_pos, queue_fr),
    }) = queue.pop()
    {
        let queue_g = dt_pos.g;
        let queue_f = queue_f + queue_offset - max_queue_offset;
        // This lookup can be unwrapped without fear of panic since the node was necessarily scored
        // before adding it to `visit_next`.
        //let g = gs[pos];
        let state = &mut states[dt_pos];
        let mut pos = dt_pos.to_pos(state.fr);

        //println!("Pop g={queue_g:3} f={queue_f:3} queue_fr={queue_fr:3} state_fr={:3} pos={pos} {dt_pos}", state.fr);

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
                    f: current_f + (max_queue_offset - queue_offset),
                    data: (dt_pos, queue_fr),
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

        stats.expanded += 1;
        if DEBUG {
            stats.expanded_states.push(pos);
        }

        if D {
            println!("Expand {pos} {}", queue_g);
        }

        // Do greedy matching within the current seed.
        // TODO: Do greedy matching on expand instead of on explore
        if graph.greedy_matching {
            while let Some(n) = graph.is_match(pos) {
                // TODO: Is pruning during greedy matching OK?
                // Never greedy expand the start of a seed.
                // Doing so may cause problems when h is not consistent and is
                // larger at the start of seed than at the position where the
                // greedy run started.
                if h.is_seed_start_or_end(pos) {
                    // Prune
                    let pq_shift = h.prune(pos, state.hint);
                    if REDUCE_RETRIES && pq_shift > 0 {
                        stats.pq_shifts += pq_shift as usize;
                        queue_offset += pq_shift;
                    }
                }

                // Explore & expand `n`
                stats.explored += 1;
                stats.expanded += 1;
                stats.greedy_expanded += 1;
                if DEBUG {
                    stats.explored_states.push(n);
                    stats.expanded_states.push(n);
                    //stats.tree.push((pos, edge));
                    //edge = Edge::GreedyMatch
                }
                if D {
                    println!("Greedy expand {n} {}", queue_g);
                }

                // Move to the pos state.
                pos = n;
                state.fr += 1;
            }
        }

        // Check if prune is needed for the final pos.
        if h.is_seed_start_or_end(pos) {
            let pq_shift = h.prune(pos, state.hint);
            if REDUCE_RETRIES && pq_shift > 0 {
                stats.pq_shifts += pq_shift as usize;
                queue_offset += pq_shift;
            }
        }

        // Copy for local usage.
        let state = *state;

        // Retrace path to root and return.
        if pos == graph.target() {
            DiagonalMapTrait::insert(&mut states, dt_pos, state);
            if D {
                println!("Reached target {pos} with state {state:?}");
            }
            dist = Some(queue_g);
            break 'outer;
        }

        graph.iterate_outgoing_edges(pos, |next, edge| {
            // Explore next
            let next_g = queue_g + edge.cost() as Cost;
            // TODO: Move this logic to some function internal to h. Not all
            // heuristics necessarily have seeds along A.

            let dt_next = DtPos::from_pos(next, next_g);
            let cur_next = DiagonalMapTrait::get_mut(&mut states, dt_next);
            // If the next state was already visited with larger FR point, skip exploring again.
            if cur_next.fr >= DtPos::fr(next) {
                return;
            };

            let next_fr = DtPos::fr(next);

            cur_next.fr = next_fr;

            let (next_h, next_hint) = h.h_with_hint(next, state.hint);
            cur_next.hint = next_hint;
            let next_f = next_g + next_h;

            //println!("Push g={next_g:3} f={next_f:3} fr={next_fr:3} pos={next} {dt_next}");
            queue.push(QueueElement {
                f: next_f + (max_queue_offset - queue_offset),
                data: (dt_next, next_fr),
            });
            if D {
                println!("Explore {next} from {pos} g {next_g}");
            }

            h.explore(next);
            stats.explored += 1;
            if DEBUG {
                stats.explored_states.push(next);
                //stats.tree.push((next, edge));
            }
        });
    }

    let Some(dist) = dist else {  return (None, stats); };

    stats.diagonalmap_capacity = states.dm_capacity();
    let traceback_start = time::Instant::now();
    let path = traceback::<H>(&states, DtPos::from_pos(graph.target(), dist));
    stats.traceback_duration = traceback_start.elapsed().as_secs_f32();
    if DEBUG {
        for &p in &stats.explored_states {
            stats.tree.push((p, pos_parent::<H>(&states, p)));
        }
    }
    (path, stats)
}

fn pos_parent<'a, H>(states: &HashMap<DtPos, State<H::Hint>>, pos: Pos) -> Edge
where
    H: HeuristicInstance<'a>,
{
    let mut g = 0;
    loop {
        if let Some(s) = DiagonalMapTrait::get(states, DtPos::from_pos(pos, g)) {
            if s.fr >= DtPos::fr(pos) {
                break;
            }
        }
        g += 1;
    }
    let dt_pos = DtPos::from_pos(pos, g);

    let mut max_fr = (0, Edge::Match);
    for edge in [Edge::Right, Edge::Down, Edge::Substitution] {
        if let Some(p) = edge.dt_back(&dt_pos) {
            if let Some(state) = DiagonalMapTrait::get(states, p) {
                if state.fr >= max_fr.0 {
                    max_fr = (state.fr, edge);
                }
            }
        }
    }
    let dt_next = max_fr
        .1
        .dt_back(&dt_pos)
        .expect("No parent found for position!");
    let next_pos = dt_next.to_pos(max_fr.0);
    if max_fr.1.back(&pos).map_or(false, |x| x != next_pos) {
        Edge::Match
    } else {
        max_fr.1
    }
}

fn dt_parent<'a, H>(states: &HashMap<DtPos, State<H::Hint>>, dt_pos: DtPos) -> (I, Edge)
where
    H: HeuristicInstance<'a>,
{
    let mut max_fr = (0, Edge::None);
    for edge in [Edge::Right, Edge::Down, Edge::Substitution] {
        if let Some(p) = edge.dt_back(&dt_pos) {
            if let Some(state) = DiagonalMapTrait::get(states, p) {
                if state.fr >= max_fr.0 {
                    max_fr = (state.fr, edge);
                }
            }
        }
    }
    max_fr
}

fn traceback<'a, H>(
    states: &HashMap<DtPos, State<H::Hint>>,
    target: DtPos,
) -> Option<(u32, Vec<Pos>)>
where
    H: HeuristicInstance<'a>,
{
    // Traceback algorithm from Ukkonen'85.
    if let Some(state) = DiagonalMapTrait::get(states, target) {
        let g = target.g;
        let mut cost = 0;
        let mut current_pos = target.to_pos(state.fr);
        let mut path = vec![current_pos];
        let mut current = target;
        // If the state is not in the map, it was found via a match.
        while current != (DtPos { diagonal: 0, g: 0 }) {
            let e = dt_parent::<H>(states, current);
            cost += e.1.cost();
            let dt_next =
                e.1.dt_back(&current)
                    .expect("No parent found for position!");
            let next_pos = dt_next.to_pos(e.0);
            // println!(
            //     "current {current} pos {current_pos} edge {e:?} dt_next {dt_next} pos {next_pos}"
            // );
            // Add matches while needed.
            while e.1.back(&current_pos).unwrap() != next_pos {
                current_pos = Edge::Match.back(&current_pos).unwrap();
                // println!(
                //     "Extra: {current_pos} with potential parent {}",
                //     e.1.back(&current_pos).unwrap()
                // );
                path.push(current_pos);
            }
            path.push(dt_next.to_pos(e.0));
            current = dt_next;
            current_pos = next_pos;
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
