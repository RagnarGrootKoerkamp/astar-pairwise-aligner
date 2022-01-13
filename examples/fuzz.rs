use pairwise_aligner::prelude::*;

fn main() {
    for n in 30.. {
        for r in 0..8000 {
            let (l, m, n, e, pruning, prune_fraction) = (8, 1, n, 0.3, true, 1.0);
            let h = GapSeedHeuristic {
                match_config: MatchConfig {
                    length: Fixed(l),
                    max_match_cost: m,
                    ..MatchConfig::default()
                },
                pruning,
                prune_fraction,
                c: PhantomData::<BruteForceContours>,
                ..GapSeedHeuristic::default()
            }
            .equal_to_seed_heuristic();

            println!("n={} r={} l={}", n, r, l);
            let (a, b, alphabet, stats) = setup_with_seed(n, e, r);
            println!("{}\n{}", to_string(&a), to_string(&b));
            let result = align(&a, &b, &alphabet, stats, h);
            result.print();
        }
    }
}
