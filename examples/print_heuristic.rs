use pairwise_aligner::{prelude::*, *};

fn main() {
    let pruning = false;
    let (l, max_match_cost) = (5, 1);
    for do_transform in [false, true] {
        for build_fast in [false] {
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
            };

            let n = 40;
            let e: f32 = 0.2;
            let (ref a, ref b, alphabet, stats) = setup(n, e);
            //let start = 0;
            //let end = 150;
            //let a = &a[start..end].to_vec();
            //let b = &b[start..end].to_vec();

            let prune = [];
            println!("{}\n{}", to_string(a), to_string(b));
            let mut h = heuristic.build(&a, &b, &alphabet);
            for p in &prune {
                h.prune(*p);
            }

            h.print(do_transform);

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
