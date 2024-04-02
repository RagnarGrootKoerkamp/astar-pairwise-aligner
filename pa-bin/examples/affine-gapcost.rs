use pa_heuristic::{
    AffineGapSeedCost, BruteForceGCSH, Heuristic, MatchConfig, Pruning, SimpleAffineCost,
};
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::{canvas::*, VisualizerInstance, VisualizerT};
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
    config.style.contour = BLACK;
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.style.draw_matches = true;

    // config.style.draw_heuristic = true;
    // config.style.draw_layers = true;
    config.style.draw_contours = true;

    let k = 3;
    let n = 30;
    let c = SimpleAffineCost {
        sub: 1,
        open: 1,
        extend: 1,
    };

    for parents in [true, false] {
        for formula in [false, true] {
            config.style.draw_parents = parents;
            config.style.draw_layers = !parents;
            // let dist = AffineGapCost { k };
            let dist = AffineGapSeedCost {
                k,
                r: 1,
                c,
                formula,
            };
            let (ref a, ref b) =
                pa_generate::generate_model(n, 0.3, pa_generate::ErrorModel::NoisyInsert, 12349);

            let h = BruteForceGCSH {
                match_config: MatchConfig {
                    length: pa_heuristic::LengthConfig::Fixed(k),
                    r: 1,
                    local_pruning: 7,
                },
                distance_function: dist,
                pruning: Pruning::both(),
            };
            let h = h.build(a, b);
            let v = &mut config.build(a, b);
            v.last_frame(None, None, Some(&h));
        }
    }
}
