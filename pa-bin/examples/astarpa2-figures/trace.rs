use pa_generate::uniform_seeded;
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::canvas::{BLACK, BLUE, RED};
use std::time::Duration;

fn main() {
    if !cfg!(feature = "example") {
        panic!("WARNING: Without the example feature, pruned matches aren't shown red for SH");
    }
    if pa_bitpacking::B::BITS != 8 {
        panic!("small_blocks feature is required for useful scale");
    }

    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save = When::None;
    config.save_last = true;
    config.delay = Duration::from_secs_f32(1.0);
    config.paused = true;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::TurboGradient(0.25..0.9);
    config.style.expanded = Gradient::Gradient((220, 220, 220, 0)..(160, 160, 160, 0));
    config.style.path_width = Some(3);
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    config.transparent_bmp = false;
    config.style.path = Some(BLACK);
    // config.style.path = Some(BLACK);
    config.style.draw_matches = false;
    config.style.draw_matches = false;
    config.style.match_width = 4;
    config.style.match_shrink = 0;
    config.style.pruned_match = RED;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.style.draw_h_calls = false;
    config.style.draw_f_calls = false;

    config.style.trace = Some(((130, 179, 102, 0), (0, 127, 255, 0)));

    config.filepath = "imgs/astarpa2-paper/trace/trace".into();
    config.cell_size = 8;
    config.downscaler = 1;

    let mut astarpa2 = astarpa2::AstarPa2Params::simple();
    astarpa2.block_width = 16;
    astarpa2.front.incremental_doubling = false;
    astarpa2.front.max_g = 6;

    let e = 0.30;
    let (mut a, mut b) = uniform_seeded(50, e, 5);
    let (mut a1, _) = uniform_seeded(40, e, 1);
    let (mut a2, mut b2) = uniform_seeded(20, e, 6);
    a.append(&mut a1);
    a.append(&mut a2);
    b.append(&mut b2);
    eprintln!("LENS {} {}", a.len(), b.len());

    let cost = astarpa2
        .make_aligner_with_visualizer(true, config)
        .align(&a, &b)
        .0;
    println!("{cost}");
}
