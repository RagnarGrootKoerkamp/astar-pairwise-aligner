//! This generates the visualizations used in figure 1 in the paper and in the slides.

#[cfg(not(feature = "sdl2"))]
fn main() {}

#[cfg(feature = "sdl2")]
fn main() {
    use std::time::Duration;

    use astar_pairwise_aligner::canvas::*;
    use astar_pairwise_aligner::{
        aligners::{astar::AStar, Aligner},
        prelude::*,
        visualizer::{Gradient, Visualizer, When},
    };

    let a = b"ACTCAGCTGTTGCCCGCTGTCGATCCGTAATTTAAAGTAGGTCGAAAC";
    let b = b"ACTCAACGTTGCGCCTGTCTATCGTAATTAAAGTGGAGAAAC";

    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::Frames(vec![0, usize::MAX]);
    config.paused = false;
    config.delay = Duration::from_secs_f32(0.0001);
    config.cell_size = 6;
    config.style.bg_color = WHITE;
    config.style.expanded = Gradient::Fixed((130, 179, 102, 0));
    config.style.explored = Some((0, 102, 204, 0));
    config.style.heuristic = Gradient::Gradient((250, 250, 250, 0)..(180, 180, 180, 0));
    config.style.max_heuristic = Some(10);
    config.style.pruned_match = RED;
    config.style.path = None;
    config.style.match_width = 3;
    config.style.draw_heuristic = true;
    config.style.draw_contours = true;
    config.style.draw_matches = true;
    config.style.contour = BLACK;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    config.filepath = "imgs/fig3".to_string();

    // First and last frame
    {
        let k = 3;
        let h = CSH {
            match_config: MatchConfig::exact(k),
            pruning: Pruning::enabled(),
            use_gap_cost: false,
            c: PhantomData::<BruteForceContours>::default(),
        };
        let mut a_star = AStar {
            diagonal_transition: false,
            greedy_edge_matching: true,
            h,
            v: Visualizer::new(config.clone(), a, b),
        };
        let cost = a_star.align(a, b).0;
        println!("Distance: {cost}");
    }

    // All frames, for video
    config.save = When::All;
    config.filepath = "imgs/fig3-video/".to_string();
    {
        let k = 3;
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
