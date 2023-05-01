//! This generates the visualizations used in figure 1 in the paper and in the slides.

use astarpa::astar;
use pa_affine_types::{AffineAligner, AffineCost};
use pa_base_algos::{nw::NW, Domain, Strategy};
use pa_generate::uniform_fixed;
use pa_heuristic::{GapCost, MatchConfig, NoCost, Pruning, GCSH};
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis_types::NoVis;
use std::{path::PathBuf, time::Duration};

fn main() {
    let n = 100;
    let e = 0.20;
    let (ref a, ref b) = uniform_fixed(n, e);
    eprintln!("Length {}", a.len());
    let cost = astar(&a, &b, &NoCost, &NoVis).0 .0;
    eprintln!("Distance {cost}");
    eprintln!("Divergence {}", cost as f32 / a.len() as f32);

    let cm = AffineCost::unit();
    let mut config = visualizer::Config::default();
    config.draw = When::All;
    config.save = When::None;
    config.save_last = false;
    config.delay = Duration::from_secs_f32(0.0001);
    //config.cell_size = 4;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path = Some((0, 0, 0, 0));
    config.style.path_width = Some(4);
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.transparent_bmp = true;
    config.draw_old_on_top = true;
    config.filepath = PathBuf::from("imgs/domains/");
    config.clear_after_meeting_point = false;

    let block_width = 4;
    let aligners: &mut [Box<dyn AffineAligner>] = &mut [
        Box::new(NW {
            cm,
            strategy: Strategy::None,
            domain: Domain::full(),
            block_width,
            v: config.with_filename("full"),
        }),
        Box::new(NW {
            cm,
            strategy: Strategy::band_doubling(),
            domain: Domain::gap_start(),
            block_width,
            v: config.with_filename("gap-start"),
        }),
        Box::new(NW {
            cm,
            strategy: Strategy::band_doubling(),
            domain: Domain::gap_gap(),
            block_width,
            v: config.with_filename("gap-gap"),
        }),
        Box::new(NW {
            cm,
            strategy: Strategy::band_doubling(),
            domain: Domain::dijkstra(),
            block_width,
            v: config.with_filename("dijkstra"),
        }),
        Box::new(NW {
            cm,
            strategy: Strategy::band_doubling(),
            domain: Domain::Astar(GapCost),
            block_width,
            v: config.with_filename("astar-gap"),
        }),
        Box::new(NW {
            cm,
            strategy: Strategy::band_doubling(),
            domain: Domain::Astar(GCSH::new(MatchConfig::exact(5), Pruning::both())),
            block_width,
            v: config.with_filename("astar-gcsh"),
        }),
    ];
    for aligner in aligners {
        aligner.align_affine(a, b);
    }
}
