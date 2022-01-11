use pairwise_aligner::prelude::*;

/// This was broken because seed_heuristic didn't clear the previous state before rebuilding.
#[test]
fn seed_heuristic_rebuild() {
    let (l, m, n, e, pruning, prune_fraction) = (4, 0, 100, 0.3, true, 1.0);
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(l),
            max_match_cost: m,
            ..MatchConfig::default()
        },
        pruning,
        prune_fraction,
        c: PhantomData::<NaiveContours<BruteForceContour>>,
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
