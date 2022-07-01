//! This generates the visualizations used in figure 1 in the paper and in the slides.
use pairwise_aligner::{
    aligners::{
        diagonal_transition::{DiagonalTransition, GapCostHeuristic},
        nw::NW,
        Aligner,
    },
    prelude::*,
    visualizer::{Draw, Gradient, Save, Visualizer},
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
    config.style.bg_color = Color::RGBA(255, 255, 255, 128);
    config.style.gradient = Gradient::TurboGradient(0.25..0.90);
    config.draw_old_on_top = true;
    let mut vis = |name: &str| {
        config.filepath = "imgs/".to_string() + name;
        Visualizer::new(config.clone(), a, b)
    };

    {
        let mut nw = NW {
            cm: cm.clone(),
            use_gap_cost_heuristic: true,
            h: ZeroCost,
            v: vis("exp_search"),
        };
        nw.align(a, b);
    }

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis("diagonal_transition"),
        );
        dt.align(a, b);
    }

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            true,
            vis("diagonal_transition_dc"),
        );
        dt.align(a, b);
    }
}
