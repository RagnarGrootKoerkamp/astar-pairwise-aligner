use pa_generate::uniform_seeded;
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis_types::canvas::{BLACK, RED};
use std::time::Duration;

fn main() {
    if !cfg!(feature = "example") {
        panic!("WARNING: Without the example feature, pruned matches aren't shown red for SH");
    }

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
    config.cell_size = 1;
    config.style.path = Some(BLACK);
    config.style.draw_matches = false;
    config.style.draw_matches = false;
    config.style.match_width = 4;
    config.style.match_shrink = 0;
    config.style.pruned_match = RED;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;

    config.filepath = "imgs/astarpa2-paper/doubling/doubling".into();

    let mut astarpa2 = astarpa2::AstarPa2Params::full();
    astarpa2.block_width = 256;

    let scale = 16;
    config.downscaler = 8;
    {
        let e = 0.16;
        let (mut a, mut b) = uniform_seeded(100 * scale, e, 0);
        let (mut a2, mut b2) = uniform_seeded(100 * scale, e, 1);
        let (mut a3, _) = uniform_seeded(100 * scale, e, 2);
        let (mut a4, mut b4) = uniform_seeded(100 * scale, e, 4);
        a.append(&mut a2);
        b.append(&mut b2);
        a.append(&mut a3);
        // b.append(&mut b3);
        a.append(&mut a4);
        b.append(&mut b4);

        let cost = astarpa2
            .make_aligner_with_visualizer(true, config)
            .align(&a, &b)
            .0;
        println!("{cost}");
    }
}
