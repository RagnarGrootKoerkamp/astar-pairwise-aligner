#![feature(
    test,
    iter_intersperse,
    is_sorted,
    exclusive_range_pattern,
    associated_type_defaults,
    generic_associated_types,
    hash_drain_filter,
    drain_filter
)]

pub mod astar;
pub mod bucket_heap;
pub mod contour;
//pub mod contour_graph;
pub mod diagonal_map;
pub mod graph;
pub mod heuristic;
pub mod random_sequence;
pub mod scored;
pub mod seeds;
pub mod thresholds;
pub mod util;

extern crate test;

#[cfg(debug_assertions)]
pub const DEBUG: bool = true;

#[cfg(not(debug_assertions))]
const DEBUG: bool = false;

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
mod binary_heap_impl {
    #[allow(dead_code)]
    pub use std::collections::BinaryHeap as Heap;
}
mod bucket_heap_impl {
    #[allow(dead_code)]
    pub use crate::bucket_heap::BucketHeap as Heap;
}

pub mod prelude {
    pub use bio_types::sequence::Sequence;
    pub use std::marker::PhantomData;

    pub use super::fx_hash_map::*;

    pub(crate) use super::bucket_heap_impl as heap;

    pub use crate::contour::*;
    pub use crate::graph::*;
    pub use crate::heuristic::*;
    pub use crate::seeds::{LengthConfig, LengthConfig::Fixed, Match, MatchConfig};
    pub use crate::util::*;
    #[allow(unused_imports)]
    pub(crate) use crate::DEBUG;
}

use csv::Writer;
use rand::SeedableRng;
use serde::Serialize;
use std::{
    cell::RefCell,
    collections::HashSet,
    fmt::{self, Display},
    path::Path,
    time,
};

use crate::random_sequence::{random_mutate, random_sequence};
use prelude::*;

#[derive(Serialize, Clone, Copy, Debug)]
pub enum Source {
    Uniform,
    Manual,
}
impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Serialize)]
pub struct SequenceStats {
    pub len_a: usize,
    pub len_b: usize,
    pub error_rate: f32,
    pub source: Source,
}

#[derive(Serialize)]
pub struct TimingStats {
    pub precomputation: f32,
    pub astar: f32,
}

#[derive(Serialize)]
pub struct AStarStats {
    pub expanded: usize,
    pub explored: usize,
    pub double_expanded: usize,
    pub retries: usize,
    #[serde(skip_serializing)]
    pub explored_states: Vec<Pos>,
    #[serde(skip_serializing)]
    pub expanded_states: Vec<Pos>,
}

#[derive(Serialize)]
pub struct HeuristicStats2 {
    pub root_h: usize,
    pub path_matches: Option<usize>,
    pub explored_matches: Option<usize>,
    pub avg_h: f32,
}

#[derive(Serialize)]
pub struct AlignResult {
    pub input: SequenceStats,
    pub heuristic_params: HeuristicParams,
    pub timing: TimingStats,
    pub astar: AStarStats,
    pub heuristic_stats2: HeuristicStats2,
    pub heuristic_stats: HeuristicStats,

    // Output
    pub answer_cost: usize,
    #[serde(skip_serializing)]
    pub path: Vec<Pos>,
}

impl AlignResult {
    fn print_opt<T: Display>(o: Option<T>) -> String {
        o.map_or("".into(), |x| x.to_string())
    }
    fn print_opt_bool(o: Option<bool>) -> String {
        o.map_or("".into(), |x| (x as u8).to_string())
    }
    pub fn print(&self) {
        static mut PRINTED_HEADER: bool = false;
        let columns: &[(String, fn(&AlignResult) -> String)] = &[
            (format!("{:>6}", "|a|"), |this: &AlignResult| {
                format!("{:>6}", this.input.len_a)
            }),
            (format!("{:>6}", "|b|"), |this: &AlignResult| {
                format!("{:>6}", this.input.len_b)
            }),
            (format!("{:>4}", "r"), |this: &AlignResult| {
                format!("{:>4.2}", this.input.error_rate)
            }),
            (format!("{:<7}", "H"), |this: &AlignResult| {
                format!("{:<7}", this.heuristic_params.name)
            }),
            (format!("{:>2}", "l"), |this: &AlignResult| {
                format!("{:>2}", AlignResult::print_opt(this.heuristic_params.l))
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
            (format!("{:<5}", "dist"), |this: &AlignResult| {
                format!(
                    "{:<5}",
                    AlignResult::print_opt(this.heuristic_params.distance_function.as_ref())
                )
            }),
            (format!("{:>7}", "seeds"), |this: &AlignResult| {
                format!(
                    "{:>7}",
                    AlignResult::print_opt(this.heuristic_stats.num_seeds)
                )
            }),
            (format!("{:>7}", "matches"), |this: &AlignResult| {
                format!(
                    "{:>7}",
                    AlignResult::print_opt(this.heuristic_stats.num_matches)
                )
            }),
            (format!("{:>9}", "expanded"), |this: &AlignResult| {
                format!("{:>9}", this.astar.expanded)
            }),
            (format!("{:>9}", "explored"), |this: &AlignResult| {
                format!("{:>9}", this.astar.explored)
            }),
            (format!("{:>7}", "dbl"), |this: &AlignResult| {
                format!("{:>7}", this.astar.double_expanded)
            }),
            (format!("{:>7}", "ret"), |this: &AlignResult| {
                format!("{:>7}", this.astar.retries)
            }),
            (format!("{:>8}", "band"), |this: &AlignResult| {
                format!(
                    "{:>8.2}",
                    this.astar.expanded as f32 / max(this.input.len_a, this.input.len_b) as f32
                )
            }),
            (format!("{:>8}", "precom"), |this: &AlignResult| {
                format!("{:>8.5}", this.timing.precomputation)
            }),
            (format!("{:>8}", "align"), |this: &AlignResult| {
                format!("{:>8.5}", this.timing.astar)
            }),
            (format!("{:>8}", "prune"), |this: &AlignResult| {
                format!(
                    "{:>8.5}",
                    AlignResult::print_opt(this.heuristic_stats.pruning_duration)
                )
            }),
            (format!("{:>5}", "dist"), |this: &AlignResult| {
                format!("{:>5}", this.answer_cost)
            }),
            (format!("{:>6}", "h(0,0)"), |this: &AlignResult| {
                format!("{:>6}", this.heuristic_stats2.root_h)
            }),
            (format!("{:>5}", "m_pat"), |this: &AlignResult| {
                format!(
                    "{:>5}",
                    AlignResult::print_opt(this.heuristic_stats2.path_matches)
                )
            }),
            (format!("{:>5}", "m_exp"), |this: &AlignResult| {
                format!(
                    "{:>5}",
                    AlignResult::print_opt(this.heuristic_stats2.explored_matches)
                )
            }),
        ];

        if unsafe { !PRINTED_HEADER } {
            for (hdr, _) in columns {
                print!("{} ", hdr);
            }
            // SAFE: We're single threaded anyway.
            unsafe {
                PRINTED_HEADER = true;
            }
            println!();
        }
        for (_, col) in columns {
            print!("{} ", col(self));
        }
        println!();
    }
    pub fn write(&self, writer: &mut Writer<std::fs::File>) {
        #[derive(Serialize)]
        struct Distance {
            distance: usize,
        }
        writer
            .serialize((
                &self.input,
                &self.heuristic_params,
                &self.timing,
                &self.astar,
                &self.heuristic_stats2,
                Distance {
                    distance: self.answer_cost,
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
    let mut expanded = 0;
    let mut explored = 0;
    let mut explored_states = Vec::new();
    let mut expanded_states = Vec::new();
    let mut double_expanded = 0;
    let mut retries = 0;

    // Instantiate the heuristic.
    let start_time = time::Instant::now();
    let h = RefCell::new(heuristic.build(a, b, alphabet));
    let root_pos = Pos(0, 0);
    let root_state = Node(root_pos, h.borrow().root_state(root_pos));
    let root_val = h.borrow().h(root_state);
    //let _ = h.borrow_mut();
    let heuristic_initialization = start_time.elapsed();

    // Run A* with heuristic.
    let start_time = time::Instant::now();
    // TODO: Make the greedy_matching bool a parameter in a struct with A* options.
    let incremental_graph = IncrementalAlignmentGraph::new(a, b, &h, true);
    let mut h_values = HashMap::<usize, usize>::default();
    let target = Pos(a.len(), b.len());
    let (distance, path) = astar::astar(
        &incremental_graph,
        root_state,
        target,
        // heuristic function
        |state| {
            let v = h.borrow().h(state);
            if DEBUG {
                *h_values.entry(v).or_insert(0) += 1;
            }
            v
        },
        /*retry_outdated*/ true,
        // Expand
        |Node(pos, _)| {
            //println!("EXPAND {:?}", pos);
            //make_dot(pos, '*', is_end_calls);
            expanded += 1;
            if DEBUG {
                expanded_states.push(pos);
            }
            h.borrow_mut().prune(pos);
        },
        // Explore
        |Node(pos, _)| {
            explored += 1;
            if DEBUG {
                explored_states.push(pos);
            }
        },
        &mut double_expanded,
        &mut retries,
    )
    .unwrap_or((0, vec![]));
    let astar_duration = start_time.elapsed();

    assert!(
        root_val <= distance,
        "Distance {} H0 {}",
        distance,
        root_val
    );

    let avg_h = {
        let mut cnt = 0;
        let mut sum = 0;
        for (x, y) in h_values {
            cnt += y;
            sum += x * y;
        }
        sum as f32 / cnt as f32
    };

    let path: Vec<Pos> = if DEBUG {
        path.into_iter().collect()
    } else {
        Default::default()
    };
    let h = h.borrow();
    let h_stats = h.stats();

    let path_matches = if DEBUG {
        h_stats
            .matches
            .as_ref()
            .map(|x| num_matches_on_path(&path, &x))
    } else {
        Default::default()
    };
    let explored_matches = if DEBUG {
        h_stats
            .matches
            .as_ref()
            .map(|x| num_matches_on_path(&explored_states, x))
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
        astar: AStarStats {
            expanded,
            explored,
            retries,
            explored_states,
            expanded_states,
            double_expanded,
        },
        heuristic_stats: h_stats,
        heuristic_stats2: HeuristicStats2 {
            root_h: root_val,
            path_matches,
            explored_matches,
            avg_h,
        },
        answer_cost: distance,
        path,
    }
}

// For quick testing
pub fn setup_with_seed(
    n: usize,
    e: f32,
    seed: u64,
) -> (Sequence, Sequence, Alphabet, SequenceStats) {
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    let alphabet = Alphabet::new(b"ACTG");
    let a = random_sequence(n, &alphabet, &mut rng);
    let b = random_mutate(&a, &alphabet, (n as f32 * e) as usize, &mut rng);

    let sequence_stats = SequenceStats {
        len_a: a.len(),
        len_b: b.len(),
        error_rate: e,
        source: Source::Uniform,
    };
    (a, b, alphabet, sequence_stats)
}
pub fn setup(n: usize, e: f32) -> (Sequence, Sequence, Alphabet, SequenceStats) {
    setup_with_seed(n, e, 31415)
}

pub fn test_heuristic<H: Heuristic, HI>(n: usize, e: f32, h: H) -> AlignResult
where
    H: for<'a> Heuristic<Instance<'a> = HI>,
    HI: for<'a> HeuristicInstance<'a, Pos = Pos>,
{
    let (a, b, alphabet, stats) = setup(n, e);
    align(&a, &b, &alphabet, stats, h)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn bicount_admissible() {
        let alphabet = &Alphabet::new(b"ACTG");

        let _n = 25;
        let _e = 0.2;
        let l = 4;
        let pattern = "AGACGTCC".as_bytes().to_vec();
        let ___text = "AGACGTCCA".as_bytes().to_vec();
        let text = ___text;

        let stats = SequenceStats {
            len_a: pattern.len(),
            len_b: text.len(),
            error_rate: 0.,
            source: Source::Manual,
        };

        let r = align(
            &pattern,
            &text,
            &alphabet,
            stats,
            SeedHeuristic {
                match_config: MatchConfig {
                    length: Fixed(l),
                    max_match_cost: 1,
                    ..MatchConfig::default()
                },
                distance_function: BiCountCost,
                pruning: false,
                ..SeedHeuristic::default()
            },
        );
        assert!(r.heuristic_stats2.root_h <= r.answer_cost);
    }

    // Failed because of match distance > 0
    #[test]
    fn consistency_1() {
        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(4),
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            pruning: false,
            c: PhantomData::<NaiveContours<BruteForceContour>>,
            ..GapSeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(2000, 0.10);
        let a = &a[361..369].to_vec();
        let b = &b[363..371].to_vec();

        println!("{}\n{}\n", to_string(&a), to_string(&b));
        align(a, b, &alphabet, stats, h);
    }

    // Failed because of match distance > 0 and stricter consistency check
    #[test]
    fn consistency_2() {
        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(5),
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            pruning: false,
            c: PhantomData::<NaiveContours<BruteForceContour>>,
            ..GapSeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(2000, 0.10);
        let a = &a[236..246].to_vec();
        let b = &b[236..246].to_vec();

        println!("{}\n{}\n", to_string(&a), to_string(&b));
        align(a, b, &alphabet, stats, h);
    }

    // Failed because of pruning
    #[test]
    #[ignore]
    fn consistency_3() {
        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(4),
                max_match_cost: 0,
                ..MatchConfig::default()
            },
            pruning: true,
            c: PhantomData::<NaiveContours<BruteForceContour>>,
            ..GapSeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(2000, 0.10);
        let a = &a.to_vec();
        let b = &b.to_vec();

        println!("{}\n{}\n", to_string(&a), to_string(&b));
        align(a, b, &alphabet, stats, h);
    }

    // Failed because of pruning and match distance
    #[test]
    fn consistency_4() {
        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(6),
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            pruning: true,
            c: PhantomData::<NaiveContours<BruteForceContour>>,
            ..GapSeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(2000, 0.10);
        let a = &a[846..870].to_vec();
        let b = &b[856..880].to_vec();
        // TTGTGGGCCCTCTTAACTTCCAAC
        // TTTTTGGGCCCTTTAACTTCCAAC

        println!("{}\n{}\n", to_string(&a), to_string(&b));
        align(a, b, &alphabet, stats, h);
    }

    // Failed because of pruning and large edit distance
    #[test]
    fn consistency_5() {
        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(4),
                max_match_cost: 0,
                ..MatchConfig::default()
            },
            pruning: true,
            c: PhantomData::<NaiveContours<BruteForceContour>>,
            ..GapSeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(2000, 0.20);
        let a = &a[200..310].to_vec();
        let b = &b[203..313].to_vec();

        println!("{}\n{}\n", to_string(&a), to_string(&b));
        align(a, b, &alphabet, stats, h);
    }
}
