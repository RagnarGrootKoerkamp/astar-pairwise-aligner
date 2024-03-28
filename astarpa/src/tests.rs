//! Tests that test A*PA with various configurations.
use crate::AstarPa;
use pa_heuristic::*;
use pa_test::*;
use pa_types::*;
use pa_vis::NoVis;

macro_rules! make_test {
    // h is a function (exact: bool, pruning: bool) -> Heuristic.
    ($name:ident, $h:ty, $slow:expr, $m:expr) => {
        mod $name {
            use super::*;

            fn test_heuristic<H: Heuristic + 'static>(h: H, dt: bool) {
                test_aligner_up_to(
                    AstarPa { dt, h, v: NoVis },
                    if $slow { 100 } else { usize::MAX },
                );
            }

            fn match_config(exact: bool, k: I) -> MatchConfig {
                match (exact) {
                    true => MatchConfig::exact(k),
                    false => MatchConfig::inexact(k),
                }
            }

            fn h(exact: bool, prune: Prune, k: I) -> impl Heuristic {
                $m(<$h>::new(match_config(exact, k), Pruning::new(prune)))
            }

            // large k variants with mostly linear matches
            #[test]
            fn exact_noprune() {
                for k in [3, 5, 6] {
                    test_heuristic(h(true, Prune::None, k), false);
                }
            }
            #[test]
            fn exact_prune() {
                for k in [4, 5, 6] {
                    test_heuristic(h(true, Prune::Both, k), false);
                }
            }
            #[test]
            fn inexact_noprune() {
                for k in [4, 7, 9] {
                    test_heuristic(h(false, Prune::None, k), false);
                }
            }
            #[test]
            fn inexact_prune() {
                for k in [5, 7, 9] {
                    test_heuristic(h(false, Prune::Both, k), false);
                }
            }
            #[test]
            fn exact_noprune_dt() {
                for k in [3, 5, 6] {
                    test_heuristic(h(true, Prune::None, k), true);
                }
            }
            #[test]
            fn exact_prune_dt() {
                for k in [4, 5, 6] {
                    test_heuristic(h(true, Prune::Both, k), true);
                }
            }
            #[test]
            fn inexact_noprune_dt() {
                for k in [4, 7, 9] {
                    test_heuristic(h(false, Prune::None, k), true);
                }
            }
            #[test]
            fn inexact_prune_dt() {
                for k in [5, 7, 9] {
                    test_heuristic(h(false, Prune::Both, k), true);
                }
            }
        }
    };
}

mod dijkstra {
    use super::*;
    #[test]
    fn exact_noprune() {
        test_aligner(AstarPa {
            dt: false,
            h: NoCost,
            v: NoVis,
        });
    }
    #[test]
    fn exact_noprune_dt() {
        test_aligner(AstarPa {
            dt: true,
            h: NoCost,
            v: NoVis,
        });
    }
}

type CSH = DefaultCSH;
make_test!(sh, SH, false, |h| h);
make_test!(csh, CSH, false, |h| h);
make_test!(gch, GCSH, false, |h| h);

// These tests are very slow
make_test!(csh_bruteforce_contours, CSH, true, |h: CSH| h
    .equal_to_bruteforce_contours());
make_test!(csh_bruteforce_csh, CSH, true, |h: CSH| h
    .equal_to_bruteforce_csh());
make_test!(gch_bruteforce_contours, GCSH, true, |h: CSH| h
    .equal_to_bruteforce_contours());
make_test!(gch_bruteforce_gcsh, GCSH, true, |h: CSH| h
    .equal_to_bruteforce_gcsh());

mod edge_cases {
    use super::*;

    /// thread 'tests::bug_in_csh_contours' panicked at 'assertion failed: new_layer <= v', src/contour/hint_contours.rs:413:17
    /// This tests that hint contours only remove contours when at least `max_len + shift - 1` layers have shifted down by `shift`.
    /// Before it only checked for at least `max_len` layers, which is wrong.
    #[test]
    fn hint_contours_overly_greedy_shift() {
        let aligner = &mut AstarPa {
            dt: false,
            h: GCSH::new(MatchConfig::new(3, 2), Pruning::both()),
            v: NoVis,
        };

        let a = "CCCGTCGTCCCTCAAACTTGGAACCCCATCGCAAATCACCCCACAGGTAACGTCATAACTACCGCATGGTACGGTACCCCTTCTGCGATAGAGATGGTAGTAGCCGATAGGCCACCCTGGGAACACTATGTCACCCTGGTGGTAACCGTCGGGTCAGAAATAGGAGAACATACGGTGGACCGCTAA".as_bytes();
        let b = "CCCGTCGTACCTCTAAACTTGGAACCCACATCGCAAATCACCCCACAGGTAACGTCATAACTACCGCATGGTTCGGGTACCCCTTCGTGCGATAGAGATGGTAGTAGCCGATAGGCCACCCTGGGAACACTATGTCACCCTGGTGGTAACCGTCGGGTCAGAAATAGGAGTACATACGGTGGACCG".as_bytes();
        test_aligner_on_input(a, b, aligner, "");

        let a = "TTCCGACACTAGCTGTCAGCCTTATAACTCATGCCCTAGTATCAACAGGCC".as_bytes();
        let b = "TTTCCGACCACTAGCTAACTCATGTCCCAGTTCAACAGGCCGTGGGAC".as_bytes();
        test_aligner_on_input(a, b, aligner, "");

        aligner.h.match_config = MatchConfig::new(4, 2);
        let a = "ATATATATTAGCGGGCATTCGCCGACCTGGAAGTGCCAGGCCATTTCGTAGCAGTAGGTCCTCACCAAGGCCAGGCAAGTCGGTAGTAAAAT".as_bytes();
        let b = "ATATATATTAAGCTGGCCTATTCGCGACCTGCGAAGGGGCCAGGCATTTCCTATCAGTAGGTCCCTCACCAAAGCCAGGT"
            .as_bytes();
        test_aligner_on_input(a, b, aligner, "");
    }

    /// Since CSH is not consistent, f may go up while extending a greedy match.
    /// This means that we can not freely extend and prune expanded states in DT-A*.
    ///
    /// Fixed by a complete rewrite of DT-A*:
    /// - We now store normal Pos in the priority queue instead of DtPos.
    /// - Like normal A*, extending is done before pushing a state onto the priority queue.
    #[test]
    fn csh_dt_inconsistent_greedy() {
        let aligner = &mut AstarPa {
            dt: true,
            h: GCSH::new(MatchConfig::new(3, 2), Pruning::both()),
            v: NoVis,
        };

        let a = "GCCGCGCGCGCAGCCGCGCGCGCGCGCGCGCCGG".as_bytes();
        let b = "GCGCCAGCGCGCGCGGGCCGCCGGCGCGCGCGCT".as_bytes();
        test_aligner_on_input(a, b, aligner, "");

        let a = "TCTCTCTCTCTG".as_bytes();
        let b = "GTCTCTCTTCTG".as_bytes();
        test_aligner_on_input(a, b, aligner, "");
    }
}
