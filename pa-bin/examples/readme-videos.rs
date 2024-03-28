#![feature(trait_upcasting)]
//! This generates the visualizations used in figure 1 in the paper and in the slides.

use astarpa::{astar, AstarPa};
use astarpa2::AstarPa2Params;
use pa_affine_types::AffineCost;
use pa_base_algos::{
    dt::{DiagonalTransition, GapCostHeuristic},
    nw::{AffineFront, NW},
    Domain,
};
use pa_generate::uniform_fixed;
use pa_heuristic::{MatchConfig, NoCost, Pruning, GCSH};
use pa_types::Aligner;
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::NoVis;
use std::{path::PathBuf, time::Duration};

fn main() {
    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = uniform_fixed(n, e);
    eprintln!("Length {}", a.len());
    let cost = astar(&a, &b, &NoCost, &NoVis).0 .0;
    eprintln!("Distance {cost}");
    eprintln!("Divergence {}", cost as f32 / a.len() as f32);

    if !cfg!(feature = "example") {
        panic!("WARNING: Without the example feature, pruned matches aren't shown red for SH");
    }
    if pa_bitpacking::B::BITS != 8 {
        panic!("small_blocks feature is required for useful scale");
    }

    let cm = AffineCost::unit();
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::Layers;
    config.save_last = false;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 1;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path = Some((0, 0, 0, 0));
    config.style.path_width = None;
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.transparent_bmp = true;
    config.draw_old_on_top = true;
    config.filepath = PathBuf::from("imgs/readme/");
    config.clear_after_meeting_point = true;

    let aligners: &mut [Box<dyn Aligner>] = &mut [
        Box::new(NW {
            cm,
            strategy: pa_base_algos::Strategy::band_doubling(),
            domain: Domain::gap_gap(),
            block_width: 1,
            v: config.with_filename("1_ukkonen"),
            front: AffineFront,
            trace: true,
            sparse_h: false,
            prune: false,
        }),
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
        Box::new(DiagonalTransition::new(
            cm,
            GapCostHeuristic::Disable,
            NoCost,
            true,
            config.with_filename("4_dt-divide-and-conquer"),
        )),
        Box::new(AstarPa {
            h: GCSH::new(MatchConfig::exact(5), Pruning::both()),
            dt: true,
            v: config.with_filename("5_astarpa"),
        }),
        {
            let mut config = config.with_filename("6_astarpa2");
            config.save = When::All;
            config.draw_old_on_top = false;
            let mut params = AstarPa2Params::full();
            params.block_width = 32;
            params.make_aligner_with_visualizer(true, config)
        },
    ];
    for aligner in aligners {
        aligner.align(a, b);
    }
}
