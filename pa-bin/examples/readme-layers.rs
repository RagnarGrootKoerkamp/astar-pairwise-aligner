//! This generates the visualizations used in figure 1 in the paper and in the slides.
use astarpa::AstarPa;
use itertools::Itertools;
use pa_generate::uniform_fixed;
use pa_heuristic::{MatchConfig, Pruning, CSH};
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::canvas::*;
use std::time::Duration;

fn main() {
    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = uniform_fixed(n, e);

    let mut config = visualizer::Config::default();
    let frames = (0..10000).step_by(10).collect_vec();
    config.draw = When::None;
    config.save = When::Frames(frames);
    config.paused = false;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 1;
    config.style.bg_color = WHITE;
    //config.style.expanded = Gradient::Fixed((130, 179, 102, 0));
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.explored = Some((0, 102, 204, 0));
    config.style.heuristic = Gradient::Gradient((250, 250, 250, 0)..(180, 180, 180, 0));
    config.style.max_heuristic = Some(35);
    config.style.pruned_match = RED;
    config.style.path = Some(BLACK);
    config.style.match_width = 3;
    config.style.draw_heuristic = true;
    config.style.draw_contours = true;
    config.style.draw_matches = true;
    config.style.draw_f = false;
    config.style.draw_dt = false;
    config.style.match_width = 2;
    config.style.contour = BLACK;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    config.filepath = "imgs/readme/layers".into();
    {
        let k = 5;
        let h = CSH::new(MatchConfig::exact(k), Pruning::both());
        let a_star = AstarPa {
            dt: true,
            h,
            v: config.clone(),
        };
        a_star.align(a, b);
    }
}
