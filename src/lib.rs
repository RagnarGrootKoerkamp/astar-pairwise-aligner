#![feature(test, iter_intersperse, exclusive_range_pattern)]
pub mod alignment_graph;
pub mod heuristic;
pub mod implicit_graph;
pub mod increasing_function;
pub mod random_sequence;
pub mod seeds;
pub mod util;

extern crate test;

use std::time;

use bio::alphabets::Alphabet;
use bio_types::sequence::Sequence;
use heuristic::*;
use util::*;

/// l: seed length
/// a: first sequence, where seeds are taken from
/// b: second sequence
pub fn align<H: Heuristic>(
    a_text: &Sequence,
    b_text: &Sequence,
    heuristic: H,
) -> Option<(usize, Vec<(Pos, ())>)> {
    let start_time = time::Instant::now();

    let _precomputation = start_time.elapsed();

    let mut is_end_calls = 0;
    let mut edge_cost_calls = 0;
    let mut heuristic_calls = 0;

    let start_time = time::Instant::now();

    // Run A* with heuristic.
    let mut astar = || -> Option<(usize, Vec<(Pos, ())>)> {
        petgraph::algo::astar(
            alignment_graph::new_alignment_graph(&a_text, &b_text),
            // start
            (Pos(0, 0), ()),
            // is end?
            |(Pos(i, j), _)| {
                //make_dot(pos, '*', is_end_calls);
                //println!("POP {:?}", pos);
                is_end_calls += 1;
                i == a_text.len() && j == b_text.len()
            },
            // edge cost
            |implicit_graph::Edge((Pos(i, j), _), (Pos(x, y), _))| {
                edge_cost_calls += 1;
                // Compute the edge weight.
                // TODO: Use different weights for indels and substitutions.
                if x > i && y > j && a_text[x - 1] == b_text[y - 1] {
                    0
                } else {
                    1
                }
            },
            |(pos, _)| {
                heuristic_calls += 1;
                let h = heuristic.h(pos);
                //println!("h {:?} = {}", pos, h);
                h
            },
        )
    };
    let path = astar();
    let _algorithm = start_time.elapsed();
    /*
    println!(
        "{:14} Matches {:7} Expanded {:7} Explored {:7} Edges {:7} precomp% {:5.2} precomp {:7.2}ms a* {:7.2}ms",
        heuristic.to_string(),
        is_end_calls, heuristic_calls, edge_cost_calls,
        (precomputation.as_secs_f64() / (precomputation+algorithm).as_secs_f64()) * 100.,
        precomputation.as_secs_f32()*1000., algorithm.as_secs_f32()*1000.,
    );
    */
    //for line in dotplot {
    //println!("{}", from_utf8(&line).unwrap());
    //}
    path
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

        let path = align(&pattern, &text, ZeroHeuristic::new());
        println!("{:?}", path);
    }

    #[test]
    fn test_heuristics() {
        let n = 2000;
        let e = 200;
        let ls = 4..=10;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(31415);
        let alphabet = &Alphabet::new(b"ACTG");
        let pattern = random_sequence(n, alphabet, &mut rng);
        let text = random_mutate(&pattern, alphabet, e, &mut rng);

        // Zero
        align(&pattern, &text, ZeroHeuristic::new());
        // Gapped
        align(
            &pattern,
            &text,
            GapHeuristic::new(&pattern, &text, &alphabet),
        );
        // Seed
        for l in ls.clone() {
            println!("n={} e={} l={}", n, e, l);
            align(
                &pattern,
                &text,
                SeedHeuristic::new(&pattern, &text, &alphabet, l),
            );
        }
        // GappedSeed
        for l in ls.clone() {
            println!("n={} e={} l={}", n, e, l);
            align(
                &pattern,
                &text,
                GappedSeedHeuristic::new(&pattern, &text, &alphabet, l),
            );
        }
    }

    fn setup() -> (
        usize,
        usize,
        usize,
        usize,
        Alphabet,
        rand_chacha::ChaCha8Rng,
    ) {
        let n = 1000;
        let e = 100;
        let l = 6;
        let alphabet = Alphabet::new(b"ACTG");
        let repeats = 10;
        let rng = rand_chacha::ChaCha8Rng::seed_from_u64(31415);
        (n, e, l, repeats, alphabet, rng)
    }

    #[bench]
    fn bench_none(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            b.iter(|| align(&pattern, &text, ZeroHeuristic::new()));
        }
    }
    #[bench]
    fn bench_seeds(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            b.iter(|| {
                align(
                    &pattern,
                    &text,
                    SeedHeuristic::new(&pattern, &text, alphabet, l),
                )
            });
        }
    }
    #[bench]
    fn bench_gap(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            b.iter(|| {
                align(
                    &pattern,
                    &text,
                    GapHeuristic::new(&pattern, &text, alphabet),
                )
            });
        }
    }
    #[bench]
    fn bench_gapped_seeds(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            b.iter(|| {
                align(
                    &pattern,
                    &text,
                    GappedSeedHeuristic::new(&pattern, &text, alphabet, l),
                )
            });
        }
    }
}
