//! This generates the visualizations used in figure 1 in the paper and in the slides.

#[cfg(not(feature = "sdl2"))]
fn main() {}

#[cfg(feature = "sdl2")]
fn main() {
    use std::{path::PathBuf, time::Duration};

    use astar_pairwise_aligner::{
        aligners::{
            astar::AStar,
            diagonal_transition::{DiagonalTransition, GapCostHeuristic},
            nw::NW,
            Aligner,
        },
        prelude::*,
        visualizer::{Gradient, Visualizer, When},
    };
    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = setup_sequences(n, e);

    let cm = LinearCost::new_unit();
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::None;
    config.save_last = true;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 4;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.path_width = Some(4);
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    let vis = |mut config: visualizer::Config, name: &str| {
        config.filepath = PathBuf::from("imgs/fig1/").join(name);
        Visualizer::new(config, a, b)
    };

    let run_all = |config: &mut visualizer::Config, video_mode| {
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
            println!("{}", nw.v.expanded.len() as f32 / a.len() as f32);
        }

        {
            let mut a_star = AStar {
                diagonal_transition: false,
                greedy_edge_matching: true,
                h: NoCost,
                v: vis(config.clone(), "2_dijkstra"),
            };
            a_star.align(a, b);
            println!("{}", a_star.v.expanded.len() as f32 / a.len() as f32);
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
            println!("{}", dt.v.borrow().expanded.len() as f32 / a.len() as f32);
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
            println!("{}", dt.v.borrow().expanded.len() as f32 / a.len() as f32);
        }

        if video_mode {
            config.save = When::All;
        }
        {
            let k = 5;
            let h = CSH {
                match_config: MatchConfig::exact(k),
                pruning: Pruning::enabled(),
                use_gap_cost: false,
                c: PhantomData::<HintContours<BruteForceContour>>::default(),
            };
            let mut a_star = AStar {
                diagonal_transition: false,
                greedy_edge_matching: true,
                h,
                v: vis(config.clone(), "5_astar-csh-pruning"),
            };
            a_star.align(a, b);
            println!("{}", a_star.v.expanded.len() as f32 / a.len() as f32);
        }
    };

    run_all(&mut config, false);
    // Run all again to make videos, with a smaller cell size.
    config.save_last = false;
    config.save = When::Layers;
    config.cell_size = 1;
    config.style.path_width = Some(2);
    run_all(&mut config, true);
}
