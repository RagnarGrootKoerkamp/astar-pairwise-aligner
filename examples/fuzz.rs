use pairwise_aligner::prelude::*;

fn main() {
    let n = 1000;
    let mut r = 0;
    loop {
        let e = 0.20;
        let l = 7;
        let max_match_cost = 1;
        let pruning = true;

        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(l),
                max_match_cost,
                ..MatchConfig::default()
            },
            pruning,
            prune_fraction: 1.0,
            c: PhantomData::<NaiveContours<LogQueryContour>>,
            ..GapSeedHeuristic::default()
        };

        println!("n={} r={}", n, r);
        let (a, b, alphabet, stats) = setup_with_seed(n, e, r);
        let result = align(&a, &b, &alphabet, stats, h);
        result.print();
        r += 1;
    }
}
