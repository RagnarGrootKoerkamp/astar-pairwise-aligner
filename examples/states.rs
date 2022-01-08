use pairwise_aligner::{prelude::*, *};

fn main() {
    let n = 2000;
    let e = 0.3;

    {
        let h = SeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(4),
                max_match_cost: 0,
                ..MatchConfig::default()
            },
            distance_function: GapCost,
            pruning: false,
            ..SeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(n, e);
        align(&a, &b, &alphabet, stats, h)
    }
    .write_explored_states("evals/stats/exact.csv");
    {
        let h = SeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(6),
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            distance_function: GapCost,
            pruning: false,
            ..SeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(n, e);
        align(&a, &b, &alphabet, stats, h)
    }
    .write_explored_states("evals/stats/inexact.csv");
    {
        let h = SeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(4),
                max_match_cost: 0,
                ..MatchConfig::default()
            },
            distance_function: GapCost,
            pruning: true,
            ..SeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(n, e);
        align(&a, &b, &alphabet, stats, h)
    }
    .write_explored_states("evals/stats/exact_pruning.csv");
    let r = {
        let h = SeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(6),
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            distance_function: ZeroCost,
            pruning: true,
            ..SeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(n, e);
        align(&a, &b, &alphabet, stats, h)
    };
    r.write_explored_states("evals/stats/inexact_pruning_zero.csv");
    println!(
        "BAND ZERO: {}",
        r.astar.expanded as f32 / r.input.len_a as f32
    );
    let r = {
        let h = SeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(6),
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            distance_function: GapCost,
            pruning: true,
            ..SeedHeuristic::default()
        };
        let (a, b, alphabet, stats) = setup(n, e);
        align(&a, &b, &alphabet, stats, h)
    };
    r.write_explored_states("evals/stats/inexact_pruning.csv");
    println!(
        "BAND COUNT: {}",
        r.astar.expanded as f32 / r.input.len_a as f32
    );
}
