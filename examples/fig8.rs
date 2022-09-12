//! This generates the visualizations used in the limitations section of the paper.
#[cfg(not(feature = "sdl2"))]
fn main() {}

#[cfg(feature = "sdl2")]
fn main() {
    use astar_pairwise_aligner::{
        aligners::{astar::AStar, Aligner},
        prelude::*,
        visualizer::{Gradient, Visualizer, When},
    };
    use sdl2::pixels::Color;

    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::None;
    config.save_last = true;
    config.delay = 0.0001;
    config.cell_size = 4;
    config.style.bg_color = Color::RGBA(255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.9);
    config.style.path_width = None;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    config.transparent_bmp = false;
    //
    config.downscaler = 10;
    config.cell_size = 1;
    config.style.path = None;
    config.style.draw_matches = true;
    config.style.match_width = 2;
    config.style.match_shrink = 0;
    config.style.pruned_match = Color::BLACK;

    {
        let (mut a, mut b) = setup_sequences_with_seed(1, 2000, 0.08);
        let (mut a2, mut b2) = setup_sequences_with_seed(2, 2000, 0.50);
        let (mut a3, mut b3) = setup_sequences_with_seed(3, 500, 0.08);
        a.append(&mut a2);
        b.append(&mut b2);
        a.append(&mut a3);
        b.append(&mut b3);
        let ref a = a;
        let ref b = b;

        config.filepath = "imgs/fig8/high-error-rate".to_string();
        let mut a_star = AStar {
            diagonal_transition: false,
            greedy_edge_matching: true,
            h: CSH {
                match_config: MatchConfig::inexact(10),
                pruning: Pruning::enabled(),
                use_gap_cost: false,
                c: PhantomData::<HintContours<BruteForceContour>>::default(),
            },
            v: Visualizer::new(config.clone(), a, b),
        };
        let cost = a_star.align(a, b).0;
        println!("cost {cost}");
    }

    {
        let (mut a, mut b) = setup_sequences_with_seed(1, 3500, 0.08);
        let (mut a2, _) = setup_sequences_with_seed(2, 500, 0.08);
        let (mut a3, mut b3) = setup_sequences_with_seed(3, 1000, 0.08);
        a.append(&mut a2);
        //b.append(&mut b2);
        a.append(&mut a3);
        b.append(&mut b3);
        let ref a = a;
        let ref b = b;

        config.filepath = "imgs/fig8/deletion".to_string();
        let mut a_star = AStar {
            diagonal_transition: false,
            greedy_edge_matching: true,
            h: CSH {
                match_config: MatchConfig::inexact(10),
                pruning: Pruning::enabled(),
                use_gap_cost: false,
                c: PhantomData::<HintContours<BruteForceContour>>::default(),
            },
            v: Visualizer::new(config.clone(), a, b),
        };
        let cost = a_star.align(a, b).0;
        println!("cost {cost}");
    }
}
