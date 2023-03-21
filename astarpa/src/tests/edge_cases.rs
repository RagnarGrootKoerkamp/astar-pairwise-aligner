//! List of tricky edge cases on which all aligners are tests.
use ::triple_accel::levenshtein;
use pa_types::{Cost, Seq};

use crate::AstarPa;
use pa_heuristic::*;
use pa_types::*;
use pa_vis_types::*;

fn test_sequences() -> Vec<(Seq<'static>, Seq<'static>)> {
    vec![
        (b"TTGGGTCAATCAGCCAGTTTTTA", b"TTTGAGTGGGTCATCACCGATTTTAT"),
        (b"ACTGACCAGT", b"CCGACAGGA"),
        (b"AGTTTTAT", b"ACCGATTTTTA"),
        (b"CTCTCTTCTCTCTCTA", b"CCTCTCTCTCTCCTCTC"),
        (b"AGTGGGTTGCCTTCATTCCG", b"AGTGGTGTCTTCAGGCCTTCATTCCG"),
        (b"GCACGTCGCCCCCCGCCCGCG", b"GCCCGCCCGCCCGCCCCCGCCCCC"),
        (b"CGCGTGTATCCGTCCACATCGAGCCGCCCTTGTTGCTTTTCGAGCGCTCATTTCCCGCAAGAGTGGCGTGCGGTCACTTTCGCGCAGCAATTAGAGTACTAACGGGTAGACGTGGCTTTCCTCCTCGTCCTGTCAACGCGCATAGGATGTCCTGCAGCAGGCCGCCGCGATTGCCTAAATCAAGGGGTTCCAATGGAGTTTCCATCTGATATCCGCGCTCCGGTTCTGAGTCTAAAGTGGAAATACTCCGAATGGGCCGGTATGAGGTTGGGTCAATCAGCCAGTTTTTA",
         b"CGCTGGGGATGCCTCCACCTTTCGAGTGCCTGTTGGTTCCGACGCTATCATAGTCCCCATGCAAGGAGATGGCTGCGCGTCCTATCGCGCGGCAAATAGAGTCTACGGGGGCGGCTGTCCTCCTCGTCCTGGTCAACGGCCATAGGATTTCCGCGATGGTCGCCCGGATGTGCCTAAACCAAGGCTCCGATGGAGCTGCCTCTGATATCCGCGCTGCCGGTTTCCTGACGTCTGAAAACGTTGGAAAATACCTCCGAATGGGCCCCGTTTGAGTGGGTCATCACCGATTTTAT"),
    ]
}

fn test_heuristic<H: Heuristic + 'static>(h: H, dt: bool) {
    let aligner = AstarPa { dt, h, v: NoVis };
    const D: bool = true;
    for (a, b) in test_sequences() {
        if D {
            eprintln!("{aligner:?}");
            eprintln!("a {}\nb {}", seq_to_string(a), seq_to_string(b));
        }
        let nw_cost = levenshtein(a, b) as Cost;
        let (cost, cigar) = aligner.align(a, b).0;
        assert_eq!(
            nw_cost,
            cost,
            "\nlet a = \"{}\".as_bytes();\nlet b = \"{}\".as_bytes();\nAligner\n{aligner:?}",
            seq_to_string(&a),
            seq_to_string(&b),
        );
        cigar.verify(&CostModel::unit(), a, b);
    }
}

fn test_heuristics<H: Heuristic + 'static, F: Fn(Prune, bool, I) -> H>(
    h_for_k: F,
    prune: Prune,
    exact: bool,
    dt: bool,
) {
    for k in 3..=15 {
        test_heuristic(h_for_k(prune, exact, k), dt);
    }
}

macro_rules! make_test {
    // h is a function (exact: bool, pruning: bool) -> Heuristic.
    ($name:ident, $h:expr) => {
        mod $name {
            use super::*;
            // large k variants with mostly linear matches
            #[test]
            fn exact_noprune() {
                test_heuristics($h, Prune::None, true, false);
            }
            #[test]
            fn exact_prune() {
                test_heuristics($h, Prune::Both, true, false);
            }
            #[test]
            fn inexact_noprune() {
                test_heuristics($h, Prune::None, false, false);
            }
            #[test]
            fn inexact_prune() {
                test_heuristics($h, Prune::Both, false, false);
            }
            #[test]
            fn exact_noprune_dt() {
                test_heuristics($h, Prune::None, true, true);
            }
            #[test]
            fn exact_prune_dt() {
                test_heuristics($h, Prune::Both, true, true);
            }
            #[test]
            fn inexact_noprune_dt() {
                test_heuristics($h, Prune::None, false, true);
            }
            #[test]
            fn inexact_prune_dt() {
                test_heuristics($h, Prune::Both, false, true);
            }
        }
    };
}

fn match_config(k: I, exact: bool) -> MatchConfig {
    match exact {
        true => MatchConfig::exact(k),
        false => MatchConfig::inexact(k),
    }
}

make_test!(sh, |prune, exact, k| SH::new(
    match_config(k, exact),
    Pruning::new(prune)
));

// The following should all be equal:
// CSH<HintContours>
// CSH<BruteforceContours>
// BruteforceCSH
make_test!(csh_contours, |prune, exact, k| CSH::new(
    match_config(k, exact),
    Pruning::new(prune)
)
.equal_to_bruteforce_contours());
make_test!(csh, |prune, exact, k| CSH::new(
    match_config(k, exact),
    Pruning::new(prune)
)
.equal_to_bruteforce_csh());

make_test!(gcsh_contours, |prune, exact, k| GCSH::new(
    match_config(k, exact),
    Pruning::new(prune)
)
.equal_to_bruteforce_contours());
make_test!(gcsh, |prune, exact, k| GCSH::new(
    match_config(k, exact),
    Pruning::new(prune)
)
.equal_to_bruteforce_gcsh());
