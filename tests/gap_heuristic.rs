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
    assert!(r.heuristic_stats2.root_h <= r.answer_cost);
}

/// In the GapSeedHeuristic, we never use a gap distance, unless it's towards the target.
/// This test makes sure that SeedHeuristic<Gap> does the same:
/// Instead of taking max(gap distance, potential distance), in cases when gap >
/// potential, this parent should be skipped completely.
#[test]
fn never_use_gap_distance() {
    let (l, m, n, e, pruning, prune_fraction) = (5, 1, 14, 0.3, true, 1.0);
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
    }
    .equal_to_seed_heuristic();

    let (_, _, alphabet, stats) = setup(n, e);
    let a = "CTAAGGAGTCCCAT".as_bytes().to_vec();
    let b = "GTAAGAGTCCACT".as_bytes().to_vec();
    println!("{}\n{}", to_string(&a), to_string(&b));
    let result = align(&a, &b, &alphabet, stats, h);
    result.print();
}
