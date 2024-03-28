use itertools::Itertools;
use pa_vis::visualizer::{self, Gradient, When};
use pa_vis::{
    canvas::{BLACK, BLUE, RED},
    VisualizerT,
};
use std::{iter::repeat, time::Duration};

fn main() {
    let mut config = visualizer::Config::default();
    config.delay = Duration::from_secs_f32(1.0);
    config.paused = true;
    config.style.bg_color = (255, 255, 255, 128);
    // config.style.expanded = Gradient::TurboGradient(0.25..0.9);
    config.style.expanded = Gradient::Gradient((200, 200, 200, 0)..(32, 32, 32, 0));
    config.style.path_width = None;
    config.draw_old_on_top = false;
    config.transparent_bmp = false;
    config.downscaler = 1;
    config.style.path = Some(BLACK);
    config.style.trace = Some((BLUE, (0, 127, 255, 0)));
    config.style.draw_matches = true;
    config.style.match_width = 4;
    config.style.match_shrink = 0;
    config.style.pruned_match = RED;
    config.style.draw_dt = false;
    config.style.draw_f = false;
    config.style.draw_labels = false;

    config.filepath = "imgs/astarpa2-paper/simd".into();
    config.cell_size = 16;

    config.draw = When::None;
    config.save = When::Frames(vec![50]);
    config.layer_drawing = true;
    // config.save = When::All;
    // config.layer_drawing = true;
    config.save_last = false;

    const N: usize = 2;
    const L: usize = 4;
    const B: usize = 4;
    let n = 13;
    let m = N * L * B;

    let a = repeat(b'A').take(n - 1).collect_vec();
    let b = repeat(b'A').take(m - 1).collect_vec();

    let v = &mut config.build(&a, &b);

    pa_bitpacking::simd::vis_block_of_rows::<N, B>(n, 0, v);
}
