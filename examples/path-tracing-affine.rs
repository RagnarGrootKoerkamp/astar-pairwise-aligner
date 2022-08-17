//! This generates the visualizations used in the blogpost on linear memory WFA.

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

    let cm = AffineCost::new_affine(1, 1, 1);
    let mut config = visualizer::Config::default();
    config.draw = When::All;
    config.save = When::None;
    config.save_last = true;
    config.delay = 0.0001;
    config.cell_size = 16;
    config.style.bg_color = Color::RGBA(255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path_width = Some(7);
    config.style.tree = Some(Color::RGB(64, 64, 64));
    config.style.tree_width = 3;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    let vis = |a, b, mut config: visualizer::Config, name: &str| {
        config.filepath = "imgs/path-tracing/".to_string() + name;
        Visualizer::new(config, a, b)
    };

    config.style.expanded = Gradient::Fixed(Color::RGB(200, 200, 200));
    config.style.extended = Some(Color::RGB(230, 230, 230));
    config.style.tree_substitution = None;
    config.style.tree = Some(Color::RGB(160, 160, 160));
    config.style.tree_fr_only = true;
    config.style.tree_affine_open = Some(Color::BLUE);

    {
        let a = b"CTTGTGGATCTTAAGGGCATCATAGTGGATCTCGTTGACTTGTGGATCTTAGCTGGATCATAGTGGTTCTTAGGGAGTCTCAAATGGATCTTAGTGGGTCTTAGTGGAAT";
        let b = b"CTTAGTGGATCTAGTGGGACTCTAGTGAATCTTAGTGGCATCTAGCTGATTCGACTAGTGGA";
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(a, b, config.clone(), "affine-repeats-open"),
        );
        dt.align(a, b);
    }
    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(a, b, config.clone(), "affine-simple-open"),
        );
        dt.align(a, b);
    }
}
