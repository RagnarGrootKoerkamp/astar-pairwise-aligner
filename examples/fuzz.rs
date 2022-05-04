use pairwise_aligner::prelude::*;

fn main() {
    for n in 40.. {
        for r in 0..10000 {
            let (k, m, n, e, pruning) = (4, 0, n, 0.9, true);
            let h = CSH {
                match_config: MatchConfig {
                    length: Fixed(k),
                    max_match_cost: m,
                    ..MatchConfig::default()
                },
                pruning,
                use_gap_cost: false,
                c: PhantomData::<HintContours<BruteForceContour>>::default(),
            };

            println!("n={} r={} k={}", n, r, k);
            let (a, b, alphabet, stats) = setup_with_seed(n, e, r);
            //println!("{}\n{}", to_string(&a), to_string(&b));
            let result = align(&a, &b, &alphabet, stats, h);
            result.print();
        }
    }
}
