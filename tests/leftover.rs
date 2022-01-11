use pairwise_aligner::prelude::*;

#[test]
fn no_leftover() {
    let pruning = true;
    let (l, max_match_cost) = (7, 1);
    let h_slow = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(l),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        c: PhantomData::<NaiveContours<BruteForceContour>>,
        ..GapSeedHeuristic::default()
    };
    let h_fast = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(l),
            max_match_cost,
            ..MatchConfig::default()
        },
        pruning,
        c: PhantomData::<NaiveContours<BruteForceContour>>,
        ..GapSeedHeuristic::default()
    };

    let n = 1000;
    let e: f32 = 0.3;
    let (a, b, alph, stats) = setup(n, e);
    let start = 679;
    let end = 750;
    let a = &a[start..end].to_vec();
    let b = &b[start..end].to_vec();

    println!("\n\n\nALIGN");
    align(
        &a,
        &b,
        &alph,
        stats,
        EqualHeuristic {
            h1: h_slow,
            h2: h_fast,
        },
    );
}

#[test]
fn needs_leftover() {
    let h_slow = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(7),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: false,
        c: PhantomData::<NaiveContours<BruteForceContour>>,
        ..GapSeedHeuristic::default()
    };
    let h_fast = GapSeedHeuristic { ..h_slow };

    let n = 1000;
    let e: f32 = 0.3;
    let (a, b, alph, stats) = setup(n, e);
    let start = 909;
    let end = 989;
    let a = &a[start..end].to_vec();
    let b = &b[start..end].to_vec();

    println!("TESTING: {:?}", h_fast);
    println!("{}\n{}", to_string(a), to_string(b));

    println!("ALIGN");
    align(
        &a,
        &b,
        &alph,
        stats,
        EqualHeuristic {
            h1: h_slow,
            h2: h_fast,
        },
    );
}
