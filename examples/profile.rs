use pairwise_aligner::{prelude::*, *};

fn main() {
    let n = 30000;
    let e = 0.10;
    let l = 5;
    let max_match_cost = 0;
    let pruning = true;

    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(l),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        prune_fraction: 0.5,
        c: PhantomData::<NaiveContours<NaiveContour>>,
        ..GapSeedHeuristic::default()
    };
    let (a, b, alphabet, stats) = setup(n, e);
    let result = align(&a, &b, &alphabet, stats, h);
    result.print();
}
