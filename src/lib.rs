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
    ops::Add,
    path::Path,
    time::{self, Duration},
};

use bio_types::sequence::Sequence;
use heuristic::*;
use util::*;

pub struct AlignResult {
    pub heuristic_name: String,
    pub heuristic_initialization: Duration,
    pub astar_duration: Duration,
    pub expanded: usize,
    pub explored: usize,
    /// Number of edges tried. More than explored states, because states can have multiple incoming edges.
    pub edges: usize,
    pub explored_states: Vec<Pos>,
}

impl AlignResult {
    fn print_header() {
        println!(
            "{:50} {:>9} {:>9} {:>9} {:>12} {:>7} {:>7}",
            "heuristic", "expanded", "explored", "edges", "precomp", "align", "h%"
        );
    }
    fn print(&self) {
        let percent_h = 100. * self.heuristic_initialization.as_secs_f64()
            / (self.heuristic_initialization.as_secs_f64() + self.astar_duration.as_secs_f64());
        println!(
            "{:50} {:>9} {:>9} {:>9} {:>12.5} {:>7.5} {:>7.3}",
            self.heuristic_name,
            self.expanded,
            self.explored,
            self.edges,
            self.heuristic_initialization.as_secs_f32(),
            self.astar_duration.as_secs_f32(),
            percent_h
        );
    }
    fn write_explored_states<P: AsRef<Path>>(&self, filename: P) {
        if !self.explored_states.is_empty() {
            let mut wtr = csv::Writer::from_path(filename).unwrap();
            wtr.write_record(&["i", "j"]).unwrap();
            for pos in &self.explored_states {
                wtr.serialize(&pos).unwrap();
            }
            wtr.flush().unwrap();
        }
    }
}

pub fn align<H: Heuristic>(
    a: &Sequence,
    b: &Sequence,
    alphabet: &Alphabet,
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
    let mut astar = || {
        let graph = alignment_graph::new_alignment_graph(&a, &b, &h);
        petgraph::algo::astar(
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
        );
    };
    let start_time = time::Instant::now();
    let _path = astar();
    let astar_duration = start_time.elapsed();
    AlignResult {
        heuristic_name: format!("{:?}", heuristic),
        heuristic_initialization,
        astar_duration,
        expanded,
        explored,
        edges,
        explored_states,
    }
}

#[cfg(test)]
mod tests {

    use rand::SeedableRng;

    use crate::random_sequence::{random_mutate, random_sequence};

    use super::*;

    #[test]
    fn test_dijkstra() {
        let pattern = b"ACTG".to_vec();
        let text = b"AACT".to_vec();
        let alphabet = &Alphabet::new(b"ACTG");

        let _result = align(&pattern, &text, &alphabet, ZeroHeuristic);
    }

    #[test]
    fn test_heuristics() {
        let n = 2000;
        let e = 200;
        let ls = 6..=6;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(31415);
        let alphabet = &Alphabet::new(b"ACTG");
        let pattern = random_sequence(n, alphabet, &mut rng);
        let text = random_mutate(&pattern, alphabet, e, &mut rng);

        AlignResult::print_header();
        align(&pattern, &text, &alphabet, ZeroHeuristic).print();
        align(&pattern, &text, &alphabet, GapHeuristic).print();
        align(&pattern, &text, &alphabet, CountHeuristic).print();
        for l in ls.clone() {
            align(
                &pattern,
                &text,
                &alphabet,
                SeedHeuristic {
                    l,
                    distance: ZeroHeuristic,
                },
            )
            .print();
        }
        for l in ls.clone() {
            align(
                &pattern,
                &text,
                &alphabet,
                SeedHeuristic {
                    l,
                    distance: GapHeuristic,
                },
            )
            .print();
        }
        for l in ls.clone() {
            align(
                &pattern,
                &text,
                &alphabet,
                SeedHeuristic {
                    l,
                    distance: CountHeuristic,
                },
            )
            .print();
        }
        // FastSeed
        //for l in ls.clone() {
        //align(&pattern, &text, &alphabet, FastSeedHeuristic { l }).print();
        //}
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

        align(&pattern, &text, &alphabet, ZeroHeuristic).write_explored_states("zero.csv");
        align(&pattern, &text, &alphabet, GapHeuristic).write_explored_states("gap.csv");
        //align(&pattern, &text, &alphabet, SeedHeuristic { l }).write_explored_states("seed.csv");
        //align(&pattern, &text, &alphabet, GappedSeedHeuristic { l })
        //.write_explored_states("gappedseed.csv");
        //align(&pattern, &text, &alphabet, FastSeedHeuristic { l })
        //.write_explored_states("zero.csv");
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
    fn bench_none(b: &mut test::Bencher) {
        let (n, e, _l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            b.iter(|| align(&pattern, &text, &alphabet, ZeroHeuristic));
        }
    }
    #[bench]
    fn bench_seeds(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            //b.iter(|| align(&pattern, &text, &alphabet, SeedHeuristic { l }));
        }
    }
    #[bench]
    fn bench_fast_seeds(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            b.iter(|| align(&pattern, &text, &alphabet, FastSeedHeuristic { l }));
        }
    }
    #[bench]
    fn bench_gap(b: &mut test::Bencher) {
        let (n, e, _l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            b.iter(|| align(&pattern, &text, &alphabet, GapHeuristic));
        }
    }
    #[bench]
    fn bench_gapped_seeds(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            //b.iter(|| align(&pattern, &text, &alphabet, GappedSeedHeuristic { l }));
        }
    }
}
