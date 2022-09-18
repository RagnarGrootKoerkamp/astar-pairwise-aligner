use astar_pairwise_aligner::prelude::*;

#[test]
#[ignore = "GapCost"]
fn contour_graph() {
    let tests = [
        (
            "GATCGCAGCAGAACTGTGCCCATTTTGTGCCT",
            "CGGATCGGCGCAGAACATGTGGTCCAATTTTGCTGCC",
        ),
        (
            "GCCTAAATGCGAACGTAGATTCGTTGTTCC",
            "GTGCCTCGCCTAAACGGGAACGTAGTTCGTTGTTC",
        ),
        // Fails with alternating [(4,0),(7,1)] seeds on something to do with leftover_at_end.
        ("GAAGGGTAACAGTGCTCG", "AGGGTAACAGTGCTCGTA"),
    ];
    for (a, b) in tests {
        println!("TEST:\n{}\n{}", a, b);
        let a = a.as_bytes().to_vec();
        let b = b.as_bytes().to_vec();
        let k = 7;
        let max_match_cost = 1;
        let h = CSH {
            match_config: MatchConfig::new(k, max_match_cost),
            pruning: Pruning::default(),
            use_gap_cost: true,
            c: PhantomData::<BruteForceContours>,
        };
        let (_, _, alph, stats) = setup(0, 0.0);

        let r = align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
        let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
        assert_eq!(r.edit_distance, dist);
    }
}

#[test]
#[ignore = "GapCost"]
fn small_test() {
    let alphabet = &Alphabet::new(b"ACTG");

    let _n = 25;
    let _e = 0.2;
    let k = 4;
    let a = "AGACGTCC".as_bytes().to_vec();
    let ___text = "AGACGTCCA".as_bytes().to_vec();
    let b = ___text;

    let stats = InputStats {
        len_a: a.len(),
        len_b: b.len(),
        error_rate: 0.,
    };

    let h = CSH {
        match_config: MatchConfig::inexact(k),
        pruning: Pruning::default(),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };
    let r = align(&a, &b, &alphabet, stats, h);
    assert!(r.heuristic_stats2.root_h <= r.edit_distance);
    let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
    assert_eq!(r.edit_distance, dist);
}

/// This was broken because seed_heuristic didn't clear the previous state before rebuilding.
#[test]
#[ignore = "GapCost"]
fn seed_heuristic_rebuild() {
    let (k, m, n, e) = (4, 0, 100, 0.3);
    let h = CSH {
        match_config: MatchConfig::new(k, m),
        pruning: Pruning::enabled(),
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

/// This and the test below are fixed by disabling greedy matching.
#[test]
#[ignore = "GapCost"]
fn no_double_expand() {
    let (k, m, n, e) = (5, 1, 78, 0.3);
    let h = CSH {
        match_config: MatchConfig::new(k, m),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<BruteForceContours>,
    };

    let (_, _, alphabet, stats) = setup(n, e);
    let a = "TTGGAGATAGTGTAGACCAGTAGACTATCAGCGCGGGACCGGTGAAACCAGGCTACTAAGTGCCCGCTACAGTGTCCG"
        .as_bytes()
        .to_vec();
    let b = "CTTTGGAGATAGTGTAGATCAGTAGGCCTATCCAGCGCGGGGACCGGTAATAAACCAGGGCTAGAGCTGCCCTACAGTAGTCCAG"
        .as_bytes()
        .to_vec();
    println!("{}\n{}", to_string(&a), to_string(&b));
    let r1 = align(&a, &b, &alphabet, stats, h.to_seed_heuristic());
    let r2 = align(&a, &b, &alphabet, stats, h);
    let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
    assert_eq!(r1.edit_distance, dist);
    assert_eq!(r2.edit_distance, dist);
}

#[test]
#[ignore = "GapCost"]
fn no_double_expand_2() {
    let (k, m, n, e) = (7, 1, 61, 0.3);
    let h = CSH {
        match_config: MatchConfig::new(k, m),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };

    let (_, _, alphabet, stats) = setup(n, e);
    let a = "TCGGTCTGTACCGCCGTGGGCGGCTTCCTATCCTCTCTTGTCCCACCGGTCTTTTCAAAGC"
        .as_bytes()
        .to_vec();
    let b = "TTGTGTCTGTACGCGCCGTGGGCGGGCTTCCGTCATTCATCTCTTGGTCCCACTCGTTTCCGGAGCC"
        .as_bytes()
        .to_vec();
    println!("{}\n{}", to_string(&a), to_string(&b));
    let r1 = align(&a, &b, &alphabet, stats, h.to_seed_heuristic());
    let r2 = align(&a, &b, &alphabet, stats, h);
    let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
    assert_eq!(r1.edit_distance, dist);
    assert_eq!(r2.edit_distance, dist);
}

/// When points are removed from a layer, we may have to add new shadow points to cover for the next layer.
#[test]
#[ignore = "GapCost"]
fn missing_shadow_points() {
    let (k, m, n, e) = (10, 1, 61, 0.3);
    let h = CSH {
        match_config: MatchConfig::new(k, m),
        pruning: Pruning::enabled(),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };

    let (_, _, alphabet, stats) = setup(n, e);
    let a = "CAGCGCGCGCGGGGAGCAAGCAGCAGCCGCTTGCCCTAGCCAATTACAAGTCGCTGTAAGGTGAAACAAACCCGCAGGCTAAATGTCGACCTCAAGACG";
    let a = a.as_bytes().to_vec();
    let b = "GGGGAGCGACAGCAGCCGCCGGCTTGCCCTAGCCAATTACTAGTCGCATTAAGGTGCAAAAAACCCCATCGGCTAAATGTGACCCTCAAGACGAGATGT";
    let b = b.as_bytes().to_vec();
    println!("{}\n{}", to_string(&a), to_string(&b));
    let r1 = align(&a, &b, &alphabet, stats, h.to_seed_heuristic());
    let r2 = align(&a, &b, &alphabet, stats, h);
    let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
    assert_eq!(r1.edit_distance, dist);
    assert_eq!(r2.edit_distance, dist);
}
