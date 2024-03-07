use pa_generate::uniform_seeded;
use pa_types::seq_to_string;
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis_types::canvas::{BLACK, RED};
use std::time::Duration;

fn main() {
    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::None;
    config.save_last = true;
    config.delay = Duration::from_secs_f32(1.0);
    config.paused = true;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.9);
    config.style.path_width = None;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    config.transparent_bmp = false;
    config.downscaler = 1;
    config.cell_size = 1;
    config.style.path = Some(BLACK);
    config.style.draw_matches = true;
    config.style.match_width = 4;
    config.style.match_shrink = 0;
    config.style.pruned_match = RED;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;

    config.filepath = "imgs/astarpa2-paper/trace".into();

    let mut astarpa2 = astarpa2::AstarPa2Params::full();
    astarpa2.heuristic.k = 8;
    astarpa2.block_width = 64;
    astarpa2.front.max_g /= 2;

    eprintln!("params: {:?}", astarpa2);
    {
        let (mut a, mut b) = uniform_seeded(110, 0.08, 0);
        let (mut a2, _) = uniform_seeded(100, 0.08, 1);
        let (mut a3, mut b3) = uniform_seeded(80, 0.08, 2);
        a.append(&mut a2);
        // b.append(&mut b2);
        a.append(&mut a3);
        b.append(&mut b3);
        eprintln!("{} {}", a.len(), b.len());
        eprintln!("{}\n{}", seq_to_string(&a), seq_to_string(&b));

        let cost = astarpa2
            .make_aligner_with_visualizer(true, config.with_filename("deletion"))
            .align(&a, &b)
            .0;
        println!("cost {cost}");
    }
}
