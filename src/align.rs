use crate::visualizer::NoVisualizer;
use crate::{astar::astar, astar_dt::astar_dt, prelude::*};

use std::{
    fmt,
    io::{stdout, Write},
    time,
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
pub struct SequenceStats {
    pub len_a: usize,
    pub len_b: usize,
    pub error_rate: f32,
    pub source: Source,
}

#[derive(Default, Clone)]
pub struct TimingStats {
    pub total: f32,
    pub total_sum_squares: f32,
    pub precomputation: f32,
    pub astar: f32,
}

#[derive(Default, Clone)]
pub struct HeuristicStats2 {
    pub root_h: Cost,
    pub root_h_end: Cost,
}

#[derive(Default, Clone)]
pub struct AlignResult {
    pub input: SequenceStats,
    pub heuristic_params: HeuristicParams,
    pub timing: TimingStats,
    pub astar: astar::AStarStats,
    pub heuristic_stats2: HeuristicStats2,
    pub heuristic_stats: HeuristicStats,

    // Output
    pub edit_distance: Cost,
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
        self.path = other.path.clone();
        self.timing.precomputation += other.timing.precomputation;
        self.timing.astar += other.timing.astar;
        self.timing.total += other.timing.total;
        self.timing.total_sum_squares += other.timing.total_sum_squares;
        self.astar.traceback_duration += other.astar.traceback_duration;
        self.astar.retries_duration += other.astar.retries_duration;
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
                format!(
                    "{:>7}",
                    this.heuristic_stats.num_seeds as usize / this.sample_size
                )
            }),
            (format!("{:>7}", "match/s"), |this: &AlignResult| {
                format!(
                    "{:>7.1}",
                    this.heuristic_stats.num_matches as f32 / this.heuristic_stats.num_seeds as f32
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
            // (format!("{:>7}", "dbl"), |this: &AlignResult| {
            //     format!("{:>7}", this.astar.double_expanded / this.sample_size)
            // }),
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
                    this.astar.expanded as f32 / this.input.len_a as f32
                )
            }),
            (format!("{:>8}", "t"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    1000. * this.timing.total / this.sample_size as f32
                )
            }),
            // (format!("{:>8}", "t_std"), |this: &AlignResult| {
            //     let n = this.sample_size as f32;
            //     let avg = this.timing.total / n;
            //     let sum_squares = this.timing.total_sum_squares;
            //     let stddev = (1. / n * (sum_squares - n * avg * avg)).sqrt();
            //     format!("{:>8.2}", 1000. * stddev)
            // }),
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
            (format!("{:>8}", "retries"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    1000. * this.astar.retries_duration / this.sample_size as f64
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
    alphabet: &Alphabet,
    sequence_stats: SequenceStats,
    heuristic: H,
) -> AlignResult
where
    H::Instance<'a>: HeuristicInstance<'a>,
{
    align_advanced(a, b, alphabet, sequence_stats, heuristic, true, false, None)
}

pub fn align_advanced<'a, H: Heuristic>(
    a: Seq<'a>,
    b: Seq<'a>,
    alphabet: &Alphabet,
    sequence_stats: SequenceStats,
    heuristic: H,
    greedy_edge_matching: bool,
    diagonal_transition: bool,
    save_last: Option<&String>,
) -> AlignResult
where
    H::Instance<'a>: HeuristicInstance<'a>,
{
    // Instantiate the heuristic.
    let start_time = time::Instant::now();
    let ref mut h = heuristic.build(a, b, alphabet);
    let heuristic_initialization = start_time.elapsed();
    let start_val = h.h(Pos(0, 0));

    // Run A* with heuristic.
    let astar_time = time::Instant::now();
    // TODO: Make the greedy_matching bool a parameter in a struct with A* options.
    let graph = EditGraph::new(a, b, greedy_edge_matching);
    let (distance_and_path, astar_stats) = if let Some(path) = save_last {
        #[cfg(feature = "sdl2")]
        {
            let mut config = visualizer::Config::default();
            //config.style.expanded = Gradient::TurboGradient(0.0..1.);
            config.save_last = true;
            config.filepath = path.clone();
            config.transparent_bmp = false;
            config.downscaler = 100;
            config.cell_size = 1;
            config.style.path = None;
            config.style.draw_matches = true;
            config.style.match_width = 1;
            config.style.match_shrink = 0;
            let mut vis = visualizer::Visualizer::new(config, a, b);
            if diagonal_transition {
                astar_dt(&graph, h, &mut vis)
            } else {
                astar(&graph, h, &mut vis)
            }
        }
        #[cfg(not(feature = "sdl2"))]
        {
            panic!("Feature sdl2 must be enabled for visualizations.");
        }
    } else {
        if diagonal_transition {
            astar_dt(&graph, h, &mut NoVisualizer)
        } else {
            astar(&graph, h, &mut NoVisualizer)
        }
    };
    let (distance, path) = distance_and_path.unwrap_or_default();
    let astar_duration = astar_time.elapsed();
    let total_duration = start_time.elapsed();
    let end_val = h.h(Pos(0, 0));

    assert!(
        start_val <= distance,
        "Distance {} H0 {}",
        distance,
        start_val
    );

    let h_stats = h.stats();

    AlignResult {
        heuristic_params: heuristic.params(),
        input: sequence_stats,
        timing: TimingStats {
            total: total_duration.as_secs_f32(),
            total_sum_squares: total_duration.as_secs_f32() * total_duration.as_secs_f32(),
            precomputation: heuristic_initialization.as_secs_f32(),
            astar: astar_duration.as_secs_f32() - astar_stats.traceback_duration,
        },
        astar: astar_stats,
        heuristic_stats: h_stats,
        heuristic_stats2: HeuristicStats2 {
            root_h: start_val,
            root_h_end: end_val,
        },
        edit_distance: distance,
        path,
        sample_size: 1,
    }
}
