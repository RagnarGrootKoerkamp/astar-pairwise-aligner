use pairwise_aligner::prelude::*;

#[test]
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
        let pruning = false;
        let h = ChainedSeedsHeuristic {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost,
                ..MatchConfig::default()
            },
            pruning,
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
fn small_test() {
    let alphabet = &Alphabet::new(b"ACTG");

    let _n = 25;
    let _e = 0.2;
    let k = 4;
    let a = "AGACGTCC".as_bytes().to_vec();
    let ___text = "AGACGTCCA".as_bytes().to_vec();
    let b = ___text;

    let stats = SequenceStats {
        len_a: a.len(),
        len_b: b.len(),
        error_rate: 0.,
        source: Source::Manual,
    };

    let h = ChainedSeedsHeuristic {
        match_config: MatchConfig {
            length: Fixed(k),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: false,
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

/// This and the test below are fixed by disabling greedy matching.
#[test]
fn no_double_expand() {
    let (k, m, n, e, pruning) = (5, 1, 78, 0.3, true);
    let h = ChainedSeedsHeuristic {
        match_config: MatchConfig {
            length: Fixed(k),
            max_match_cost: m,
            ..MatchConfig::default()
        },
        pruning,
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
fn no_double_expand_2() {
    let (k, m, n, e, pruning) = (7, 1, 61, 0.3, true);
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
fn missing_shadow_points() {
    let (k, m, n, e, pruning) = (10, 1, 61, 0.3, true);
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
