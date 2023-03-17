use pa_affine_types::AffineCost;
use pa_base_algos::nw::NW;
use pa_generate::uniform_fixed;
use pa_heuristic::{MatchConfig, Pruning, CSH};
use pa_vis::visualizer::{self, Gradient, When};
use std::{path::PathBuf, time::Duration};

fn main() {
    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = uniform_fixed(n, e);

    let cm = AffineCost::unit();
    let mut config = visualizer::Config::default();
    config.draw = When::Layers;
    config.save = When::None;
    config.save_last = true;
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
    config.filepath = PathBuf::from("imgs/example/local-doubling/");

    let mut aligner = NW {
        cm,
        use_gap_cost_heuristic: false,
        exponential_search: false,
        local_doubling: true,
        h: CSH::new(MatchConfig::exact(5), Pruning::both()),
        v: config,
    };
    aligner.align(a, b);
}
