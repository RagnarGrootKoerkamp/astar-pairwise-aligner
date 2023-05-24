//! Tests that compare CSH and GCSH heuristics using HintContours to equivalents
//! using BruteForceGCSH.
use crate::astar;
use pa_generate::uniform_fixed;
use pa_heuristic::*;
use pa_types::*;
use pa_vis_types::*;

#[test]
fn exact_no_pruning_gap() {
    for k in [4, 5] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GCSH::new(MatchConfig::exact(k), Pruning::disabled());
                let (a, b) = uniform_fixed(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_bruteforce_gcsh(), &NoVis);
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
                let h = GCSH::new(MatchConfig::inexact(k), Pruning::disabled());
                let (a, b) = uniform_fixed(n, e);
                //print(h, &a, &b);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_bruteforce_gcsh(), &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
            }
        }
    }
}

#[test]
fn pruning_bruteforce_gap() {
    for (k, r) in [(4, 1), (5, 1), (6, 2), (7, 2)] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = GCSH::new(MatchConfig::new(k, r), Pruning::both());
                let (a, b) = uniform_fixed(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_bruteforce_gcsh(), &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
            }
        }
    }
}

#[test]
fn pruning_hint_bruteforce_gap() {
    for (k, r) in [(4, 1), (5, 1), (6, 2), (7, 2)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = GCSH::new(MatchConfig::new(k, r), Pruning::both());
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
                let h = CSH::new(MatchConfig::exact(k), Pruning::disabled());
                let (a, b) = uniform_fixed(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_bruteforce_csh(), &NoVis);
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
                let h = CSH::new(MatchConfig::inexact(k), Pruning::disabled());
                let (a, b) = uniform_fixed(n, e);
                //print(h, &a, &b);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_bruteforce_csh(), &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
            }
        }
    }
}

#[test]
fn pruning_bruteforce() {
    for (k, r) in [(4, 1), (5, 1), (6, 2), (7, 2)] {
        for n in [40, 100, 200, 500] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH::new(MatchConfig::new(k, r), Pruning::both());
                let (a, b) = uniform_fixed(n, e);
                println!("TESTING n {} e {}: {:?}", n, e, h);
                let r = astar(&a, &b, &h.equal_to_bruteforce_csh(), &NoVis);
                let dist = bio::alignment::distance::simd::levenshtein(&a, &b) as Cost;
                assert_eq!(r.0 .0, dist);
            }
        }
    }
}

#[test]
fn pruning_hint_bruteforce_no_gap() {
    for (k, r) in [(4, 1), (5, 1), (6, 2), (7, 2)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = CSH::new(MatchConfig::new(k, r), Pruning::both());
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
    for (k, r) in [(4, 1), (5, 1), (6, 2), (7, 2)] {
        for n in [40, 100, 200, 500, 1000] {
            for e in [0.1, 0.3, 1.0] {
                let h = SH {
                    match_config: MatchConfig::new(k, r),
                    pruning: Pruning::both(),
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
