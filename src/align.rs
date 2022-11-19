use crate::astar::{AstarStats, Timing};
use crate::visualizer::{NoVisualizer, VisualizerT};
use crate::{astar::astar, astar_dt::astar_dt, prelude::*};

use std::default;
use std::{
    fmt,
    io::{stdout, Write},
};

#[derive(Clone, Copy, Debug, Default)]
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

#[derive(Clone, Copy, Default)]
pub struct InputStats {
    pub len_a: usize,
    pub len_b: usize,
    pub error_rate: f32,
}

#[derive(Default, Clone)]
pub struct AlignResult {
    pub input: InputStats,
    pub heuristic_params: HeuristicParams,
    pub astar: astar::AstarStats,

    // Output
    pub edit_distance: Cost,

    // For averaging
    pub sample_size: usize,
}

impl AlignResult {
    pub fn new(a: Seq, b: Seq, cost: u32, total_duration: f32) -> AlignResult {
        AlignResult {
            sample_size: 1,
            input: InputStats {
                len_a: a.len(),
                len_b: b.len(),
                ..Default::default()
            },
            edit_distance: cost as Cost,
            astar: AstarStats {
                timing: Timing {
                    total: total_duration,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn add_sample(&mut self, other: &AlignResult) {
        if self.sample_size == 0 {
            *self = (*other).clone();
            return;
        }

        self.input.len_a += other.input.len_a;
        self.input.len_b += other.input.len_b;
        self.astar += other.astar;
        self.edit_distance += other.edit_distance;
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
            (format!("{:>4}", "e"), |this: &AlignResult| {
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
            // (format!("{:>2}", "pr"), |this: &AlignResult| {
            //     format!(
            //         "{:>2}",
            //         if this.heuristic_params.pruning.enabled {
            //             1
            //         } else {
            //             0
            //         }
            //     )
            // }),
            // (format!("{:<5}", "d-f"), |this: &AlignResult| {
            //     format!("{:<5}", this.heuristic_params.distance_function)
            // }),
            (format!("{:>7}", "seeds"), |this: &AlignResult| {
                format!("{:>7}", this.astar.h.num_seeds as usize / this.sample_size)
            }),
            (format!("{:>7}", "match/s"), |this: &AlignResult| {
                format!(
                    "{:>7.1}",
                    this.astar.h.num_matches as f32 / this.astar.h.num_seeds as f32
                )
            }),
            // (format!("{:>7}", "f-match"), |this: &AlignResult| {
            //     format!(
            //         "{:>7}",
            //         this.heuristic_stats.num_filtered_matches / this.sample_size
            //     )
            // }),
            (format!("{:>9}", "expanded"), |this: &AlignResult| {
                format!("{:>9}", this.astar.expanded / this.sample_size)
            }),
            (format!("{:>9}", "explored"), |this: &AlignResult| {
                format!("{:>9}", this.astar.explored / this.sample_size)
            }),
            (format!("{:>9}", "greedy"), |this: &AlignResult| {
                format!("{:>9}", this.astar.greedy_expanded / this.sample_size)
            }),
            (format!("{:>7}", "ret"), |this: &AlignResult| {
                format!("{:>7}", this.astar.retries / this.sample_size)
            }),
            (format!("{:>7}", "prunes"), |this: &AlignResult| {
                format!("{:>7}", this.astar.h.num_pruned / this.sample_size)
            }),
            (format!("{:>7}", "shift"), |this: &AlignResult| {
                format!("{:>7}", this.astar.pq_shifts / this.sample_size)
            }),
            (format!("{:>8}", "band"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    this.astar.expanded as f32 / this.input.len_a as f32
                )
            }),
            (format!("{:>8}", "t"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    1000. * this.astar.timing.total / this.sample_size as f32
                )
            }),
            (format!("{:>8}", "precom"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    1000. * this.astar.timing.precomp / this.sample_size as f32
                )
            }),
            (format!("{:>8}", "align"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    1000. * this.astar.timing.total / this.sample_size as f32
                )
            }),
            (format!("{:>8}", "prune"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    1000. * this.astar.h.pruning_duration / this.sample_size as f32
                )
            }),
            (format!("{:>8}", "retries"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    1000. * this.astar.timing.retries / this.sample_size as f32
                )
            }),
            // (format!("{:>8}", "trace"), |this: &AlignResult| {
            //     format!(
            //         "{:>8.2}",
            //         1000. * this.astar.traceback_duration / this.sample_size as f32
            //     )
            // }),
            (format!("{:>7}", "ed"), |this: &AlignResult| {
                format!(
                    "{:>7.0}",
                    this.edit_distance as f32 / this.sample_size as f32
                )
            }),
            (format!("{:>4}", "e%"), |this: &AlignResult| {
                format!(
                    "{:>4.1}",
                    this.edit_distance as f32 / this.input.len_a as f32 * 100.0
                )
            }),
            (format!("{:>6}", "h0"), |this: &AlignResult| {
                format!("{:>6.0}", this.astar.h.h0 as f32 / this.sample_size as f32)
            }),
            (format!("{:>6}", "h0end"), |this: &AlignResult| {
                format!(
                    "{:>6.0}",
                    this.astar.h.h0_end as f32 / this.sample_size as f32
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
}

pub fn align<'a, H: Heuristic>(
    a: Seq<'a>,
    b: Seq<'a>,
    sequence_stats: InputStats,
    heuristic: H,
) -> AlignResult
where
    H::Instance<'a>: HeuristicInstance<'a>,
{
    align_advanced(a, b, sequence_stats, heuristic, false, &mut NoVisualizer)
}

pub fn align_advanced<'a, H: Heuristic>(
    a: Seq<'a>,
    b: Seq<'a>,
    sequence_stats: InputStats,
    heuristic: H,
    diagonal_transition: bool,
    vis: &mut impl VisualizerT,
) -> AlignResult
where
    H::Instance<'a>: HeuristicInstance<'a>,
{
    // Instantiate the heuristic.
    let ref mut h = heuristic.build(a, b);

    // Run A* with heuristic.
    // TODO: Make the greedy_matching bool a parameter in a struct with A* options.
    let graph = EditGraph::new(a, b, true);
    let (distance_and_path, astar_stats) = if diagonal_transition {
        astar_dt(&graph, h, vis)
    } else {
        astar(&graph, h, vis)
    };
    let (distance, _) = distance_and_path;

    AlignResult {
        heuristic_params: heuristic.params(),
        input: sequence_stats,
        astar: astar_stats,
        edit_distance: distance,
        sample_size: 1,
    }
}
