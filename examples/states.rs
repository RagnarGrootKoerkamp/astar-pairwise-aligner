use pairwise_aligner::{prelude::*, *};

fn main() {
    let n = 2000;
    let e = 0.3;

    test_heuristic(
        n,
        e,
        SeedHeuristic {
            l: 4,
            match_distance: 0,
            distance_function: CountHeuristic,
            pruning: false,
        },
    )
    .write_explored_states("evals/stats/exact.csv");
    test_heuristic(
        n,
        e,
        SeedHeuristic {
            l: 6,
            match_distance: 1,
            distance_function: CountHeuristic,
            pruning: false,
        },
    )
    .write_explored_states("evals/stats/inexact.csv");
    test_heuristic(
        n,
        e,
        SeedHeuristic {
            l: 4,
            match_distance: 0,
            distance_function: CountHeuristic,
            pruning: true,
        },
    )
    .write_explored_states("evals/stats/exact_pruning.csv");
    let r = test_heuristic(
        n,
        e,
        SeedHeuristic {
            l: 6,
            match_distance: 1,
            distance_function: ZeroHeuristic,
            pruning: true,
        },
    );
    r.write_explored_states("evals/stats/inexact_pruning_zero.csv");
    println!(
        "BAND ZERO: {}",
        r.astar.expanded as f32 / r.input.len_a as f32
    );
    let r = test_heuristic(
        n,
        e,
        SeedHeuristic {
            l: 6,
            match_distance: 1,
            distance_function: CountHeuristic,
            pruning: true,
        },
    );
    r.write_explored_states("evals/stats/inexact_pruning.csv");
    println!(
        "BAND COUNT: {}",
        r.astar.expanded as f32 / r.input.len_a as f32
    );
}
