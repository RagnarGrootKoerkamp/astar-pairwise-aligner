//! This generates the visualizations used in figure 1 in the paper and in the slides.

use astarpa::{AstarPa, HeuristicParams, astar};
use astarpa2::*;
use pa_affine_types::AffineCost;
use pa_base_algos::{
    dt::{DiagonalTransition, GapCostHeuristic},
    nw::{AffineFront, NW},
};
use pa_generate::uniform_fixed;
use pa_heuristic::{CSH, GCSH, GapCost, MatchConfig, NoCost, Pruning, SH};
use pa_types::Aligner;
use pa_vis::NoVis;
use pa_vis::visualizer::{self, Gradient, When};
use std::{path::PathBuf, time::Duration};

fn main() {
    let n = 3000;
    let e = 0.25;
    let (ref a, ref b) = uniform_fixed(n, e);
    eprintln!("Length {}", a.len());
    let cost = astar(&a, &b, &NoCost, &NoVis).0.0;
    eprintln!("Distance {cost}");
    eprintln!("Divergence {}", cost as f32 / a.len() as f32);

    let cm = AffineCost::unit();
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::None;
    config.save_last = true;
    config.delay = Duration::from_secs_f32(0.0001);
    config.downscaler = 2;
    config.cell_size = 1;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.trace = None;
    config.style.path = Some((0, 0, 0, 0));
    config.style.path_width = Some(4);
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.transparent_bmp = true;
    config.draw_old_on_top = true;
    config.filepath = PathBuf::from("imgs/thesis-cover");
    config.clear_after_meeting_point = false;

    let block_params = AstarPa2Params {
        name: "simple".into(),
        domain: Domain::Astar(()),
        heuristic: HeuristicParams {
            heuristic: pa_heuristic::HeuristicType::Gap,
            ..Default::default()
        },
        doubling: DoublingType::BandDoubling {
            start: DoublingStart::Gap,
            factor: 2.0,
        },
        block_width: 1,
        front: astarpa2::BlockParams {
            sparse: true,
            simd: false,
            no_ilp: true,
            incremental_doubling: false,
            dt_trace: false,
            ..astarpa2::BlockParams::default()
        },
        sparse_h: false,
        prune: false,
        viz: false,
    };

    let aligners: &mut [Box<dyn Aligner>] = &mut [
        // Box::new(NW {
        //     cm,
        //     strategy: pa_base_algos::Strategy::None,
        //     domain: pa_base_algos::Domain::full(),
        //     block_width: 1,
        //     v: config.with_filename("00_NW"),
        //     front: AffineFront,
        //     trace: true,
        //     sparse_h: false,
        //     prune: false,
        // }),
        // Box::new(NW {
        //     cm,
        //     strategy: pa_base_algos::Strategy::band_doubling(),
        //     domain: pa_base_algos::Domain::dijkstra(),
        //     block_width: 1,
        //     v: config.with_filename("01_doubling_start"),
        //     front: AffineFront,
        //     trace: true,
        //     sparse_h: false,
        //     prune: false,
        // }),
        Box::new(AstarPa {
            dt: false,
            h: NoCost,
            v: config.with_filename("00_dijkstra"),
        }),
        Box::new(DiagonalTransition::new(
            cm,
            GapCostHeuristic::Disable,
            NoCost,
            false,
            config.with_filename("01_dt"),
        )),
        Box::new(AstarPa {
            dt: false,
            h: GapCost,
            v: config.with_filename("02_dijkstra_gap"),
        }),
        Box::new(AstarPa {
            h: GapCost,
            dt: true,
            v: config.with_filename("03_dt_gap"),
        }),
        // Box::new(AstarPa {
        //     h: SH::new(MatchConfig::exact(5), Pruning::disabled()),
        //     dt: true,
        //     v: config.with_filename("04_sh"),
        // }),
        Box::new(AstarPa {
            h: CSH::new(MatchConfig::exact(5), Pruning::disabled()),
            dt: true,
            v: config.with_filename("05_csh"),
        }),
        Box::new(AstarPa {
            h: GCSH::new(MatchConfig::exact(5), Pruning::disabled()),
            dt: true,
            v: config.with_filename("06_gcsh"),
        }),
        Box::new(AstarPa {
            h: GCSH::new(MatchConfig::exact(5), Pruning::both()),
            dt: true,
            v: config.with_filename("07_pruning"),
        }),
        Box::new(NW {
            cm,
            strategy: pa_base_algos::Strategy::band_doubling(),
            domain: pa_base_algos::Domain::gap_start(),
            block_width: 1,
            v: config.with_filename("21_doubling_gap_start"),
            front: AffineFront,
            trace: true,
            sparse_h: false,
            prune: false,
        }),
        Box::new(NW {
            cm,
            strategy: pa_base_algos::Strategy::band_doubling(),
            domain: pa_base_algos::Domain::gap_gap(),
            block_width: 1,
            v: config.with_filename("22_doubling_gap_gap"),
            front: AffineFront,
            trace: true,
            sparse_h: false,
            prune: false,
        }),
        Box::new(NW {
            cm,
            strategy: pa_base_algos::Strategy::BandDoubling {
                start: pa_base_algos::DoublingStart::Gap,
                factor: 2.0,
            },
            domain: pa_base_algos::Domain::dist_gap(),
            block_width: 1,
            v: config.with_filename("23_doubling_gap_dist"),
            front: AffineFront,
            trace: true,
            sparse_h: false,
            prune: false,
        }),
        block_params.make_aligner_with_visualizer(true, config.with_filename("24_bitpacking")),
        {
            let mut bp = block_params;
            bp.block_width = 32;
            bp
        }
        .make_aligner_with_visualizer(true, config.with_filename("25_bitpacking_2")),
        // Box::new(AstarPa {
        //     h: GCSH::new(MatchConfig::exact(5), Pruning::both()),
        //     dt: false,
        //     v: config.with_filename("25_astarpa-prune"),
        // }),
        // astarpa2::AstarPa2Params::simple()
        //     .make_aligner_with_visualizer(true, config.with_filename("26_astarpa2_simple")),
        {
            let mut params = astarpa2::AstarPa2Params::full();
            params.block_width = 128;
            params.heuristic.k = 5;
            params
        }
        .make_aligner_with_visualizer(true, config.with_filename("26_astarpa2_full")),
        astarpa2::AstarPa2Params::full()
            .make_aligner_with_visualizer(true, config.with_filename("27_astarpa2_full")),
    ];
    for aligner in aligners {
        aligner.align(a, b);
    }
}
