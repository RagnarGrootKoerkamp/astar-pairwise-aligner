use astarpa::AstarPa;
use itertools::Itertools;
use pa_bitpacking::search;
use pa_heuristic::contour::rotate_to_front::RotateToFrontContour;
use pa_heuristic::contour::{BruteForceContour, BruteForceContours, HintContours};
use pa_heuristic::{MatchConfig, Prune, Pruning, CSH, GCSH, SH};
use pa_types::I;
use pa_vis::canvas::*;
use pa_vis::visualizer::{self, Gradient, When};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::None;
    config.save_last = false;
    config.paused = false;
    config.delay = Duration::from_secs_f32(0.0000);
    config.cell_size = 6;
    config.draw_old_on_top = false;
    config.style.bg_color = WHITE;
    config.style.expanded = Gradient::Fixed((130, 179, 102, 0));
    config.style.explored = Some((0, 102, 204, 0));
    config.style.heuristic = Gradient::Gradient((250, 250, 250, 0)..(180, 180, 180, 0));
    config.style.max_heuristic = Some(10);
    config.style.max_layer = Some(6);
    config.style.pruned_match = RED;
    config.style.path = None;
    config.style.match_width = 3;
    config.style.draw_heuristic = false;
    config.style.draw_layers = false;
    config.style.draw_contours = false;
    config.style.draw_matches = true;
    config.style.contour = BLACK;
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;

    config.filepath = PathBuf::from("imgs/semi-global/layers/");

    if !cfg!(feature = "example") {
        panic!("WARNING: Without the example feature, pruned matches aren't shown red for SH");
    }

    for _ in 0..10000 {
        let a_len = rand::random_range(0..20);
        let b_len = rand::random_range(0..20);
        let rate = rand::random_range(0.0..0.10);
        let (ref b, b2) = pa_generate::uniform_random(b_len, rate);
        let mut a = pa_generate::random_seq(a_len);
        let splice_1 = rand::random_range(0..=a_len);
        a.splice(splice_1..splice_1, b2.iter().copied());

        let splice_1 = rand::random_range(0..=a_len);
        a.splice(splice_1..splice_1, b2.iter().copied());

        let a = &a;
        let k = rand::random_range(1..20);
        let lp = rand::random_range(0..=4);

        // config.draw = When::All;
        // config.style.draw_heuristic = true;
        // let a = b"ACCCCGCAACATTCCGTGGAGGACAAGCACTAAATTTGCGTGGGACGAAGTTCGCACGTTCGCTATTGCGAGGTTTGTGCCTCACAGCCTGCTTCTACCGCTGGCGATGACTGGCATGTCAAGAAAGGACAAGCACTAAATTTGCGTGGGACGAAGTTCGCACGTTCGCTATTGCGAGGTTTGTGCCTCACAGCCTGCTTCTACCGCTGGCGATGACTGGCATGTCAAGAAGGCACTAGGGTATATCCGGGGCAAAACCCAAGCGCCTTCATTCCTCTTGCTCTATGCTTTTCATTCCTCACAGCCTAC";
        // let b = b"GAGGACAAGCACTAAATTTGACGTGGGACGAAGTTCGCACGTTCGCTATTGCGAGGTTTGTGCCTCACAGCCTGCTTCTACCGCTGGCGATGACTGGCATGTCAAGAA";
        // let k = 17;
        // let lp = 10;

        eprintln!("let a = b\"{}\";", String::from_utf8_lossy(a));
        eprintln!("let b = b\"{}\";", String::from_utf8_lossy(b));
        eprintln!("let k = {};", k);
        eprintln!("let lp = {};", lp);

        for pruning in [/*Prune::None,*/ Prune::Both] {
            let suf = if pruning.is_enabled() { "" } else { "-noprune" };
            // AstarPa {
            //     dt: false,
            //     h: SH::new(MatchConfig::exact(k), Pruning::new(pruning)),
            //     v: config.with_filename(&("sh".to_string() + suf)),
            // }
            // .align(a, b);
            // AstarPa {
            //     dt: false,
            //     h: CSH::new(MatchConfig::exact(k), Pruning::new(pruning)),
            //     v: config.with_filename(&("csh".to_string() + suf)),
            // }
            // .align(a, b);

            let mut match_config = MatchConfig::exact(k);
            match_config.local_pruning = lp;

            let pruning1 = Pruning::new(pruning);

            let h = CSH {
                match_config,
                pruning: pruning1,
                use_gap_cost: true,
                // c: PhantomData::<BruteForceContours>,
                c: PhantomData::<HintContours<BruteForceContour>>,
                // c: PhantomData::<HintContours<RotateToFrontContour>>,
            };
            let ((dist, _cigar), _stats) = AstarPa {
                dt: true,
                // h: GCSH::new(match_config, pruning1),
                h,
                v: config.with_filename(&("gcsh".to_string() + suf)),
            }
            .align(a, b);

            let ds = search(b, a, 1.0);
            let dist2 = *ds.out.iter().min().unwrap();
            let i2 = ds.out.iter().position_min().unwrap();
            let (cigar, path) = ds.trace(i2);
            // eprintln!("{:?}", &ds.out[i2..i2 + 10]);
            eprintln!("Dist {dist} dist 2 {dist2}");
            eprintln!("Pos2 {i2}");
            eprintln!("Path {} {}", path.first().unwrap(), path.last().unwrap());
            eprintln!("Cigar {}", cigar.to_string());

            // eprintln!("k {k}");
            assert_eq!(dist, dist2);
        }
    }
}

/*
TCAAACGGGGCAGCTCTGTAGACTCCAAGGTCGAGGATGAGCTCACTTCAAGGTGCTGATGGG**ATAAGGGGATGACCAAGTACCTGTTGAAGGGGAAGGCCTTCGTGTTGCCAACCTACAAAGCGATCCTCAACGATACGGGAGCTTTTTTAGCATCAAGGGAGTCACATATTTCCGACGACTTCCAGCTCCGTACGG";
                                                     *
                                                  AGGCGCTGATGGGCTATAAGGGGATGACCAAGTACCTGTTGAAGGGGAAG";
*/
