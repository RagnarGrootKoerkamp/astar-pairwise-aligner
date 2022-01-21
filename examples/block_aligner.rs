use itertools::Itertools;
use pairwise_aligner::prelude::*;

// Compare with block aligner:
// They do 10k pairs of length 10k and distance 10% in 2s!
fn main() {
    let ns = [10_000];
    let es = [0.10];

    for (&n, e) in ns.iter().cartesian_product(es) {
        for k in [8, 9, 10, 11, 12] {
            {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
                        max_match_cost: 1,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<NaiveContours<LogQueryContour>>,
                    ..GapSeedHeuristic::default()
                };
                let (a, b, alphabet, stats) = setup(n, e);
                align(&a, &b, &alphabet, stats, h)
            }
            .print();
        }
    }
}
