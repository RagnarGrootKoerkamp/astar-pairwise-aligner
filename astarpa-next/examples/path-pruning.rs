use astarpa_next::path_pruning::PathHeuristic;
use pa_affine_types::AffineCost;
use pa_base_algos::{
    nw::{AffineFront, NW},
    Domain,
};
use pa_generate::uniform_fixed;
use pa_heuristic::{MatchConfig, Pruning, GCSH};
use pa_vis::visualizer::{self, Gradient, When};
use std::{path::PathBuf, time::Duration};

fn main() {
    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = uniform_fixed(n, e);

    let cm = AffineCost::unit();
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::LayersStepBy(20);
    config.save_last = false;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 2;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path_width = Some(2);
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.transparent_bmp = true;
    config.draw_old_on_top = true;
    config.filepath = PathBuf::from("imgs/slides/");

    let aligner = NW {
        cm,
        strategy: pa_base_algos::Strategy::LocalDoubling,
        domain: Domain::Astar(PathHeuristic {
            h: GCSH::new(MatchConfig::exact(5), Pruning::both()),
        }),
        block_width: 1,
        v: config.with_filename("path-pruning"),
        front: AffineFront,
        trace: true,
        sparse_h: true,
        prune: true,
    };
    aligner.align(a, b);
}
