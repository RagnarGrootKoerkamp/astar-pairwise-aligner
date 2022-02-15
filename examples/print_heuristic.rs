use pairwise_aligner::prelude::*;

fn main() {
    let pruning = false;
    for do_transform in [false, true] {
        let n = 50;
        let e: f32 = 0.2;
        let k = 4;
        let max_match_cost = 1;

        let heuristic = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost,
                ..MatchConfig::default()
            },
            pruning,
            c: PhantomData::<HintContours<BruteForceContour>>,
            ..GapSeedHeuristic::default()
        };

        let (ref a, ref b, alphabet, stats) = setup(n, e);
        println!("{}\n{}", to_string(a), to_string(b));
        let mut h = heuristic.build(&a, &b, &alphabet);

        h.print(do_transform, false);

        if do_transform {
            align(
                &a,
                &b,
                &alphabet,
                stats,
                heuristic.equal_to_seed_heuristic(),
            );
        }
    }
}
