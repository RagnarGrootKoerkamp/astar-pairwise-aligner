use itertools::Itertools;
use pairwise_aligner::{prelude::*, *};

fn main() {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("evals/stats/table.csv")
        .unwrap();

    let ns = [2_000];
    let es = [0.10, 0.20, 0.30];
    let lm = [(4, 0), (5, 0), (7, 1), (8, 1)];
    let prunings = [false, true];

    AlignResult::print_header();
    for (&n, e) in ns.iter().cartesian_product(es) {
        for pruning in prunings {
            for (l, max_match_cost) in lm {
                for build_fast in [false, true] {
                    for make_consistent in [true, false] {
                        let result = test_heuristic(
                            n,
                            e,
                            SeedHeuristic {
                                l,
                                max_match_cost,
                                distance_function: GapHeuristic,
                                pruning,
                                build_fast,
                                query_fast: build_fast,
                                make_consistent,
                            },
                        );
                        result.print();
                        result.write(&mut wtr);
                    }
                }
            }
        }
        println!("");
    }
    AlignResult::print_header();
}
