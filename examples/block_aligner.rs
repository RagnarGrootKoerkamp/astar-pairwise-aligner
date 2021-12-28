use itertools::Itertools;
use pairwise_aligner::{prelude::*, *};

// Compare with block aligner:
// They do 10k pairs of length 10k and distance 10% in 2s!
fn main() {
    let ns = [10_000];
    let es = [0.10];

    AlignResult::print_header();
    for (&n, e) in ns.iter().cartesian_product(es) {
        for l in [8, 9, 10, 11, 12] {
            test_heuristic(
                n,
                e,
                SeedHeuristic {
                    l,
                    max_match_cost: 1,
                    distance_function: CountHeuristic,
                    pruning: true,
                    build_fast: false,
                    query_fast: false,
                },
            )
            .print();
        }
    }
    AlignResult::print_header();
}
