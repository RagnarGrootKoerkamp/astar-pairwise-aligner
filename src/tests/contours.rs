use crate::{heuristic::SH, prelude::*, visualizer::NoVis};

#[test]
fn exact_no_pruning_gap() {
    for k in [4, 5] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH {
                    match_config: MatchConfig::exact(k),
                    pruning: Pruning::default(),
                    use_gap_cost: true,
                    c: PhantomData::<BruteForceContours>,
                };
                let (a, b) = uniform_fixed(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_seed_heuristic(), &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
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
                    pruning: Pruning::default(),
                    use_gap_cost: true,
                    c: PhantomData::<BruteForceContours>,
                };
                let (a, b) = uniform_fixed(n, e);
                //print(h, &a, &b);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_seed_heuristic(), &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
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
                    pruning: Pruning::enabled(),
                    use_gap_cost: true,
                    c: PhantomData::<BruteForceContours>,
                };
                let (a, b) = uniform_fixed(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_seed_heuristic(), &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
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
                    pruning: Pruning::enabled(),
                    use_gap_cost: true,
                    c: PhantomData::<HintContours<BruteForceContour>>,
                };
                let (a, b) = uniform_fixed(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_bruteforce_contours(), &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
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
                    pruning: Pruning::default(),
                    use_gap_cost: false,
                    c: PhantomData::<BruteForceContours>,
                };
                let (a, b) = uniform_fixed(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_zero_cost_seed_heuristic(), &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
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
                    pruning: Pruning::default(),
                    use_gap_cost: false,
                    c: PhantomData::<BruteForceContours>,
                };
                let (a, b) = uniform_fixed(n, e);
                //print(h, &a, &b);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_zero_cost_seed_heuristic(), &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
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
                    pruning: Pruning::enabled(),
                    use_gap_cost: false,
                    c: PhantomData::<BruteForceContours>,
                };
                let (a, b) = uniform_fixed(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_zero_cost_seed_heuristic(), &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
            }
        }
    }
}

#[test]
fn pruning_hint_bruteforce_no_gap() {
    for (k, max_match_cost) in [(4, 0), (5, 0), (6, 1), (7, 1)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH {
                    match_config: MatchConfig::new(k, max_match_cost),
                    pruning: Pruning::enabled(),
                    use_gap_cost: false,
                    c: PhantomData::<HintContours<BruteForceContour>>,
                };
                let (a, b) = uniform_fixed(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_bruteforce_contours(), &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
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
                    pruning: Pruning::enabled(),
                };
                let (a, b) = uniform_fixed(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h, &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
            }
        }
    }
}
