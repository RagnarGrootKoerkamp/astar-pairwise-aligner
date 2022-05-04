use pairwise_aligner::prelude::*;

fn main() {
    let pruning = true;
    let n = 55;
    let e: f32 = 0.2;
    let _k = 10;
    let max_match_cost = 1;

    let heuristic = SH {
        match_config: MatchConfig {
            length:
            //Fixed(6),
            LengthConfig::Max(MaxMatches {
                max_matches: 1,
                k_min: 4,
                k_max: 10,
            }),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
    };

    let (ref a, ref b, alphabet, stats) = setup(n, e);
    println!("{}\n{}", to_string(a), to_string(b));
    let h = heuristic.build(&a, &b, &alphabet);

    PRINT.store(true, std::sync::atomic::Ordering::Relaxed);
    h.print(false, false);

    align(&a, &b, &alphabet, stats, heuristic).print();
}
