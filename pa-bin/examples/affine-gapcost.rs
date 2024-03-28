use astarpa::AstarPa;
use pa_heuristic::{AffineGapCost, BruteForceGCSH, GapCost, MatchConfig, Pruning, ZeroCost};
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::canvas::*;
use std::time::Duration;

fn main() {
    let mut config = visualizer::Config::default();
    config.draw = When::All;
    config.save = When::None;
    config.save_last = false;
    config.paused = true;
    config.delay = Duration::from_secs_f32(0.0001);
    //config.cell_size = 5;
    config.draw_old_on_top = false;
    config.style.bg_color = WHITE;
    config.style.expanded = Gradient::Fixed((130, 179, 102, 0));
    config.style.explored = Some((0, 102, 204, 0));
    config.style.heuristic = Gradient::Gradient((250, 250, 250, 0)..(180, 180, 180, 0));
    config.style.max_heuristic = Some(5);
    //config.style.max_layer = Some(6);
    config.style.pruned_match = RED;
    config.style.path = None;
    config.style.match_width = 3;
    config.style.draw_layers = false;
    config.style.draw_contours = true;
    config.style.draw_matches = true;
    config.style.contour = BLACK;
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;

    config.style.draw_heuristic = false;
    config.style.draw_parents = true;

    let k = 3;
    let n = 70;
    let (ref a, ref b) = pa_generate::uniform_fixed(n, 0.3);

    let h = BruteForceGCSH {
        match_config: MatchConfig::new(k, 1),
        distance_function: ZeroCost,
        pruning: Pruning::both(),
    };

    AstarPa {
        dt: false,
        h,
        v: config.clone(),
    }
    .align(a, b);

    let h = BruteForceGCSH {
        match_config: MatchConfig::new(k, 1),
        distance_function: GapCost,
        pruning: Pruning::both(),
    };

    AstarPa {
        dt: false,
        h,
        v: config.clone(),
    }
    .align(a, b);

    let h = BruteForceGCSH {
        match_config: MatchConfig::new(k, 1),
        distance_function: AffineGapCost { k },
        pruning: Pruning::both(),
    };

    AstarPa {
        dt: false,
        h,
        v: config.clone(),
    }
    .align(a, b);
}
