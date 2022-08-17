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

    let cm = LinearCost::new_unit();
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

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(a, b, config.clone(), "forward-greedy"),
        );
        dt.align(a, b);
    }
    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(a, b, config.clone(), "backward-greedy"),
        );
        dt.path_tracing_method = PathTracingMethod::ReverseGreedy;
        dt.align(a, b);
    }

    config.style.expanded = Gradient::Fixed(Color::RGB(200, 200, 200));
    config.style.extended = Some(Color::RGB(230, 230, 230));

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(a, b, config.clone(), "forward-greedy-grey"),
        );
        dt.align(a, b);
    }

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(a, b, config.clone(), "backward-greedy-grey"),
        );
        dt.path_tracing_method = PathTracingMethod::ReverseGreedy;
        dt.align(a, b);
    }

    config.style.tree_substitution = Some(Color::RED);

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(a, b, config.clone(), "forward-greedy-subs"),
        );
        dt.align(a, b);
    }
    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(a, b, config.clone(), "backward-greedy-subs"),
        );
        dt.path_tracing_method = PathTracingMethod::ReverseGreedy;
        dt.align(a, b);
    }
    {
        let b = b"AXBDBBC";
        let a = b"ABDBBYDC";
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(a, b, config.clone(), "detail"),
        );
        dt.path_tracing_method = PathTracingMethod::ReverseGreedy;
        dt.align(a, b);
    }
    {
        let a = b"CCGGGGTGCTCG";
        let b = b"GTGCCCGTGGGTG";
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(a, b, config.clone(), "detail-tricky"),
        );
        dt.align(a, b);
    }

    {
        let a = b"CTTGTGGATCTTAAGGGCATCATAGTGGATCTCGTTGACTTGTGGATCTTAGCTGGATCATAGTGGTTCTTAGGGAGTCTCAAATGGATCTTAGTGGGTCTTAGTGGAAT";
        let b = b"CTTAGTGGATCTAGTGGGACTCTAGTGAATCTTAGTGGCATCTAGCTGATTCGACTAGTGGA";

        {
            let mut dt = DiagonalTransition::new(
                cm.clone(),
                GapCostHeuristic::Disable,
                ZeroCost,
                false,
                vis(a, b, config.clone(), "repeats"),
            );
            dt.align(a, b);
        }

        config.style.tree_match = Some(Color::RGB(160, 160, 160));
        {
            let mut dt = DiagonalTransition::new(
                cm.clone(),
                GapCostHeuristic::Disable,
                ZeroCost,
                false,
                vis(a, b, config.clone(), "repeats-no-matches"),
            );
            dt.align(a, b);
        }

        config.style.tree = Some(Color::RGB(160, 160, 160));
        {
            let mut dt = DiagonalTransition::new(
                cm.clone(),
                GapCostHeuristic::Disable,
                ZeroCost,
                false,
                vis(a, b, config.clone(), "repeats-subs"),
            );
            dt.align(a, b);
        }

        config.style.tree_fr_only = true;
        {
            let mut dt = DiagonalTransition::new(
                cm.clone(),
                GapCostHeuristic::Disable,
                ZeroCost,
                false,
                vis(a, b, config.clone(), "repeats-active"),
            );
            dt.align(a, b);
        }

        {
            config.style.tree_direction_change = Some(Color::BLUE);
            let mut dt = DiagonalTransition::new(
                cm.clone(),
                GapCostHeuristic::Disable,
                ZeroCost,
                false,
                vis(a, b, config.clone(), "repeats-fixed"),
            );
            dt.align(a, b);
        }
    }
    config.style.expanded = Gradient::Fixed(Color::RGB(200, 200, 200));
    config.style.extended = Some(Color::RGB(230, 230, 230));
    config.style.tree_substitution = Some(Color::RED);
    config.style.tree = Some(Color::RGB(160, 160, 160));
    config.style.tree_fr_only = true;
    config.style.tree_direction_change = Some(Color::BLUE);

    {
        let mut dt = DiagonalTransition::new(
            cm.clone(),
            GapCostHeuristic::Disable,
            ZeroCost,
            false,
            vis(a, b, config.clone(), "simple-final"),
        );
        dt.align(a, b);
    }
}
