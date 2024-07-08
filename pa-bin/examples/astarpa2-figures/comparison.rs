#![feature(trait_upcasting)]
//! This generates the visualizations used in the limitations section of the paper.

use astarpa::{astar, AstarPa, HeuristicParams};
use astarpa2::*;
use pa_affine_types::AffineCost;
use pa_base_algos::{
    dt::{DiagonalTransition, GapCostHeuristic},
    nw::{AffineFront, NW},
};
use pa_generate::uniform_seeded;
use pa_heuristic::{MatchConfig, NoCost, Pruning, GCSH};
use pa_types::Aligner;
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::NoVis;
use std::time::Duration;

fn main() {
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::None;
    config.save_last = true;
    config.delay = Duration::from_secs_f32(0.0001);
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.9);
    config.style.path_width = Some(3);
    config.draw_old_on_top = true;
    config.layer_drawing = false;
    config.transparent_bmp = false;
    let scale = 10;
    //config.downscaler = scale as u32;
    config.downscaler = 5;
    config.cell_size = 1;
    config.style.draw_matches = true;
    config.style.match_width = 5;
    config.style.match_shrink = 0;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.transparent_bmp = true;

    config.filepath = "imgs/astarpa2-paper/comparison".into();

    // 50 3%
    let (mut a, mut b) = uniform_seeded(100 * scale, 0.03, 1);
    // 150 Noisy region
    let (mut x, mut y) = uniform_seeded(100 * scale, 0.40, 2);
    a.append(&mut x);
    b.append(&mut y);
    // 100 4%
    let (mut x, mut y) = uniform_seeded(125 * scale, 0.04, 3);
    a.append(&mut x);
    b.append(&mut y);
    // 75 Gap
    let (mut x, _y) = uniform_seeded(50 * scale, 0.08, 8);
    b.append(&mut x);
    // 100 4%
    let (mut x, mut y) = uniform_seeded(25 * scale, 0.04, 9);
    a.append(&mut x);
    b.append(&mut y);
    // 2x75 repeat @4%, with same seed
    let (mut x, mut y) = uniform_seeded(100 * scale, 0.10, 4);
    a.append(&mut x);
    b.append(&mut y);
    let (mut x, mut y) = uniform_seeded(100 * scale, 0.12, 4);
    a.append(&mut x);
    b.append(&mut y);
    // 50 3%
    let (mut x, mut y) = uniform_seeded(50 * scale, 0.03, 3);
    a.append(&mut x);
    b.append(&mut y);
    // another repeat
    let (mut x, mut y) = uniform_seeded(50 * scale, 0.04, 11);
    a.append(&mut x);
    b.append(&mut y);
    let (mut x, mut y) = uniform_seeded(50 * scale, 0.05, 11);
    a.append(&mut x);
    b.append(&mut y);
    // 100 8%
    let (mut x, mut y) = uniform_seeded(25 * scale, 0.08, 5);
    a.append(&mut x);
    b.append(&mut y);
    // 75 Gap
    let (mut x, _y) = uniform_seeded(50 * scale, 0.08, 6);
    a.append(&mut x);
    // 175 6%
    let (mut x, mut y) = uniform_seeded(175 * scale, 0.06, 7);
    a.append(&mut x);
    b.append(&mut y);
    // 150 3%
    let (mut x, mut y) = uniform_seeded(50 * scale, 0.03, 10);
    a.append(&mut x);
    b.append(&mut y);

    let a = &a;
    let b = &b;

    let cost = astar(&a, &b, &NoCost, &NoVis).0 .0;
    println!("{} {}", a.len(), b.len());
    println!("cost {cost}");

    let block_params = AstarPa2Params {
        name: "simple".into(),
        domain: Domain::Astar(()),
        heuristic: HeuristicParams {
            heuristic: pa_heuristic::HeuristicType::Gap,
            ..Default::default()
        },
        doubling: DoublingType::BandDoubling {
            start: DoublingStart::H0,
            factor: 2.0,
        },
        block_width: 1,
        front: astarpa2::BlockParams {
            sparse: true,
            simd: false,
            no_ilp: true,
            incremental_doubling: false,
            dt_trace: false,
            ..astarpa2::BlockParams::default()
        },
        sparse_h: false,
        prune: false,
        viz: false,
    };

    let cm = AffineCost::unit();

    let aligners: &mut [Box<dyn Aligner>] = &mut [
        Box::new(NW {
            cm,
            strategy: pa_base_algos::Strategy::band_doubling(),
            domain: pa_base_algos::Domain::gap_gap(),
            block_width: 1,
            v: config.with_filename("0_gap-gap"),
            front: AffineFront,
            trace: true,
            sparse_h: false,
            prune: false,
        }),
        block_params.make_aligner_with_visualizer(true, config.with_filename("0_bitpacking")),
        Box::new(AstarPa {
            dt: false,
            h: NoCost,
            v: config.with_filename("2_dijkstra"),
        }),
        Box::new(DiagonalTransition::new(
            cm,
            GapCostHeuristic::Disable,
            NoCost,
            false,
            config.with_filename("3_diagonal-transition"),
        )),
        Box::new(AstarPa {
            h: GCSH::new(MatchConfig::exact(6), Pruning::both()),
            dt: true,
            v: config.with_filename("5_astarpa-prune"),
        }),
        astarpa2::AstarPa2Params::simple()
            .make_aligner_with_visualizer(true, config.with_filename("6_astarpa2_simple")),
        {
            let mut params = astarpa2::AstarPa2Params::full();
            params.heuristic.k = 6;
            params
        }
        .make_aligner_with_visualizer(true, config.with_filename("7_astarpa2_full")),
    ];
    for aligner in aligners {
        aligner.align(a, b);
    }
}
