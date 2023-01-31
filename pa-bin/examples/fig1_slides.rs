//! This generates the visualizations used in figure 1 in the paper and in the slides.

use astarpa::AstarPa;
use pa_affine_types::AffineCost;
use pa_base_algos::{
    dt::{DiagonalTransition, GapCostHeuristic},
    nw::NW,
};
use pa_generate::uniform_fixed;
use pa_heuristic::{MatchConfig, NoCost, Pruning, CSH};
use pa_vis::visualizer::{self, Gradient, When};
use std::{path::PathBuf, time::Duration};

fn main() {
    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = uniform_fixed(n, e);

    let cm = AffineCost::unit();
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::Last;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 4;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path_width = None;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    let vis = |mut config: visualizer::Config, name: &str| {
        config.filepath = PathBuf::from("imgs/fig1-slides/").join(name);
        config
    };

    {
        config.draw_old_on_top = true;
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: true,
            exponential_search: true,
            local_doubling: false,
            h: NoCost,
            v: vis(config.clone(), "1_ukkonen"),
        };
        nw.align(a, b);
    }

    {
        let a_star = AstarPa {
            dt: false,
            h: NoCost,
            v: vis(config.clone(), "2_dijkstra"),
        };
        a_star.align(a, b);
    }

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            NoCost,
            false,
            vis(config.clone(), "3_diagonal-transition"),
        );
        dt.align(a, b);
    }

    {
        config.draw_old_on_top = false;
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            NoCost,
            true,
            vis(config.clone(), "4_dt-divide-and-conquer"),
        );
        dt.align(a, b);
    }

    {
        let k = 5;
        let h = CSH::new(MatchConfig::exact(k), Pruning::disabled());
        let a_star = AstarPa {
            dt: false,
            h,
            v: vis(config.clone(), "5_astar-csh"),
        };
        a_star.align(a, b);
    }

    {
        let k = 5;
        let h = CSH::new(MatchConfig::exact(k), Pruning::enabled());
        let a_star = AstarPa {
            dt: false,
            h,
            v: vis(config.clone(), "6_astar-csh-pruning"),
        };
        a_star.align(a, b);
    }
}
