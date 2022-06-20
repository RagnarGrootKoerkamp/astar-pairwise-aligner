use std::{cell::Cell, ops::Range, time::Duration};

use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
    Sdl,
};

use crate::prelude::Pos;

/// A visualizer can be used to visualize progress of an implementation.
pub trait VisualizerT {
    fn explore(&mut self, _pos: Pos) {}
    fn expand(&mut self, _pos: Pos) {}
}

/// A trivial visualizer that does not do anything.
pub struct NoVisualizer;
impl VisualizerT for NoVisualizer {}

#[derive(Clone)]
pub enum Gradient {
    NoGradient { expand: Color, explore: Color },
    Gradient(Range<Color>),
    // 0 <= start < end <= 1
    TurboGradient(Range<f32>),
}

#[derive(Clone)]
pub struct ColorScheme {
    gradient: Gradient,
    bg_color: Color,
}

#[derive(Clone)]
pub struct Config {
    cell_size: usize,
    prescaler: usize, //for scaling image
    filepath: String, //maybe &str instead
    drawing: bool,
    delay: Cell<f32>,
    saving: bool,
    colors: ColorScheme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cell_size: 8,
            prescaler: 1,
            saving: false,
            filepath: String::from(""),
            drawing: false,
            delay: Cell::new(0.2),
            colors: ColorScheme {
                gradient: Gradient::NoGradient {
                    expand: Color::BLUE,
                    explore: Color::RGB(128, 0, 128),
                },
                bg_color: Color::BLACK,
            },
        }
    }
}

//Saves canvas to bmp file
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

struct Visualizer {
    canvas: Option<sdl2::render::Canvas<sdl2::video::Window>>,
    sdl_context: Sdl,
    config: Config,
    expanded: Vec<Pos>,
    explored: Vec<Pos>,
    width: u32,
    height: u32,
    file_number: usize,
}

impl Visualizer {
    fn init(&mut self, config: &Config, len1: u32, len2: u32) {
        self.file_number = 0;
        self.width = len1;
        self.height = len2;
        self.config = config.clone();
        self.sdl_context = sdl2::init().unwrap();
        let canvas_size_cells = Pos(len1 + 1, len2 + 1);
        let video_subsystem = self.sdl_context.video().unwrap();
        video_subsystem.gl_attr().set_double_buffer(true);
        let window = if config.drawing || config.saving {
            Some(
                video_subsystem
                    .window(
                        "A*PA",
                        canvas_size_cells.0 as u32
                            * config.cell_size as u32
                            * config.prescaler as u32,
                        (canvas_size_cells.1 as u32)
                            * config.cell_size as u32
                            * config.prescaler as u32,
                    )
                    //.borderless()
                    .build()
                    .unwrap(),
            )
        } else {
            None
        };
        self.canvas = window.map(|w| w.into_canvas().build().unwrap());
    }
    fn draw(&mut self) {
        if !self.config.saving && !self.config.drawing {
            return;
        }
        let scale = self.config.prescaler as u32;
        let cell_size = self.config.cell_size as u32;

        let cell_begin = |Pos(i, j): Pos| -> Point {
            Point::new((i * cell_size) as i32, (j * cell_size) as i32)
        };

        let draw_pixel = |canvas: &mut Canvas<Window>, p: Pos, c: Color| {
            canvas.set_draw_color(c);
            let mut begin = cell_begin(p);
            begin *= scale as i32;
            canvas
                .fill_rect(Rect::new(
                    begin.x,
                    begin.y,
                    cell_size * scale,
                    cell_size * scale,
                ))
                .unwrap();
        };

        if let Some(canvas) = &mut self.canvas {
            canvas.set_draw_color(self.config.colors.bg_color);
            canvas
                .fill_rect(Rect::new(0, 0, self.width, self.height))
                .unwrap();
            match &self.config.colors.gradient {
                Gradient::NoGradient { expand, explore } => {
                    for pos in &self.explored {
                        draw_pixel(canvas, *pos, *explore);
                    }
                    for pos in &self.expanded {
                        draw_pixel(canvas, *pos, *expand);
                    }
                }
                Gradient::Gradient(range) => {
                    //Draws only expnded states!
                    fn gradient(f: f32, c1: Color, c2: Color) -> Color {
                        let frac =
                            |a: u8, b: u8| -> u8 { ((1. - f) * a as f32 + f * b as f32) as u8 };

                        return Color::RGB(frac(c1.r, c2.r), frac(c1.g, c2.g), frac(c1.b, c2.b));
                    }
                    let mut i: f32 = 0.;
                    let d = self.expanded.len() as f32;
                    for pos in &self.expanded {
                        draw_pixel(canvas, *pos, gradient(i / d, range.start, range.end));
                        i += 1.;
                    }
                }
                Gradient::TurboGradient(range) => {
                    //Draws only expnded states!
                    let grad = colorgrad::turbo();
                    let mut i: f64 = 0.;
                    let d = self.expanded.len() as f64;
                    let coef = (range.end - range.start) as f64;
                    for pos in &self.expanded {
                        let val = i / d;
                        let clr = grad.at(range.start as f64 + (val * coef)).rgba_u8();
                        draw_pixel(canvas, *pos, Color::RGBA(clr.0, clr.1, clr.2, clr.3));
                        i += 1.;
                    }
                }
            }
            if self.config.saving {
                save_canvas(&canvas, &self.config.filepath, self.file_number);
                self.file_number += 1;
            }
            if self.config.drawing {
                ::std::thread::sleep(Duration::from_secs_f32(self.config.delay.get()));
            }
        } else {
            return;
        }

        //Keyboard events

        let sleep_duration = 0.01;
        let mut duration: f32 = 0.;
        let mut delay_tmp = &self.config.delay.get();
        let mut is_playing: bool = true;
        if self.config.drawing {
            if self.skip == 1 {
                return;
            } else if self.skip == 2 {
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
                                //pause
                                is_playing = !is_playing;
                            }
                            Keycode::Escape => {
                                //next frame
                                break 'outer;
                            }
                            Keycode::F => {
                                //faster
                                delay_tmp *= 0.8;
                            }
                            Keycode::S => {
                                //slower
                                delay_tmp /= 0.8;
                            }
                            Keycode::A => {
                                //skip to the last frame
                                if self.skip == 0 {
                                    self.skip = 1;
                                }
                                break 'outer;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                ::std::thread::sleep(Duration::from_secs_f32(sleep_duration));
                if is_playing {
                    duration += sleep_duration;
                    if duration >= delay_tmp {
                        break 'outer;
                    }
                }
            }
        }
        if self.skip == 2 {
            self.skip == 0;
        }
        config.delay.set(delay_tmp);
        return;
    }
}

impl VisualizerT for Visualizer {
    fn expand(&mut self, _pos: Pos) {
        self.expanded.push(_pos);
        self.draw();
    }

    fn explore(&mut self, _pos: Pos) {
        self.explored.push(_pos);
        self.draw();
    }
}
