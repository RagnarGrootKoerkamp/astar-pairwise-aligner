use pairwise_aligner::{prelude::*, *};

fn main() {
    let pruning = false;
    for do_transform in [false, true] {
        let n = 50;
        let e: f32 = 0.2;
        let l = 4;
        let max_match_cost = 1;
        let prune = [];

        let heuristic = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(l),
                max_match_cost,
                ..MatchConfig::default()
            },
            pruning,
            c: PhantomData::<NaiveContours<NaiveContour>>,
            ..GapSeedHeuristic::default()
        };

        let (ref a, ref b, alphabet, stats) = setup(n, e);
        println!("{}\n{}", to_string(a), to_string(b));
        let mut h = heuristic.build(&a, &b, &alphabet);
        for p in &prune {
            h.prune(*p);
        }

        h.print(do_transform, false);

        if do_transform {
            // TODO: Convert to slow variant.
            let h2 = heuristic.as_seed_heuristic();

            align(
                &a,
                &b,
                &alphabet,
                stats,
                EqualHeuristic {
                    h1: h2,
                    h2: heuristic,
                },
            );
        }
    }
}
