//! This generates the visualizations used in figure 1 in the paper and in the slides.

#[cfg(not(feature = "sdl2"))]
fn main() {}

#[cfg(feature = "sdl2")]
fn main() {
    use astar_pairwise_aligner::canvas::*;
    use astar_pairwise_aligner::{
        aligners::{astar::AStar, Aligner},
        prelude::*,
        visualizer::{Gradient, Visualizer, When},
    };
    use itertools::Itertools;

    let n = 500;
    let e = 0.20;
    let (ref a, ref b) = setup_sequences(n, e);

    let mut config = visualizer::Config::default();
    let frames = (0..10000).step_by(4).collect_vec();
    config.draw = When::Frames(frames.clone());
    config.save = When::Frames(frames);
    config.paused = false;
    config.delay = 0.0001;
    config.cell_size = 2;
    config.style.bg_color = WHITE;
    //config.style.expanded = Gradient::Fixed((130, 179, 102, 0));
    config.style.expanded = Gradient::TurboGradient(0.25..0.90);
    config.style.explored = Some((0, 102, 204, 0));
    config.style.heuristic = Gradient::Gradient((250, 250, 250, 0)..(180, 180, 180, 0));
    config.style.max_heuristic = Some(35);
    config.style.pruned_match = GREEN;
    config.style.path = Some(BLACK);
    config.style.match_width = 3;
    config.style.draw_heuristic = true;
    config.style.draw_contours = true;
    config.style.draw_matches = true;
    config.style.match_width = 2;
    config.style.contour = BLACK;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    config.filepath = "imgs/fig-readme".to_string();
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
            v: Visualizer::new(config.clone(), a, b),
        };
        a_star.align(a, b);
    }
}
