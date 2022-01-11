use pairwise_aligner::prelude::*;

fn main() {
    let mut r = 0;
    loop {
        let (l, m, n, e, pruning, prune_fraction) = (4, 0, 16, 0.3, true, 1.0);
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

        println!("n={} r={}", n, r);
        let (a, b, alphabet, stats) = setup_with_seed(n, e, r);
        println!("{}\n{}", to_string(&a), to_string(&b));
        let result = align(&a, &b, &alphabet, stats, h);
        result.print();
        r += 1;
    }
}
