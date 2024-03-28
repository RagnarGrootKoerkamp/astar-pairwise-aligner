//! This generates the visualizations used in figure 1 in the paper and in the slides.

use astarpa::{astar, AstarPa};
use pa_generate::uniform_fixed;
use pa_heuristic::{MatchConfig, NoCost, Pruning, GCSH, SH};
use pa_types::Aligner;
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::NoVis;
use std::{path::PathBuf, time::Duration};

fn main() {
    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = uniform_fixed(n, e);
    eprintln!("Length {}", a.len());
    let cost = astar(&a, &b, &NoCost, &NoVis).0 .0;
    eprintln!("Distance {cost}");
    eprintln!("Divergence {}", cost as f32 / a.len() as f32);

    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::Layers;
    config.save_last = false;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 1;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path = Some((0, 0, 0, 0));
    config.style.path_width = None;
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.transparent_bmp = true;
    config.draw_old_on_top = true;
    config.filepath = PathBuf::from("imgs/slides/");
    config.clear_after_meeting_point = true;

    let aligners: &mut [Box<dyn Aligner>] = &mut [
        Box::new(AstarPa {
            dt: false,
            h: SH::new(MatchConfig::exact(5), Pruning::disabled()),
            v: config.with_filename("sh-noprune"),
        }),
        Box::new(AstarPa {
            dt: false,
            h: SH::new(MatchConfig::exact(5), Pruning::both()),
            v: config.with_filename("sh"),
        }),
        Box::new(AstarPa {
            dt: false,
            h: GCSH::new(MatchConfig::exact(5), Pruning::disabled()),
            v: config.with_filename("gcsh-noprune"),
        }),
        Box::new(AstarPa {
            dt: false,
            h: GCSH::new(MatchConfig::exact(5), Pruning::both()),
            v: config.with_filename("gcsh"),
        }),
        Box::new(AstarPa {
            dt: true,
            h: GCSH::new(MatchConfig::exact(5), Pruning::both()),
            v: config.with_filename("gcsh-dt"),
        }),
    ];
    for aligner in aligners {
        aligner.align(a, b);
    }
}
