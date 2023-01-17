//! This generates the visualizations used in the limitations section of the paper.
#[cfg(not(feature = "vis"))]
fn main() {}

#[cfg(feature = "vis")]
fn main() {
    use std::time::Duration;

    use astarpa::{
        aligners::{astar::Astaras, Aligner},
        canvas::BLACK,
        prelude::*,
        visualizer::{Gradient, Visualizer, When},
    };
    use rand::SeedableRng;

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

        config.filepath = std::path::PathBuf::from("imgs/fig8-extra/high-error-rate-dt/");
        let mut a_star = Astar {
            // NOTE: TRUE HERE
            dt: true,
            h: CSH {
                match_config: MatchConfig::inexact(10),
                pruning: Pruning::enabled(),
                use_gap_cost: false,
                c: PhantomData::<HintContours<BruteForceContour>>::default(),
            },
            v: config.clone(),
        };
        let cost = a_star.align(a, b).0;
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

        config.filepath = "imgs/fig8-extra/deletion-dt".into();
        let mut a_star = Astar {
            // NOTE: TRUE
            dt: true,
            h: CSH {
                match_config: MatchConfig::inexact(10),
                pruning: Pruning::enabled(),
                use_gap_cost: false,
                c: PhantomData::<HintContours<BruteForceContour>>::default(),
            },
            v: config.clone(),
        };
        let cost = a_star.align(a, b).0;
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

        config.filepath = "imgs/fig8-extra/deletion-dt-gapcost".into();
        let mut a_star = Astar {
            // NOTE: TRUE
            dt: true,
            h: CSH {
                match_config: MatchConfig::inexact(10),
                pruning: Pruning::enabled(),
                // NOTE: TRUE
                use_gap_cost: true,
                c: PhantomData::<HintContours<BruteForceContour>>::default(),
            },
            v: config.clone(),
        };
        let cost = a_star.align(a, b).0;
        println!("cost {cost}");
    }

    {
        let (mut a, mut b) = uniform_seeded(100 * scale, 0.08, 1);
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(2 as u64);
        let (mut a2, mut b2) = generate_pair(
            &GenerateOptions {
                length: 300 * scale,
                error_rate: 0.08,
                error_model: ErrorModel::DoubleMutatedRepeat,
                pattern_length: 10 * scale,
                m: None,
            },
            &mut rng,
        );
        let (mut a3, mut b3) = uniform_seeded(100 * scale, 0.08, 3);
        a.append(&mut a2);
        b.append(&mut b2);
        a.append(&mut a3);
        b.append(&mut b3);
        let ref a = a;
        let ref b = b;

        config.filepath = "imgs/fig8-extra/repeats-variable-k".into();
        let mut a_star = Astar {
            // NOTE: TRUE
            dt: true,
            h: CSH {
                // NOTE: At most 2 matches
                match_config: MatchConfig {
                    length: LengthConfig::Max(MaxMatches {
                        max_matches: 15,
                        k_min: 8,
                        k_max: 14,
                    }),
                    max_match_cost: 0,
                    window_filter: false,
                },
                pruning: Pruning::enabled(),
                // NOTE: TRUE
                use_gap_cost: true,
                c: PhantomData::<HintContours<BruteForceContour>>::default(),
            },
            v: config.clone(),
        };
        let cost = a_star.align(a, b).0;
        println!("cost {cost}");
    }
}