use pa_generate::uniform_seeded;
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::canvas::{BLACK, RED};
use std::time::Duration;

fn main() {
    if !cfg!(feature = "example") {
        panic!("WARNING: Without the example feature, pruned matches aren't shown red for SH");
    }
    if pa_bitpacking::B::BITS != 8 {
        panic!("Small blocks feature is required for useful scale");
    }

    let mut config = visualizer::Config::default();
    config.draw = When::None;
    config.save_last = false;
    config.delay = Duration::from_secs_f32(1.0);
    config.paused = true;
    config.style.bg_color = (255, 255, 255, 128);
    config.style.expanded = Gradient::BoundedGradient((220, 220, 220, 0)..(120, 120, 120, 0), 30);
    config.style.fixed = Some((0, 255, 0, 0));
    config.style.path_width = None;
    config.draw_old_on_top = false;
    config.layer_drawing = false;
    config.transparent_bmp = false;
    config.style.path = Some(BLACK);
    config.style.draw_matches = false;
    config.style.draw_matches = false;
    config.style.match_width = 4;
    config.style.match_shrink = 0;
    config.style.pruned_match = RED;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;
    config.style.draw_h_calls = false;
    config.style.draw_ranges = true;
    config.style.draw_fixed_h = true;

    config.filepath = "imgs/astarpa2-paper/doubling".into();

    let mut astarpa2 = astarpa2::AstarPa2Params::simple();
    astarpa2.doubling = astarpa2::DoublingType::BandDoublingStartIncrement {
        start: astarpa2::DoublingStart::H0,
        factor: 2.,
        start_increment: 32,
    };
    astarpa2.block_width = 8;
    astarpa2.front.incremental_doubling = true;
    config.cell_size = 6;
    config.downscaler = 1;
    config.save = When::Frames(vec![19, 67]);
    // config.save = When::All;

    let n = 50;
    let m = 50;
    let s1 = 1;
    let s2 = 105;
    eprintln!("LEN {} {}", s1, s2);
    let (a, _) = uniform_seeded(n, 0., s1);
    let (b, _) = uniform_seeded(m, 0., s2);

    let cost = astarpa2
        .make_aligner_with_visualizer(true, config.clone())
        .align(&a, &b)
        .0;
    println!("{cost}");
}
