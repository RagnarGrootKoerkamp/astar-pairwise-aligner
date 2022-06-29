use std::{
    ops::Range,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
    Sdl,
};

use crate::prelude::{Pos, Seq};

/// A visualizer can be used to visualize progress of an implementation.
pub trait VisualizerT {
    fn explore(&mut self, _pos: Pos) {}
    fn expand(&mut self, _pos: Pos) {}

    //This function may be called after the main loop to display final image.
    fn last_frame(&mut self) {}
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

impl Gradient {
    fn expand(&self, f: f32) -> Color {
        match self {
            Gradient::NoGradient { expand, .. } => *expand,
            Gradient::Gradient(range) => {
                let frac = |a: u8, b: u8| -> u8 { ((1. - f) * a as f32 + f * b as f32) as u8 };
                Color::RGB(
                    frac(range.start.r, range.end.r),
                    frac(range.start.g, range.end.g),
                    frac(range.start.b, range.end.b),
                )
            }
            Gradient::TurboGradient(range) => {
                let f = range.start + f * (range.end - range.start);
                let c = colorgrad::turbo().at(f as f64).rgba_u8();
                Color::RGBA(c.0, c.1, c.2, c.3)
            }
        }
    }

    fn explore(&self, _f: f32) -> Color {
        match self {
            Gradient::NoGradient { explore, .. } => *explore,
            Gradient::Gradient(_) => todo!(),
            Gradient::TurboGradient(_) => todo!(),
        }
    }
}

#[derive(Clone)]
pub struct ColorScheme {
    pub gradient: Gradient,
    pub bg_color: Color,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Save {
    None,
    Last,
    All,
}

impl Save {
    fn do_save(&self, is_last: bool) -> bool {
        match &self {
            Save::None => false,
            Save::Last => is_last,
            Save::All => true,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Draw {
    None,
    Last,
    All,
}

impl Draw {
    fn do_draw(&self, is_last: bool) -> bool {
        match &self {
            Draw::None => false,
            Draw::Last => is_last,
            Draw::All => true,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub cell_size: usize,
    pub prescaler: usize, //for scaling image
    pub filepath: String, //maybe &str instead
    pub draw: Draw,
    pub delay: f32,
    pub save: Save,
    pub colors: ColorScheme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cell_size: 8,
            prescaler: 1,
            save: Save::None,
            filepath: String::from(""),
            draw: Draw::None,
            delay: 0.2,
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

pub struct Visualizer {
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
    pub fn new(config: Config, a: Seq, b: Seq) -> Self {
        let sdl_context = sdl2::init().unwrap();
        Visualizer {
            canvas: {
                let canvas_size_cells = Pos::from(a.len() + 1, b.len() + 1);
                let video_subsystem = sdl_context.video().unwrap();
                video_subsystem.gl_attr().set_double_buffer(true);
                if config.draw != Draw::None || config.save != Save::None {
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
                            .unwrap()
                            .into_canvas()
                            .build()
                            .unwrap(),
                    )
                } else {
                    None
                }
            },
            sdl_context,
            config: config.clone(),
            expanded: vec![],
            explored: vec![],
            width: a.len() as u32 + 1,
            height: b.len() as u32 + 1,
            file_number: 0,
        }
    }

    //Saves canvas to bmp file
    pub fn save_canvas(&self, last: bool) {
        let path = if last {
            let file = Path::new(&self.config.filepath);
            if let Some(parent) = file.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            file.with_extension("bmp").to_owned()
        } else {
            // Make sure the directory exists.
            let mut dir = PathBuf::from(&self.config.filepath);
            std::fs::create_dir_all(&dir).unwrap();
            dir.push(self.file_number.to_string());
            dir.set_extension("bmp");
            dir
        };

        let canvas = self.canvas.as_ref().unwrap();

        let pixel_format = canvas.default_pixel_format();
        let mut pixels = canvas.read_pixels(canvas.viewport(), pixel_format).unwrap();
        let (width, height) = canvas.output_size().unwrap();
        let pitch = pixel_format.byte_size_of_pixels(width as usize);
        let mut surf = sdl2::surface::Surface::from_data(
            pixels.as_mut_slice(),
            width,
            height,
            pitch as u32,
            pixel_format,
        )
        .unwrap();
        surf.set_color_key(true, self.config.colors.bg_color)
            .unwrap();

        surf.save_bmp(path).unwrap_or_else(|error| {
            print!("Problem saving the file: {:?}", error);
        });
    }

    fn draw(&mut self, is_last: bool) {
        if !self.config.draw.do_draw(is_last) && !self.config.save.do_save(is_last) {
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

        let Some(canvas) = &mut self.canvas else {return;};

        // Draw background.
        canvas.set_draw_color(self.config.colors.bg_color);
        canvas
            .fill_rect(Rect::new(
                0,
                0,
                cell_size * self.width,
                cell_size * self.height,
            ))
            .unwrap();

        for (i, pos) in self.explored.iter().enumerate() {
            draw_pixel(
                canvas,
                *pos,
                self.config
                    .colors
                    .gradient
                    .explore(i as f32 / self.explored.len() as f32),
            );
        }
        for (i, pos) in self.expanded.iter().enumerate() {
            draw_pixel(
                canvas,
                *pos,
                self.config
                    .colors
                    .gradient
                    .expand(i as f32 / self.expanded.len() as f32),
            );
        }

        // SAVE

        if self.config.save.do_save(is_last) {
            if is_last {
                self.save_canvas(is_last);
            } else {
                self.save_canvas(is_last);
                self.file_number += 1;
            }
        }

        // DRAW

        if !self.config.draw.do_draw(is_last) {
            return;
        }

        //Keyboard events

        let sleep_duration = 0.00001;
        let mut paused = false;
        self.canvas.as_mut().unwrap().present();
        let mut start_time = Instant::now();
        'outer: loop {
            for event in self.sdl_context.event_pump().unwrap().poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::X),
                        ..
                    } => {
                        panic!("Running aborted by user!");
                    }
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => match key {
                        Keycode::P => {
                            //pause
                            if paused {
                                paused = false;
                                start_time = Instant::now();
                            } else {
                                paused = true;
                            }
                        }
                        Keycode::Escape | Keycode::Space => {
                            //next frame
                            break 'outer;
                        }
                        Keycode::F => {
                            //faster
                            self.config.delay *= 0.8;
                        }
                        Keycode::S => {
                            //slower
                            self.config.delay /= 0.8;
                        }
                        Keycode::Q => {
                            self.config.draw = Draw::Last;
                            break 'outer;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            ::std::thread::sleep(Duration::from_secs_f32(sleep_duration));

            if !paused && !is_last && start_time.elapsed().as_secs_f32() >= self.config.delay {
                break 'outer;
            }
        }
    }
}

impl VisualizerT for Visualizer {
    fn expand(&mut self, pos: Pos) {
        self.expanded.push(pos);
        self.draw(false);
    }

    fn explore(&mut self, pos: Pos) {
        self.explored.push(pos);
        self.draw(false);
    }

    fn last_frame(&mut self) {
        self.draw(true);
    }
}
