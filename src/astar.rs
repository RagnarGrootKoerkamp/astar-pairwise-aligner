use std::{
    fmt::Display,
    io::{stdout, Write},
};

use derive_more::AddAssign;

use crate::{
    aligners::cigar::Cigar,
    alignment_graph::EditGraph,
    prelude::*,
    visualizer_trait::{VisualizerConfig, VisualizerT},
};

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

#[derive(Default, Clone, Copy, AddAssign)]
pub struct Timing {
    /// precomp + astar
    pub total: f32,
    /// building the heuristic
    pub precomp: f32,
    /// running A*
    pub astar: f32,

    pub traceback: f32,
    pub retries: f32,
}

#[derive(Default, Clone, AddAssign)]
pub struct AstarStats {
    pub len_a: usize,
    pub len_b: usize,
    /// The known error rate of the generated input.
    pub error_rate: f32,
    /// The computed distance.
    pub distance: Cost,
    /// states popped from PQ
    pub expanded: usize,
    /// states pushed to PQ
    pub explored: usize,
    /// states skipped through by greedy matching
    pub extended: usize,
    /// Number of times a node was popped and found to have an outdated value of h after pruning.
    pub retries: usize,
    /// Total priority queue shift after pruning.
    pub pq_shifts: usize,
    /// Number of states allocated in the DiagonalMap
    pub diagonalmap_capacity: usize,

    pub h_params: HeuristicParams,
    pub h: HeuristicStats,

    pub timing: Timing,
    pub sample_size: usize,
}

impl AstarStats {
    pub fn new(a: Seq, b: Seq, cost: Cost, total_duration: f32) -> Self {
        Self {
            len_a: a.len(),
            len_b: b.len(),
            distance: cost,
            sample_size: 1,
            timing: Timing {
                total: total_duration,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    pub fn print(&self) {
        self.print_internal(true);
    }
    pub fn print_no_newline(&self) {
        self.print_internal(false);
    }

    fn format_scl<T: Display + num_traits::AsPrimitive<f32>>(
        &self,
        align: char,
        width: usize,
        title: &str,
        val: T,
    ) -> (String, String) {
        if align == '<' {
            (
                format!("{:<width$}", title),
                format!("{:<width$}", val.as_() / self.sample_size as f32),
            )
        } else {
            (
                format!("{:>width$}", title),
                format!("{:>width$}", val.as_() / self.sample_size as f32),
            )
        }
    }

    fn format_raw<T: Display>(
        &self,
        align: char,
        width: usize,
        title: &str,
        val: T,
    ) -> (String, String) {
        if align == '<' {
            (format!("{:<width$}", title), format!("{:<width$}", val))
        } else {
            (format!("{:>width$}", title), format!("{:>width$}", val))
        }
    }

    pub fn values(&self) -> (Vec<String>, Vec<String>) {
        [
            self.format_raw('>', 7, "nr", self.sample_size),
            self.format_scl('>', 10, "|a|", self.len_a),
            self.format_scl('>', 10, "|b|", self.len_b),
            self.format_scl('>', 4, "e", self.error_rate),
            self.format_raw('<', 7, "H", self.h_params.name.clone()),
            self.format_raw('>', 2, "k", self.h_params.k),
            self.format_raw('>', 2, "m", self.h_params.max_match_cost),
            self.format_scl('>', 7, "seeds", self.h.num_seeds),
            self.format_scl('>', 7, "match/s", self.h.num_matches),
            self.format_scl('>', 9, "expanded", self.expanded),
            self.format_scl('>', 9, "explored", self.explored),
            self.format_scl('>', 9, "greedy", self.extended),
            self.format_scl('>', 7, "pruned", self.h.num_pruned),
            self.format_scl('>', 7, "shift", self.pq_shifts),
            self.format_raw('>', 8, "band", self.expanded as f32 / self.len_a as f32),
            self.format_scl('>', 8, "t", 1000. * self.timing.total),
            self.format_scl('>', 8, "precom", 1000. * self.timing.precomp),
            self.format_scl('>', 8, "prune", 1000. * self.h.pruning_duration),
            self.format_scl('>', 8, "retries", 1000. * self.timing.retries),
            self.format_scl('>', 7, "ed", self.distance),
            self.format_raw(
                '>',
                4,
                "e%",
                100.0 * self.distance as f32 / self.len_a as f32,
            ),
            self.format_scl('>', 6, "h0", self.h.h0),
            self.format_scl('>', 6, "h0end", self.h.h0_end),
        ]
        .into_iter()
        .unzip()
    }

    fn print_internal(&self, newline: bool) {
        let (header, values) = self.values();
        static mut PRINTED_HEADER: bool = false;
        if unsafe { !PRINTED_HEADER } {
            // SAFE: We're single threaded anyway.
            unsafe {
                PRINTED_HEADER = true;
            }
            println!("{}", header.join(" "));
        }
        print!("{}", values.join(" "));
        if newline {
            println!();
        } else {
            stdout().flush().unwrap();
        }
    }
}

pub fn astar<'a, H: Heuristic>(
    a: Seq<'a>,
    b: Seq<'a>,
    h: &H,
    v: &impl VisualizerConfig,
) -> ((Cost, Cigar), AstarStats) {
    let start = instant::Instant::now();
    let ref graph = EditGraph::new(a, b, true);
    let ref mut h = h.build(a, b);
    let precomp = start.elapsed().as_secs_f32();
    let ref mut v = v.build(a, b);

    let mut stats = AstarStats::default();
    stats.len_a = a.len();
    stats.len_b = b.len();
    stats.sample_size = 1;

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
        HashMap::<Pos, State<<H::Instance<'a> as HeuristicInstance<'a>>::Hint>>::default();

    let mut max_f = 0;
    v.new_layer_with_h(Some(h));

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

        let state = DiagonalMapTrait::get_mut(&mut states, pos);

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
                    v.explore_with_h(next, queue_g, queue_f, Some(h));
                    v.expand_with_h(next, queue_g, queue_f, Some(h));
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
            v.explore_with_h(next, next_g, next_f, Some(h));
        });
    };

    stats.diagonalmap_capacity = states.dm_capacity();
    let traceback_start = instant::Instant::now();
    let (d, path) = traceback(&states, graph.target());
    let cigar = Cigar::from_path(graph.a, graph.b, &path);
    stats.timing.traceback = traceback_start.elapsed().as_secs_f32();
    v.last_frame_with_h(Some(&cigar), None, Some(h));
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
            if let Some(state) = DiagonalMapTrait::get(states, p) {
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
    let Some(state) = DiagonalMapTrait::get(states, target) else {
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
