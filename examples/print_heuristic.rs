use pairwise_aligner::prelude::*;

fn main() {
    let pruning = true;
    let n = 100;
    let e: f32 = 0.3;
    let k = 4;
    let max_match_cost = 0;

    let heuristic = SH {
        match_config: MatchConfig {
            length:
            //Fixed(6),
            LengthConfig::Fixed(k),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
    };

    let (ref a, ref b, alphabet, stats) = setup(n, e);
    println!("{}\n{}", to_string(a), to_string(b));
    //let h = heuristic.build(&a, &b, &alphabet);

    PRINT.store(true, std::sync::atomic::Ordering::Relaxed);
    h.terminal_print(Pos::from_length(a, b));

    align(&a, &b, &alphabet, stats, heuristic).print();
}
