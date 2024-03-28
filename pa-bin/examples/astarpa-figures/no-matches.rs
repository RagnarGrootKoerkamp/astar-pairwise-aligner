use astarpa::AstarPa;
use pa_generate::uniform_fixed;
use pa_heuristic::{MatchConfig, Pruning, SH};
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::canvas::*;
use std::{path::PathBuf, time::Duration};

fn main() {
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::None;
    config.save_last = true;
    config.paused = false;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 12;
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
    config.style.draw_heuristic = true;
    //config.style.draw_layers = true;
    //config.style.draw_contours = true;
    config.style.draw_matches = true;
    config.style.contour = BLACK;
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;

    config.filepath = PathBuf::from("imgs/astarpa-paper/no-matches/");
    if !cfg!(feature = "example") {
        panic!("WARNING: Without the example feature, pruned matches aren't shown red for SH");
    }

    let n = 50;
    let k = 5;
    let (ref a, _) = uniform_fixed(n, 0.0);
    let mut b = a.clone();
    for i in (0..n).step_by(k as _) {
        b[i] = if a[i] == b'A' { b'C' } else { b'A' }
    }
    let b = &b;

    {
        let h = SH::new(MatchConfig::exact(k), Pruning::both());
        let a_star = AstarPa {
            dt: false,
            h,
            v: config.clone(),
        };
        a_star.align(a, b);
    }
}
