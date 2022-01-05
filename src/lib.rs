#![feature(
    test,
    iter_intersperse,
    exclusive_range_pattern,
    associated_type_defaults,
    generic_associated_types,
    hash_drain_filter
)]

pub mod astar;
pub mod bucket_heap;
pub mod contour_graph;
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

    pub use super::fx_hash_map::*;

    pub(crate) use super::bucket_heap_impl as heap;

    pub use crate::graph::*;
    pub use crate::heuristic::*;
    pub use crate::seeds::{Match, MatchConfig};
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
        let columns: [(String, fn(&AlignResult) -> String); 24] = [
            (format!("{:>6}", "|a|"), |this: &AlignResult| {
                format!("{:>6}", this.input.len_a)
            }),
            (format!("{:>6}", "|b|"), |this: &AlignResult| {
                format!("{:>6}", this.input.len_b)
            }),
            (format!("{:>4}", "r"), |this: &AlignResult| {
                format!("{:>4.2}", this.input.error_rate)
            }),
            (format!("{:<5}", "H"), |this: &AlignResult| {
                format!("{:<5}", this.heuristic_params.name)
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
            (format!("{:>2}", "qf"), |this: &AlignResult| {
                format!(
                    "{:>2}",
                    AlignResult::print_opt_bool(
                        this.heuristic_params.query_fast.map(|x| x.enabled())
                    )
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
            (format!("{:>6}", "band"), |this: &AlignResult| {
                format!(
                    "{:>6.2}",
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
            for (hdr, _) in &columns {
                print!("{} ", hdr);
            }
            // SAFE: We're single threaded anyway.
            unsafe {
                PRINTED_HEADER = true;
            }
            println!();
        }
        for (_, col) in &columns {
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
    let incremental_graph = IncrementalAlignmentGraph::new(a, b, &h);
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
    fn test_dijkstra() {
        let pattern = b"ACTG".to_vec();
        let text = b"AACT".to_vec();
        let alphabet = &Alphabet::new(b"ACTG");

        let _result = align(
            &pattern,
            &text,
            &alphabet,
            SequenceStats {
                len_a: pattern.len(),
                len_b: text.len(),
                error_rate: 0.,
                source: Source::Manual,
            },
            ZeroHeuristic,
        );
    }

    #[test]
    fn visualize_gapped_seed() {
        let alphabet = &Alphabet::new(b"ACTG");

        let l = 3;
        let a = "ACTTGG".as_bytes().to_vec();
        let b = "ACTGG".as_bytes().to_vec();

        // Instantiate the heuristic.
        let h = SeedHeuristic {
            match_config: MatchConfig {
                l,
                max_match_cost: 0,
                ..MatchConfig::default()
            },
            distance_function: GapHeuristic,
            pruning: false,
            build_fast: false,
            query_fast: QueryMode::Off,
        }
        .build(&a, &b, alphabet);

        for j in 0..=b.len() {
            println!(
                "{:?}",
                (0..=a.len())
                    .map(|i| h.h(Node(Pos(i, j), Default::default())))
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn bicount_admissible() {
        let alphabet = &Alphabet::new(b"ACTG");

        let _n = 25;
        let _e = 0.2;
        let l = 4;
        let pattern = "AGACGTCC".as_bytes().to_vec();
        let ___text = "AGACGTCCA".as_bytes().to_vec();
        let text = ___text;

        //let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(31415);
        //let pattern = random_sequence(n, alphabet, &mut rng);
        //let text = random_mutate(&pattern, alphabet, (e * n as f32) as usize, &mut rng);

        let stats = SequenceStats {
            len_a: pattern.len(),
            len_b: text.len(),
            error_rate: 0.,
            source: Source::Manual,
        };

        println!(
            "{}\n{}\n",
            String::from_utf8(pattern.clone()).unwrap(),
            String::from_utf8(text.clone()).unwrap()
        );

        let r = align(
            &pattern,
            &text,
            &alphabet,
            stats,
            SeedHeuristic {
                match_config: MatchConfig {
                    l,
                    max_match_cost: 1,
                    ..MatchConfig::default()
                },
                distance_function: BiCountHeuristic,
                pruning: false,
                build_fast: false,
                query_fast: QueryMode::Off,
            },
        );
        assert!(r.heuristic_stats2.root_h <= r.answer_cost);
    }

    // Failed because of match distance > 0
    #[test]
    fn consistency_1() {
        let h = SeedHeuristic {
            match_config: MatchConfig {
                l: 4,
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            distance_function: GapHeuristic,
            pruning: false,
            build_fast: false,
            query_fast: QueryMode::Off,
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
        let h = SeedHeuristic {
            match_config: MatchConfig {
                l: 5,
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            distance_function: GapHeuristic,
            pruning: false,
            build_fast: false,
            query_fast: QueryMode::Off,
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
        let h = SeedHeuristic {
            match_config: MatchConfig {
                l: 4,
                max_match_cost: 0,
                ..MatchConfig::default()
            },
            distance_function: GapHeuristic,
            pruning: true,
            build_fast: false,
            query_fast: QueryMode::Off,
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
        let h = SeedHeuristic {
            match_config: MatchConfig {
                l: 6,
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            distance_function: GapHeuristic,
            pruning: true,
            build_fast: false,
            query_fast: QueryMode::Off,
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
        let h = SeedHeuristic {
            match_config: MatchConfig {
                l: 4,
                max_match_cost: 0,
                ..MatchConfig::default()
            },
            distance_function: GapHeuristic,
            pruning: true,
            build_fast: false,
            query_fast: QueryMode::Off,
        };
        let (a, b, alphabet, stats) = setup(2000, 0.20);
        let a = &a[200..310].to_vec();
        let b = &b[203..313].to_vec();

        println!("{}\n{}\n", to_string(&a), to_string(&b));
        align(a, b, &alphabet, stats, h);
    }
}

// TODO: Statistics
// - avg total estimated distance
// - max number of consecutive matches
// - contribution to h from matches and distance heuristic
// - heuristic time
// - number of skipped matches
// - pruning time
//
// TODO: Code
// - fuzzing/testing that fast impls equal slow impls
// - efficient pruning: skip explored states that have outdated heuristic value (aka pruning with offset)
// - Investigate doing long jumps on matching diagonals.
// - Rename max_match_cost to something that includes the +1 that's present everywhere.
// - Make a separate type for transformed positions
//
// TODO: MSA (delayed; pruning complications)
// - instantiate one heuristic per pair of sequences
// - run A* on the one-by-one step graph
//
// TODO: Edit Distance
// - Run SeedHeuristic with l=1 as edit distance computation algorithm.
//   - This generalizes the LCS Contours algorithm to edit distance.
//   - For l>1, it generalizes the LCS_{k[+]}  algorithm and provides a lower bound.
//
// TODO: Seeds
// - Dynamic seeding, either greedy or using some DP[i, j, distance].
//   - Maximize h(0,0) or (max_match_cost+1)/l
//   - Minimize number of extra seeds.
// - choosing seeds bases on guessed alignment
// - Fix the gap heuristic transpose to take the seeds into account.
// - Strategies for choosing seeds:
//   - A: Each seed does not match, and covers exactly max_dist+1 mutations.
//     - This way, no pruning is needed because there are no matches on the
//     diagonal, and h(0,0) exactly equals the actual distance, so that only a
//     very narrow region is expanded.
//   - B: Maximize the number of seeds that matches exactly (at most 10 times).
//   - Experiment: make one mutation every l positions, and make seeds of length l.
//
// TODO: Pruning
// - In-place bruteforce pruning for IncreasingFunction datastructure
// - Partial pruning: only prune matches where it is cheap to do so
// - Lazy pruning with offset.
// - Proof that pruning doesn't interact badly with consistency
// - Implementation for fast partial pruning:
//   - If the current match has no prev/next on the pareto front, *all* previous points must have optimal paths through this match.
//   - Removing this match decreases h for *all* previous matches
//   - Either bruteforce decrement the value at previous nodes, or keep some log-time datastructure for this.
//   - Most of the time, the match will be at the very front and there are going
//     to be very few expanded states in front, so we can do an offset and only
//     update h for those expanded states beyond this match.
// - Pruning with offset
//   - Need to figure out when all previous vertices depend on the current match
// - Remove matches from indels at the start and ends of seeds. Replace by doing a wider lookup along the diagonal.
//
// TODO: Performance
// - Use Pos(u32,u32) instead of Pos(usize,usize)
// - Use array + sorting + binary search to find optimal path.
// - Do Greedy extending of edges along diagonals
//   - NOTE: This should also expand (and prune) all in-between states.
//     Tight coupling with A* is needed to do this.
// - Replace QGramIndex by something less memory hungry
// - Skip insertions at the start/end of seeds.
// - Prune only half (some fixed %) of matches. This should result in O(matches) total pruning time.
// - Prune only matches at (or close to) the 'front': with so far maximal i and j, for not having to update the priority queue.
// - Do not generate dist-1 matches with insertions at the start and/or end.
// - Do not generate dist-1 matches with deletions at the end.
//   - Can deletions at the start also be pruned? It may screw up heuristic values right next to it. Does that matter?
//   - Definitely cannot skip deletions at both start and end.
// - Replace IncreasingFunction by a vector: value -> position, instead of the current position->value map.
//   This is sufficient, because values only increase by 1 or 2 at a time anyway, and set lookup becomes binary search.
// - ContourGraph: Add child pointer to incremental state, for faster moving diagonally.
// - Investigate gap between h(0,0) and the actual distance.
//   - For exact matches, do we want exactly 1 mutation per seed? That way h(0,0) is as large as possible, and we don't have any matches.
// - When building ContourGraphs, to get the value at the end of a match,
//   instead of walking there using incremental steps, compute and store the value
//   of the match once then end-column is processed, but insert it only when the
//   start-column is being processed.
//
//
// DONE: Fast Seed+Gap heuristic implementation:
// - Bruteforce from bottom right to top left, fully processing everything all
//   matches that are 'shadowed', i.e. only matter for going left/up, but not diagonally anymore.

// NOTE: Optimizations done:
// - Seed Heuristic
// - Count Heuristic
// - Inexact matches
// - Pruning
// - sort nodes closer to target first, among those with equal distance+h estimate
//   - this almost halves the part of the bandwidth above 1.
// - Pruning correctness: Do not prune matches that are next to a better match.
// - A* optimizations: together 4x speedup
//   - HashMap -> FxHashMap: a faster hash function for ints
//   - HashMap -> DiagonalMap: for expanded/explored states, since these are dense on the diagonal.
//   - BinaryHeap -> BucketHeap: much much faster; turns log(n) pop into O(1) push&pop
//     - For unknown reasons, sorting positions before popping them makes more expanded states, but faster code.
// - delete consistency code
// - delete incoming edges code
// - more efficient edges iteration
// - Pre-allocate DiagonalMap edges
// - Do internal iteration over outgoing edges, instead of collecting them.
// - Sort nodes in IncreasingFunction for better caching
// - incremental_h is slowly becoming more efficient (move fewer steps backwards)
// - incremental_h: Add Pos==Hint check to incremental_h
