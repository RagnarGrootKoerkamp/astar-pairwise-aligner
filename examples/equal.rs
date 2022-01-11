use pairwise_aligner::prelude::*;

fn main() {
    let n = 2000;
    let e = 0.20;
    let l = 7;
    let max_match_cost = 1;
    let pruning = true;

    let h_fast = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(l),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        prune_fraction: 0.5,
        c: PhantomData::<NaiveContours<SetContour>>,
        ..GapSeedHeuristic::default()
    };
    let h_base = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(l),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        prune_fraction: 0.5,
        c: PhantomData::<NaiveContours<LogQueryContour>>,
        ..GapSeedHeuristic::default()
    };
    let h = EqualHeuristic {
        h1: h_base,
        h2: h_fast,
    };

    let (a, b, alphabet, stats) = setup(n, e);
    let start = 910;
    let end = 1050;
    let a = (&a[start..end - 20]).to_vec();
    let b = (&b[start + 20..end]).to_vec();
    let result = align(&a, &b, &alphabet, stats, h);
    result.print();
}
