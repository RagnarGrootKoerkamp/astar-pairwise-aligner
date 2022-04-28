use pairwise_aligner::prelude::{UnorderedHeuristic, *};

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
                    use_gap_cost: true,
                    c: PhantomData::<BruteForceContours>,
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
                assert_eq!(r.edit_distance, dist);
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
                    use_gap_cost: true,
                    c: PhantomData::<BruteForceContours>,
                };
                let (a, b, alph, stats) = setup(n, e);
                //print(h, &a, &b, &alph);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
                assert_eq!(r.edit_distance, dist);
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
                    use_gap_cost: true,
                    c: PhantomData::<BruteForceContours>,
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = align(&a, &b, &alph, stats, h.equal_to_seed_heuristic());
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
                assert_eq!(r.edit_distance, dist);
            }
        }
    }
}

#[test]
fn incremental_pruning_hint_bruteforce() {
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
                    use_gap_cost: true,
                    c: PhantomData::<HintContours<BruteForceContour>>,
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = align(&a, &b, &alph, stats, h.equal_to_bruteforce_contours());
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
                assert_eq!(r.edit_distance, dist);
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
                    use_gap_cost: true,
                    c: PhantomData::<HintContours<CentralContour>>,
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = align(&a, &b, &alph, stats, h.equal_to_bruteforce_contours());
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
                assert_eq!(r.edit_distance, dist);
            }
        }
    }
}

#[test]
fn unordered() {
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = UnorderedHeuristic {
                    match_config: MatchConfig {
                        length: Fixed(k),
                        max_match_cost,
                        ..MatchConfig::default()
                    },
                    pruning: true,
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = align(&a, &b, &alph, stats, h);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
                assert_eq!(r.edit_distance, dist);
            }
        }
    }
}

#[test]
fn no_gap_cost() {
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
                    use_gap_cost: false,
                    c: PhantomData::<HintContours<CentralContour>>,
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = align(&a, &b, &alph, stats, h);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
                assert_eq!(r.edit_distance, dist);
            }
        }
    }
}
