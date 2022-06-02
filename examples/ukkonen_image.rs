// To create a video from images use this command:

// ffmpeg -framerate 4 -i %d.bmp -vf fps=4 -pix_fmt yuv420p output1.mp4

// (You need to have ffmpeg installed. And make sure that binary is in the folder that is included in PATH (I have no idea tbh does Mac have PATH or not. Maybe this thing with PATH is only for windows))

// Sometimes there can be an error like this: height(or width) not divisible by 2. Use this command in this case:

// ffmpeg -framerate 4 -i %d.bmp -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" -pix_fmt yuv420p output1.mp4

use std::cell::Cell;

use num_traits::abs;
use pairwise_aligner::{
    astar::Config,
    diagonal_transition::{biwfa, biwfa5, Args},
    drawing::save_canvas,
    prelude::*,
    ukkonen::ukkonen_vis,
    ukkonen::{ukkonen, ukkonen2},
};
use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};

fn main() {
    let n = 500;
    let e = 0.2;

    let m = 0;
    let k = 3;

    let (ref a, ref b, ref alphabet, stats) = setup(n, e);

    println!("{}\n{}\n", to_string(a), to_string(b));

    let _target = Pos::from_length(&a, &b);
    //Ukkonen

    let mut d = max(2, abs(a.len() as i32 - b.len() as i32) as usize);
    let mut r = d + 1;
    let mut explored = vec![];
    let mut path = vec![];
    while r > d {
        (r, path) = ukkonen2(a, b, d, &mut explored);

        println!("d = {} r = {}", d, r);
        d *= 2;
        r *= 2;
    }

    println!("Ukkonen says that edit distance is {}", r / 2);

    let low = Pos(0, 0);
    let high = _target;
    let width = high.0 - low.0 + 1;
    let height = high.1 - low.1 + 1;

    let mut sdl_context = sdl2::init().unwrap();
    let canvas_size_cells = Pos(high.0 - low.0 + 1, high.1 - low.1 + 1);
    let video_subsystem = sdl_context.video().unwrap();
    const CELL_SIZE: u32 = 1;
    const SCALE: u32 = 1;
    video_subsystem.gl_attr().set_double_buffer(true);
    let window = video_subsystem
        .window(
            "A*PA",
            canvas_size_cells.0 as u32 * CELL_SIZE * SCALE,
            (canvas_size_cells.1 as u32) * CELL_SIZE * SCALE + v_offset * SCALE,
        )
        .borderless()
        .build()
        .unwrap();
    let ref mut canvas = window.into_canvas().build().unwrap();

    let gray_bg = 0.97; //BG COLOR
    canvas.set_draw_color(Color::RGB(
        (gray_bg * 255.) as u8,
        (gray_bg * 255.) as u8,
        (gray_bg * 255.) as u8,
    ));
    //canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
    canvas.fill_rect(Rect::new(0, 0, width, height));

    let grad = colorgrad::turbo();
    let min_step = 0;
    let max_steps = explored.len() - 1;
    explored.reverse();
    for (i, pos) in explored.iter().enumerate() {
        let val = (max_steps - i - min_step) as f64 / (max_steps - min_step) as f64;
        let clr = grad.at(0.25 + (val * 0.65)).rgba_u8();
        canvas.set_draw_color(Color::RGB(clr.0, clr.1, clr.2));
        canvas.draw_point(Point::new(pos.1 as i32, pos.0 as i32));
        //imgbuf.put_pixel(pos.0, pos.1, image::Rgb([clr.0, clr.1, clr.2]));
    }

    let draw_thick_line_diag =
        |canvas: &mut Canvas<Window>, mut from: Point, mut to: Point, mut width: usize| {
            to *= SCALE as i32;
            from *= SCALE as i32;
            width *= SCALE as usize;
            canvas.draw_line(from, to).unwrap();
            for mut w in 1..width as i32 {
                if w % 2 == 1 {
                    w = (w + 1) / 2;
                    canvas
                        .draw_line(
                            Point::new(from.x + w, from.y - w + 1),
                            Point::new(to.x + w - 1, to.y - w),
                        )
                        .unwrap();
                    canvas
                        .draw_line(
                            Point::new(from.x - w, from.y + w - 1),
                            Point::new(to.x - w + 1, to.y + w),
                        )
                        .unwrap();
                    canvas
                        .draw_line(
                            Point::new(from.x + w - 1, from.y - w),
                            Point::new(to.x + w, to.y - w + 1),
                        )
                        .unwrap();
                    canvas
                        .draw_line(
                            Point::new(from.x - w + 1, from.y + w),
                            Point::new(to.x - w, to.y + w - 1),
                        )
                        .unwrap();
                } else {
                    w /= 2;
                    canvas
                        .draw_line(
                            Point::new(from.x + w, from.y - w),
                            Point::new(to.x + w, to.y - w),
                        )
                        .unwrap();
                    canvas
                        .draw_line(
                            Point::new(from.x - w, from.y + w),
                            Point::new(to.x - w, to.y + w),
                        )
                        .unwrap();
                }
            }
        };

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    let mut prev = Pos(0, 0);
    let path1 = path.clone();
    for p in path {
        draw_thick_line_diag(
            canvas,
            Point::new(prev.1 as i32, prev.0 as i32),
            Point::new(p.1 as i32, p.0 as i32),
            1,
        );
        prev = p;
    }

    save_canvas(&canvas, "evals/astar-visualization/edlib2", 0);

    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.gl_attr().set_double_buffer(true);
    let window = video_subsystem
        .window(
            "A*PA",
            canvas_size_cells.0 as u32 * CELL_SIZE * SCALE,
            (canvas_size_cells.1 as u32) * CELL_SIZE * SCALE + v_offset * SCALE,
        )
        .borderless()
        .build()
        .unwrap();
    let ref mut canvas = window.into_canvas().build().unwrap();

    let args = Args {
        a1: 0,
        a2: a.len(),
        b1: 0,
        b2: b.len(),
        x_offset: 0,
        y_offset: 0,
    };

    let mut sz = 0;
    let mut explored = vec![];
    let mut queue: Vec<Args> = vec![];
    queue.push(args);
    let mut file_number = 0;
    let mut dist = 0;

    while sz < queue.len() {
        let arg = queue[sz].clone();
        (dist, file_number) = biwfa5(
            &a[arg.a1..arg.a2].to_vec(),
            &b[arg.b1..arg.b2].to_vec(),
            &mut explored,
            arg.x_offset,
            arg.y_offset,
            _target,
            file_number,
            canvas,
            &mut queue,
        );
        sz += 1;
    }

    let gray_bg = 0.97; //BG COLOR
    canvas.set_draw_color(Color::RGB(
        (gray_bg * 255.) as u8,
        (gray_bg * 255.) as u8,
        (gray_bg * 255.) as u8,
    ));

    //canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
    canvas.fill_rect(Rect::new(0, 0, width, height));

    let grad = colorgrad::turbo();
    let min_step = 0;
    let max_steps = explored.len() - 1;
    for (i, pos) in explored.iter().enumerate() {
        let val = (i - min_step) as f64 / (max_steps - min_step) as f64;
        let clr = grad.at(0.25 + (val * 0.65)).rgba_u8();
        canvas.set_draw_color(Color::RGB(clr.0, clr.1, clr.2));
        canvas.draw_point(Point::new(pos.0 as i32, pos.1 as i32));
    }

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    let mut prev = Pos(0, 0);
    let path2 = path1.clone();
    for p in path1 {
        draw_thick_line_diag(
            canvas,
            Point::new(prev.1 as i32, prev.0 as i32),
            Point::new(p.1 as i32, p.0 as i32),
            1,
        );
        prev = p;
    }

    println!("Expl: {}\n", explored.len());

    save_canvas(&canvas, "evals/astar-visualization/biwfa2", 0);

    let mut explored = vec![];
    let mut dist = biwfa(a, b, &mut explored);

    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem.gl_attr().set_double_buffer(true);
    let window = video_subsystem
        .window(
            "A*PA",
            canvas_size_cells.0 as u32 * CELL_SIZE * SCALE,
            (canvas_size_cells.1 as u32) * CELL_SIZE * SCALE + v_offset * SCALE,
        )
        .borderless()
        .build()
        .unwrap();
    let ref mut canvas = window.into_canvas().build().unwrap();

    let gray_bg = 0.97; //BG COLOR
    canvas.set_draw_color(Color::RGB(
        (gray_bg * 255.) as u8,
        (gray_bg * 255.) as u8,
        (gray_bg * 255.) as u8,
    ));

    //canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
    canvas.fill_rect(Rect::new(0, 0, width, height));

    let grad = colorgrad::turbo();
    let min_step = 0;
    let max_steps = explored.len() - 1;
    for (i, pos) in explored.iter().enumerate() {
        let val = (i - min_step) as f64 / (max_steps - min_step) as f64;
        let clr = grad.at(0.25 + (val * 0.65)).rgba_u8();
        canvas.set_draw_color(Color::RGB(clr.0, clr.1, clr.2));
        canvas.draw_point(Point::new(pos.0 as i32, pos.1 as i32));
    }

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    let mut prev = Pos(0, 0);
    for p in path2 {
        draw_thick_line_diag(
            canvas,
            Point::new(prev.1 as i32, prev.0 as i32),
            Point::new(p.1 as i32, p.0 as i32),
            1,
        );
        prev = p;
    }

    save_canvas(&canvas, "evals/astar-visualization/biwfa_short2", 0);
}
