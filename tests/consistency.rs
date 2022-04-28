use pairwise_aligner::prelude::*;

#[test]
fn bicount_admissible() {
    let alphabet = &Alphabet::new(b"ACTG");

    let _n = 25;
    let _e = 0.2;
    let k = 4;
    let pattern = "AGACGTCC".as_bytes().to_vec();
    let ___text = "AGACGTCCA".as_bytes().to_vec();
    let text = ___text;

    let stats = SequenceStats {
        len_a: pattern.len(),
        len_b: text.len(),
        error_rate: 0.,
        source: Source::Manual,
    };

    let r = align(
        &pattern,
        &text,
        &alphabet,
        stats,
        SeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(k),
                max_match_cost: 1,
                ..MatchConfig::default()
            },
            distance_function: BiCountCost,
            pruning: false,
        },
    );
    assert!(r.heuristic_stats2.root_h <= r.edit_distance);
}

// Failed because of match distance > 0
#[test]
fn consistency_1() {
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(4),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: false,
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };
    let (a, b, alphabet, stats) = setup(2000, 0.10);
    let a = &a[361..369].to_vec();
    let b = &b[363..371].to_vec();

    println!("{}\n{}\n", to_string(&a), to_string(&b));
    align(a, b, &alphabet, stats, h);
}

// Failed because of match distance > 0 and stricter consistency check
#[test]
fn consistency_2() {
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(5),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: false,
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };
    let (a, b, alphabet, stats) = setup(2000, 0.10);
    let a = &a[236..246].to_vec();
    let b = &b[236..246].to_vec();

    println!("{}\n{}\n", to_string(&a), to_string(&b));
    align(a, b, &alphabet, stats, h);
}

// Failed because of pruning
#[test]
#[ignore]
fn consistency_3() {
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(4),
            max_match_cost: 0,
            ..MatchConfig::default()
        },
        pruning: true,
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };
    let (a, b, alphabet, stats) = setup(2000, 0.10);
    let a = &a.to_vec();
    let b = &b.to_vec();

    println!("{}\n{}\n", to_string(&a), to_string(&b));
    align(a, b, &alphabet, stats, h);
}

// Failed because of pruning and match distance
#[test]
fn consistency_4() {
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(6),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: true,
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };
    let (a, b, alphabet, stats) = setup(2000, 0.10);
    let a = &a[846..870].to_vec();
    let b = &b[856..880].to_vec();
    // TTGTGGGCCCTCTTAACTTCCAAC
    // TTTTTGGGCCCTTTAACTTCCAAC

    println!("{}\n{}\n", to_string(&a), to_string(&b));
    align(a, b, &alphabet, stats, h);
}

// Failed because of pruning and large edit distance
#[test]
fn consistency_5() {
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(4),
            max_match_cost: 0,
            ..MatchConfig::default()
        },
        pruning: true,
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>,
    };
    let (a, b, alphabet, stats) = setup(2000, 0.20);
    let a = &a[200..310].to_vec();
    let b = &b[203..313].to_vec();

    println!("{}\n{}\n", to_string(&a), to_string(&b));
    align(a, b, &alphabet, stats, h);
}
