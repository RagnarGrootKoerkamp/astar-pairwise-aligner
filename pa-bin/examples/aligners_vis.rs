#![allow(unused_imports)]
use astarpa::AstarPa;
use pa_affine_types::AffineCost;
use pa_base_algos::{
    dt::{DiagonalTransition, GapCostHeuristic},
    nw::{AffineFront, BitFront, NW},
    Domain,
};
use pa_generate::uniform_fixed;
use pa_heuristic::{MatchConfig, NoCost, Pruning, CSH, GCSH, SH};
use pa_types::seq_to_string;
use pa_vis::visualizer::{self, Config, Gradient, When};
use std::{path::PathBuf, time::Duration};

fn main() {
    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = uniform_fixed(n, e);
    println!("{}\n{}\n", seq_to_string(a), seq_to_string(b));

    let cm = AffineCost::unit();
    let mut config = visualizer::Config::default();
    config.draw = When::Layers;
    config.save = When::None;
    config.save_last = true;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 0;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path = Some((0, 0, 0, 0));
    config.style.path_width = Some(2);
    config.layer_drawing = false;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.style.draw_matches = true;
    config.style.draw_contours = true;
    config.style.draw_heuristic = false;
    config.style.max_heuristic = Some(100);
    config.style.heuristic = Gradient::Gradient((255, 255, 255, 255)..(100, 100, 100, 255));
    config.style.match_width = 2;
    config.transparent_bmp = true;
    config.draw_old_on_top = true;
    config.clear_after_meeting_point = false;
    config.paused = true;

    let mut vis = |name: &str| {
        config.filepath = PathBuf::from("imgs/talk/").join(name);
        config.clone()
    };

    let sh = SH {
        match_config: MatchConfig::exact(4),
        pruning: Pruning::disabled(),
    };
    let sh_prune = SH::new(MatchConfig::exact(4), Pruning::start());
    let csh_prune = CSH::new(MatchConfig::exact(4), Pruning::start());
    let gcsh_prune = GCSH::new(MatchConfig::exact(4), Pruning::start());
    let gcsh_prune_inexact = GCSH::new(MatchConfig::inexact(8), Pruning::start());
    let gcsh_prune_local = GCSH::new(
        MatchConfig {
            length: pa_heuristic::LengthConfig::Fixed(4),
            r: 1,
            local_pruning: 1,
        },
        Pruning::start(),
    );

    {
        let nw = NW {
            cm: cm.clone(),
            strategy: pa_base_algos::Strategy::None,
            domain: Domain::full(),
            block_width: 1,
            v: vis("01-nw"),
            front: AffineFront,
            trace: true,
            sparse_h: false,
            prune: false,
        };
        nw.align(a, b);
    }
    {
        let aligner = AstarPa {
            dt: false,
            h: NoCost,
            v: vis("02-dijkstra"),
        };
        aligner.align(a, b);
    }
    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            NoCost,
            false,
            vis("03-dt"),
        );
        dt.align(a, b);
    }
    {
        let nw = NW {
            cm: cm.clone(),
            strategy: pa_base_algos::Strategy::band_doubling(),
            domain: Domain::gap_start(),
            block_width: 1,
            v: vis("04-nw_doubling"),
            front: AffineFront,
            trace: true,
            sparse_h: false,
            prune: false,
        };
        nw.align(a, b);
    }
    {
        let nw = NW {
            cm: cm.clone(),
            strategy: pa_base_algos::Strategy::band_doubling(),
            domain: Domain::gap_gap(),
            block_width: 1,
            v: vis("05-nw_gapcost"),
            front: AffineFront,
            trace: true,
            sparse_h: false,
            prune: false,
        };
        nw.align(a, b);
    }
    config.style.draw_heuristic = true;
    let mut vis = |name: &str| {
        config.filepath = PathBuf::from("imgs/talk/").join(name);
        config.clone()
    };
    {
        let aligner = AstarPa {
            dt: false,
            h: sh,
            v: vis("06-a*pa-sh"),
        };
        aligner.align(a, b);
    }
    {
        let aligner = AstarPa {
            dt: false,
            h: sh_prune,
            v: vis("07-a*pa-sh-prune"),
        };
        aligner.align(a, b);
    }
    {
        let aligner = AstarPa {
            dt: false,
            h: csh_prune,
            v: vis("08-a*pa-csh-prune"),
        };
        aligner.align(a, b);
    }
    {
        let aligner = AstarPa {
            dt: false,
            h: gcsh_prune,
            v: vis("09-a*pa-gcsh-prune"),
        };
        aligner.align(a, b);
    }
    config.style.draw_contours = false;
    let mut vis = |name: &str| {
        config.filepath = PathBuf::from("imgs/talk/").join(name);
        config.clone()
    };
    {
        let aligner = AstarPa {
            dt: true,
            h: gcsh_prune,
            v: vis("10-a*pa-gcsh-prune-dt"),
        };
        aligner.align(a, b);
    }
    {
        let mut config = Config {
            draw: When::All,
            ..config.clone()
        };
        let mut vis = |name: &str| {
            config.filepath = PathBuf::from("imgs/talk/").join(name);
            config.clone()
        };
        let nw = NW {
            cm: cm.clone(),
            strategy: pa_base_algos::Strategy::BandDoubling {
                start: pa_base_algos::DoublingStart::H0,
                factor: 1.5,
            },
            domain: Domain::astar(gcsh_prune),
            block_width: 1,
            v: vis("11-nw-gcsh-prune"),
            front: AffineFront,
            trace: true,
            sparse_h: false,
            prune: true,
        };
        nw.align(a, b);
    }
    let mut vis = |name: &str| {
        config.filepath = PathBuf::from("imgs/talk/").join(name);
        config.clone()
    };
    {
        let nw = NW {
            cm: cm.clone(),
            strategy: pa_base_algos::Strategy::band_doubling(),
            domain: Domain::astar(gcsh_prune),
            block_width: 16,
            v: vis("12-nw-gcsh-prune-block"),
            front: BitFront::default(),
            trace: true,
            sparse_h: false,
            prune: true,
        };
        nw.align(a, b);
    }

    // {
    //     let mut dt = DiagonalTransition::new(
    //         cm.clone(),
    //         GapCostHeuristic::Disable,
    //         NoCost,
    //         true,
    //         vis("20-dt-divide-and-conquer"),
    //     );
    //     dt.align(a, b);
    // }
    // {
    //     let mut dt = DiagonalTransition::new(
    //         cm.clone(),
    //         GapCostHeuristic::Disable,
    //         gcsh_prune,
    //         false,
    //         vis("21-dt-gcsh-prune"),
    //     );
    //     dt.align(a, b);
    // }

    let (ref a, ref b) = uniform_fixed(10 * n, e);
    config.downscaler = 10;
    config.style.draw_heuristic = false;
    config.style.draw_contours = false;
    let mut vis = |name: &str| {
        config.filepath = PathBuf::from("imgs/talk/").join(name);
        config.clone()
    };

    {
        let nw = NW {
            cm: cm.clone(),
            strategy: pa_base_algos::Strategy::band_doubling(),
            domain: Domain::astar(gcsh_prune),
            block_width: 64,
            v: vis("13-nw-gcsh-prune-block-large"),
            front: BitFront::default(),
            trace: true,
            sparse_h: false,
            prune: true,
        };
        nw.align(a, b);
    }

    {
        let nw = NW {
            cm: cm.clone(),
            strategy: pa_base_algos::Strategy::band_doubling(),
            domain: Domain::astar(gcsh_prune_inexact),
            block_width: 64,
            v: vis("14-nw-gcsh-prune-block-large-inexact"),
            front: BitFront::default(),
            trace: true,
            sparse_h: false,
            prune: true,
        };
        nw.align(a, b);
    }

    {
        let nw = NW {
            cm: cm.clone(),
            strategy: pa_base_algos::Strategy::band_doubling(),
            domain: Domain::astar(gcsh_prune_local),
            block_width: 64,
            v: vis("15-nw-gcsh-prune-block-large-local"),
            front: BitFront::default(),
            trace: true,
            sparse_h: false,
            prune: true,
        };
        nw.align(a, b);
    }
}
