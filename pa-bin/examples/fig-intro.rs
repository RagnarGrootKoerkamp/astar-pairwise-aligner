//! This generates the visualizations used in figure 1 in the paper and in the slides.

use astarpa::AstarPa;
use pa_affine_types::{AffineAligner, AffineCost};
use pa_base_algos::{
    dt::{DiagonalTransition, GapCostHeuristic},
    nw::NW,
};
use pa_generate::uniform_fixed;
use pa_heuristic::{MatchConfig, NoCost, Pruning, CSH};
use pa_vis::visualizer::{self, Gradient, When};
use std::{path::PathBuf, time::Duration};

fn main() {
    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = uniform_fixed(n, e);

    let cm = AffineCost::unit();
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::Layers;
    config.save_last = true;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 4;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path = Some((128, 128, 128, 0));
    config.style.path_width = Some(4);
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.transparent_bmp = true;
    config.draw_old_on_top = true;
    config.filepath = PathBuf::from("imgs/paper/intro");

    let aligners: &mut [Box<dyn AffineAligner>] = &mut [
        Box::new(NW {
            cm,
            use_gap_cost_heuristic: true,
            exponential_search: true,
            local_doubling: false,
            h: NoCost,
            v: config.with_filename("1_ukkonen"),
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
            h: CSH::new(MatchConfig::exact(5), Pruning::enabled()),
            dt: true,
            v: config.with_filename("5_astar-csh-pruning"),
        }),
    ];
    for aligner in aligners {
        aligner.align(a, b);
    }
}
