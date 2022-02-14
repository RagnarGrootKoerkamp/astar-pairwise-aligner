use pairwise_aligner::prelude::*;

/// This was broken because seed_heuristic didn't clear the previous state before rebuilding.
#[test]
fn seed_heuristic_rebuild() {
    let (k, m, n, e, pruning, prune_fraction) = (4, 0, 100, 0.3, true, 1.0);
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(k),
            max_match_cost: m,
            ..MatchConfig::default()
        },
        pruning,
        prune_fraction,
        c: PhantomData::<HintContours<BruteForceContour>>,
        ..GapSeedHeuristic::default()
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
    align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
}

/// In the GapSeedHeuristic, we never use a gap distance, unless it's towards the target.
/// This test makes sure that SeedHeuristic<Gap> does the same:
/// Instead of taking max(gap distance, potential distance), in cases when gap >
/// potential, this parent should be skipped completely.
#[test]
fn never_use_gap_distance() {
    let (k, m, n, e, pruning, prune_fraction) = (5, 1, 14, 0.3, true, 1.0);
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(k),
            max_match_cost: m,
            ..MatchConfig::default()
        },
        pruning,
        prune_fraction,
        c: PhantomData::<BruteForceContours>,
        ..GapSeedHeuristic::default()
    }
    .equal_to_seed_heuristic();

    let (_, _, alphabet, stats) = setup(n, e);
    let a = "CTAAGGAGTCCCAT".as_bytes().to_vec();
    let b = "GTAAGAGTCCACT".as_bytes().to_vec();
    println!("{}\n{}", to_string(&a), to_string(&b));
    let result = align(&a, &b, &alphabet, stats, h);
    result.print();
}
