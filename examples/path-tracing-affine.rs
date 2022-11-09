//! This generates the visualizations used in the blogpost on linear memory WFA.

#[cfg(not(feature = "sdl2"))]
fn main() {}

#[cfg(feature = "sdl2")]
fn main() {
    use astar_pairwise_aligner::{
        aligners::{
            diagonal_transition::{DiagonalTransition, GapCostHeuristic},
            Aligner,
        },
        canvas::BLUE,
        prelude::*,
        visualizer::{Gradient, Visualizer, When},
    };
    let a = b"CACTGCAATCGGGAGTCAGTTCAGTAACAAGCGTACGACGCCGATACATGCTACGATCGA";
    let b = b"CATCTGCTCTCTGAGTCAGTGCAGTAACAGCGTACG";

    let cm = AffineCost::new_affine(1, 1, 1);
    let mut config = visualizer::Config::default();
    config.draw = When::All;
    config.save = When::None;
    config.save_last = true;
    config.delay = 0.0001;
    config.cell_size = 16;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path_width = Some(7);
    config.style.tree = Some((64, 64, 64, 0));
    config.style.tree_width = 3;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    let vis = |a, b, mut config: visualizer::Config, name: &str| {
        config.filepath = "imgs/path-tracing/".to_string() + name;
        Visualizer::new(config, a, b)
    };

    config.style.expanded = Gradient::Fixed((200, 200, 200, 0));
    config.style.extended = Some((230, 230, 230, 0));
    config.style.tree_substitution = None;
    config.style.tree = Some((160, 160, 160, 0));
    config.style.tree_fr_only = true;
    config.style.tree_affine_open = Some(BLUE);

    {
        let a = b"CTTGTGGATCTTAAGGGCATCATAGTGGATCTCGTTGACTTGTGGATCTTAGCTGGATCATAGTGGTTCTTAGGGAGTCTCAAATGGATCTTAGTGGGTCTTAGTGGAAT";
        let b = b"CTTAGTGGATCTAGTGGGACTCTAGTGAATCTTAGTGGCATCTAGCTGATTCGACTAGTGGA";
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            NoCost,
            false,
            vis(a, b, config.clone(), "affine-repeats-open"),
        );
        dt.align(a, b);
    }
    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            NoCost,
            false,
            vis(a, b, config.clone(), "affine-simple-open"),
        );
        dt.align(a, b);
    }
}
