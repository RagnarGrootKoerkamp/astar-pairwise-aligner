use pa_affine_types::AffineCost;
use pa_base_algos::{
    nw::{BitFront, NW},
    Domain, Strategy,
};
use pa_generate::uniform_fixed;
use pa_heuristic::{GapCost, MatchConfig, Pruning, GCSH};
use pa_types::Aligner;
use pa_vis::visualizer::{self, Gradient, When};
use std::{path::PathBuf, time::Duration};

fn main() {
    let n = 5000;
    let e = 0.20;
    let (ref a, ref b) = uniform_fixed(n, e);
    eprintln!("Length {} {}", a.len(), b.len());

    let cm = AffineCost::unit();
    let mut config = visualizer::Config::default();
    config.draw = When::All;
    config.save = When::None;
    config.save_last = false;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 0;
    config.downscaler = 8;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path = Some((0, 0, 0, 0));
    config.style.path_width = Some(4);
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.transparent_bmp = true;
    config.draw_old_on_top = false;
    config.filepath = PathBuf::from("imgs/domains/");
    config.clear_after_meeting_point = false;

    let block_width = 32;
    let front = BitFront {
        sparse: true,
        simd: true,
    };
    let aligners: &mut [Box<dyn Aligner>] = &mut [
        // Box::new(NW {
        //     cm,
        //     strategy: Strategy::None,
        //     domain: Domain::full(),
        //     block_width,
        //     v: config.with_filename("full"),
        //     front,
        // }),
        Box::new(NW {
            cm,
            strategy: Strategy::band_doubling(),
            domain: Domain::gap_start(),
            block_width,
            v: config.with_filename("gap-start"),
            front,
            trace: true,
            sparse_h: true,
        }),
        Box::new(NW {
            cm,
            strategy: Strategy::band_doubling(),
            domain: Domain::gap_gap(),
            block_width,
            v: config.with_filename("gap-gap"),
            front,
            trace: true,
            sparse_h: true,
        }),
        Box::new(NW {
            cm,
            strategy: Strategy::band_doubling(),
            domain: Domain::dijkstra(),
            block_width,
            v: config.with_filename("dijkstra"),
            front,
            trace: true,
            sparse_h: true,
        }),
        Box::new(NW {
            cm,
            strategy: Strategy::band_doubling(),
            domain: Domain::Astar(GapCost),
            block_width,
            v: config.with_filename("astar-gap"),
            front,
            trace: true,
            sparse_h: true,
        }),
        Box::new(NW {
            cm,
            strategy: Strategy::band_doubling(),
            domain: Domain::Astar(GCSH::new(MatchConfig::exact(5), Pruning::both())),
            block_width,
            v: config.with_filename("astar-gcsh"),
            front,
            trace: true,
            sparse_h: true,
        }),
    ];
    for aligner in aligners {
        aligner.align(a, b);
    }
}
