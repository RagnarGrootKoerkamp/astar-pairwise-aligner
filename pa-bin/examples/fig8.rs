//! This generates the visualizations used in the limitations section of the paper.

use astarpa::astar;
use pa_generate::{uniform_seeded, ErrorModel};
use pa_heuristic::{MatchConfig, Pruning, CSH};
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis_types::canvas::BLACK;
use rand::SeedableRng;
use std::time::Duration;

fn main() {
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::None;
    config.save_last = true;
    config.delay = Duration::from_secs_f32(0.0001);
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.9);
    config.style.path_width = None;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    config.transparent_bmp = false;
    let scale = 2;
    //config.downscaler = scale as u32;
    config.downscaler = 1;
    config.cell_size = 1;
    config.style.path = None;
    config.style.draw_matches = true;
    config.style.match_width = 4;
    config.style.match_shrink = 0;
    config.style.pruned_match = BLACK;

    {
        let (mut a, mut b) = uniform_seeded(250 * scale, 0.08, 6);
        let (mut a2, mut b2) = uniform_seeded(200 * scale, 0.60, 2);
        let (mut a3, mut b3) = uniform_seeded(50 * scale, 0.08, 3);
        a.append(&mut a2);
        b.append(&mut b2);
        a.append(&mut a3);
        b.append(&mut b3);
        let ref a = a;
        let ref b = b;

        config.filepath = "imgs/fig8/high-error-rate".into();
        let cost = astar(
            a,
            b,
            &CSH::new(MatchConfig::inexact(10), Pruning::enabled()),
            &config,
        )
        .0
         .0;
        println!("cost {cost}");
    }

    {
        let (mut a, mut b) = uniform_seeded(350 * scale, 0.08, 5);
        let (mut a2, _) = uniform_seeded(50 * scale, 0.08, 8);
        let (mut a3, mut b3) = uniform_seeded(100 * scale, 0.08, 9);
        a.append(&mut a2);
        //b.append(&mut b2);
        a.append(&mut a3);
        b.append(&mut b3);
        let ref a = a;
        let ref b = b;

        config.filepath = "imgs/fig8/deletion".into();
        let cost = astar(
            a,
            b,
            &CSH::new(MatchConfig::inexact(10), Pruning::enabled()),
            &config,
        )
        .0
         .0;
        println!("cost {cost}");
    }

    {
        let (mut a, mut b) = uniform_seeded(100 * scale, 0.08, 1);
        let rng = &mut rand_chacha::ChaCha8Rng::seed_from_u64(2 as u64);
        let (mut a2, mut b2) = pa_generate::SeqPairGenerator {
            length: 300 * scale,
            error_rate: 0.08,
            error_model: ErrorModel::SymmetricRepeat,
            pattern_length: Some(10 * scale),
        }
        .generate(rng);
        let (mut a3, mut b3) = uniform_seeded(100 * scale, 0.08, 3);
        a.append(&mut a2);
        b.append(&mut b2);
        a.append(&mut a3);
        b.append(&mut b3);
        let ref a = a;
        let ref b = b;

        config.filepath = "imgs/fig8/repeats".into();
        let cost = astar(
            a,
            b,
            &CSH::new(MatchConfig::inexact(10), Pruning::enabled()),
            &config,
        )
        .0
         .0;
        println!("cost {cost}");
    }
}