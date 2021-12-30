use pairwise_aligner::{prelude::*, *};

fn main() {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("evals/stats/table.csv")
        .unwrap();

    let n = 10_000;
    let e = 0.10;
    let l = 7;
    let max_match_cost = 1;
    let pruning = false;
    let build_fast = true;
    let query_fast = true;

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
