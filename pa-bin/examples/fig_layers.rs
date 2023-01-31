use astarpa::AstarPa;
use pa_heuristic::{MatchConfig, Pruning, CSH, GCSH, SH};
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis_types::canvas::*;
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
    config.style.bg_color = WHITE;
    config.style.expanded = Gradient::Fixed((130, 179, 102, 0));
    config.style.explored = Some((0, 102, 204, 0));
    config.style.heuristic = Gradient::Gradient((250, 250, 250, 0)..(180, 180, 180, 0));
    config.style.max_heuristic = Some(10);
    config.style.max_layer = Some(6);
    config.style.pruned_match = RED;
    config.style.path = None;
    config.style.match_width = 3;
    config.style.draw_heuristic = false;
    config.style.draw_layers = true;
    config.style.draw_contours = true;
    config.style.draw_matches = true;
    config.style.contour = BLACK;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;

    let dir = PathBuf::from("imgs/layers/");

    let k = 3;
    for pruning in [false, true] {
        let suf = if pruning { "-pruning" } else { "" };
        // SH
        {
            config.filepath = dir.join("sh".to_string() + suf);
            let h = SH {
                match_config: MatchConfig::exact(k),
                pruning: Pruning::new(pruning),
            };
            let a_star = AstarPa {
                dt: false,
                h,
                v: config.clone(),
            };
            let cost = a_star.align(a, b).0 .0;
            println!("Distance: {cost}");
        }

        // CSH
        {
            config.filepath = dir.join("csh".to_string() + suf);
            let h = CSH::new(MatchConfig::exact(k), Pruning::new(pruning));
            let a_star = AstarPa {
                dt: false,
                h,
                v: config.clone(),
            };
            a_star.align(a, b);
        }

        // GCSH
        {
            config.filepath = dir.join("gch".to_string() + suf);
            let h = GCSH::new(MatchConfig::exact(k), Pruning::new(pruning));
            let a_star = AstarPa {
                dt: false,
                h,
                v: config.clone(),
            };
            a_star.align(a, b);
        }
    }

    // // All frames, for video
    // config.save = When::All;
    // config.filepath = "imgs/layers-video/".to_string();
    // {
    //     let k = 3;
    //     let h = CSH {
    //         match_config: MatchConfig::exact(k),
    //         pruning: Pruning::enabled(),
    //         use_gap_cost: false,
    //         c: PhantomData::<HintContours<BruteForceContour>>::default(),
    //     };
    //     let mut a_star = Astar {
    //         dt: false,
    //         h,
    // v: config.clone()
    //     };
    //     a_star.align(a, b);
    // }
}
