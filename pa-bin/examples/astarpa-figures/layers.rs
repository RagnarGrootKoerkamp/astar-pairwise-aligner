use astarpa::AstarPa;
use pa_heuristic::contour::BruteForceContours;
use pa_heuristic::{MatchConfig, Prune, Pruning, CSH, GCSH, SH};
use pa_vis::canvas::*;
use pa_vis::visualizer::{self, Gradient, When};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    // let a = b"ACTCAGCTGTTGCCCGCTGTCGATACTCAGCTGTTGCCCGCTGTCGATCCGTAATTTAAAGTAGGTCGAAACCCGTAATTTAAAGTAGGTCGAAAC";
    let b = b"ACTCAACGTTGCGCCTGTCTATCGTAATTAAAGTGGAGAAAC";
    let (ref b, b2) = pa_generate::uniform_random(50, 0.15);
    let mut a = pa_generate::random_seq(150);
    a.splice(50..50, b2.iter().copied());
    a.splice(100..100, b2.iter().copied());
    let a = &a;
    eprintln!("a: {}", String::from_utf8_lossy(a));
    eprintln!("b: {}", String::from_utf8_lossy(b));

    let mut config = visualizer::Config::default();
    config.draw = When::All;
    config.save = When::None;
    config.save_last = false;
    config.paused = false;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 15;
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
    config.style.draw_layers = false;
    config.style.draw_contours = false;
    config.style.draw_matches = true;
    config.style.contour = BLACK;
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;

    config.filepath = PathBuf::from("imgs/semi-global/layers/");

    if !cfg!(feature = "example") {
        panic!("WARNING: Without the example feature, pruned matches aren't shown red for SH");
    }

    let k = 3;
    for pruning in [Prune::Both /*Prune::Both*/] {
        let suf = if pruning.is_enabled() { "" } else { "-noprune" };
        // AstarPa {
        //     dt: false,
        //     h: SH::new(MatchConfig::exact(k), Pruning::new(pruning)),
        //     v: config.with_filename(&("sh".to_string() + suf)),
        // }
        // .align(a, b);
        // AstarPa {
        //     dt: false,
        //     h: CSH::new(MatchConfig::exact(k), Pruning::new(pruning)),
        //     v: config.with_filename(&("csh".to_string() + suf)),
        // }
        // .align(a, b);

        let mut match_config = MatchConfig::exact(k);
        match_config.local_pruning = 2 * k as usize;

        let pruning1 = Pruning::new(pruning);

        let h = CSH {
            match_config,
            pruning: pruning1,
            use_gap_cost: true,
            c: PhantomData::<BruteForceContours>,
        };
        AstarPa {
            dt: true,
            // h: GCSH::new(match_config, pruning1),
            h,
            v: config.with_filename(&("gcsh".to_string() + suf)),
        }
        .align(a, b);
    }
}
