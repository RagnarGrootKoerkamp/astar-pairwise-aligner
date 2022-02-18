use pairwise_aligner::prelude::{UnorderedHeuristic, *};

fn main() {
    for n in 10.. {
        for r in 0..100 {
            let (k, m, n, e, pruning) = (6, 1, n, 0.1, true);
            let h = UnorderedHeuristic {
                match_config: MatchConfig {
                    length: Fixed(k),
                    max_match_cost: m,
                    ..MatchConfig::default()
                },
                pruning,
            };

            println!("n={} r={} k={}", n, r, k);
            let (a, b, alphabet, stats) = setup_with_seed(n, e, r);
            println!("{}\n{}", to_string(&a), to_string(&b));
            let result = align(&a, &b, &alphabet, stats, h);
            result.print();
        }
    }
}
