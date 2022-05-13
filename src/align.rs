use crate::{astar::Config, prelude::*};

use csv::Writer;
use serde::Serialize;
use std::{
    fmt,
    io::{stdout, Write},
    path::Path,
    time,
};

#[derive(Serialize, Clone, Copy, Debug, Default)]
pub enum Source {
    Uniform,
    Manual,
    #[default]
    Extern,
}
impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Serialize, Default)]
pub struct SequenceStats {
    pub len_a: usize,
    pub len_b: usize,
    pub error_rate: f32,
    pub source: Source,
}

#[derive(Serialize, Default, Clone)]
pub struct TimingStats {
    pub precomputation: f32,
    pub astar: f32,
}

#[derive(Serialize, Default, Clone)]
pub struct HeuristicStats2 {
    pub root_h: Cost,
    pub root_h_end: Cost,
    pub path_matches: usize,
    pub explored_matches: usize,
}

#[derive(Serialize, Default, Clone)]
pub struct AlignResult {
    pub input: SequenceStats,
    pub heuristic_params: HeuristicParams,
    pub timing: TimingStats,
    pub astar: astar::AStarStats,
    pub heuristic_stats2: HeuristicStats2,
    pub heuristic_stats: HeuristicStats,

    // Output
    pub edit_distance: Cost,
    #[serde(skip_serializing)]
    pub path: Vec<Pos>,

    // For averaging
    pub sample_size: usize,
}

impl AlignResult {
    pub fn add_sample(&mut self, other: &AlignResult) {
        if self.sample_size == 0 {
            *self = (*other).clone();
            return;
        }

        self.input.len_a += other.input.len_a;
        self.input.len_b += other.input.len_b;
        self.heuristic_stats.num_seeds += other.heuristic_stats.num_seeds;
        self.heuristic_stats.num_matches += other.heuristic_stats.num_matches;
        self.heuristic_stats.num_filtered_matches += other.heuristic_stats.num_filtered_matches;
        self.heuristic_stats.num_prunes += other.heuristic_stats.num_prunes;
        self.astar.expanded += other.astar.expanded;
        self.astar.explored += other.astar.explored;
        self.astar.double_expanded += other.astar.double_expanded;
        self.astar.greedy_expanded += other.astar.greedy_expanded;
        self.astar.retries += other.astar.retries;
        self.astar.pq_shifts += other.astar.pq_shifts;
        self.astar.diagonalmap_capacity += other.astar.diagonalmap_capacity;
        self.astar.expanded_states = other.astar.expanded_states.clone();
        self.astar.explored_states = other.astar.explored_states.clone();
        self.path = other.path.clone();
        self.timing.precomputation += other.timing.precomputation;
        self.timing.astar += other.timing.astar;
        self.astar.traceback_duration += other.astar.traceback_duration;
        self.heuristic_stats.pruning_duration += other.heuristic_stats.pruning_duration;
        self.edit_distance += other.edit_distance;
        self.heuristic_stats2.root_h += other.heuristic_stats2.root_h;
        self.heuristic_stats2.root_h_end += other.heuristic_stats2.root_h_end;
        self.sample_size += other.sample_size;
    }

    pub fn print(&self) {
        self.print_internal(true);
    }
    pub fn print_no_newline(&self) {
        self.print_internal(false);
    }

    pub fn values(&self) -> (Vec<String>, Vec<String>) {
        type ColumnType = (String, fn(&AlignResult) -> String);
        let columns: &[ColumnType] = &[
            (format!("{:>7}", "nr"), |this: &AlignResult| {
                format!("{:>7}", this.sample_size)
            }),
            (format!("{:>10}", "|a|"), |this: &AlignResult| {
                format!("{:>10}", this.input.len_a / this.sample_size)
            }),
            (format!("{:>10}", "|b|"), |this: &AlignResult| {
                format!("{:>10}", this.input.len_b / this.sample_size)
            }),
            (format!("{:>4}", "r"), |this: &AlignResult| {
                format!("{:>4.2}", this.input.error_rate)
            }),
            (format!("{:<7}", "H"), |this: &AlignResult| {
                format!("{:<7}", this.heuristic_params.name)
            }),
            (format!("{:>2}", "k"), |this: &AlignResult| {
                format!("{:>2}", this.heuristic_params.k)
            }),
            (format!("{:>2}", "m"), |this: &AlignResult| {
                format!("{:>2}", this.heuristic_params.max_match_cost)
            }),
            (format!("{:>2}", "pr"), |this: &AlignResult| {
                format!("{:>2}", if this.heuristic_params.pruning { 1 } else { 0 })
            }),
            (format!("{:>2}", "bf"), |this: &AlignResult| {
                format!(
                    "{:>2}",
                    if this.heuristic_params.build_fast {
                        1
                    } else {
                        0
                    }
                )
            }),
            (format!("{:<5}", "d-f"), |this: &AlignResult| {
                format!("{:<5}", this.heuristic_params.distance_function)
            }),
            (format!("{:>7}", "seeds"), |this: &AlignResult| {
                format!(
                    "{:>7}",
                    this.heuristic_stats.num_seeds as usize / this.sample_size
                )
            }),
            (format!("{:>7}", "matches"), |this: &AlignResult| {
                format!("{:>7}", this.heuristic_stats.num_matches / this.sample_size)
            }),
            (format!("{:>7}", "f-match"), |this: &AlignResult| {
                format!(
                    "{:>7}",
                    this.heuristic_stats.num_filtered_matches / this.sample_size
                )
            }),
            (format!("{:>9}", "expanded"), |this: &AlignResult| {
                format!("{:>9}", this.astar.expanded / this.sample_size)
            }),
            (format!("{:>9}", "explored"), |this: &AlignResult| {
                format!("{:>9}", this.astar.explored / this.sample_size)
            }),
            (format!("{:>9}", "greedy"), |this: &AlignResult| {
                format!("{:>9}", this.astar.greedy_expanded / this.sample_size)
            }),
            (format!("{:>7}", "dbl"), |this: &AlignResult| {
                format!("{:>7}", this.astar.double_expanded / this.sample_size)
            }),
            (format!("{:>7}", "ret"), |this: &AlignResult| {
                format!("{:>7}", this.astar.retries / this.sample_size)
            }),
            (format!("{:>7}", "prunes"), |this: &AlignResult| {
                format!("{:>7}", this.heuristic_stats.num_prunes / this.sample_size)
            }),
            (format!("{:>7}", "shift"), |this: &AlignResult| {
                format!("{:>7}", this.astar.pq_shifts / this.sample_size)
            }),
            (format!("{:>8}", "band"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    this.astar.explored as f32 / max(this.input.len_a, this.input.len_b) as f32
                )
            }),
            (format!("{:>8}", "precom"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    1000. * this.timing.precomputation / this.sample_size as f32
                )
            }),
            (format!("{:>8}", "align"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    1000. * this.timing.astar / this.sample_size as f32
                )
            }),
            (format!("{:>8}", "prune"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    1000. * this.heuristic_stats.pruning_duration / this.sample_size as f32
                )
            }),
            (format!("{:>8}", "trace"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    1000. * this.astar.traceback_duration / this.sample_size as f32
                )
            }),
            (format!("{:>7}", "ed"), |this: &AlignResult| {
                format!(
                    "{:>7.0}",
                    this.edit_distance as f32 / this.sample_size as f32
                )
            }),
            (format!("{:>6}", "h0"), |this: &AlignResult| {
                format!(
                    "{:>6.0}",
                    this.heuristic_stats2.root_h as f32 / this.sample_size as f32
                )
            }),
            (format!("{:>6}", "h0end"), |this: &AlignResult| {
                format!(
                    "{:>6.0}",
                    this.heuristic_stats2.root_h_end as f32 / this.sample_size as f32
                )
            }),
            // (format!("{:>5}", "m_pat"), |this: &AlignResult| {
            //     format!(
            //         "{:>5}",
            //         this.heuristic_stats2.path_matches
            //     )
            // }),
            // (format!("{:>5}", "m_exp"), |this: &AlignResult| {
            //     format!(
            //         "{:>5}",
            //         this.heuristic_stats2.explored_matches
            //     )
            // }),
            // (format!("{:>5}", "dm-fr"), |this: &AlignResult| {
            //     format!(
            //         "{:>5.3}",
            //         this.astar.explored as f32 / this.astar.diagonalmap_capacity as f32
            //     )
            // }),
        ];

        let mut header = Vec::new();
        let mut vals = Vec::new();
        for (hdr, f) in columns {
            header.push(hdr.clone());
            vals.push(f(self));
        }
        (header, vals)
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
    pub fn write(&self, writer: &mut Writer<std::fs::File>) {
        #[derive(Serialize)]
        struct Distance {
            distance: Cost,
        }
        writer
            .serialize((
                &self.input,
                &self.heuristic_params,
                &self.timing,
                &self.astar,
                &self.heuristic_stats2,
                Distance {
                    distance: self.edit_distance,
                },
            ))
            .unwrap();
    }
    pub fn write_explored_states<P: AsRef<Path>>(&self, filename: P) {
        if self.astar.explored_states.is_empty() {
            return;
        }
        let mut wtr = csv::Writer::from_path(filename).unwrap();
        // type: Explored, Expanded, Path, Match
        // Match does not have step set
        wtr.write_record(&["i", "j", "type", "step", "match_cost"])
            .unwrap();
        for (i, pos) in self.astar.explored_states.iter().enumerate() {
            wtr.serialize((pos.0, pos.1, "Explored", i, -1)).unwrap();
        }
        for (i, pos) in self.astar.expanded_states.iter().enumerate() {
            wtr.serialize((pos.0, pos.1, "Expanded", i, -1)).unwrap();
        }
        for pos in &self.path {
            wtr.serialize((pos.0, pos.1, "Path", -1, -1)).unwrap();
        }
        for Match {
            start, match_cost, ..
        } in &self.heuristic_stats.matches
        {
            wtr.serialize((start.0, start.1, "Match", -1, match_cost))
                .unwrap();
        }
        wtr.flush().unwrap();
    }
}

fn num_matches_on_path(path: &[Pos], matches: &[Match]) -> usize {
    let matches = {
        let mut s = HashSet::<Pos>::default();
        for &Match { start, .. } in matches {
            s.insert(start);
        }
        s
    };
    path.iter()
        .map(|p| if matches.contains(p) { 1 } else { 0 })
        .sum()
}

pub fn align<'a, H: Heuristic>(
    a: &'a Sequence,
    b: &'a Sequence,
    alphabet: &Alphabet,
    sequence_stats: SequenceStats,
    heuristic: H,
) -> AlignResult
where
    H::Instance<'a>: HeuristicInstance<'a>,
{
    align_advanced(a, b, alphabet, sequence_stats, heuristic, true)
}

pub fn align_advanced<'a, H: Heuristic>(
    a: &'a Sequence,
    b: &'a Sequence,
    alphabet: &Alphabet,
    sequence_stats: SequenceStats,
    heuristic: H,
    greedy_edge_matching: bool,
) -> AlignResult
where
    H::Instance<'a>: HeuristicInstance<'a>,
{
    // Instantiate the heuristic.
    let start_time = time::Instant::now();
    let mut h = heuristic.build(a, b, alphabet);
    let heuristic_initialization = start_time.elapsed();
    let start_val = h.h(Pos(0, 0));

    // Run A* with heuristic.
    let start_time = time::Instant::now();
    // TODO: Make the greedy_matching bool a parameter in a struct with A* options.
    let graph = EditGraph::new(a, b, greedy_edge_matching);
    let (distance_and_path, astar_stats) = astar::astar(
        &graph,
        Pos(0, 0),
        &mut h,
        &Config {
            saving: false,
            filepath: String::from("c:/rust_photos3/1/"),
            drawing: false,
            ..Default::default()
        },
    );
    let (distance, path) = distance_and_path.unwrap_or_default();
    let astar_duration = start_time.elapsed();
    let end_val = h.h(Pos(0, 0));

    assert!(
        start_val <= distance,
        "Distance {} H0 {}",
        distance,
        start_val
    );

    let path: Vec<Pos> = if DEBUG {
        path.into_iter().collect()
    } else {
        Default::default()
    };
    let h_stats = h.stats();

    let path_matches = if DEBUG {
        num_matches_on_path(&path, &h_stats.matches)
    } else {
        Default::default()
    };
    let explored_matches = if DEBUG {
        num_matches_on_path(&astar_stats.explored_states, &h_stats.matches)
    } else {
        Default::default()
    };
    AlignResult {
        heuristic_params: heuristic.params(),
        input: sequence_stats,
        timing: TimingStats {
            precomputation: heuristic_initialization.as_secs_f32(),
            astar: astar_duration.as_secs_f32() - astar_stats.traceback_duration,
        },
        astar: astar_stats,
        heuristic_stats: h_stats,
        heuristic_stats2: HeuristicStats2 {
            root_h: start_val,
            root_h_end: end_val,
            path_matches,
            explored_matches,
        },
        edit_distance: distance,
        path,
        sample_size: 1,
    }
}
