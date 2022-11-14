use std::marker::PhantomData;

use crate::{
    align::align_advanced,
    heuristic::{Heuristic, Pruning, CSH},
    matches::MatchConfig,
    prelude::{BruteForceContour, HintContours},
    visualizer::{Config, NoVisualizer, Visualizer, VisualizerStyle},
};

mod contours;

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

#[test]
fn bug_in_csh_contours() {
    let h = CSH {
        match_config: MatchConfig::new(3, 1),
        pruning: Pruning::new(true),
        use_gap_cost: true,
        c: PhantomData::<HintContours<BruteForceContour>>::default(),
    };

    let a = "CCCGTCGTCCCTCAAACTTGGAACCCCATCGCAAATCACCCCACAGGTAACGTCATAACTACCGCATGGTACGGTACCCCTTCTGCGATAGAGATGGTAGTAGCCGATAGGCCACCCTGGGAACACTATGTCACCCTGGTGGTAACCGTCGGGTCAGAAATAGGAGAACATACGGTGGACCGCTAA".as_bytes();
    let b = "CCCGTCGTACCTCTAAACTTGGAACCCACATCGCAAATCACCCCACAGGTAACGTCATAACTACCGCATGGTTCGGGTACCCCTTCGTGCGATAGAGATGGTAGTAGCCGATAGGCCACCCTGGGAACACTATGTCACCCTGGTGGTAACCGTCGGGTCAGAAATAGGAGTACATACGGTGGACCG".as_bytes();
    test_input(a, b, h);
}
