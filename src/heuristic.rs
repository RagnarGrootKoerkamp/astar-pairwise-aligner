pub mod bruteforce_csh;
pub mod chained_seed;
pub mod distance;
pub mod equal;
pub mod max;
pub mod mirror;
pub mod perfect;
pub mod seed;
pub mod symmetric;

pub use bruteforce_csh::*;
pub use chained_seed::*;
pub use distance::*;
pub use equal::*;
pub use max::*;
pub use mirror::*;
pub use perfect::*;

use sdl2::{pixels::Color, render::BlendMode};
pub use seed::*;
pub use symmetric::*;

use serde::Serialize;
use std::{cell::Cell};

use crate::{astar::Config, matches::Match, prelude::*};

#[derive(Serialize, Default, Clone)]
pub struct HeuristicParams {
    pub name: String,
    pub distance_function: String,
    pub k: I,
    pub max_match_cost: MatchCost,
    pub pruning: bool,
    pub build_fast: bool,
}

#[derive(Serialize, Clone)]
pub struct HeuristicStats {
    pub num_seeds: I,
    pub num_matches: usize,
    pub num_filtered_matches: usize,
    #[serde(skip_serializing)]
    pub matches: Vec<Match>,
    pub pruning_duration: f32,
    pub num_prunes: usize,
}

impl Default for HeuristicStats {
    fn default() -> Self {
        Self {
            num_seeds: 0,
            num_matches: 0,
            num_filtered_matches: 0,
            matches: Default::default(),
            pruning_duration: 0.,
            num_prunes: 0,
        }
    }
}

/// An object containing the settings for a heuristic.
pub trait Heuristic: std::fmt::Debug + Copy {
    type Instance<'a>: HeuristicInstance<'a>;

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::Instance<'a>;

    // Heuristic properties.
    fn name(&self) -> String;

    fn params(&self) -> HeuristicParams {
        HeuristicParams {
            name: self.name(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub struct DisplayOptions {}

/// An instantiation of a heuristic for a specific pair of sequences.
pub trait HeuristicInstance<'a> {
    fn h(&self, pos: Pos) -> Cost;

    /// The internal contour value at the given position, if available.
    fn contour_value(&self, _pos: Pos) -> Option<Cost> {
        None
    }

    fn h_with_parent(&self, pos: Pos) -> (Cost, Pos) {
        (self.h(pos), Pos::default())
    }

    type Hint: Copy + Default + std::fmt::Debug = ();
    fn h_with_hint(&self, pos: Pos, _hint: Self::Hint) -> (Cost, Self::Hint) {
        (self.h(pos), Default::default())
    }

    fn root_state(&self, _root_pos: Pos) -> Self::Hint {
        Default::default()
    }

    fn root_potential(&self) -> Cost {
        0
    }

    /// A* will checked for consistency whenever this returns true.
    fn is_seed_start_or_end(&self, _pos: Pos) -> bool {
        true
    }

    /// Returns the offset by which all expanded states in the priority queue can be shifted.
    ///
    /// `seed_cost`: The cost made in the seed ending at pos.
    fn prune(&mut self, _pos: Pos, _hint: Self::Hint, _seed_cost: MatchCost) -> Cost {
        0
    }

    /// Tells the heuristic that the position was explored, so it knows which
    /// positions need to be updated when propagating the pruning to the
    /// priority queue.
    fn explore(&mut self, _pos: Pos) {}

    fn stats(&self) -> HeuristicStats {
        Default::default()
    }

    fn matches(&self) -> Option<Vec<Match>> {
        None
    }

    fn seeds(&self) -> Option<&Vec<Seed>> {
        None
    }

    /// Display is only implemented when the `sdl2` feature is enabled.
    #[cfg(not(feature = "sdl2"))]
    fn display(
        &self,
        _target: Pos,
        _max_val: Option<Cost>,
        _explored: Option<Vec<Pos>>,
        _expanded: Option<Vec<Pos>>,
        _path: Option<Vec<Pos>>,
        _tree: Option<Vec<(Pos, Edge)>>,
    ) {
    }

    // `max_val` is used to cap the color gradient.
    #[cfg(feature = "sdl2")]

    fn save_canvas(
        canvas: &sdl2::render::Canvas<sdl2::video::Window>,
        filepath: &str,
        number: u32,
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

    fn display2(
        &self,
        target: Pos,
        _explored: Option<&Vec<Pos>>,
        _expanded: Option<&Vec<Pos>>,
        path: Option<&Vec<Pos>>,
        tree: Option<Vec<(Pos, Edge)>>,
        canvas_size_cells: Pos,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        sdl_context: &mut sdl2::Sdl,
        mut is_playing: bool,
        config: &Config,
        file_number: u32,
        skip: u32, // 0 - do not skip, 1 - skip, 2 - stay on the picture
        PATH_COLOR1: Color,
    ) -> (bool, u32, u32) {
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
        const CONTOUR_WIDTH: i32 = 2;

        const radius: i32 = (0.5 * SCALE as f32) as i32; // radius of small dots on the background

        const SEED_COLOR: Color = Color::RGB(0, 0, 0);
        const MATCH_COLOR: Color = Color::RGB(0, 150, 0);
        const PRUNED_MATCH_COLOR: Color = Color::RGB(0, 216, 0);
        const CONTOUR_COLOR: Color = Color::RGB(0, 150, 0);
        const TREE_COLOR: Color = Color::BLUE;
        const TREE_COLOR_MATCH: Color = Color::CYAN;
        const PATH_COLOR: Color = Color::BLUE;
        const H_COLOR: Color = Color::RGB(64, 64, 64);
        const EXPANDED_COLOR: Color = Color::BLUE;
        const EXPLORED_COLOR: Color = Color::RGB(128, 0, 128);

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

        let draw_thick_line_horizontal =
            |canvas: &mut Canvas<Window>,
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

        let max_h_val = config
            .hmax
            .unwrap_or_else(|| max(self.h(Pos(0, 0)), self.h(Pos(0, target.1))));

        // Draw the heuristic.
        let h_gradient = |h: Cost| -> Color {
            if h <= max_h_val {
                gradient(h as f32 / max_h_val as f32, Color::WHITE, H_COLOR)
            } else {
                H_COLOR
            }
        };
        const color_offset: u8 = 50;
        for i in 0..canvas_size_cells.0 {
            for j in 0..canvas_size_cells.1 {
                let pos = Pos(i, j);
                let v = self.h(pos);
                let h_color = h_gradient(v);
                draw_pixel(canvas, pos, h_color, false);
                let f: u8 = h_color.r.saturating_sub(color_offset);
                let color = Color::RGB(f, f, f);
                canvas.set_draw_color(color);
                let point = cell_center(pos) * SCALE as i32;
                //draw_pixel(canvas, pos, Color::BLACK, true);
                for y in -radius..radius + 1 {
                    let x = ((radius * radius - y * y) as f32).sqrt();
                    canvas.draw_line(
                        Point::new(point.x - x as i32, point.y + y),
                        Point::new(point.x + x as i32, point.y + y),
                    );
                }
            }
        }

        /*canvas.set_draw_color(Color::RGB(150, 150, 150));
        for i in 0..target.0 + 1 {
            for j in 0..target.1 + 1 {
                let pos = Pos(i, j);
                let point = cell_center(pos) * SCALE as i32;
                //draw_pixel(canvas, pos, Color::BLACK, true);
                for y in -radius..radius + 1 {
                    let x = ((radius * radius - y * y) as f32).sqrt();
                    canvas.draw_line(
                        Point::new(point.x - x as i32, point.y + y),
                        Point::new(point.x + x as i32, point.y + y),
                    );
                }
            }
        }*/

        // // Draw explored
        // if let Some(explored) = explored {
        //     for p in explored {
        //         draw_pixel(canvas, *p, EXPLORED_COLOR, false);
        //     }
        // }

        // // Draw expanded
        // if let Some(expanded) = expanded {
        //     for p in expanded {
        //         draw_pixel(canvas, *p, EXPANDED_COLOR, false);
        //     }
        // }

        // Draw matches
        if let Some(matches) = self.matches() {
            for m in &matches {
                if m.match_cost > 0 {
                    continue;
                }
                if true {
                    canvas.set_draw_color(match m.pruned {
                        MatchStatus::Active => MATCH_COLOR,
                        MatchStatus::Pruned => PRUNED_MATCH_COLOR,
                    });
                    let center1 = cell_center(m.start);
                    let center2 = cell_center(m.end);
                    let point1 = Point::new(center1.x + radius, center1.y + radius);
                    let point2 = Point::new(center2.x - radius, center2.y - radius);
                    draw_thick_line_diag(canvas, point1, point2, 4);
                } else {
                    let mut p = m.start;
                    draw_pixel(canvas, p, MATCH_COLOR, false);
                    draw_pixel(canvas, p, MATCH_COLOR, false);
                    loop {
                        p = p.add_diagonal(1);
                        draw_pixel(canvas, p, MATCH_COLOR, false);
                        if p == m.end {
                            break;
                        }
                    }
                }
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

        // Draw contours
        if let Some(_) = self.contour_value(Pos(0, 0)) {
            canvas.set_draw_color(CONTOUR_COLOR);
            /*let draw_right_border = |canvas: &mut Canvas<Window>, Pos(i, j): Pos| {
                canvas
                    .draw_line(
                        cell_begin(Pos(i + 1, j)) * SCALE as i32,
                        cell_begin(Pos(i + 1, j + 1)) * SCALE as i32,
                    )
                    .unwrap();
            };
            let draw_bottom_border = |canvas: &mut Canvas<Window>, Pos(i, j): Pos| {
                canvas
                    .draw_line(
                        cell_begin(Pos(i, j + 1)) * SCALE as i32,
                        cell_begin(Pos(i + 1, j + 1)) * SCALE as i32,
                    )
                    .unwrap();
            };*/

            let draw_right_border = |canvas: &mut Canvas<Window>, Pos(i, j): Pos| {
                draw_thick_line_vertical(
                    canvas,
                    cell_begin(Pos(i + 1, j)),
                    cell_begin(Pos(i + 1, j + 1)),
                    CONTOUR_WIDTH,
                    0,
                );
            };
            let draw_bottom_border = |canvas: &mut Canvas<Window>, Pos(i, j): Pos| {
                draw_thick_line_horizontal(
                    canvas,
                    cell_begin(Pos(i, j + 1)),
                    cell_begin(Pos(i + 1, j + 1)),
                    CONTOUR_WIDTH,
                    0,
                );
            };

            // Right borders
            for i in 0..canvas_size_cells.0 - 1 {
                for j in 0..canvas_size_cells.1 {
                    let pos = Pos(i, j);
                    let v = self.contour_value(pos).unwrap();
                    let pos_r = Pos(i + 1, j);
                    let v_r = self.contour_value(pos_r).unwrap();
                    if v_r != v {
                        draw_right_border(canvas, pos);
                    }
                }
            }
            // Bottom borders
            for i in 0..canvas_size_cells.0 {
                for j in 0..canvas_size_cells.1 - 1 {
                    let pos = Pos(i, j);
                    let v = self.contour_value(pos).unwrap();
                    let pos_l = Pos(i, j + 1);
                    let v_l = self.contour_value(pos_l).unwrap();
                    if v_l != v {
                        draw_bottom_border(canvas, pos);
                    }
                }
            }
        }

        // Draw seeds and potentials
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.fill_rect(Rect::new(0, 0, canvas.window().size().0, v_offset * SCALE));
        if let Some(seeds) = self.seeds() {
            canvas.set_draw_color(SEED_COLOR);
            for s in seeds {
                draw_thick_line_horizontal(
                    canvas,
                    Point::new(
                        cell_center(Pos(s.start, 0)).x,
                        ((CELL_SIZE / 2) + v_offset / 2 - 1) as i32,
                    ),
                    Point::new(
                        cell_center(Pos(s.end, 0)).x,
                        ((CELL_SIZE / 2) + v_offset / 2 - 1) as i32,
                    ),
                    3,
                    2,
                );
            }
            for i in 0..canvas_size_cells.0 {
                let pos = Pos(i, 0);
                let v = self.h(pos) + self.contour_value(pos).unwrap();
                let h_color = h_gradient(v);
                canvas.set_draw_color(h_color);
                let mut begin = cell_begin(pos);
                begin *= SCALE as i32;
                canvas
                    .fill_rect(Rect::new(begin.x, 0, CELL_SIZE * SCALE, 5 * SCALE))
                    .unwrap();
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
            Self::save_canvas(canvas, &config.filepath, file_number);
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

    fn display(
        &self,
        target: Pos,
        _max_val: Option<Cost>,
        explored: Option<Vec<Pos>>,
        expanded: Option<Vec<Pos>>,
        _path: Option<Vec<Pos>>,
        tree: Option<Vec<(Pos, Edge)>>,
    ) {
        use sdl2::{
            pixels::Color,
        };
        

        // Pos: position in edit graph
        // Cell: position in drawing, of size CELL_SIZE x CELL_SIZE
        // Pixel: one pixel

        const CELL_SIZE: u32 = 8;
        const SMALL_CELL_MARGIN: u32 = 2;

        const MATCH_COLOR: Color = Color::RGB(0, 200, 0);
        const CONTOUR_COLOR: Color = Color::RGB(0, 216, 0);
        const PATH_COLOR: Color = Color::RED;
        const H_COLOR: Color = Color::BLACK;
        const EXPANDED_COLOR: Color = Color::BLUE;
        const EXPLORED_COLOR: Color = Color::RGB(128, 0, 128);

        let low = Pos(0, 0);
        let high = target;

        let mut sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        video_subsystem.gl_attr().set_double_buffer(true);
        let canvas_size_cells = Pos(high.0 - low.0 + 1, high.1 - low.1 + 1);

        let window = video_subsystem
            .window(
                "A*PA",
                canvas_size_cells.0 as u32 * CELL_SIZE,
                canvas_size_cells.1 as u32 * CELL_SIZE,
            )
            .borderless()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        self.display2(
            target,
            explored.as_ref(),
            expanded.as_ref(),
            None,
            tree,
            canvas_size_cells,
            &mut canvas,
            &mut sdl_context,
            false,
            &Config {
                filepath: String::from(""),
                saving: false,
                drawing: true,
                hmax: Some(0),
                delay: Cell::new(0.2),
            },
            0,
            0,
            Color::BLUE,
        );
    }
}

fn useless() -> u32 {
    5
}
