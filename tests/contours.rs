use pairwise_aligner::prelude::*;

#[test]
fn exact_no_pruning() {
    for k in [4, 5] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
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
    for k in [6, 7] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
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
    for k in [4, 5] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
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
    for k in [6, 7] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
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
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
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
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
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
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<NaiveContours<LogQueryContour>>,
                    ..GapSeedHeuristic::default()
                };
                let h_base = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
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
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<NaiveContours<SetContour>>,
                    ..GapSeedHeuristic::default()
                };
                let h_base = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
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
fn incremental_pruning_naive_central() {
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<NaiveContours<CentralContour>>,
                    ..GapSeedHeuristic::default()
                };
                let h_base = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
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
fn incremental_pruning_hint_central() {
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<HintContours<CentralContour>>,
                    ..GapSeedHeuristic::default()
                };
                let h_base = GapSeedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                    c: PhantomData::<NaiveContours<CentralContour>>,
                    ..GapSeedHeuristic::default()
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                align(&a, &b, &alph, stats, EqualHeuristic { h1: h_base, h2: h });
            }
        }
    }
}
