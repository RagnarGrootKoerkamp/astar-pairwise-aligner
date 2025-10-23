//! This generates the visualizations used in figure 1 in the paper and in the slides.

use astarpa::{AstarPa, astar};
use astarpa2::{AstarPa2Params, DoublingStart, DoublingType};
use pa_affine_types::AffineCost;
use pa_base_algos::{
    Domain,
    dt::{DiagonalTransition, GapCostHeuristic},
    nw::{AffineFront, NW},
};
use pa_generate::{uniform_fixed, uniform_seeded};
use pa_heuristic::{GCSH, MatchConfig, NoCost, Pruning};
use pa_types::Aligner;
use pa_vis::NoVis;
use pa_vis::visualizer::{self, Gradient, When};
use std::{path::PathBuf, time::Duration};

fn main() {
    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = uniform_seeded(n, e, 1234);
    eprintln!("Length {}", a.len());
    let cost = astar(&a, &b, &NoCost, &NoVis).0.0;
    eprintln!("Distance {cost}");
    eprintln!("Divergence {}", cost as f32 / a.len() as f32);

    if !cfg!(feature = "example") {
        panic!("WARNING: Without the example feature, pruned matches aren't shown red for SH");
    }
    if pa_bitpacking::B::BITS != 8 {
        panic!("small_blocks feature is required for useful scale");
    }

    let cm = AffineCost::unit();
    let mut config = visualizer::Config::default();
    config.draw = When::Layers;
    config.save = When::Layers;
    config.save_last = false;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 1;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path = Some((0, 0, 0, 0));
    config.style.path_width = None;
    config.layer_drawing = true;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.transparent_bmp = true;
    config.draw_old_on_top = true;
    config.filepath = PathBuf::from("imgs/readme/");
    config.clear_after_meeting_point = true;

    let aligners: &mut [Box<dyn Aligner>] = &mut [
        Box::new(NW {
            cm,
            strategy: pa_base_algos::Strategy::None,
            domain: Domain::full(),
            block_width: 1,
            v: {
                let mut config = config.with_filename("0_nw");
                config.save = When::LayersStepBy(5);
                config.draw = When::LayersStepBy(5);
                config.num_layers = Some(500);
                config
            },
            front: AffineFront,
            trace: true,
            sparse_h: false,
            prune: false,
        }),
        Box::new(NW {
            cm,
            strategy: pa_base_algos::Strategy::band_doubling(),
            domain: Domain::gap_gap(),
            block_width: 1,
            v: {
                let mut config = config.with_filename("1_ukkonen");
                config.layer_drawing = true;
                config.draw_old_on_top = false;
                config.draw = When::All;
                config.save = When::StepBy(15);
                config.num_layers = Some(7);
                config
            },
            front: AffineFront,
            trace: true,
            sparse_h: false,
            prune: false,
        }),
        Box::new(NW {
            cm,
            strategy: pa_base_algos::Strategy::band_doubling(),
            domain: Domain::dist_gap(),
            block_width: 1,
            v: {
                let mut config = config.with_filename("1_edlib");
                config.layer_drawing = true;
                config.draw_old_on_top = false;
                config.draw = When::All;
                config.save = When::StepBy(5);
                config.num_layers = Some(7);
                config
            },
            front: AffineFront,
            trace: true,
            sparse_h: false,
            prune: false,
        }),
        Box::new(AstarPa {
            dt: false,
            h: NoCost,
            v: {
                let mut config = config.with_filename("2_dijkstra");
                config.num_layers = Some(78);
                config
            },
        }),
        Box::new(DiagonalTransition::new(
            cm,
            GapCostHeuristic::Disable,
            NoCost,
            false,
            {
                let mut config = config.with_filename("3_diagonal-transition");
                config.num_layers = Some(76);
                config
            },
        )),
        Box::new(DiagonalTransition::new(
            cm,
            GapCostHeuristic::Disable,
            NoCost,
            true,
            config.with_filename("4_dt-divide-and-conquer"),
        )),
        Box::new(AstarPa {
            h: GCSH::new(MatchConfig::exact(5), Pruning::both()),
            dt: true,
            v: {
                let mut config = config.with_filename("5_astarpa");
                config.num_layers = Some(20);
                config
            },
        }),
        {
            let mut config = config.with_filename("6_astarpa2");
            config.layer_drawing = false;
            config.draw = When::All;
            config.save = When::All;
            config.draw_old_on_top = false;
            config.num_layers = Some(32);
            let mut params = AstarPa2Params::full();
            if let DoublingType::BandDoubling { start, .. } = &mut params.doubling {
                *start = DoublingStart::Gap;
            }
            params.front.incremental_doubling = false;
            params.block_width = 32;
            params.make_aligner_with_visualizer(true, config)
        },
    ];
    for aligner in aligners {
        aligner.align(a, b);
    }

    let scale = 1;
    let (mut a, mut b) = uniform_seeded(250 * scale, 0.08, 6);
    let (mut a2, mut b2) = uniform_seeded(200 * scale, 0.60, 2);
    let (mut a3, mut b3) = uniform_seeded(50 * scale, 0.08, 3);
    a.append(&mut a2);
    b.append(&mut b2);
    a.append(&mut a3);
    b.append(&mut b3);
    let a = &a;
    let b = &b;

    let astarpa_noisy = AstarPa {
        h: GCSH::new(MatchConfig::exact(5), Pruning::both()),
        dt: true,
        v: {
            let mut config = config.with_filename("5_astarpa_noisy");
            config.num_layers = Some(39);
            config
        },
    };
    astarpa_noisy.align(a, b);
}
