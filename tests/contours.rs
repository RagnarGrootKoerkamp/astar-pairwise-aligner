use pairwise_aligner::prelude::*;

#[test]
fn exact_no_pruning() {
    for l in [4, 5] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost: 0,
                        ..MatchConfig::default()
                    },
                    pruning: false,
                    c: PhantomData::<BruteForceContours>,
                    ..GapSeedHeuristic::default()
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
            }
        }
    }
}

#[test]
fn inexact_no_pruning() {
    for l in [6, 7] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost: 1,
                        ..MatchConfig::default()
                    },
                    pruning: false,
                    c: PhantomData::<BruteForceContours>,
                    ..GapSeedHeuristic::default()
                };
                let (a, b, alph, stats) = setup(n, e);
                //print(h, &a, &b, &alph);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
            }
        }
    }
}

#[test]
fn exact_pruning() {
    for l in [4, 5] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost: 0,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<BruteForceContours>,
                    ..GapSeedHeuristic::default()
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
            }
        }
    }
}

#[test]
fn inexact_pruning() {
    for l in [6, 7] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost: 1,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<BruteForceContours>,
                    ..GapSeedHeuristic::default()
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
            }
        }
    }
}

#[test]
fn incremental_pruning_bruteforce() {
    for (l, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<BruteForceContours>,
                    ..GapSeedHeuristic::default()
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                align(&a, &b, &alph, stats, h.equal_to_bruteforce_contours());
            }
        }
    }
}

#[test]
fn incremental_pruning_naive_naive() {
    for (l, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<NaiveContours<BruteForceContour>>,
                    ..GapSeedHeuristic::default()
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                align(&a, &b, &alph, stats, h.equal_to_bruteforce_contours());
            }
        }
    }
}

#[test]
fn incremental_pruning_naive_log() {
    for (l, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<NaiveContours<LogQueryContour>>,
                    ..GapSeedHeuristic::default()
                };
                let h_base = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<NaiveContours<BruteForceContour>>,
                    ..GapSeedHeuristic::default()
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                align(&a, &b, &alph, stats, EqualHeuristic { h1: h_base, h2: h });
            }
        }
    }
}

#[test]
fn incremental_pruning_naive_set() {
    for (l, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<NaiveContours<SetContour>>,
                    ..GapSeedHeuristic::default()
                };
                let h_base = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(l),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<NaiveContours<BruteForceContour>>,
                    ..GapSeedHeuristic::default()
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                align(&a, &b, &alph, stats, EqualHeuristic { h1: h_base, h2: h });
            }
        }
    }
}

#[test]
fn exact_pruning_bad_case() {
    let (l, m, n, e, pruning, prune_fraction) = (4, 0, 100, 0.3, true, 1.0);
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
    let (_a, _b, alph, stats) = setup(n, e);
    let a = "TCGTCCCAACTGCGTGCAGACGTCCTGAGGACGTGGTCGCGACGCTATAGGCAGGGTACATCGAGATGCCGCCTAAATGCGAACGTAGATTCGTTGTTCC".as_bytes().to_vec();
    let b = "TCAGTCCCACACTCCTAGCAGACGTTCCTGCAGGACAGTGGACGCTGACGCCTATAGGAGAGGCATCGAGGTGCCTCGCCTAAACGGGAACGTAGTTCGTTGTTC".as_bytes().to_vec();
    println!("TESTING n {} e {}: {:?}", n, e, h);
    println!("{}\n{}", to_string(&a), to_string(&b));
    align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
}
