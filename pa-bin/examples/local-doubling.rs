#![allow(unused)]
use astarpa::AstarPa;
use pa_affine_types::AffineCost;
use pa_base_algos::{
    nw::{AffineFront, BitFront, NW},
    Domain, Strategy,
};
use pa_generate::{uniform_fixed, uniform_seeded};
use pa_heuristic::{MatchConfig, Pruning, GCSH};
use pa_types::Sequence;
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::canvas::RED;
use std::{path::PathBuf, time::Duration};

fn complex_ab(scale: usize) -> (Sequence, Sequence) {
    // 200 4%
    let (mut a, mut b) = uniform_seeded(200 * scale, 0.04, 1);
    // 200 Noisy region
    let (mut x, mut y) = uniform_seeded(200 * scale, 0.60, 2);
    a.append(&mut x);
    b.append(&mut y);
    // 100 4%
    let (mut x, mut y) = uniform_seeded(25 * scale, 0.04, 3);
    a.append(&mut x);
    b.append(&mut y);
    // 2x75 repeat @4%, with same seed
    let (mut x, mut y) = uniform_seeded(150 * scale, 0.04, 4);
    a.append(&mut x);
    b.append(&mut y);
    let (mut x, mut y) = uniform_seeded(150 * scale, 0.05, 4);
    a.append(&mut x);
    b.append(&mut y);
    // 100 8%
    let (mut x, mut y) = uniform_seeded(25 * scale, 0.08, 5);
    a.append(&mut x);
    b.append(&mut y);
    // 75 Gap
    let (mut x, _y) = uniform_seeded(75 * scale, 0.08, 6);
    a.append(&mut x);
    // 175 8%
    let (mut x, mut y) = uniform_seeded(175 * scale, 0.08, 7);
    a.append(&mut x);
    b.append(&mut y);

    (a, b)
}

fn main() {
    let n = 50000;
    let e = 0.20;
    let (ref a, ref b) = uniform_fixed(n, e);
    let (ref a, ref b) = complex_ab(16);
    eprintln!("a: {}\nb: {}", a.len(), b.len());

    let cm = AffineCost::unit();
    let mut config = visualizer::Config::default();
    config.draw = When::LayersStepBy(1);
    config.save = When::None; //When::LayersStepBy(30);
    config.save_last = false;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 1;
    config.downscaler = 16;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path_width = None;
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_heuristic = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.transparent_bmp = true;
    config.draw_old_on_top = true;
    config.paused = true;
    config.filepath = PathBuf::from("imgs/slides/");

    config.style.pruned_match = RED;
    config.style.match_width = 1;
    config.style.draw_matches = true;

    let mut aligner = NW {
        cm,
        strategy: Strategy::BandDoubling {
            start: pa_base_algos::DoublingStart::H0,
            factor: 2.,
        },
        domain: Domain::Astar(GCSH::new(MatchConfig::exact(5), Pruning::start())),
        block_width: 256,
        v: config.with_filename("local-doubling"),
        front: BitFront::default(),
        trace: true,
        sparse_h: true,
        prune: true,
    };
    aligner.align(a, b);
    aligner.strategy = Strategy::LocalDoubling;
    aligner.align(a, b);
}
