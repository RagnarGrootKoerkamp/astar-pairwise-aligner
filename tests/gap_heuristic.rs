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
        let l = 7;
        let max_match_cost = 1;
        let pruning = false;
        let h = GapSeedHeuristic {
            match_config: MatchConfig {
                length: Fixed(l),
                max_match_cost,
                ..MatchConfig::default()
            },
            pruning,
            c: PhantomData::<BruteForceContours>,
            ..GapSeedHeuristic::default()
        };
        let (_, _, alph, stats) = setup(0, 0.0);

        align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
    }
}

#[test]
fn pruning_and_inexact_matches() {
    let pruning = true;
    let (l, max_match_cost) = (7, 1);
    for do_transform in [false, true] {
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
        let h_slow = GapSeedHeuristic { ..h_fast };

        let n = 1000;
        let e: f32 = 0.3;
        let (a, b, alph, stats) = setup(n, e);
        let start = 951;
        let end = 986;
        let a = &a[start..end].to_vec();
        let b = &b[start..end].to_vec();

        println!("TESTING: {:?}", h_fast);
        println!("{}\n{}", to_string(a), to_string(b));

        if do_transform {
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
    }
}

#[test]
fn small_test() {
    let alphabet = &Alphabet::new(b"ACTG");

    let _n = 25;
    let _e = 0.2;
    let l = 4;
    let pattern = "AGACGTCC".as_bytes().to_vec();
    let ___text = "AGACGTCCA".as_bytes().to_vec();
    let text = ___text;

    let stats = SequenceStats {
        len_a: pattern.len(),
        len_b: text.len(),
        error_rate: 0.,
        source: Source::Manual,
    };

    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(l),
            max_match_cost: 1,
            ..MatchConfig::default()
        },
        pruning: false,
        c: PhantomData::<NaiveContours<BruteForceContour>>,
        ..GapSeedHeuristic::default()
    };
    let r = align(&pattern, &text, &alphabet, stats, h);
    assert!(r.heuristic_stats2.root_h <= r.edit_distance);
}

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

/// This and the test below are fixed by disabling greedy matching.
#[test]
fn no_double_expand() {
    let (l, m, n, e, pruning, prune_fraction) = (5, 1, 78, 0.3, true, 1.0);
    let h = GapSeedHeuristic {
        match_config: MatchConfig {
            length: Fixed(l),
            max_match_cost: m,
            ..MatchConfig::default()
        },
        pruning,
        prune_fraction,
        c: PhantomData::<BruteForceContours>,
        ..GapSeedHeuristic::default()
    };

    let (_, _, alphabet, stats) = setup(n, e);
    let a = "TTGGAGATAGTGTAGACCAGTAGACTATCAGCGCGGGACCGGTGAAACCAGGCTACTAAGTGCCCGCTACAGTGTCCG"
        .as_bytes()
        .to_vec();
    let b = "CTTTGGAGATAGTGTAGATCAGTAGGCCTATCCAGCGCGGGGACCGGTAATAAACCAGGGCTAGAGCTGCCCTACAGTAGTCCAG"
        .as_bytes()
        .to_vec();
    println!("{}\n{}", to_string(&a), to_string(&b));
    align(&a, &b, &alphabet, stats, h.to_seed_heuristic()).print();
    align(&a, &b, &alphabet, stats, h).print();
}

#[test]
fn no_double_expand_2() {
    let (l, m, n, e, pruning, prune_fraction) = (7, 1, 61, 0.3, true, 1.0);
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

    let (_, _, alphabet, stats) = setup(n, e);
    let a = "TCGGTCTGTACCGCCGTGGGCGGCTTCCTATCCTCTCTTGTCCCACCGGTCTTTTCAAAGC"
        .as_bytes()
        .to_vec();
    let b = "TTGTGTCTGTACGCGCCGTGGGCGGGCTTCCGTCATTCATCTCTTGGTCCCACTCGTTTCCGGAGCC"
        .as_bytes()
        .to_vec();
    println!("{}\n{}", to_string(&a), to_string(&b));
    align(&a, &b, &alphabet, stats, h.to_seed_heuristic()).print();
    align(&a, &b, &alphabet, stats, h).print();
}
