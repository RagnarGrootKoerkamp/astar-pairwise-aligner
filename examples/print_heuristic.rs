use pairwise_aligner::{prelude::*, *};

fn main() {
    let pruning = false;
    for do_transform in [false, true] {
        for build_fast in [true] {
            let n = 50;
            let e: f32 = 0.2;
            let l = 4;
            let max_match_cost = 1;
            let prune = [];

            let heuristic = SeedHeuristic {
                match_config: MatchConfig {
                    length: Fixed(l),
                    max_match_cost,
                    ..MatchConfig::default()
                },
                distance_function: GapHeuristic,
                pruning,
                build_fast,
                query_fast: QueryMode::Off,
                ..SeedHeuristic::default()
            };

            let (ref a, ref b, alphabet, stats) = setup(n, e);
            println!("{}\n{}", to_string(a), to_string(b));
            let mut h = heuristic.build(&a, &b, &alphabet);
            for p in &prune {
                h.prune(*p);
            }

            h.print(do_transform, false);

            if do_transform {
                let h2 = SeedHeuristic {
                    build_fast: false,
                    query_fast: QueryMode::Off,
                    ..heuristic
                };

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
}
