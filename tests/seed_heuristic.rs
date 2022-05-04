use pairwise_aligner::prelude::*;

/// This was broken because seed_heuristic didn't clear the previous state before rebuilding.
#[test]
fn seed_heuristic_rebuild() {
    let (k, m, n, e, pruning) = (4, 0, 100, 0.3, true);
    let h = ChainedSeedsHeuristic {
        match_config: MatchConfig {
            length: Fixed(k),
            max_match_cost: m,
            ..MatchConfig::default()
        },
        pruning,
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };
    let (_a, _b, alph, stats) = setup(n, e);

    let a = "TGAGTTAAGCCGATTG".as_bytes().to_vec();
    let b = "AGAGTTTAAGCCGGATG".as_bytes().to_vec();
    println!("TESTING n {} e {}: {:?}", n, e, h);
    println!("{}\n{}", to_string(&a), to_string(&b));
    align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());

    let a = "TCGTCCCAACTGCGTGCAGACGTCCTGAGGACGTGGTCGCGACGCTATAGGCAGGGTACATCGAGATGCCGCCTAAATGCGAACGTAGATTCGTTGTTCC".as_bytes().to_vec();
    let b = "TCAGTCCCACACTCCTAGCAGACGTTCCTGCAGGACAGTGGACGCTGACGCCTATAGGAGAGGCATCGAGGTGCCTCGCCTAAACGGGAACGTAGTTCGTTGTTC".as_bytes().to_vec();
    println!("TESTING n {} e {}: {:?}", n, e, h);
    println!("{}\n{}", to_string(&a), to_string(&b));
    let r = align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
    let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
    assert_eq!(r.edit_distance, dist);
}

/// In the ChainedSeedsHeuristic, we never use a gap distance, unless it's towards the target.
/// This test makes sure that SeedHeuristic<Gap> does the same:
/// Instead of taking max(gap distance, potential distance), in cases when gap >
/// potential, this parent should be skipped completely.
#[test]
fn never_use_gap_distance() {
    let (k, m, n, e, pruning) = (5, 1, 14, 0.3, true);
    let h = ChainedSeedsHeuristic {
        match_config: MatchConfig {
            length: Fixed(k),
            max_match_cost: m,
            ..MatchConfig::default()
        },
        pruning,
        use_gap_cost: true,
        c: PhantomData::<BruteForceContours>,
    }
    .equal_to_seed_heuristic();

    let (_, _, alphabet, stats) = setup(n, e);
    let a = "CTAAGGAGTCCCAT".as_bytes().to_vec();
    let b = "GTAAGAGTCCACT".as_bytes().to_vec();
    println!("{}\n{}", to_string(&a), to_string(&b));
    let r = align(&a, &b, &alphabet, stats, h);
    r.print();
    let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
    assert_eq!(r.edit_distance, dist);
}

/// Zero distance should be consistent.
#[test]
#[ignore = "ZeroCost heuristic is not decreasing on diagonals"]
fn seed_heuristic_zero_dist_consistent() {
    for (k, m) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = SeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
                        max_match_cost: m,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    distance_function: ZeroCost,
                };

                println!("TESTING n {} e {}: {:?}", n, e, h);

                let (a, b, alphabet, stats) = setup(n, e);
                let r = align(&a, &b, &alphabet, stats, h);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
                assert_eq!(r.edit_distance, dist);
            }
        }
    }
}
