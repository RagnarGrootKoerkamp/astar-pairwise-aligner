use pairwise_aligner::prelude::*;

fn main() {
    let n = 100000;
    let e = 0.20;
    let k = 7;
    let max_match_cost = 1;
    let pruning = true;

    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(k),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        prune_fraction: 0.5,
        c: PhantomData::<NaiveContours<LogQueryContour>>,
        ..GapSeedHeuristic::default()
    };

    let (a, b, alphabet, stats) = setup(n, e);
    let result = align(&a, &b, &alphabet, stats, h);
    result.print();
}
