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

use std::{
    collections::HashSet,
    fmt,
    path::Path,
    time::{self, Duration},
};

use bio_types::sequence::Sequence;
use heuristic::*;
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

#[derive(Clone, Copy)]
pub struct SequenceStats {
    len_a: usize,
    len_b: usize,
    error_rate: f32,
    source: Source,
}

pub struct AlignResult {
    // Input
    pub sequence_stats: SequenceStats,

    // Timing
    pub heuristic_initialization: Duration,
    pub astar_duration: Duration,

    // Stats
    pub heuristic_name: String,
    pub expanded: usize,
    pub explored: usize,
    /// Number of edges tried. More than explored states, because states can have multiple incoming edges.
    pub edges: usize,
    pub explored_states: Vec<Pos>,

    // Output
    pub distance: usize,
    pub path: Vec<Pos>,
}

impl AlignResult {
    pub fn print_header() {
        println!(
            "{:>6} {:>6} {:>5} {:>10} {:50} {:>9} {:>9} {:>9} {:>12} {:>9} {:>7} {:>5}",
            "len a",
            "len b",
            "rate",
            "model",
            "heuristic",
            "expanded",
            "explored",
            "edges",
            "precomp",
            "align",
            "h%",
            "dist"
        );
    }
    pub fn print(&self) {
        let percent_h = 100. * self.heuristic_initialization.as_secs_f64()
            / (self.heuristic_initialization.as_secs_f64() + self.astar_duration.as_secs_f64());
        println!(
            "{:>6} {:>6} {:>5.3} {:>10} {:50} {:>9} {:>9} {:>9} {:>12.5} {:>9.5} {:>7.3} {:>5}",
            self.sequence_stats.len_a,
            self.sequence_stats.len_b,
            self.sequence_stats.error_rate,
            self.sequence_stats.source.to_string(),
            self.heuristic_name,
            self.expanded,
            self.explored,
            self.edges,
            self.heuristic_initialization.as_secs_f32(),
            self.astar_duration.as_secs_f32(),
            percent_h,
            self.distance
        );
    }
    pub fn write_explored_states<P: AsRef<Path>>(&self, filename: P) {
        if self.explored_states.is_empty() {
            return;
        }
        let mut path = HashSet::new();
        for p in &self.path {
            path.insert(p);
        }
        let mut wtr = csv::Writer::from_path(filename).unwrap();
        wtr.write_record(&["i", "j", "inpath"]).unwrap();
        for pos in &self.explored_states {
            wtr.serialize((pos.0, pos.1, path.contains(pos))).unwrap();
        }
        wtr.flush().unwrap();
    }
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

    // Instantiate the heuristic.
    let start_time = time::Instant::now();
    let h = heuristic.build(a, b, alphabet);
    let heuristic_initialization = start_time.elapsed();

    // Run A* with heuristic.
    let start_time = time::Instant::now();
    let graph = alignment_graph::new_alignment_graph(&a, &b, &h);
    let (distance, path) = petgraph::algo::astar(
        &graph,
        // start
        (Pos(0, 0), h.root_state()),
        // is end?
        |(Pos(i, j), _)| {
            //make_dot(pos, '*', is_end_calls);
            //println!("POP {:?}", pos);
            expanded += 1;
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
            h.h(state)
        },
    )
    .unwrap_or((0, vec![]));
    let astar_duration = start_time.elapsed();

    let path = path.into_iter().map(|(pos, _)| pos).collect();

    AlignResult {
        sequence_stats,
        heuristic_name: format!("{:?}", heuristic),
        heuristic_initialization,
        astar_duration,
        expanded,
        explored,
        edges,
        explored_states,
        distance,
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
    fn bugfix() {
        let alphabet = &Alphabet::new(b"ACTG");

        AlignResult::print_header();
        let l = 3;
        let pattern = "ACTTGG".as_bytes().to_vec();
        let text = "ACTGG".as_bytes().to_vec();
        let stats = SequenceStats {
            len_a: pattern.len(),
            len_b: text.len(),
            error_rate: 0.,
            source: Source::Uniform,
        };

        println!(
            "{}\n{}\n",
            String::from_utf8(pattern.clone()).unwrap(),
            String::from_utf8(text.clone()).unwrap()
        );

        align(&pattern, &text, &alphabet, stats, FastSeedHeuristic { l }).print();
        align(
            &pattern,
            &text,
            &alphabet,
            stats,
            SeedHeuristic {
                l,
                distance: ZeroHeuristic,
            },
        )
        .print();
        align(&pattern, &text, &alphabet, stats, MergedSeedHeuristic { l }).print();
    }

    #[test]
    fn test_heuristics() {
        let ns = [10_000];
        let es = [0.05f32, 0.10, 0.20, 0.30];
        let ls = 6..=6;
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

            //align(&pattern, &text, &alphabet, stats, ZeroHeuristic).print();
            //align(&pattern, &text, &alphabet, stats, GapHeuristic).print();
            //align(&pattern, &text, &alphabet, stats, CountHeuristic).print();
            for l in ls.clone() {
                align(
                    &pattern,
                    &text,
                    &alphabet,
                    stats,
                    SeedHeuristic {
                        l,
                        distance: ZeroHeuristic,
                    },
                )
                .print();
            }
            for l in ls.clone() {
                align(&pattern, &text, &alphabet, stats, FastSeedHeuristic { l }).print();
            }
            // for l in ls.clone() {
            //     align(
            //         &pattern,
            //         &text,
            //         &alphabet,
            //         stats,
            //         SeedHeuristic {
            //             l,
            //             distance: GapHeuristic,
            //         },
            //     )
            //     .print();
            // }
            // for l in ls.clone() {
            //     align(
            //         &pattern,
            //         &text,
            //         &alphabet,
            //         stats,
            //         SeedHeuristic {
            //             l,
            //             distance: CountHeuristic,
            //         },
            //     )
            //     .print();
            // }
        }
    }

    #[test]
    #[ignore]
    fn print_states() {
        let n = 2000;
        let e = 200;
        let l = 6;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(31415);
        let alphabet = &Alphabet::new(b"ACTG");
        let pattern = random_sequence(n, alphabet, &mut rng);
        let text = random_mutate(&pattern, alphabet, e, &mut rng);

        let stats = SequenceStats {
            len_a: pattern.len(),
            len_b: text.len(),
            error_rate: e as f32 / n as f32,
            source: Source::Manual,
        };

        align(&pattern, &text, &alphabet, stats, ZeroHeuristic)
            .write_explored_states("evals/stats/zero.csv");
        align(&pattern, &text, &alphabet, stats, GapHeuristic)
            .write_explored_states("evals/stats/gap.csv");
        align(&pattern, &text, &alphabet, stats, CountHeuristic)
            .write_explored_states("evals/stats/count.csv");
        align(
            &pattern,
            &text,
            &alphabet,
            stats,
            SeedHeuristic {
                l,
                distance: ZeroHeuristic,
            },
        )
        .write_explored_states("evals/stats/seed.csv");
        align(&pattern, &text, &alphabet, stats, FastSeedHeuristic { l })
            .write_explored_states("evals/stats/seed_fast.csv");
        align(
            &pattern,
            &text,
            &alphabet,
            stats,
            SeedHeuristic {
                l,
                distance: GapHeuristic,
            },
        )
        .write_explored_states("evals/stats/seedgap.csv");
        align(
            &pattern,
            &text,
            &alphabet,
            stats,
            SeedHeuristic {
                l,
                distance: CountHeuristic,
            },
        )
        .write_explored_states("evals/stats/seedcnt.csv");
        //align(&pattern, &text, &alphabet, FastSeedHeuristic { l }) .write_explored_states("zero.csv");
    }

    fn setup() -> (
        usize,
        usize,
        usize,
        usize,
        Alphabet,
        rand_chacha::ChaCha8Rng,
    ) {
        let n = 100;
        let e = 10;
        let l = 6;
        let alphabet = Alphabet::new(b"ACTG");
        let repeats = 10;
        let rng = rand_chacha::ChaCha8Rng::seed_from_u64(31415);
        (n, e, l, repeats, alphabet, rng)
    }

    #[bench]
    fn bench_zero(b: &mut test::Bencher) {
        let (n, e, _l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            let stats = SequenceStats {
                len_a: pattern.len(),
                len_b: text.len(),
                error_rate: e as f32 / n as f32,
                source: Source::Uniform,
            };

            b.iter(|| align(&pattern, &text, &alphabet, stats, ZeroHeuristic));
        }
    }
    #[bench]
    fn bench_gap(b: &mut test::Bencher) {
        let (n, e, _l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            let stats = SequenceStats {
                len_a: pattern.len(),
                len_b: text.len(),
                error_rate: e as f32 / n as f32,
                source: Source::Uniform,
            };
            b.iter(|| align(&pattern, &text, &alphabet, stats, GapHeuristic));
        }
    }
    #[bench]
    fn bench_count(b: &mut test::Bencher) {
        let (n, e, _l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            let stats = SequenceStats {
                len_a: pattern.len(),
                len_b: text.len(),
                error_rate: e as f32 / n as f32,
                source: Source::Uniform,
            };
            b.iter(|| align(&pattern, &text, &alphabet, stats, CountHeuristic));
        }
    }
    #[bench]
    fn bench_seeds(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            let stats = SequenceStats {
                len_a: pattern.len(),
                len_b: text.len(),
                error_rate: e as f32 / n as f32,
                source: Source::Uniform,
            };
            b.iter(|| {
                align(
                    &pattern,
                    &text,
                    &alphabet,
                    stats,
                    SeedHeuristic {
                        l,
                        distance: ZeroHeuristic,
                    },
                )
            });
        }
    }
    #[bench]
    fn bench_seeds_gap(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            let stats = SequenceStats {
                len_a: pattern.len(),
                len_b: text.len(),
                error_rate: e as f32 / n as f32,
                source: Source::Uniform,
            };
            b.iter(|| {
                align(
                    &pattern,
                    &text,
                    &alphabet,
                    stats,
                    SeedHeuristic {
                        l,
                        distance: GapHeuristic,
                    },
                )
            });
        }
    }
    #[bench]
    fn bench_seeds_count(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            let stats = SequenceStats {
                len_a: pattern.len(),
                len_b: text.len(),
                error_rate: e as f32 / n as f32,
                source: Source::Uniform,
            };
            b.iter(|| {
                align(
                    &pattern,
                    &text,
                    &alphabet,
                    stats,
                    SeedHeuristic {
                        l,
                        distance: CountHeuristic,
                    },
                )
            });
        }
    }
    #[bench]
    fn bench_fast_seed(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        let n = 400;
        let e = 10;
        for _ in 0..1 {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            let stats = SequenceStats {
                len_a: pattern.len(),
                len_b: text.len(),
                error_rate: e as f32 / n as f32,
                source: Source::Uniform,
            };
            b.iter(|| align(&pattern, &text, &alphabet, stats, FastSeedHeuristic { l }));
        }
    }
}

// Statistics:
// - number of matches
// - number of seeds
// - greedy matching count
// - average value of heuristic
// - contribution to h from matches and distance heuristic
