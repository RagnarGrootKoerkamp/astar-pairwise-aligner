use crate::{aligners::Path, prelude::Pos};

/// A visualizer can be used to visualize progress of an implementation.
pub trait VisualizerT {
    fn explore(&mut self, _pos: Pos) {}
    fn expand(&mut self, _pos: Pos) {}

    /// This function should be called after completing each layer
    fn new_layer(&mut self) {}

    /// This function may be called after the main loop to display final image.
    fn last_frame(&mut self, _path: Option<&Path>) {}
}

/// A trivial visualizer that does not do anything.
pub struct NoVisualizer;
impl VisualizerT for NoVisualizer {}

#[cfg(feature = "sdl2")]
pub use with_sdl2::*;

#[cfg(feature = "sdl2")]
mod with_sdl2 {
    use super::*;
    use crate::prelude::Seq;
    use itertools::Itertools;
    use sdl2::{
        event::Event,
        keyboard::Keycode,
        pixels::Color,
        rect::{Point, Rect},
        render::Canvas,
        video::Window,
        Sdl,
    };
    use std::{
        cell::RefCell,
        ops::Range,
        path,
        time::{Duration, Instant},
    };

    pub struct Visualizer {
        canvas: Option<RefCell<Canvas<Window>>>,
        sdl_context: Sdl,
        config: Config,
        expanded: Vec<Pos>,
        explored: Vec<Pos>,
        width: u32,
        height: u32,
        file_number: usize,
        layer: Option<usize>,
        expanded_layers: Vec<usize>,
    }

    impl VisualizerT for Visualizer {
        fn expand(&mut self, pos: Pos) {
            self.expanded.push(pos);
            self.draw(false, None, false);
        }

        fn explore(&mut self, pos: Pos) {
            self.explored.push(pos);
            self.draw(false, None, false);
        }

        fn new_layer(&mut self) {
            if let Some(layer) = self.layer {
                self.layer = Some(layer + 1);
                self.expanded_layers.push(self.expanded.len());
            }
            self.draw(false, None, true);
        }

        fn last_frame(&mut self, path: Option<&Path>) {
            self.draw(true, path, true);
        }
    }

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

        fn explore(&self, _f: f32) -> Option<Color> {
            match self {
                Gradient::NoGradient { explore, .. } => Some(*explore),
                Gradient::Gradient(_) => None,
                Gradient::TurboGradient(_) => None,
            }
        }
    }

    #[derive(Clone)]
    pub struct Style {
        pub gradient: Gradient,
        pub bg_color: Color,
        pub path: Color,
        /// None to draw cells.
        pub path_width: Option<usize>,
    }

    #[derive(PartialEq, Eq, Clone, Copy)]
    pub enum Save {
        None,
        Last,
        All,
        Layers,
    }

    impl Save {
        fn do_save(&self, is_last: bool, new_layer: bool) -> bool {
            match &self {
                Save::None => false,
                Save::Last => is_last,
                Save::All => !new_layer,
                Save::Layers => new_layer,
            }
        }
    }

    #[derive(PartialEq, Eq, Clone, Copy)]
    pub enum Draw {
        None,
        Last,
        All,
        Layers,
    }

    impl Draw {
        fn do_draw(&self, is_last: bool, new_layer: bool) -> bool {
            match &self {
                Draw::None => false,
                Draw::Last => is_last,
                Draw::All => !new_layer,
                Draw::Layers => new_layer,
            }
        }
    }

    #[derive(Clone)]
    pub struct Config {
        pub cell_size: usize,
        pub prescaler: usize,
        pub filepath: String,
        pub draw: Draw,
        pub delay: f32,
        pub paused: bool,
        pub save: Save,
        pub style: Style,
        pub draw_old_on_top: bool,
        pub layer_drawing: bool,
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
                paused: false,
                style: Style {
                    gradient: Gradient::NoGradient {
                        expand: Color::BLUE,
                        explore: Color::RGB(128, 0, 128),
                    },
                    bg_color: Color::WHITE,
                    path: Color::BLACK,
                    path_width: Some(2),
                },
                draw_old_on_top: true,
                layer_drawing: false,
            }
        }
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
                        Some(RefCell::new(
                            video_subsystem
                                .window(
                                    &config.filepath,
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
                        ))
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
                layer: if config.layer_drawing { Some(0) } else { None },
                expanded_layers: vec![],
            }
        }

        fn cell_begin(&self, Pos(i, j): Pos) -> Point {
            Point::new(
                (i * self.config.cell_size as u32) as i32,
                (j * self.config.cell_size as u32) as i32,
            )
        }
        fn cell_center(&self, Pos(i, j): Pos) -> Point {
            Point::new(
                (i * self.config.cell_size as u32 + self.config.cell_size as u32 / 2) as i32,
                (j * self.config.cell_size as u32 + self.config.cell_size as u32 / 2) as i32,
            )
        }

        fn draw_pixel(&self, canvas: &mut Canvas<Window>, p: Pos, c: Color) {
            canvas.set_draw_color(c);
            let mut begin = self.cell_begin(p);
            begin *= self.config.prescaler as i32;
            canvas
                .fill_rect(Rect::new(
                    begin.x,
                    begin.y,
                    (self.config.cell_size * self.config.prescaler) as u32,
                    (self.config.cell_size * self.config.prescaler) as u32,
                ))
                .unwrap();
        }

        fn draw_diag_line(
            canvas: &mut Canvas<Window>,
            from: Point,
            to: Point,
            color: Color,
            width: usize,
        ) {
            canvas.set_draw_color(color);
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
        }

        #[allow(unused)]
        fn draw_thick_line_horizontal(
            canvas: &mut Canvas<Window>,
            from: Point,
            to: Point,
            width: i32,
            margin: i32,
        ) {
            for w in -width / 2..width - width / 2 {
                canvas
                    .draw_line(
                        Point::new(from.x + margin, from.y + w),
                        Point::new(to.x - margin, to.y + w),
                    )
                    .unwrap();
            }
        }

        //Saves canvas to bmp file
        fn save_canvas(&self, canvas: &mut Canvas<Window>, last: bool) {
            let path = if last {
                let file = path::Path::new(&self.config.filepath);
                if let Some(parent) = file.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                file.with_extension("bmp").to_owned()
            } else {
                // Make sure the directory exists.
                let mut dir = path::PathBuf::from(&self.config.filepath);
                std::fs::create_dir_all(&dir).unwrap();
                dir.push(self.file_number.to_string());
                dir.set_extension("bmp");
                dir
            };

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
            surf.set_color_key(true, self.config.style.bg_color)
                .unwrap();

            surf.save_bmp(path).unwrap_or_else(|error| {
                print!("Problem saving the file: {:?}", error);
            });
        }

        fn draw(&mut self, is_last: bool, path: Option<&Path>, is_new_layer: bool) {
            if !self.config.draw.do_draw(is_last, is_new_layer)
                && !self.config.save.do_save(is_last, is_new_layer)
            {
                return;
            }

            let cell_size = self.config.cell_size as u32;

            let Some(canvas) = &self.canvas else {return;};
            let mut canvas = canvas.borrow_mut();

            // DRAW

            // Draw background.
            canvas.set_draw_color(self.config.style.bg_color);
            canvas
                .fill_rect(Rect::new(
                    0,
                    0,
                    cell_size * self.width,
                    cell_size * self.height,
                ))
                .unwrap();

            // Draw explored and expanded.
            if self.config.draw_old_on_top {
                for (i, pos) in self.explored.iter().enumerate().rev() {
                    if let Some(color) = self
                        .config
                        .style
                        .gradient
                        .explore(i as f32 / self.explored.len() as f32)
                    {
                        self.draw_pixel(&mut canvas, *pos, color);
                    }
                }
                let mut current_layer = if let Some(layer) = self.layer {
                    layer
                } else {
                    0
                };
                for (i, pos) in self.expanded.iter().enumerate().rev() {
                    self.draw_pixel(
                    &mut canvas,
                    *pos,
                    self.config.style.gradient.expand(
                        if let Some(layer) = self.layer && layer != 0 {
                            if current_layer > 0
                                && i < self.expanded_layers[current_layer - 1]
                            {
                                current_layer -= 1;
                            }
                            current_layer as f32 / layer as f32
                        } else {
                                 i as f32 / self.expanded.len() as f32
                        },
                        ),
                    );
                }
            } else {
                for (i, pos) in self.explored.iter().enumerate() {
                    if let Some(color) = self
                        .config
                        .style
                        .gradient
                        .explore(i as f32 / self.explored.len() as f32)
                    {
                        self.draw_pixel(&mut canvas, *pos, color);
                    }
                }
                let mut current_layer = 0;
                for (i, pos) in self.expanded.iter().enumerate() {
                    self.draw_pixel(
                    &mut canvas,
                    *pos,
                    self.config.style.gradient.expand(
                        if let Some(layer) = self.layer && layer != 0 {
                            if current_layer < layer && i >= self.expanded_layers[current_layer] {
                                current_layer += 1;
                            }
                            current_layer as f32 / layer as f32
                        } else {
                                 i as f32 / self.expanded.len() as f32
                        },
                        ),
                );
                }
            }

            // Draw path.
            if let Some(path) = path {
                if let Some(path_width) = self.config.style.path_width {
                    for (from, to) in path.iter().tuple_windows() {
                        Self::draw_diag_line(
                            &mut canvas,
                            self.cell_center(*from),
                            self.cell_center(*to),
                            self.config.style.path,
                            path_width,
                        );
                    }
                } else {
                    for p in path {
                        self.draw_pixel(&mut canvas, *p, self.config.style.path)
                    }
                }
            }

            // SAVE

            if self.config.save.do_save(is_last, is_new_layer) {
                if is_last {
                    self.save_canvas(&mut canvas, is_last);
                } else {
                    self.save_canvas(&mut canvas, is_last);
                    self.file_number += 1;
                }
            }

            // SHOW

            if !self.config.draw.do_draw(is_last, is_new_layer) {
                return;
            }

            //Keyboard events

            let sleep_duration = 0.001;
            canvas.present();
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
                                if self.config.paused {
                                    self.config.paused = false;
                                    start_time = Instant::now();
                                } else {
                                    self.config.paused = true;
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

                if !self.config.paused
                    && !is_last
                    && start_time.elapsed().as_secs_f32() >= self.config.delay
                {
                    break 'outer;
                }
            }
        }
    }
}
