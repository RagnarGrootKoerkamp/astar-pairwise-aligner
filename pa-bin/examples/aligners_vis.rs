use astarpa::AstarPa;
use pa_affine_types::AffineCost;
use pa_base_algos::{
    dt::{DiagonalTransition, GapCostHeuristic},
    nw::{AffineFront, NW},
    Domain,
};
use pa_generate::uniform_fixed;
use pa_heuristic::{GapCost, MatchConfig, NoCost, Pruning, CSH, GCSH, SH};
use pa_types::seq_to_string;
use pa_vis::visualizer::{self, Gradient, When};
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
    let csh = CSH::new(MatchConfig::exact(4), Pruning::disabled());
    let gcsh = GCSH::new(MatchConfig::exact(4), Pruning::disabled());
    let gcsh_prune = GCSH::new(MatchConfig::exact(4), Pruning::start());

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

    // {
    //     let nw = NW {
    //         cm: cm.clone(),
    //         strategy: pa_base_algos::Strategy::band_doubling(),
    //         domain: Domain::dist_gap(),
    //         block_width: 1,
    //         v: vis("nw_gapcost_h"),
    //         front: AffineFront,
    //         trace: true,
    //         sparse_h: false,
    //         prune: false,
    //     };

    //     nw.align(a, b);
    // }

    // {
    //     let nw = NW {
    //         cm: cm.clone(),
    //         strategy: pa_base_algos::Strategy::band_doubling(),
    //         domain: Domain::astar(sh),
    //         block_width: 1,
    //         v: vis("nw_sh"),
    //         front: AffineFront,
    //         trace: true,
    //         sparse_h: false,
    //         prune: false,
    //     };

    //     nw.align(a, b);
    // }

    // {
    //     let nw = NW {
    //         cm: cm.clone(),
    //         strategy: pa_base_algos::Strategy::band_doubling(),
    //         domain: Domain::astar(csh),
    //         block_width: 1,
    //         v: vis("nw_csh"),
    //         front: AffineFront,
    //         trace: true,
    //         sparse_h: false,
    //         prune: false,
    //     };

    //     nw.align(a, b);
    // }

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            NoCost,
            true,
            vis("dt_dc"),
        );
        dt.align(a, b);
    }

    // {
    //     let mut dt = DiagonalTransition::new(
    //         cm.clone(),
    //         GapCostHeuristic::Enable,
    //         NoCost,
    //         false,
    //         vis("dt_gapcost"),
    //     );
    //     dt.align(a, b);
    // }

    // {
    //     let mut dt = DiagonalTransition::new(
    //         cm.clone(),
    //         GapCostHeuristic::Disable,
    //         GapCost,
    //         false,
    //         vis("dt_gapcost_h"),
    //     );
    //     dt.align(a, b);
    // }

    // {
    //     let mut dt = DiagonalTransition::new(
    //         cm.clone(),
    //         GapCostHeuristic::Disable,
    //         sh,
    //         false,
    //         vis("dt_sh"),
    //     );
    //     dt.align(a, b);
    // }

    // {
    //     let mut dt = DiagonalTransition::new(
    //         cm.clone(),
    //         GapCostHeuristic::Disable,
    //         csh,
    //         false,
    //         vis("dt_csh"),
    //     );
    //     dt.align(a, b);
    // }
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
            h: csh,
            v: vis("07-a*pa-csh"),
        };
        aligner.align(a, b);
    }
    {
        let aligner = AstarPa {
            dt: false,
            h: gcsh,
            v: vis("08-a*pa-gcsh"),
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
    {
        let aligner = AstarPa {
            dt: true,
            h: gcsh_prune,
            v: vis("10-a*pa-gcsh-prune-dt"),
        };
        aligner.align(a, b);
    }
    // {
    //     let aligner = AstarPa {
    //         dt: true,
    //         h: sh,
    //         v: vis("a*pa-sh-dt"),
    //     };
    //     aligner.align(a, b);
    // }
    // {
    //     let aligner = AstarPa {
    //         dt: true,
    //         h: gcsh,
    //         v: vis("a*pa-gcsh-dt"),
    //     };
    //     aligner.align(a, b);
    // }
}
