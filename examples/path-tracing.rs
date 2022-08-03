//! This generates the visualizations used in figure 1 in the paper and in the slides.

#[cfg(not(feature = "sdl2"))]
fn main() {}

#[cfg(feature = "sdl2")]
fn main() {
    use astar_pairwise_aligner::{
        aligners::{
            diagonal_transition::{DiagonalTransition, GapCostHeuristic, PathTracingMethod},
            Aligner,
        },
        prelude::*,
        visualizer::{Gradient, Visualizer, When},
    };
    use sdl2::pixels::Color;
    let a = b"CACTGCAATCGGGAGTCAGTTCAGTAACAAGCGTACGACGCCGATACATGCTACGATCGA";
    let b = b"CATCTGCTCTCTGAGTCAGTGCAGTAACAGCGTACG";

    let cm = LinearCost::new_unit();
    let mut config = visualizer::Config::default();
    config.draw = When::All;
    config.save = When::None;
    config.save_last = true;
    config.delay = 0.0001;
    config.cell_size = 16;
    config.style.bg_color = Color::RGBA(255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path_width = Some(6);
    config.style.tree = Some(Color::RGB(64, 64, 64));
    config.style.tree_width = 3;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    let vis = |mut config: visualizer::Config, name: &str| {
        config.filepath = "imgs/path-tracing/".to_string() + name;
        Visualizer::new(config, a, b)
    };

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(config.clone(), "forward-greedy"),
        );
        dt.align(a, b);
    }
    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(config.clone(), "backward-greedy"),
        );
        dt.path_tracing_method = PathTracingMethod::ReverseGreedy;
        dt.align(a, b);
    }
}
