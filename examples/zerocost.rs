use pairwise_aligner::{prelude::*, *};

fn main() {
    let n = 4000;
    let e = 0.10;
    let l = 5;
    let max_match_cost = 0;
    let pruning = true;
    let prune_fraction = 1.0;

    let h = SeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(l),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        prune_fraction,
        distance_function: ZeroCost,
        ..SeedHeuristic::default()
    };
    let (a, b, alphabet, stats) = setup(n, e);
    let result = align(&a, &b, &alphabet, stats, h);
    result.print();

    let h = SeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(l),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        prune_fraction,
        distance_function: GapCost,
        ..SeedHeuristic::default()
    };
    let (a, b, alphabet, stats) = setup(n, e);
    let result = align(&a, &b, &alphabet, stats, h);
    result.print();
}
