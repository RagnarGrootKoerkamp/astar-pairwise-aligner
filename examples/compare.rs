use itertools::Itertools;
use pairwise_aligner::{prelude::*, *};

fn main() {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("evals/stats/table.csv")
        .unwrap();

    let ns = [2_000];
    let es = [0.10, 0.20, 0.30];
    let lm = [(5, 0), (6, 0), (7, 1), (8, 1), (9, 1)];
    let prunings = [false, true];
    let build_fast = [(false, false), (true, false), (true, true)];
    let consistent = [false];

    AlignResult::print_header();
    for (&n, e) in ns.iter().cartesian_product(es) {
        for (l, max_match_cost) in lm {
            for pruning in prunings {
                for make_consistent in consistent {
                    for (build_fast, query_fast) in build_fast {
                        if pruning && query_fast {
                            continue;
                        }
                        let result = test_heuristic(
                            n,
                            e,
                            SeedHeuristic {
                                l,
                                max_match_cost,
                                distance_function: GapHeuristic,
                                pruning,
                                build_fast,
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
