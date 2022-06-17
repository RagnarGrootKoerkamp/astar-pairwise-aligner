//! This module contains implementations of other alignment algorithms.

use std::{cell::Cell, time::Duration};

use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
    Sdl,
};

use crate::prelude::{Cost, Pos, Sequence};

use self::{cigar::Cigar, nw::PATH};

pub mod cigar;
pub mod diagonal_transition;
pub mod layer;
pub mod nw;
pub mod nw_lib;

#[derive(Clone)]
enum Gradient {
    NoGradient(Color, Color), //(expanded_color,explored color)
    Gradient(Color, Color),   //(start color, end color)
    TurboGradient(f32, f32), //(start value, end value); start < end; start > 0 && end > 0; start < 1 && end <= 1
}

#[derive(Clone)]
struct ColorScheme {
    gradient: Gradient,
    bg_color: Color,
}

// let default_colors: ColorScheme = ColorScheme{};

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
                gradient: Gradient::NoGradient(Color::BLUE, Color::RGB(128, 0, 128)),
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

/// A visualizer can be used to visualize progress of an implementation.
pub trait Visualizer {
    fn init(&mut self, config: &Config, len1: u32, len2: u32) {}
    fn explore(&mut self, _pos: Pos) {}
    fn expand(&mut self, _pos: Pos) {}
    fn draw(&mut self) {}
}

/// A trivial visualizer that does not do anything.
struct NoVisualizer;
impl Visualizer for NoVisualizer {}

/// A trivial visualizer that does not do anything.
struct Visualizer_obj {
    canvas: Option<sdl2::render::Canvas<sdl2::video::Window>>,
    sdl_context: Sdl,
    config: Config,
    expanded: Vec<Pos>,
    explored: Vec<Pos>,
    width: u32,
    height: u32,
    file_number: usize,
}

impl Visualizer for Visualizer_obj {
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

    fn expand(&mut self, _pos: Pos) {
        self.expanded.push(_pos);
        self.draw();
    }

    fn explore(&mut self, _pos: Pos) {
        self.explored.push(_pos);
        self.draw();
    }
    fn draw(&mut self) {
        if !self.config.saving && !self.config.drawing {
            return;
        }
        let SCALE = self.config.prescaler as u32;
        let CELL_SIZE = self.config.cell_size as u32;

        let cell_begin = |Pos(i, j): Pos| -> Point {
            Point::new((i * CELL_SIZE) as i32, (j * CELL_SIZE) as i32)
        };

        let draw_pixel = |canvas: &mut Canvas<Window>, p: Pos, c: Color| {
            canvas.set_draw_color(c);
            let mut begin = cell_begin(p);
            begin *= SCALE as i32;
            canvas
                .fill_rect(Rect::new(
                    begin.x,
                    begin.y,
                    CELL_SIZE * SCALE,
                    CELL_SIZE * SCALE,
                ))
                .unwrap();
        };

        if let Some(canvas) = &mut self.canvas {
            canvas.set_draw_color(self.config.colors.bg_color);
            canvas
                .fill_rect(Rect::new(0, 0, self.width, self.height))
                .unwrap();
            match self.config.colors.gradient {
                Gradient::NoGradient(expanded_color, explored_color) => {
                    for pos in &self.explored {
                        draw_pixel(canvas, *pos, explored_color);
                    }
                    for pos in &self.expanded {
                        draw_pixel(canvas, *pos, expanded_color);
                    }
                }
                Gradient::Gradient(c1, c2) => {
                    //Draws only expnded states!
                    fn gradient(f: f32, c1: Color, c2: Color) -> Color {
                        let frac =
                            |a: u8, b: u8| -> u8 { ((1. - f) * a as f32 + f * b as f32) as u8 };

                        return Color::RGB(frac(c1.r, c2.r), frac(c1.g, c2.g), frac(c1.b, c2.b));
                    }
                    let mut i: f32 = 0.;
                    let d = self.expanded.len() as f32;
                    for pos in &self.expanded {
                        draw_pixel(canvas, *pos, gradient(i / d, c1, c2));
                        i += 1.;
                    }
                }
                Gradient::TurboGradient(start, end) => {
                    //Draws only expnded states!
                    let grad = colorgrad::turbo();
                    let mut i: f64 = 0.;
                    let d = self.expanded.len() as f64;
                    let coef = (end - start) as f64;
                    for pos in &self.expanded {
                        let val = i / d;
                        let clr = grad.at(start as f64 + (val * coef)).rgba_u8();
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
        }
    }
}

/// An aligner is a type that supports aligning sequences using some algorithm.
/// It should implement the most general of the methods below.
/// The cost-only variant can sometimes be implemented using less memory.
///
/// There is one function for each cost model:
/// - LinearCost
/// - AffineCost
///
/// The output can be:
/// - cost only
/// - cost and alignment
/// - cost, alignment, and a visualization.
///
/// Note that insertions are when `b` has more characters than `a`, and deletions are when `b` has less characters than `a`.
pub trait Aligner {
    fn cost(&self, a: &Sequence, b: &Sequence) -> Cost {
        self.align(a, b).0
    }

    /// TODO: Make this return a path as well.
    fn align(&self, a: &Sequence, b: &Sequence) -> (Cost, PATH, Cigar) {
        self.visualize(a, b, &mut NoVisualizer)
    }

    fn visualize(
        &self,
        _a: &Sequence,
        _b: &Sequence,
        _visualizer: &mut impl Visualizer,
    ) -> (Cost, PATH, Cigar) {
        unimplemented!("This aligner does not support visualizations!");
    }
}

#[cfg(test)]
mod tests;
