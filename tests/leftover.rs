use pairwise_aligner::prelude::*;

#[test]
fn no_leftover() {
    let pruning = true;
    let (k, max_match_cost) = (7, 1);
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(k),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };

    let n = 1000;
    let e: f32 = 0.3;
    let (a, b, alph, stats) = setup(n, e);
    let start = 679;
    let end = 750;
    let a = &a[start..end].to_vec();
    let b = &b[start..end].to_vec();

    println!("\n\n\nALIGN");
    align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
}

#[test]
fn needs_leftover() {
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(7),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: false,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };

    let n = 1000;
    let e: f32 = 0.3;
    let (a, b, alph, stats) = setup(n, e);
    let start = 909;
    let end = 989;
    let a = &a[start..end].to_vec();
    let b = &b[start..end].to_vec();

    println!("TESTING: {:?}", h);
    println!("{}\n{}", to_string(a), to_string(b));

    println!("ALIGN");
    align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
}
