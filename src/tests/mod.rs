use std::marker::PhantomData;

use crate::{
    align::align_advanced,
    heuristic::{Heuristic, Pruning, CSH},
    matches::MatchConfig,
    prelude::{BruteForceContour, HintContours},
    visualizer::{Config, NoVisualizer, Visualizer, VisualizerStyle},
};

mod contours;

#[allow(unused)]
fn test_input(a: &[u8], b: &[u8], h: impl Heuristic) {
    align_advanced(
        &a,
        &b,
        Default::default(),
        h,
        true,
        false,
        &mut NoVisualizer,
    );
}

#[allow(unused)]
fn test_input_and_visualize(a: &[u8], b: &[u8], h: impl Heuristic) {
    align_advanced(
        &a,
        &b,
        Default::default(),
        h,
        true,
        false,
        &mut Visualizer::new(Config::new(VisualizerStyle::Test), a, b),
    );
}

/// thread 'tests::bug_in_csh_contours' panicked at 'assertion failed: new_layer <= v', src/contour/hint_contours.rs:413:17
#[test]
fn bug_in_csh_contours_1() {
    let h = CSH {
        match_config: MatchConfig::new(3, 1),
        pruning: Pruning::new(true),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>::default(),
    }
    .equal_to_bruteforce_contours();

    let a = "CCCGTCGTCCCTCAAACTTGGAACCCCATCGCAAATCACCCCACAGGTAACGTCATAACTACCGCATGGTACGGTACCCCTTCTGCGATAGAGATGGTAGTAGCCGATAGGCCACCCTGGGAACACTATGTCACCCTGGTGGTAACCGTCGGGTCAGAAATAGGAGAACATACGGTGGACCGCTAA".as_bytes();
    let b = "CCCGTCGTACCTCTAAACTTGGAACCCACATCGCAAATCACCCCACAGGTAACGTCATAACTACCGCATGGTTCGGGTACCCCTTCGTGCGATAGAGATGGTAGTAGCCGATAGGCCACCCTGGGAACACTATGTCACCCTGGTGGTAACCGTCGGGTCAGAAATAGGAGTACATACGGTGGACCG".as_bytes();
    test_input_and_visualize(a, b, h);
}

#[test]
fn bug_in_csh_contours_2() {
    let h = CSH {
        match_config: MatchConfig::new(4, 1),
        pruning: Pruning::new(true),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>::default(),
    }
    .equal_to_bruteforce_contours();

    let a = "ATATATATTAGCGGGCATTCGCCGACCTGGAAGTGCCAGGCCATTTCGTAGCAGTAGGTCCTCACCAAGGCCAGGCAAGTCGGTAGTAAAAT".as_bytes();
    let b = "ATATATATTAAGCTGGCCTATTCGCGACCTGCGAAGGGGCCAGGCATTTCCTATCAGTAGGTCCCTCACCAAAGCCAGGT"
        .as_bytes();
    test_input_and_visualize(a, b, h);
}

#[test]
fn bug_in_csh_contours_3() {
    let h = CSH {
        match_config: MatchConfig::new(3, 1),
        pruning: Pruning::new(true),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>::default(),
    };
    //.equal_to_bruteforce_contours();

    let a = "TTCCGACACTAGCTGTCAGCCTTATAACTCATGCCCTAGTATCAACAGGCC".as_bytes();
    let b = "TTTCCGACCACTAGCTAACTCATGTCCCAGTTCAACAGGCCGTGGGAC".as_bytes();
    test_input_and_visualize(a, b, h);
}
