//This function is a copy of same-named function in the "heuristic.rs" file. I copied it here because drawing logic should be separated from heuristic for independent visualition of all algorithms no matter do they use heuristic or not. If approves this, I will delete the duplicate function in the "heuristuc.rs" file. (The same applies to function save_canvas)
//But I have troubles with exporting and implementing as an argument heuristic (heuristic + contours + matches/seeds + gradient), so for now I commented it

use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{pixels::Color, render::BlendMode};

use crate::{astar::Config, prelude::*};

use colorgrad::Color as OtherColor;
use image::{save_buffer, GenericImage, GenericImageView, ImageBuffer, RgbImage};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn save_canvas(
    canvas: &sdl2::render::Canvas<sdl2::video::Window>,
    filepath: &str,
    number: usize,
) {
    let pixel_format = canvas.default_pixel_format();
    let mut pixels = canvas.read_pixels(canvas.viewport(), pixel_format).unwrap();
    let (width, height) = canvas.output_size().unwrap();
    let pitch = pixel_format.byte_size_of_pixels(width as usize);
    let surf = sdl2::surface::Surface::from_data(
        pixels.as_mut_slice(),
        width,
        height,
        pitch as u32,
        pixel_format,
    )
    .unwrap();
    surf.save_bmp(format!("{}{}.bmp", filepath, number))
        .unwrap_or_else(|error| {
            print!("Problem saving the file: {:?}", error);
        });
}

pub fn display2(
    target: Pos,
    _explored: Option<&Vec<Pos>>,
    _expanded: Option<&Vec<Pos>>,
    _prev: Option<&Vec<Vec<Pos>>>, //explored in previous iteration
    path: Option<&Vec<Pos>>,
    tree: Option<Vec<(Pos, Edge)>>,
    canvas_size_cells: Pos,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    sdl_context: &mut sdl2::Sdl,
    mut is_playing: bool,
    config: &Config,
    file_number: usize,
    skip: usize, // 0 - do not skip, 1 - skip, 2 - stay on the picture
    PATH_COLOR1: Color,
) -> (bool, usize, usize) {
    //is_playing, file_number, skip
    if !config.drawing && !config.saving {
        return (is_playing, file_number + 1, skip);
    }
    use sdl2::{
        event::Event,
        keyboard::Keycode,
        pixels::Color,
        rect::{Point, Rect},
        render::Canvas,
        video::Window,
    };
    use std::time::Duration;

    // Pos: position in edit graph
    // Cell: position in drawing, of size CELL_SIZE x CELL_SIZE
    // Pixel: one pixel

    const CELL_SIZE: u32 = 8;
    const SMALL_CELL_MARGIN: u32 = 2;
    const TREE_COLOR: Color = Color::BLUE;
    const TREE_COLOR_MATCH: Color = Color::CYAN;
    const EXPANDED_COLOR: Color = Color::BLUE;
    const EXPLORED_COLOR: Color = Color::RGB(128, 0, 128);
    const PREV_COLOR: Color = Color::RGBA(0, 255, 255, 100);

    canvas.set_blend_mode(BlendMode::Blend);

    // Conversions
    let cell_center = |Pos(i, j): Pos| -> Point {
        Point::new(
            (i * CELL_SIZE + CELL_SIZE / 2) as i32,
            (j * CELL_SIZE + CELL_SIZE / 2) as i32 + v_offset as i32,
        )
    };
    let cell_begin = |Pos(i, j): Pos| -> Point {
        Point::new(
            (i * CELL_SIZE) as i32,
            (j * CELL_SIZE) as i32 + v_offset as i32,
        )
    };

    canvas.set_draw_color(Color::WHITE);
    canvas.clear();

    let draw_pixel = |canvas: &mut Canvas<Window>, p: Pos, c: Color, small: bool| {
        canvas.set_draw_color(c);
        let mut begin = cell_begin(p);
        begin *= SCALE as i32;
        canvas
            .fill_rect(Rect::new(
                begin.x,
                begin.y,
                (CELL_SIZE - if small { 2 * SMALL_CELL_MARGIN } else { 0 }) * SCALE,
                (CELL_SIZE - if small { 2 * SMALL_CELL_MARGIN } else { 0 }) * SCALE,
            ))
            .unwrap();
    };

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

    let draw_thick_line_horizontal = |canvas: &mut Canvas<Window>,
                                      mut from: Point,
                                      mut to: Point,
                                      mut width: i32,
                                      mut margin: i32| {
        from *= SCALE as i32;
        to *= SCALE as i32;
        width *= SCALE as i32;
        margin *= SCALE as i32;
        for w in -width / 2..width - width / 2 {
            canvas
                .draw_line(
                    Point::new(from.x + margin, from.y + w),
                    Point::new(to.x - margin, to.y + w),
                )
                .unwrap();
        }
    };

    let draw_thick_line_vertical = |canvas: &mut Canvas<Window>,
                                    mut from: Point,
                                    mut to: Point,
                                    mut width: i32,
                                    mut margin: i32| {
        from *= SCALE as i32;
        to *= SCALE as i32;
        width *= SCALE as i32;
        margin *= SCALE as i32;
        for w in -width / 2..width - width / 2 {
            canvas
                .draw_line(
                    Point::new(from.x + w, from.y + margin),
                    Point::new(to.x + w, to.y - margin),
                )
                .unwrap();
        }
    };

    fn gradient(f: f32, c1: Color, c2: Color) -> Color {
        let frac = |a: u8, b: u8| -> u8 { ((1. - f) * a as f32 + f * b as f32) as u8 };

        return Color::RGB(frac(c1.r, c2.r), frac(c1.g, c2.g), frac(c1.b, c2.b));
    }

    // Draw explored
    if let Some(explored) = _explored {
        for p in explored {
            draw_pixel(canvas, *p, EXPLORED_COLOR, false);
        }
    }
    //Draw cells, explored in previous iteration
    if let Some(prev) = _prev {
        for i in prev {
            for p in i {
                draw_pixel(canvas, *p, PREV_COLOR, false);
            }
        }
    }
    // Draw expanded
    if let Some(expanded) = _expanded {
        for p in expanded {
            draw_pixel(canvas, *p, EXPANDED_COLOR, false);
        }
    }

    // Draw tree
    if let Some(tree) = tree {
        for (p, e) in tree {
            if let Some(prev) = e.back(&p) {
                canvas.set_draw_color(if e == Edge::Match {
                    TREE_COLOR_MATCH
                } else {
                    TREE_COLOR
                });
                draw_thick_line_diag(canvas, cell_center(prev), cell_center(p), 1);
            }
        }
    }

    // Draw path
    if let Some(path) = path {
        canvas.set_draw_color(PATH_COLOR1);
        let mut prev = Pos(0, 0);
        for p in path {
            draw_thick_line_diag(canvas, cell_center(prev), cell_center(*p), 2);
            prev = *p;
        }
    }

    // Draw path
    if let Some(path) = path {
        canvas.set_draw_color(PATH_COLOR1);
        let mut prev = Pos(0, 0);
        for p in path {
            draw_thick_line_diag(canvas, cell_center(prev), cell_center(*p), 2);
            prev = *p;
        }
    }
    if config.saving {
        save_canvas(canvas, &config.filepath, file_number);
    }
    let sleep_duration = 0.01;
    let mut duration: f32 = 0.;
    let delay_tmp = config.delay.get();
    if config.drawing {
        if skip == 1 {
            return (is_playing, file_number + 1, skip);
        } else if skip == 2 {
            config.delay.set(1000.);
        }
        canvas.present();
        'outer: loop {
            for event in sdl_context.event_pump().unwrap().poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Q),
                        ..
                    } => {
                        panic!();
                    }
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => match key {
                        Keycode::P => {
                            config.delay.set(delay_tmp);
                            is_playing = !is_playing;
                            return (is_playing, file_number + 1, 0);
                        }
                        Keycode::Escape => {
                            config.delay.set(delay_tmp);
                            break 'outer;
                        }
                        Keycode::F => {
                            config.delay.set(delay_tmp);
                            config.delay.set(0.8 * config.delay.get());
                        }
                        Keycode::S => {
                            config.delay.set(delay_tmp);
                            config.delay.set(1. / 0.8 * config.delay.get());
                        }
                        Keycode::A => {
                            config.delay.set(delay_tmp);
                            return (is_playing, file_number + 1, 1);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            ::std::thread::sleep(Duration::from_secs_f32(sleep_duration));
            duration += sleep_duration;
            if is_playing && duration >= config.delay.get() {
                config.delay.set(delay_tmp);
                return (is_playing, file_number + 1, 0);
            }
        }
    }
    config.delay.set(delay_tmp);
    (is_playing, file_number + 1, skip)
}

pub fn draw_explored_states1(r: &AlignResult, filename: &str) {
    if r.astar.explored_states.is_empty() {
        return;
    }

    let low = Pos(0, 0);
    let high = Pos(r.input.len_a as u32, r.input.len_b as u32);
    let width = high.0 - low.0 + 1;
    let height = high.1 - low.1 + 1;
    let mut imgbuf = ImageBuffer::new(width, height);
    let gray_bg = 0.97;

    for pixel in imgbuf.enumerate_pixels_mut() {
        *pixel.2 = image::Rgb([gray_bg, gray_bg, gray_bg]);
    }

    let grad = colorgrad::turbo();
    let min_step = 0;
    let max_steps = r.astar.explored_states.len() - 1;

    for (i, pos) in r.astar.explored_states.iter().enumerate() {
        let val = (i - min_step) / (max_steps - min_step);
        let clr = grad.at(0.25 + (val as f64 * 1.3)).rgba();
        imgbuf.put_pixel(pos.0, pos.1, image::Rgb([clr.0, clr.1, clr.2]));
    }

    for pos in &r.path {
        //img[row.i-lbox, row.j-lbox] = mcolors.to_rgba('xkcd:black')
        imgbuf.put_pixel(pos.0, pos.1, image::Rgb([0., 0., 0.]));
    }
}

pub fn draw_explored_states(r: &AlignResult, filename: &str) {
    if r.astar.explored_states.is_empty() {
        return;
    }

    let low = Pos(0, 0);
    let high = Pos(r.input.len_a as u32, r.input.len_b as u32);
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
    canvas.set_blend_mode(BlendMode::Blend);

    /*let gray_bg = 0.97; // BG COLOR
    canvas.set_draw_color(Color::RGB(
        (gray_bg * 255.) as u8,
        (gray_bg * 255.) as u8,
        (gray_bg * 255.) as u8,
    ));*/
    canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
    canvas.fill_rect(Rect::new(0, 0, width, height));

    let grad = colorgrad::turbo();
    let min_step = 0;
    let max_steps = r.astar.explored_states.len() - 1;

    for (i, pos) in r.astar.explored_states.iter().enumerate() {
        let val = (i - min_step) as f64 / (max_steps - min_step) as f64;
        let clr = grad.at(0.25 + (val * 0.65)).rgba_u8();
        canvas.set_draw_color(Color::RGBA(clr.0, clr.1, clr.2, clr.3));
        canvas.draw_point(Point::new(pos.0 as i32, pos.1 as i32));
    }

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    for pos in &r.path {
        canvas.draw_point(Point::new(pos.0 as i32, pos.1 as i32));
    }

    save_canvas(&canvas, filename, 0);
}

pub fn display3(
    //Attention! Saves only to fixed path
    _target: Pos,
    explored: &Vec<Pos>,
    file_number: usize,
    canvas: &mut Canvas<Window>,
    filepath: &str,
) {
    let low = Pos(0, 0);
    let high = _target;
    let width = high.0 - low.0 + 1;
    let height = high.1 - low.1 + 1;

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
    let max_steps = 11618 - 1;
    for (i, pos) in explored.iter().enumerate() {
        let val = (i - min_step) as f64 / (max_steps - min_step) as f64;
        let clr = grad.at(0.25 + (val * 0.65)).rgba_u8();
        canvas.set_draw_color(Color::RGB(clr.0, clr.1, clr.2));
        canvas.draw_point(Point::new(pos.1 as i32, pos.0 as i32));
    }

    save_canvas(&canvas, filepath, file_number);
}
