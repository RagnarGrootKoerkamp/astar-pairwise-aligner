use pairwise_aligner::{prelude::*, *};

fn main() {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("evals/stats/table.csv")
        .unwrap();

    let n = 8_000;
    let e = 0.20;
    let l = 7;
    let max_match_cost = 1;
    let pruning = false;
    let build_fast = true;
    let query_fast = true;
    let make_consistent = false;

    AlignResult::print_header();
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
    AlignResult::print_header();
}
