use astar_pairwise_aligner::prelude::{SH, *};

#[test]
fn exact_no_pruning_gap() {
    for k in [4, 5] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH {
                    match_config: MatchConfig::exact(k),
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
fn inexact_no_pruning_gap() {
    for k in [6, 7] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH {
                    match_config: MatchConfig::inexact(k),
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
fn pruning_bruteforce_gap() {
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH {
                    match_config: MatchConfig::new(k, max_match_cost),
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
fn pruning_hint_bruteforce_gap() {
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH {
                    match_config: MatchConfig::new(k, max_match_cost),
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
fn pruning_hint_central_gap() {
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH {
                    match_config: MatchConfig::new(k, max_match_cost),
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
fn exact_no_pruning() {
    for k in [4, 5] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH {
                    match_config: MatchConfig::exact(k),
                    pruning: false,
                    use_gap_cost: false,
                    c: PhantomData::<BruteForceContours>,
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = align(&a, &b, &alph, stats, h.equal_to_zero_cost_seed_heuristic());
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
                let h = CSH {
                    match_config: MatchConfig::inexact(k),
                    pruning: false,
                    use_gap_cost: false,
                    c: PhantomData::<BruteForceContours>,
                };
                let (a, b, alph, stats) = setup(n, e);
                //print(h, &a, &b, &alph);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = align(&a, &b, &alph, stats, h.equal_to_zero_cost_seed_heuristic());
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
                assert_eq!(r.edit_distance, dist);
            }
        }
    }
}

#[test]
fn pruning_bruteforce() {
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH {
                    match_config: MatchConfig::new(k, max_match_cost),
                    pruning: true,
                    use_gap_cost: false,
                    c: PhantomData::<BruteForceContours>,
                };
                let (a, b, alph, stats) = setup(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = align(&a, &b, &alph, stats, h.equal_to_zero_cost_seed_heuristic());
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b);
                assert_eq!(r.edit_distance, dist);
            }
        }
    }
}

#[test]
fn pruning_hint_bruteforce() {
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH {
                    match_config: MatchConfig::new(k, max_match_cost),
                    pruning: true,
                    use_gap_cost: false,
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
fn pruning_hint_central() {
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH {
                    match_config: MatchConfig::new(k, max_match_cost),
                    pruning: true,
                    use_gap_cost: false,
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
                let h = SH {
                    match_config: MatchConfig::new(k, max_match_cost),
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
