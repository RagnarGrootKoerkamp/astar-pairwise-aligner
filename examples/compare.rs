use itertools::Itertools;
use pairwise_aligner::{prelude::*, *};

fn main() {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("evals/stats/table.csv")
        .unwrap();

    let ns = [4_000, 8_000, 16_000, 32_000];
    let es = [0.20];
    let lm = [(8, 1), (9, 1)];
    let prunings = [false, true];
    let build_fast = [(true, false), (true, true)];

    AlignResult::print_header();
    for (&n, e) in ns.iter().cartesian_product(es) {
        for (l, max_match_cost) in lm {
            for pruning in prunings {
                for (build_fast, query_fast) in build_fast {
                    if pruning && query_fast {
                        continue;
                    }
                    if !pruning && !query_fast {
                        continue;
                    }
                    let result = {
                        let h = SeedHeuristic {
                            l,
                            max_match_cost,
                            distance_function: GapHeuristic,
                            pruning,
                            build_fast,
                            query_fast,
                        };
                        let (a, b, alphabet, stats) = setup(n, e);
                        align(&a, &b, &alphabet, stats, h)
                    };
                    result.print();
                    result.write(&mut wtr);
                }
            }
        }
        println!("");
    }
    AlignResult::print_header();
}
