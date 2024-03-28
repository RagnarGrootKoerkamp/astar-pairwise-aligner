use astarpa::AstarPa;
use pa_heuristic::{MatchConfig, Prune, Pruning, CSH, GCSH, SH};
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::canvas::*;
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    let a = b"ACTCAGCTGTTGCCCGCTGTCGATCCGTAATTTAAAGTAGGTCGAAAC";
    let b = b"ACTCAACGTTGCGCCTGTCTATCGTAATTAAAGTGGAGAAAC";

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
    config.style.max_layer = Some(20);
    config.style.pruned_match = RED;
    config.style.path = None;
    config.style.match_width = 3;
    config.style.draw_heuristic = false;
    config.style.draw_layers = true;
    config.style.draw_contours = true;
    config.style.draw_matches = true;
    config.style.contour = BLACK;
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;

    config.filepath = PathBuf::from("imgs/astarpa2-paper/layers/");

    if !cfg!(feature = "example") {
        panic!("WARNING: Without the example feature, pruned matches aren't shown red for SH");
    }

    let mut astarpa2 = astarpa2::AstarPa2Params::full();
    astarpa2.heuristic.k = 3;

    let k = 2;
    let mut match_config = MatchConfig {
        length: pa_heuristic::LengthConfig::Fixed(k),
        r: 1,
        local_pruning: 0,
    };
    let pruning = Prune::None;
    for p in [0, 5] {
        match_config.local_pruning = p;
        let suf = if p > 0 { "-lp" } else { "" };
        AstarPa {
            dt: false,
            h: SH::new(match_config, Pruning::new(pruning)),
            v: config.with_filename(&("sh".to_string() + suf)),
        }
        .align(a, b);
        AstarPa {
            dt: false,
            h: CSH::new(match_config, Pruning::new(pruning)),
            v: config.with_filename(&("csh".to_string() + suf)),
        }
        .align(a, b);
        AstarPa {
            dt: false,
            h: GCSH::new(match_config, Pruning::new(pruning)),
            v: config.with_filename(&("gcsh".to_string() + suf)),
        }
        .align(a, b);
    }
}
