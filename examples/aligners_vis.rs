use pairwise_aligner::{
    aligners::{
        diagonal_transition::{DiagonalTransition, GapCostHeuristic},
        nw::NW,
        Aligner,
    },
    prelude::*,
    visualizer::{Draw, Gradient, Save, Visualizer, VisualizerT},
};
use sdl2::pixels::Color;

fn main() {
    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = setup_sequences(n, e);
    println!("{}\n{}\n", to_string(a), to_string(b));

    let cm = LinearCost::new_unit();
    let mut config = visualizer::Config::default();
    config.draw = Draw::All;
    config.save = Save::Last;
    config.delay = 0.0001;
    config.cell_size = 2;
    config.colors.bg_color = Color::RGBA(255, 255, 255, 128);
    config.colors.gradient = Gradient::TurboGradient(0.25..0.90);
    config.draw_old_on_top = true;
    let mut vis = |name: &str| {
        config.filepath = "imgs/".to_string() + name;
        Visualizer::new(config.clone(), a, b)
    };

    let sh = SH {
        match_config: MatchConfig::exact(4),
        pruning: false,
    };
    let csh = CSH {
        match_config: MatchConfig::exact(4),
        pruning: false,
        use_gap_cost: false,
        c: PhantomData::<BruteForceContours>,
    };

    {
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: false,
            h: ZeroCost,
            v: vis("nw"),
        };
        nw.cost(a, b);
        nw.v.last_frame();
    }

    {
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: true,
            h: ZeroCost,
            v: vis("nw_gapcost"),
        };
        nw.cost(a, b);
        nw.v.last_frame();
    }

    {
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: false,
            h: GapCost,
            v: vis("nw_gapcost_h"),
        };

        nw.cost(a, b);
        nw.v.last_frame();
    }

    {
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: false,
            h: sh,
            v: vis("nw_sh"),
        };

        nw.cost(a, b);
        nw.v.last_frame();
    }

    {
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: false,
            h: csh,
            v: vis("nw_csh"),
        };

        nw.cost(a, b);
        nw.v.last_frame();
    }

    {
        let mut dt =
            DiagonalTransition::new(cm.clone(), GapCostHeuristic::Disable, ZeroCost, vis("dt"));
        dt.cost(a, b);
        dt.v.last_frame();
    }

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Enable,
            ZeroCost,
            vis("dt_gapcost"),
        );
        dt.cost(a, b);
        dt.v.last_frame();
    }

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            GapCost,
            vis("dt_gapcost_h"),
        );
        dt.cost(a, b);
        dt.v.last_frame();
    }

    {
        let mut dt =
            DiagonalTransition::new(cm.clone(), GapCostHeuristic::Disable, sh, vis("dt_sh"));
        dt.cost(a, b);
        dt.v.last_frame();
    }

    {
        let mut dt =
            DiagonalTransition::new(cm.clone(), GapCostHeuristic::Disable, csh, vis("dt_csh"));
        dt.cost(a, b);
        dt.v.last_frame();
    }
}
