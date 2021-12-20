#![feature(
    test,
    iter_intersperse,
    exclusive_range_pattern,
    associated_type_defaults,
    generic_associated_types
)]
pub mod alignment_graph;
pub mod astar;
pub mod heuristic;
pub mod implicit_graph;
pub mod increasing_function;
pub mod random_sequence;
pub mod scored;
pub mod seeds;
pub mod util;

extern crate test;

pub mod prelude {
    pub use bio_types::sequence::Sequence;

    pub use crate::alignment_graph::Node;
    pub use crate::heuristic::*;
    pub use crate::seeds::Match;
    pub use crate::util::*;
}

use csv::Writer;
use rand::SeedableRng;
use serde::Serialize;
use std::{cell::RefCell, collections::HashSet, fmt, path::Path, time};

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
    /// Number of edges tried. More than explored states, because states can have multiple incoming edges.
    pub edges: usize,
    #[serde(skip_serializing)]
    pub explored_states: Vec<Pos>,
    #[serde(skip_serializing)]
    pub expanded_states: Vec<Pos>,
}

#[derive(Serialize)]
pub struct HeuristicStats {
    pub seeds: Option<usize>,
    #[serde(skip_serializing)]
    pub matches: Option<Vec<Match>>,
    pub num_matches: Option<usize>,
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
    pub heuristic_stats: HeuristicStats,

    // Output
    pub answer_cost: usize,
    #[serde(skip_serializing)]
    pub path: Vec<Pos>,
}

impl AlignResult {
    pub fn print_header() {
        println!(
            "{:>6} {:>6} {:>5} {:>10} {:10} {:>3} {:>9} {:>7} {:10} {:>7} {:>7} {:>9} {:>9} {:>9} {:>9} {:>10} {:>9} {:>9} {:>12} {:>9} {:>7} {:>5} {:>6} {:>10} {:>10} {:>7}",
            "len a",
            "len b",
            "rate",
            "model",
            "h name", "l", "matchdist", "pruning", "dist",
            "seeds", "matches",
            "expanded",
            "explored",
            "do expa", "retried",
            "e/max(n,m)",
            "e/nm",
            "edges",
            "precomp",
            "align",
            "h%",
            "dist",
            "h(0,0)",
            "path_match",
            "expl_match",
            "avg.h"
        );
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
                &self.heuristic_stats,
                Distance {
                    distance: self.answer_cost,
                },
            ))
            .unwrap();
    }
    pub fn print(&self) {
        let percent_h =
            100. * self.timing.precomputation / (self.timing.precomputation + self.timing.astar);
        println!(
            "{:>6} {:>6} {:>5.3} {:>10} {:10} {:>3} {:>9} {:>7} {:10} {:>7} {:>7} {:>9} {:>9} {:>9} {:>9} {:>10.2} {:>9.5} {:>9} {:>12.5} {:>9.5} {:>7.3} {:>5} {:>6} {:>10} {:>10} {:>7.1}",
            self.input.len_a,
            self.input.len_b,
            self.input.error_rate,
            self.input.source.to_string(),
            self.heuristic_params.heuristic,
            self.heuristic_params.l.map_or("".into(), |x| x.to_string()),
            self.heuristic_params.max_match_cost.map_or("".into(), |x| x.to_string()),
            self.heuristic_params.pruning.map_or("".into(), |x| x.to_string()),
            self.heuristic_params.distance_function.as_ref().unwrap_or(&"".to_string()),
            self.heuristic_stats.seeds.map(|x| x.to_string()).unwrap_or_default(),
            self.heuristic_stats.num_matches.map(|x| x.to_string()).unwrap_or_default(),
            self.astar.expanded,
            self.astar.explored,
            self.astar.double_expanded,
            self.astar.retries,
            self.astar.expanded as f32 / max(self.input.len_a, self.input.len_b) as f32,
            self.astar.explored as f32 / (self.input.len_a * self.input.len_b) as f32,
            self.astar.edges,
            self.timing.precomputation,
            self.timing.astar,
            percent_h,
            self.answer_cost,
            self.heuristic_stats.root_h,
            self.heuristic_stats.path_matches.map(|x| x.to_string()).unwrap_or_default(),
            self.heuristic_stats.explored_matches.map(|x| x.to_string()).unwrap_or_default(),
            self.heuristic_stats.avg_h,
        );
    }
    pub fn write_explored_states<P: AsRef<Path>>(&self, filename: P) {
        if self.astar.explored_states.is_empty() {
            return;
        }
        let mut wtr = csv::Writer::from_path(filename).unwrap();
        // type: Explored, Expanded, Path, Match
        // Match does not have step set
        wtr.write_record(&["i", "j", "type", "step", "match_distance"])
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

fn num_matches_on_path(path: &Vec<Pos>, matches: &Vec<Match>) -> usize {
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
    a: &Sequence,
    b: &Sequence,
    alphabet: &Alphabet,
    sequence_stats: SequenceStats,
    heuristic: H,
) -> AlignResult {
    let mut expanded = 0;
    let mut explored = 0;
    let mut edges = 0;
    let mut explored_states = Vec::new();
    let mut expanded_states = Vec::new();
    let mut double_expanded = 0;
    let mut retries = 0;

    // The base graph.
    let graph = alignment_graph::new_alignment_graph(&a, &b);

    // Instantiate the heuristic.
    let start_time = time::Instant::now();
    let h = RefCell::new(heuristic.build(a, b, alphabet, &graph));
    let root_state = Node(Pos(0, 0), h.borrow().root_state());
    let root_val = h.borrow().h(root_state);
    //let _ = h.borrow_mut();
    let heuristic_initialization = start_time.elapsed();

    // Run A* with heuristic.
    let start_time = time::Instant::now();
    let incremental_graph = alignment_graph::new_incremental_alignment_graph(&graph, &h);
    let mut h_values = HashMap::<usize, usize>::new();
    let (distance, path) = astar::astar(
        &incremental_graph,
        root_state,
        // is end?
        |Node(Pos(i, j), _)| i == a.len() && j == b.len(),
        // edge cost
        |implicit_graph::Edge(_, _, cost)| {
            edges += 1;
            cost
        },
        // heuristic function
        |state| {
            let v = h.borrow().h(state);
            *h_values.entry(v).or_insert(0) += 1;
            v
        },
        // Expand
        |Node(pos, _)| {
            //make_dot(pos, '*', is_end_calls);
            expanded += 1;
            expanded_states.push(pos);
            h.borrow_mut().prune(pos);
        },
        // Explore
        |Node(pos, _)| {
            explored += 1;
            explored_states.push(pos);
        },
        false,
        &mut double_expanded,
        &mut retries,
    )
    .unwrap_or((0, vec![]));
    let astar_duration = start_time.elapsed();

    assert!(distance >= root_val);

    let avg_h = {
        let mut cnt = 0;
        let mut sum = 0;
        for (x, y) in h_values {
            cnt += y;
            sum += x * y;
        }
        sum as f32 / cnt as f32
    };

    let path = path.into_iter().map(|Node(pos, _)| pos).collect();
    let h = h.borrow();

    let path_matches = h.matches().map(|x| num_matches_on_path(&path, x));
    let explored_matches = h
        .matches()
        .map(|x| num_matches_on_path(&explored_states, x));
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
            edges,
            explored_states,
            expanded_states,
            double_expanded,
        },
        heuristic_stats: HeuristicStats {
            root_h: root_val,
            seeds: h.num_seeds(),
            matches: h.matches().cloned(),
            num_matches: h.num_matches(),
            path_matches,
            explored_matches,
            avg_h,
        },
        answer_cost: distance,
        path,
    }
}

// For quick testing
pub fn setup(n: usize, e: f32) -> (Sequence, Sequence, Alphabet, SequenceStats) {
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(31415);
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

pub fn test_heuristic<H: Heuristic>(n: usize, e: f32, h: H) -> AlignResult {
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

        AlignResult::print_header();
        let l = 3;
        let a = "ACTTGG".as_bytes().to_vec();
        let b = "ACTGG".as_bytes().to_vec();

        let graph = alignment_graph::new_alignment_graph(&a, &b);

        // Instantiate the heuristic.
        let h = SeedHeuristic {
            l,
            max_match_cost: 0,
            distance_function: GapHeuristic,
            pruning: false,
            build_fast: false,
        }
        .build(&a, &b, alphabet, &graph);

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
                l,
                max_match_cost: 1,
                distance_function: BiCountHeuristic,
                pruning: false,
                build_fast: false,
            },
        );
        assert!(r.heuristic_stats.root_h <= r.answer_cost);
    }

    // Failed because of match distance > 0
    #[test]
    fn consistency_1() {
        let h = SeedHeuristic {
            l: 4,
            max_match_cost: 1,
            distance_function: GapHeuristic,
            pruning: false,
            build_fast: false,
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
            l: 5,
            max_match_cost: 1,
            distance_function: GapHeuristic,
            pruning: false,
            build_fast: false,
        };
        let (a, b, alphabet, stats) = setup(2000, 0.10);
        let a = &a[236..246].to_vec();
        let b = &b[236..246].to_vec();

        println!("{}\n{}\n", to_string(&a), to_string(&b));
        align(a, b, &alphabet, stats, h);
    }

    // Failed because of pruning
    #[test]
    fn consistency_3() {
        let h = SeedHeuristic {
            l: 4,
            max_match_cost: 0,
            distance_function: GapHeuristic,
            pruning: true,
            build_fast: false,
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
            l: 6,
            max_match_cost: 1,
            distance_function: GapHeuristic,
            pruning: true,
            build_fast: false,
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
            l: 4,
            max_match_cost: 0,
            distance_function: GapHeuristic,
            pruning: true,
            build_fast: false,
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
//
// TODO: Code
// - fuzzing/testing that fast impls equal slow impls
// - efficient pruning: skip explored states that have outdated heuristic value (aka pruning with offset)
// - Expanded count counts identical nodes once for each pop
// - Why is pruning worse for 0.05 edit distance?
// - Pruning with offset
//   - Need to figure out when all previous vertices depend on the current match
// - Simulate efficient pruning by re-pushing explored states with outdated heuristic value
// - Investigate doing long jumps on matching diagonals.
//
// TODO: Seeds
// - Dynamic seeding, either greedy or using some DP[i, j, distance].
//   - Maximize h(0,0) or (max_match_cost+1)/l
//   - Minimize number of extra seeds.
// - choosing seeds bases on guessed alignment
//
// TODO: Fast Seed+Gap heuristic implementation:
// - Bruteforce from bottom right to top left, fully processing everything all
//   matches that are 'shadowed', i.e. only matter for going left/up, but not diagonally anymore.

// NOTE: Optimizations done:
// - Seed Heuristic
// - Count Heuristic
// - Inexact matches
// - Pruning
// - sort nodes closer to target first, among those with equal distance+h estimate
//   - this almost halves the part of the bandwidth above 1.

// NOTE: Expanded states is counted as:
