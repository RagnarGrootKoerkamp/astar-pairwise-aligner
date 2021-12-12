#![feature(
    test,
    iter_intersperse,
    exclusive_range_pattern,
    associated_type_defaults
)]
pub mod alignment_graph;
pub mod heuristic;
pub mod implicit_graph;
pub mod increasing_function;
pub mod random_sequence;
pub mod seeds;
pub mod util;

extern crate test;

use std::{cell::RefCell, collections::HashSet, fmt, path::Path, time};

use bio_types::sequence::Sequence;
use csv::Writer;
use heuristic::*;
use seeds::Match;
use serde::Serialize;
use util::*;

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
            "{:>6} {:>6} {:>5} {:>10} {:15} {:>3} {:>9} {:>7} {:15} {:>7} {:>7} {:>9} {:>9} {:>10} {:>9} {:>9} {:>12} {:>9} {:>7} {:>5} {:>6} {:>10} {:>10} {:>7}",
            "len a",
            "len b",
            "rate",
            "model",
            "h name", "l", "matchdist", "pruning", "dist",
            "seeds", "matches",
            "expanded",
            "explored",
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
            "{:>6} {:>6} {:>5.3} {:>10} {:15} {:>3} {:>9} {:>7} {:15} {:>7} {:>7} {:>9} {:>9} {:>10.2} {:>9.5} {:>9} {:>12.5} {:>9.5} {:>7.3} {:>5} {:>6} {:>10} {:>10} {:>7.1}",
            self.input.len_a,
            self.input.len_b,
            self.input.error_rate,
            self.input.source.to_string(),
            self.heuristic_params.heuristic,
            self.heuristic_params.l.map_or("".into(), |x| x.to_string()),
            self.heuristic_params.match_distance.map_or("".into(), |x| x.to_string()),
            self.heuristic_params.pruning.map_or("".into(), |x| x.to_string()),
            self.heuristic_params.distance_function.as_ref().unwrap_or(&"".to_string()),
            self.heuristic_stats.seeds.map(|x| x.to_string()).unwrap_or_default(),
            self.heuristic_stats.num_matches.map(|x| x.to_string()).unwrap_or_default(),
            self.astar.expanded,
            self.astar.explored,
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
                start,
                end: _,
                match_distance,
            } in matches
            {
                wtr.serialize((start.0, start.1, "Match", -1, match_distance))
                    .unwrap();
            }
        }
        wtr.flush().unwrap();
    }
}

fn num_matches_on_path(path: &Vec<Pos>, matches: &Vec<Match>) -> usize {
    let matches = {
        let mut s = HashSet::<Pos>::new();
        for &Match {
            start,
            end: _,
            match_distance: _,
        } in matches
        {
            s.insert(start);
        }
        s
    };
    path.iter()
        .map(|p| if matches.contains(p) { 1 } else { 0 })
        .sum()
}

pub fn align<H: Heuristic>(
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

    // Instantiate the heuristic.
    let start_time = time::Instant::now();
    let h = RefCell::new(heuristic.build(a, b, alphabet));
    let root_state = (Pos(0, 0), h.borrow().root_state());
    let root_val = h.borrow().h(root_state);
    //let _ = h.borrow_mut();
    let heuristic_initialization = start_time.elapsed();

    // Run A* with heuristic.
    let start_time = time::Instant::now();
    let graph = alignment_graph::new_alignment_graph(&a, &b, &h);
    let mut h_values = HashMap::<usize, usize>::new();
    let (distance, path) = petgraph::algo::astar(
        &graph,
        root_state,
        // is end?
        |(pos @ Pos(i, j), _)| {
            //make_dot(pos, '*', is_end_calls);
            expanded += 1;
            expanded_states.push(pos);
            h.borrow_mut().expand(pos);
            i == a.len() && j == b.len()
        },
        // edge cost
        |implicit_graph::Edge((Pos(i, j), _), (Pos(x, y), _))| {
            edges += 1;
            // Compute the edge weight.
            // TODO: Use different weights for indels and substitutions.
            if x > i && y > j && a[x - 1] == b[y - 1] {
                0
            } else {
                1
            }
        },
        |state| {
            explored += 1;
            explored_states.push(state.0);
            let v = h.borrow().h(state);
            *h_values.entry(v).or_insert(0) += 1;
            v
        },
    )
    .unwrap_or((0, vec![]));
    let astar_duration = start_time.elapsed();

    let avg_h = {
        let mut cnt = 0;
        let mut sum = 0;
        for (x, y) in h_values {
            cnt += y;
            sum += x * y;
        }
        sum as f32 / cnt as f32
    };

    let path = path.into_iter().map(|(pos, _)| pos).collect();
    let h = h.into_inner();

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
            edges,
            explored_states,
            expanded_states,
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

#[cfg(test)]
mod tests {

    use itertools::Itertools;
    use rand::SeedableRng;

    use crate::random_sequence::{random_mutate, random_sequence};

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
        let pattern = "ACTTGG".as_bytes().to_vec();
        let text = "ACTGG".as_bytes().to_vec();

        // Instantiate the heuristic.
        let h = SeedHeuristic {
            l,
            match_distance: 0,
            distance_function: GapHeuristic,
            pruning: false,
        }
        .build(&pattern, &text, alphabet);

        for j in 0..=pattern.len() {
            println!(
                "{:?}",
                (0..=text.len())
                    .map(|i| h.h((Pos(i, j), Default::default())))
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
                match_distance: 1,
                distance_function: BiCountHeuristic,
                pruning: false,
            },
        );
        assert!(r.heuristic_stats.root_h <= r.answer_cost);
    }

    #[test]
    fn test_heuristics() {
        let ns = [2_000];
        let es = [0.05, 0.10, 0.20, 0.30];
        let lm = [(4, 0), (6, 1), (7, 1)];
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(31415);
        let alphabet = &Alphabet::new(b"ACTG");

        AlignResult::print_header();
        for (&n, e) in ns.iter().cartesian_product(es) {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, (e * n as f32) as usize, &mut rng);
            let stats = SequenceStats {
                len_a: pattern.len(),
                len_b: text.len(),
                error_rate: e,
                source: Source::Uniform,
            };

            for pruning in [true] {
                for (l, match_distance) in lm {
                    align(
                        &pattern,
                        &text,
                        &alphabet,
                        stats,
                        SeedHeuristic {
                            l,
                            match_distance,
                            distance_function: CountHeuristic,
                            pruning,
                        },
                    )
                    .print();
                }
            }
            println!("");
        }
        AlignResult::print_header();
    }

    #[test]
    #[ignore]
    fn csv() {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(true)
            .from_path("evals/stats/table.csv")
            .unwrap();

        let ns = [2_000, 4_000];
        let es = [0.05, 0.10, 0.20, 0.30];
        let lm = [
            (4, 0),
            (5, 0),
            (6, 0),
            (7, 0),
            (6, 1),
            (7, 1),
            (8, 1),
            (9, 1),
        ];
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(31415);
        let alphabet = &Alphabet::new(b"ACTG");

        for (&n, e) in ns.iter().cartesian_product(es) {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, (e * n as f32) as usize, &mut rng);
            let stats = SequenceStats {
                len_a: pattern.len(),
                len_b: text.len(),
                error_rate: e,
                source: Source::Uniform,
            };

            for pruning in [false, true] {
                for (l, match_distance) in lm {
                    align(
                        &pattern,
                        &text,
                        &alphabet,
                        stats,
                        SeedHeuristic {
                            l,
                            match_distance,
                            distance_function: CountHeuristic,
                            pruning,
                        },
                    )
                    .write(&mut wtr);
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn print_states() {
        let n = 2000;
        let e = 400;
        let _l = 6;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(31415);
        let alphabet = &Alphabet::new(b"ACTG");
        let pattern = random_sequence(n, alphabet, &mut rng);
        let text = random_mutate(&pattern, alphabet, e, &mut rng);

        let stats = SequenceStats {
            len_a: pattern.len(),
            len_b: text.len(),
            error_rate: e as f32 / n as f32,
            source: Source::Uniform,
        };

        align(
            &pattern,
            &text,
            &alphabet,
            stats,
            SeedHeuristic {
                l: 4,
                match_distance: 0,
                distance_function: CountHeuristic,
                pruning: false,
            },
        )
        .write_explored_states("evals/stats/exact.csv");
        align(
            &pattern,
            &text,
            &alphabet,
            stats,
            SeedHeuristic {
                l: 6,
                match_distance: 1,
                distance_function: CountHeuristic,
                pruning: false,
            },
        )
        .write_explored_states("evals/stats/inexact.csv");
        align(
            &pattern,
            &text,
            &alphabet,
            stats,
            SeedHeuristic {
                l: 4,
                match_distance: 0,
                distance_function: CountHeuristic,
                pruning: true,
            },
        )
        .write_explored_states("evals/stats/exact_pruning.csv");
        align(
            &pattern,
            &text,
            &alphabet,
            stats,
            SeedHeuristic {
                l: 6,
                match_distance: 1,
                distance_function: CountHeuristic,
                pruning: true,
            },
        )
        .write_explored_states("evals/stats/inexact_pruning.csv");
    }
}

// TODO:
// Statistics:
// - avg total estimated distance
// - max number of consecutive matches
// - contribution to h from matches and distance heuristic
// - heuristic time
// - number of skipped matches
//
// Code:
// - fuzzing/testing that fast impls equal slow impls
// - efficient pruning: skip explored states that have outdated heuristic value
// - choosing seeds bases on guessed alignment
