//! This generates the visualizations used in the limitations section of the paper.

use astarpa::{astar, AstarPa};
use pa_generate::uniform_seeded;
use pa_heuristic::{MatchConfig, NoCost, Prune, Pruning, CSH, GCSH, SH};
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::NoVis;
use std::time::Duration;

fn main() {
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::None;
    config.save_last = true;
    config.delay = Duration::from_secs_f32(0.0001);
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.9);
    config.style.path_width = Some(3);
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    config.transparent_bmp = false;
    let scale = 1;
    //config.downscaler = scale as u32;
    config.downscaler = 1;
    config.cell_size = 1;
    config.style.draw_matches = true;
    config.style.match_width = 5;
    config.style.match_shrink = 0;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.transparent_bmp = true;

    config.filepath = "imgs/astarpa-paper/comparison".into();

    // 200 4%
    let (mut a, mut b) = uniform_seeded(200 * scale, 0.04, 1);
    // 200 Noisy region
    let (mut x, mut y) = uniform_seeded(200 * scale, 0.60, 2);
    a.append(&mut x);
    b.append(&mut y);
    // 100 4%
    let (mut x, mut y) = uniform_seeded(25 * scale, 0.04, 3);
    a.append(&mut x);
    b.append(&mut y);
    // 2x75 repeat @4%, with same seed
    let (mut x, mut y) = uniform_seeded(150 * scale, 0.04, 4);
    a.append(&mut x);
    b.append(&mut y);
    let (mut x, mut y) = uniform_seeded(150 * scale, 0.05, 4);
    a.append(&mut x);
    b.append(&mut y);
    // 100 8%
    let (mut x, mut y) = uniform_seeded(25 * scale, 0.08, 5);
    a.append(&mut x);
    b.append(&mut y);
    // 75 Gap
    let (mut x, _y) = uniform_seeded(75 * scale, 0.08, 6);
    a.append(&mut x);
    // 175 8%
    let (mut x, mut y) = uniform_seeded(175 * scale, 0.08, 7);
    a.append(&mut x);
    b.append(&mut y);

    let a = &a;
    let b = &b;

    let cost = astar(&a, &b, &NoCost, &NoVis).0 .0;
    println!("{} {}", a.len(), b.len());
    println!("cost {cost}");

    let k = 10;
    for pruning in [Prune::None, Prune::Both] {
        for dt in [false, true] {
            let suf1 = if pruning.is_enabled() { "" } else { "-noprune" };
            let suf2 = if dt { "-dt" } else { "" };
            if pruning == Prune::None {
                AstarPa {
                    dt,
                    h: NoCost,
                    v: config.with_filename(&("dijkstra".to_string() + suf1 + suf2)),
                }
                .align(a, b);
            }
            AstarPa {
                dt,
                h: SH::new(MatchConfig::inexact(k), Pruning::new(pruning)),
                v: config.with_filename(&("sh".to_string() + suf1 + suf2)),
            }
            .align(a, b);
            AstarPa {
                dt,
                h: CSH::new(MatchConfig::inexact(k), Pruning::new(pruning)),
                v: config.with_filename(&("csh".to_string() + suf1 + suf2)),
            }
            .align(a, b);
            AstarPa {
                dt,
                h: GCSH::new(MatchConfig::inexact(k), Pruning::new(pruning)),
                v: config.with_filename(&("gcsh".to_string() + suf1 + suf2)),
            }
            .align(a, b);
        }
    }
}
