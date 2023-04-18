use std::{
    fmt::Display,
    io::{stdout, Write},
};

use derive_more::AddAssign;
use pa_types::{Cost, Seq};

use pa_heuristic::HeuristicStats;

#[derive(Default, Clone, Copy, AddAssign, Debug)]
pub struct Timing {
    /// precomp + astar
    pub total: f64,
    /// building the heuristic
    pub precomp: f64,
    /// running A*
    pub astar: f64,

    pub traceback: f64,
    pub reordering: f64,
}

#[derive(Default, Clone, AddAssign, Debug)]
pub struct AstarStats {
    pub len_a: usize,
    pub len_b: usize,
    /// The computed distance.
    pub distance: Cost,
    /// states popped from PQ
    pub expanded: usize,
    /// states pushed to PQ
    pub explored: usize,
    /// states skipped through by greedy matching
    pub extended: usize,
    /// Number of times a node was popped and found to have an outdated value of h after pruning.
    pub reordered: usize,
    /// Total priority queue shift after pruning.
    pub pq_shifts: usize,
    /// Number of states allocated in the DiagonalMap
    pub hashmap_capacity: usize,

    pub h: HeuristicStats,

    pub timing: Timing,
    pub sample_size: usize,
}

impl AstarStats {
    pub fn init(a: Seq, b: Seq) -> Self {
        Self {
            len_a: a.len(),
            len_b: b.len(),
            sample_size: 1,
            ..Default::default()
        }
    }
    pub fn new(a: Seq, b: Seq, cost: Cost, total_duration: f64) -> Self {
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

    fn format_flt<T: Display>(
        &self,
        align: char,
        mut width: usize,
        title: &str,
        val: T,
    ) -> (String, String) {
        let val = val.to_string();
        // make sure to not discard things before the decimal point.
        let point = val.find('.').unwrap_or(val.len());
        width = width.max(point);
        let mut val = val.as_str();
        if val.len() > width {
            val = &val[0..width];
        }
        if align == '<' {
            (format!("{:<width$}", title), format!("{:<width$}", val))
        } else {
            (format!("{:>width$}", title), format!("{:>width$}", val))
        }
    }

    fn format_avg<T: Display + num_traits::AsPrimitive<f32>>(
        &self,
        align: char,
        width: usize,
        title: &str,
        val: T,
    ) -> (String, String) {
        self.format_flt(align, width, title, val.as_() / self.sample_size as f32)
    }

    pub fn values(&self) -> (Vec<String>, Vec<String>) {
        [
            self.format_raw('>', 7, "nr", self.sample_size),
            self.format_avg('>', 10, "|a|", self.len_a),
            self.format_avg('>', 10, "|b|", self.len_b),
            self.format_avg('>', 7, "seeds", self.h.num_seeds),
            self.format_flt(
                '>',
                7,
                "match/s",
                self.h.num_matches as f32 / self.h.num_seeds as f32,
            ),
            self.format_avg('>', 9, "expanded", self.expanded),
            self.format_avg('>', 9, "explored", self.explored),
            self.format_avg('>', 9, "extended", self.extended),
            self.format_avg('>', 9, "reorders", self.reordered),
            self.format_avg('>', 7, "pruned", self.h.num_pruned),
            self.format_avg('>', 7, "shift", self.pq_shifts),
            self.format_flt('>', 8, "band", self.expanded as f32 / self.len_a as f32),
            self.format_avg('>', 8, "t", 1000. * self.timing.total),
            self.format_avg('>', 5, "pre", 1000. * self.timing.precomp),
            self.format_avg('>', 5, "A*", 1000. * self.timing.astar),
            self.format_avg('>', 5, "h()", 1000. * self.h.h_duration),
            self.format_avg('>', 5, "prune", 1000. * self.h.prune_duration),
            self.format_avg('>', 5, "cntrs", 1000. * self.h.contours_duration),
            self.format_avg('>', 5, "reord", 1000. * self.timing.reordering),
            self.format_avg('>', 7, "n h()", self.h.h_calls),
            self.format_avg('>', 7, "n prune", self.h.prune_calls),
            self.format_avg('>', 7, "n cntrs", self.h.contours_calls),
            self.format_avg('>', 8, "trace", 1000. * self.timing.traceback),
            self.format_avg('>', 7, "ed", self.distance),
            self.format_flt(
                '>',
                4,
                "e%",
                100.0 * self.distance as f32 / self.len_a as f32,
            ),
            self.format_avg('>', 6, "h0", self.h.h0),
            self.format_avg('>', 6, "h0end", self.h.h0_end),
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
