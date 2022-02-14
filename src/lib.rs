#![feature(
    test,
    iter_intersperse,
    derive_default_enum,
    min_specialization,
    is_sorted,
    exclusive_range_pattern,
    associated_type_defaults,
    generic_associated_types,
    hash_drain_filter,
    drain_filter
)]

#[macro_use]
extern crate lazy_static;

pub mod astar;
pub mod bucket_queue;
pub mod contour;
//pub mod contour_graph;
pub mod algorithms;
pub mod config;
pub mod costmodel;
pub mod diagonal_map;
pub mod generate;
pub mod graph;
pub mod heuristic;
pub mod scored;
pub mod seeds;
pub mod trie;
pub mod util;

// Include one of these to switch to the faster FxHashMap hashing algorithm.
mod hash_map {
    #[allow(dead_code)]
    pub type HashMap<K, V> =
        std::collections::HashMap<K, V, std::collections::hash_map::RandomState>;
    #[allow(dead_code)]
    pub type HashSet<K> = std::collections::HashSet<K, std::collections::hash_map::RandomState>;
}
mod fx_hash_map {
    #[allow(dead_code)]
    pub use rustc_hash::FxHashMap as HashMap;
    #[allow(dead_code)]
    pub use rustc_hash::FxHashSet as HashSet;
}

// Include one of these heap implementations.
mod binary_queue_impl {
    #[allow(dead_code)]
    pub use std::collections::BinaryHeap as Heap;
}
mod bucket_queue_impl {
    #[allow(dead_code)]
    pub use crate::bucket_queue::BucketQueue as Heap;
}

pub mod prelude {
    pub use bio_types::sequence::Sequence;
    pub use std::marker::PhantomData;

    pub use super::fx_hash_map::*;

    pub use config::*;

    pub(crate) use super::bucket_queue_impl as heap;

    pub use super::*;
    pub use crate::algorithms::*;
    pub use crate::contour::*;
    pub use crate::generate::*;
    pub use crate::graph::*;
    pub use crate::heuristic::*;
    pub use crate::seeds::{LengthConfig, LengthConfig::Fixed, Match, MatchConfig};
    pub use crate::util::*;
}

use csv::Writer;
use rand::SeedableRng;
use serde::Serialize;
use std::{
    collections::HashSet,
    fmt::{self, Display},
    io::{stdout, Write},
    path::Path,
    time,
};

use prelude::*;

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
    pub path_matches: Option<usize>,
    pub explored_matches: Option<usize>,
}

#[derive(Serialize, Default, Clone)]
pub struct AlignResult {
    pub input: SequenceStats,
    pub heuristic_params: HeuristicParams,
    pub timing: TimingStats,
    pub astar: astar::AStarStats<Pos>,
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
    fn print_opt<T: Display>(o: Option<T>) -> String {
        o.map_or("".into(), |x| x.to_string())
    }
    fn print_opt_sampled(o: Option<usize>, sample_size: usize) -> String {
        o.map_or("".into(), |x| (x / sample_size).to_string())
    }
    fn print_opt_bool(o: Option<bool>) -> String {
        o.map_or("".into(), |x| (x as u8).to_string())
    }

    pub fn add_sample(&mut self, other: &AlignResult) {
        if self.sample_size == 0 {
            *self = (*other).clone();
            return;
        }

        self.input.len_a += other.input.len_a;
        self.input.len_b += other.input.len_b;
        if let Some(x) = &mut self.heuristic_stats.num_seeds {
            *x += other.heuristic_stats.num_seeds.unwrap_or_default();
        }
        if let Some(x) = &mut self.heuristic_stats.num_matches {
            *x += other.heuristic_stats.num_matches.unwrap_or_default();
        }
        if let Some(x) = &mut self.heuristic_stats.num_filtered_matches {
            *x += other
                .heuristic_stats
                .num_filtered_matches
                .unwrap_or_default();
        }
        if let Some(x) = &mut self.heuristic_stats.num_prunes {
            *x += other.heuristic_stats.num_prunes.unwrap_or_default();
        }
        self.astar.expanded += other.astar.expanded;
        self.astar.explored += other.astar.explored;
        self.astar.double_expanded += other.astar.double_expanded;
        self.astar.retries += other.astar.retries;
        self.astar.pq_shifts += other.astar.pq_shifts;
        self.astar.diagonalmap_capacity += other.astar.diagonalmap_capacity;
        self.astar.expanded_states = other.astar.expanded_states.clone();
        self.astar.explored_states = other.astar.explored_states.clone();
        self.path = other.path.clone();
        self.timing.precomputation += other.timing.precomputation;
        self.timing.astar += other.timing.astar;
        if let Some(x) = &mut self.heuristic_stats.pruning_duration {
            *x += other.heuristic_stats.pruning_duration.unwrap_or_default();
        }
        self.edit_distance += other.edit_distance;
        self.heuristic_stats2.root_h += other.heuristic_stats2.root_h;
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
                format!("{:>2}", AlignResult::print_opt(this.heuristic_params.k))
            }),
            (format!("{:>2}", "md"), |this: &AlignResult| {
                format!(
                    "{:>2}",
                    AlignResult::print_opt(this.heuristic_params.max_match_cost)
                )
            }),
            (format!("{:>2}", "pr"), |this: &AlignResult| {
                format!(
                    "{:>2}",
                    AlignResult::print_opt_bool(this.heuristic_params.pruning)
                )
            }),
            (format!("{:>2}", "bf"), |this: &AlignResult| {
                format!(
                    "{:>2}",
                    AlignResult::print_opt_bool(this.heuristic_params.build_fast)
                )
            }),
            (format!("{:<5}", "d-f"), |this: &AlignResult| {
                format!(
                    "{:<5}",
                    AlignResult::print_opt(this.heuristic_params.distance_function.as_ref())
                )
            }),
            (format!("{:>7}", "seeds"), |this: &AlignResult| {
                format!(
                    "{:>7}",
                    AlignResult::print_opt_sampled(
                        this.heuristic_stats.num_seeds.map(|x| x as usize),
                        this.sample_size
                    )
                )
            }),
            (format!("{:>7}", "matches"), |this: &AlignResult| {
                format!(
                    "{:>7}",
                    AlignResult::print_opt_sampled(
                        this.heuristic_stats.num_matches,
                        this.sample_size
                    )
                )
            }),
            (format!("{:>7}", "f-match"), |this: &AlignResult| {
                format!(
                    "{:>7}",
                    AlignResult::print_opt_sampled(
                        this.heuristic_stats.num_filtered_matches,
                        this.sample_size
                    )
                )
            }),
            (format!("{:>9}", "expanded"), |this: &AlignResult| {
                format!("{:>9}", this.astar.expanded / this.sample_size)
            }),
            (format!("{:>9}", "explored"), |this: &AlignResult| {
                format!("{:>9}", this.astar.explored / this.sample_size)
            }),
            (format!("{:>7}", "dbl"), |this: &AlignResult| {
                format!("{:>7}", this.astar.double_expanded / this.sample_size)
            }),
            (format!("{:>7}", "ret"), |this: &AlignResult| {
                format!("{:>7}", this.astar.retries / this.sample_size)
            }),
            (format!("{:>7}", "prunes"), |this: &AlignResult| {
                format!(
                    "{:>7}",
                    AlignResult::print_opt_sampled(
                        this.heuristic_stats.num_prunes,
                        this.sample_size
                    )
                )
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
                this.heuristic_stats
                    .pruning_duration
                    .map_or("".into(), |x| {
                        format!("{:>8.2}", 1000. * x / this.sample_size as f32)
                    })
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
            // (format!("{:>5}", "m_pat"), |this: &AlignResult| {
            //     format!(
            //         "{:>5}",
            //         AlignResult::print_opt(this.heuristic_stats2.path_matches)
            //     )
            // }),
            // (format!("{:>5}", "m_exp"), |this: &AlignResult| {
            //     format!(
            //         "{:>5}",
            //         AlignResult::print_opt(this.heuristic_stats2.explored_matches)
            //     )
            // }),
            (format!("{:>5}", "dm-fr"), |this: &AlignResult| {
                format!(
                    "{:>5.3}",
                    this.astar.explored as f32 / this.astar.diagonalmap_capacity as f32
                )
            }),
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
        if let Some(matches) = &self.heuristic_stats.matches {
            for Match {
                start, match_cost, ..
            } in matches
            {
                wtr.serialize((start.0, start.1, "Match", -1, match_cost))
                    .unwrap();
            }
        }
        wtr.flush().unwrap();
    }
}

fn num_matches_on_path(path: &[Pos], matches: &[Match]) -> usize {
    let matches = {
        let mut s = HashSet::<Pos>::new();
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
    H::Instance<'a>: HeuristicInstance<'a, Pos = Pos>,
{
    // Instantiate the heuristic.
    let start_time = time::Instant::now();
    let mut h = heuristic.build(a, b, alphabet);
    let heuristic_initialization = start_time.elapsed();
    let start_val = h.h(Pos(0, 0));

    // Run A* with heuristic.
    let start_time = time::Instant::now();
    // TODO: Make the greedy_matching bool a parameter in a struct with A* options.
    let graph = AlignmentGraph::new(a, b, GREEDY_EDGE_MATCHING);
    let (distance, path, astar_stats) =
        astar::astar(&graph, Pos(0, 0), Pos::from_length(a, b), &mut h).unwrap_or_default();
    let astar_duration = start_time.elapsed();

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
        h_stats
            .matches
            .as_ref()
            .map(|x| num_matches_on_path(&path, x))
    } else {
        Default::default()
    };
    let explored_matches = if DEBUG {
        h_stats
            .matches
            .as_ref()
            .map(|x| num_matches_on_path(&astar_stats.explored_states, x))
    } else {
        Default::default()
    };
    AlignResult {
        heuristic_params: heuristic.params(),
        input: sequence_stats,
        timing: TimingStats {
            precomputation: heuristic_initialization.as_secs_f32(),
            astar: astar_duration.as_secs_f32(),
        },
        astar: astar_stats,
        heuristic_stats: h_stats,
        heuristic_stats2: HeuristicStats2 {
            root_h: start_val,
            path_matches,
            explored_matches,
        },
        edit_distance: distance,
        path,
        sample_size: 1,
    }
}
