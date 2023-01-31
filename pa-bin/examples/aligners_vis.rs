use pa_affine_types::AffineCost;
use pa_base_algos::{
    dt::{DiagonalTransition, GapCostHeuristic},
    nw::NW,
};
use pa_generate::uniform_fixed;
use pa_heuristic::{GapCost, MatchConfig, NoCost, Pruning, CSH, SH};
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
    config.draw = When::All;
    config.save = When::Last;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 2;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.draw_old_on_top = true;
    let mut vis = |name: &str| {
        config.filepath = PathBuf::from("imgs/").join(name);
        config.clone()
    };

    let sh = SH {
        match_config: MatchConfig::exact(4),
        pruning: Pruning::disabled(),
    };
    let csh = CSH::new(MatchConfig::exact(4), Pruning::disabled());

    {
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: false,
            exponential_search: false,
            local_doubling: false,
            h: NoCost,
            v: vis("nw"),
        };
        nw.align(a, b);
    }

    {
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: true,
            exponential_search: true,
            local_doubling: false,
            h: NoCost,
            v: vis("nw_gapcost"),
        };
        nw.align(a, b);
    }

    {
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: false,
            exponential_search: true,
            local_doubling: false,
            h: GapCost,
            v: vis("nw_gapcost_h"),
        };

        nw.align(a, b);
    }

    {
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: false,
            exponential_search: true,
            local_doubling: false,
            h: sh,
            v: vis("nw_sh"),
        };

        nw.align(a, b);
    }

    {
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: false,
            exponential_search: true,
            local_doubling: false,
            h: csh,
            v: vis("nw_csh"),
        };

        nw.align(a, b);
    }

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            NoCost,
            false,
            vis("dt"),
        );
        dt.align(a, b);
    }

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

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Enable,
            NoCost,
            false,
            vis("dt_gapcost"),
        );
        dt.align(a, b);
    }

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            GapCost,
            false,
            vis("dt_gapcost_h"),
        );
        dt.align(a, b);
    }

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            sh,
            false,
            vis("dt_sh"),
        );
        dt.align(a, b);
    }

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            csh,
            false,
            vis("dt_csh"),
        );
        dt.align(a, b);
    }
}
