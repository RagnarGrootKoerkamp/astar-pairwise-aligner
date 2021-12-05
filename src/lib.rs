#![feature(test, iter_intersperse, exclusive_range_pattern)]
pub mod alignment_graph;
pub mod random_sequence;

extern crate test;

use alignment_graph::{Edge, Pos};
use std::{
    cmp::{max, min},
    collections::{BTreeMap, HashMap},
    fmt, time,
};

use bio::{
    alphabets::{Alphabet, RankTransform},
    data_structures::qgram_index::QGramIndex,
};
use bio_types::sequence::Sequence;

fn abs_diff(i: usize, j: usize) -> usize {
    (i as isize - j as isize).abs() as usize
}

#[derive(Debug, Clone, Copy)]
pub enum Heuristic {
    None,
    Seeds,
    Gap,
    GappedSeeds,
}
impl fmt::Display for Heuristic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// l: seed length
/// a: first sequence, where seeds are taken from
/// b: second sequence
pub fn align(
    l: usize,
    a_text: &Sequence,
    b_text: &Sequence,
    text_alphabet: &Alphabet,
    heuristic: Heuristic,
) -> Option<(usize, Vec<Pos>)> {
    // Convert to a binary sequences.
    let rank_transform = RankTransform::new(text_alphabet);
    let a = rank_transform.transform(a_text);
    let b = rank_transform.transform(b_text);

    // Split a into seeds of size l, which are encoded as `usize`.
    let seed_qgrams: Vec<(usize, usize)> = a_text
        .chunks_exact(l)
        .enumerate()
        .map(|(i, s)| (l * i, s))
        .intersperse_with({
            let mut iter = a_text[l / 2..]
                .chunks_exact(l)
                .enumerate()
                .map(|(i, s)| (l * i + l / 2, s));
            move || iter.next().unwrap()
        })
        // A chunk of size l has exactly one qgram of length l.
        .map(|(i, seed)| (i, rank_transform.qgrams(l as u32, seed).next().unwrap()))
        .collect::<Vec<_>>();

    // Find matches of the seeds of a in b.
    // NOTE: This uses O(alphabet^l) memory.
    let qgram_index = QGramIndex::new(l as u32, b_text, &text_alphabet);

    // For each seed, the positions where it matches.
    let mut histogram = BTreeMap::<usize, usize>::new();
    let mut num_matches = 0;
    let match_positions: Vec<(usize, &[usize])> = seed_qgrams
        .iter()
        .map(|&(i, seed)| {
            let matches = qgram_index.qgram_matches(seed);
            *histogram.entry(matches.len()).or_insert(0) += 1;
            num_matches += matches.len();
            (i, matches)
        })
        .collect::<Vec<_>>();

    //println!("Seeds {}\t Matches {}", seed_qgrams.len(), num_matches);
    if false {
        for (k, v) in histogram {
            println!("{}\t {}", k, v);
        }
    }

    // Create an iterator over all matches right-below the given position.
    let matches_iter = match_positions
        .iter()
        .map(|&(x, ys)| ys.iter().map(move |&y| Pos(x, y)))
        .flatten();

    // potential: the number of seeds starting at or after position i.
    let potential = |Pos(i, _)| (a.len() + l / 2) / l - min(i + l - 1, a.len()) / l;

    // Heuristics
    // 1. Always return 0. -> A* becomes Dijkstra.
    let zero_heuristic = |_: Pos| 0usize;
    // 2. Lower bound by the distance from the main diagonal.
    let gap_heuristic = |Pos(i, j)| abs_diff(a.len() - i, b.len() - j);

    // Seed heuristic: for Pos(i,j), the number of seeds we will not hit on the way to the end.
    let seed_heuristic = || {
        // Compute heuristic at matches.
        let mut max_matches = HashMap::new();
        max_matches.insert(Pos(a.len(), b.len()), 0);
        for pos @ Pos(i, j) in matches_iter.clone().rev() {
            // Value is 1 + max over matches bottom right of this one.
            // TODO: Make this faster.
            // TODO: Make sure seeds do not overlap.
            let val = max_matches
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i + l && y >= j + l)
                .map(|(_, &val)| val)
                .max()
                .unwrap();
            max_matches.insert(pos, 1 + val);
        }

        move |pos @ Pos(i, j): Pos| {
            // TODO: Find a datastructure for log-time lookup.
            let cnt = max_matches
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i && y >= j)
                .map(|(_, &val)| val)
                .max()
                .unwrap();
            potential(pos) - cnt
        }
    };

    let gapped_seed_heuristic = || {
        let skipped: &mut usize = &mut 0;
        // Precompute the values for all match positions.
        let mut h_map = HashMap::new();

        // TODO: Faster precomputation & querying.
        // 1. Do precomputation using a right-to-left front. The front is just an increasing function.
        // 2. Store which matches are at some point neighbours on the front.
        // 3. When querying and coming from a given position linked to a given match, only consider neighbours of that match for the new position.

        h_map.insert(Pos(a.len(), b.len()), 0);
        for pos @ Pos(i, j) in matches_iter.clone().rev() {
            let update_val = h_map
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i + l && y >= j + l)
                .map(|(&frompos @ Pos(x, y), &val)| {
                    val + abs_diff(x - i, y - j) + (max(potential(pos) - potential(frompos), 1) - 1)
                })
                .min()
                .unwrap();
            let query_val = h_map
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i && y >= j)
                .map(|(&frompos @ Pos(x, y), &val)| {
                    val + abs_diff(x - i, y - j) + (potential(pos) - potential(frompos))
                })
                .min()
                .unwrap();

            if update_val < query_val {
                h_map.insert(pos, update_val);
            } else {
                *skipped += 1;
            }
            //println!("{:?} => {}", pos, val);
        }
        println!("Skipped matches: {}", skipped);
        move |pos @ Pos(i, j): Pos| {
            // TODO: Find a datastructure for log-time lookup.
            h_map
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i && y >= j)
                .map(|(&frompos @ Pos(x, y), &val)| {
                    // TODO: Should there be a +- 1 here? Or take into account
                    // whether the current position/column is a match?
                    val + abs_diff(x - i, y - j) + (potential(pos) - potential(frompos))
                })
                .min()
                .unwrap()
        }
    };

    let start_time = time::Instant::now();
    let heuristic_fn: Box<dyn Fn(Pos) -> usize> = match heuristic {
        Heuristic::None => Box::new(zero_heuristic),
        Heuristic::Gap => Box::new(gap_heuristic),
        Heuristic::Seeds => Box::new(seed_heuristic()),
        Heuristic::GappedSeeds => Box::new(gapped_seed_heuristic()),
    };

    let precomputation = start_time.elapsed();

    let mut is_end_calls = 0;
    let mut edge_cost_calls = 0;
    let mut heuristic_calls = 0;

    const A_SIZE: usize = 50;
    const B_SIZE: usize = 50;
    let mut dotplot: [[u8; A_SIZE]; B_SIZE] = [[' ' as u8; A_SIZE]; B_SIZE];
    let a_factor = a.len() / A_SIZE + 1;
    let b_factor = b.len() / B_SIZE + 1;

    let mut make_dot = |Pos(i, j), mut ch, is_end_calls| {
        if ch == '*' {
            let n = a.len();
            ch = match is_end_calls / n {
                0..2 => '-',
                2..4 => '+',
                4..8 => 'o',
                8..16 => 'O',
                _ => '*',
            };
        }
        let var_name: &mut [u8; A_SIZE] = dotplot.get_mut(j / b_factor).unwrap();
        let chr: &mut u8 = var_name.get_mut(i / a_factor).unwrap();
        if *chr == ' ' as u8 || *chr == '.' as u8 {
            *chr = ch as u8;
        }
    };
    for pos in matches_iter.clone() {
        make_dot(pos, '.', 0);
    }

    let start_time = time::Instant::now();

    // Run A* with heuristic.
    let mut astar = || -> Option<(usize, Vec<Pos>)> {
        petgraph::algo::astar(
            alignment_graph::AlignmentGraph::new(&a, &b),
            // start
            Pos(0, 0),
            // is end?
            |Pos(i, j)| {
                //make_dot(pos, '*', is_end_calls);
                //println!("POP {:?}", pos);
                is_end_calls += 1;
                i == a.len() && j == b.len()
            },
            // edge cost
            |Edge(Pos(i, j), Pos(x, y))| {
                edge_cost_calls += 1;
                // Compute the edge weight.
                // TODO: Use different weights for indels and substitutions.
                if x > i && y > j && a[x - 1] == b[y - 1] {
                    0
                } else {
                    1
                }
            },
            |pos| {
                heuristic_calls += 1;
                let h = (&heuristic_fn)(pos);
                //println!("h {:?} = {}", pos, h);
                h
            },
        )
    };
    let path = astar();
    let algorithm = start_time.elapsed();
    println!(
        "{:14} Matches {:7} Expanded {:7} Explored {:7} Edges {:7} precomp% {:5.2} precomp {:7.2}ms a* {:7.2}ms",
        heuristic.to_string(),
        num_matches,
        is_end_calls, heuristic_calls, edge_cost_calls,
        (precomputation.as_secs_f64() / (precomputation+algorithm).as_secs_f64()) * 100.,
        precomputation.as_secs_f32()*1000., algorithm.as_secs_f32()*1000.,
    );
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

        let path = align(2, &pattern, &text, alphabet, Heuristic::None);
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

        for l in ls {
            println!("n={} e={} l={}", n, e, l);
            let _paths = [
                //Heuristic::None,
                //Heuristic::Seeds,
                //Heuristic::Gap,
                Heuristic::GappedSeeds,
            ]
            .iter()
            .map(|heuristic| {
                let (dist, _p) = align(l, &pattern, &text, alphabet, *heuristic).unwrap();
                dist
            })
            .collect::<Vec<_>>();
            //assert_eq!(paths[0], paths[1]);
            //assert_eq!(paths[0], paths[2]);
            //assert_eq!(paths[0], paths[3]);
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
            b.iter(|| align(l, &pattern, &text, alphabet, Heuristic::None));
        }
    }
    #[bench]
    fn bench_seeds(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            b.iter(|| align(l, &pattern, &text, alphabet, Heuristic::Seeds));
        }
    }
    #[bench]
    fn bench_gap(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            b.iter(|| align(l, &pattern, &text, alphabet, Heuristic::Gap));
        }
    }
    #[bench]
    fn bench_gapped_seeds(b: &mut test::Bencher) {
        let (n, e, l, repeats, ref alphabet, mut rng) = setup();
        for _ in 0..repeats {
            let pattern = random_sequence(n, alphabet, &mut rng);
            let text = random_mutate(&pattern, alphabet, e, &mut rng);
            b.iter(|| align(l, &pattern, &text, alphabet, Heuristic::GappedSeeds));
        }
    }
}
