use itertools::Itertools;
use pairwise_aligner::{prelude::*, *};

fn main() {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("evals/stats/table.csv")
        .unwrap();

    let ns = [2_000, 4_000, 8_000, 16_000, 32_000];
    let es = [0.10];
    let lm = [(5, 0), (6, 0), (8, 1), (9, 1), (10, 1)];
    let prunings = [true];
    let query_fast = [false];
    let consistent = [false];

    AlignResult::print_header();
    for (&n, e) in ns.iter().cartesian_product(es) {
        for pruning in prunings {
            for (l, max_match_cost) in lm {
                for query_fast in query_fast {
                    for make_consistent in consistent {
                        let result = test_heuristic(
                            n,
                            e,
                            SeedHeuristic {
                                l,
                                max_match_cost,
                                distance_function: GapHeuristic,
                                pruning,
                                build_fast: true,
                                query_fast,
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
