pub mod bruteforce_csh;
pub mod chained_seed;
pub mod distance;
pub mod equal;
pub mod max;
pub mod mirror;
pub mod perfect;
pub mod seed;
pub mod symmetric;

use std::time::Duration;

pub use bruteforce_csh::*;
pub use chained_seed::*;
pub use distance::*;
pub use equal::*;
pub use max::*;
pub use mirror::*;
pub use perfect::*;
use rand::{prelude::Distribution, SeedableRng};
use rand_chacha::ChaCha8Rng;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};
pub use seed::*;
pub use symmetric::*;

use serde::Serialize;

use crate::{matches::Match, prelude::*};

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

    fn matches(&self) -> Option<&SeedMatches> {
        None
    }

    fn terminal_print(&self, target: Pos) {
        if !crate::config::print() {
            return;
        }

        let mut ps = HashMap::default();
        let mut rng = ChaCha8Rng::seed_from_u64(3144);
        let dist = rand::distributions::Uniform::new_inclusive(0u8, 255u8);
        let Pos(a, b) = target;
        let mut pixels = vec![vec![(None, None, false, false); 20 * b as usize]; 20 * a as usize];
        for i in 0..=a {
            for j in 0..=b {
                let p = Pos(i, j);
                let pixel = &mut pixels[p.0 as usize][p.1 as usize];

                let (val, parent_pos) = self.h_with_parent(p);
                let k = ps.len();
                let (_parent_id, color) = ps.entry(parent_pos).or_insert((
                    k,
                    termion::color::Rgb(
                        dist.sample(&mut rng),
                        dist.sample(&mut rng),
                        dist.sample(&mut rng),
                    ),
                ));
                if self.is_seed_start_or_end(p) {
                    pixel.2 = true;
                }
                pixel.0 = Some(*color);
                pixel.1 = Some(val);
            }
        }
        let print = |i: I, j: I| {
            let pixel = pixels[i as usize][j as usize];
            if pixel.2 {
                print!(
                    "{}{}",
                    termion::color::Fg(termion::color::Black),
                    termion::style::Bold
                );
            } else if pixel.3 {
                print!(
                    "{}{}",
                    termion::color::Fg(termion::color::Rgb(100, 100, 100)),
                    termion::style::Bold
                );
            }
            print!(
                "{}{:3} ",
                termion::color::Bg(pixel.0.unwrap_or(termion::color::Rgb(0, 0, 0))),
                pixel.1.map(|x| format!("{:3}", x)).unwrap_or_default()
            );
            print!(
                "{}{}",
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset)
            );
        };
        for j in 0..=b {
            for i in 0..=a {
                print(i, j);
            }
            println!(
                "{}{}",
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset)
            );
        }
    }

    // `max_val` is used to cap the color gradient.
    fn display(
        &self,
        a: &Sequence,
        b: &Sequence,
        target: Pos,
        _display_type: DisplayOptions,
        max_val: Option<Cost>,
        explored: Option<Vec<Pos>>,
        expanded: Option<Vec<Pos>>,
        path: Option<Vec<Pos>>,
    ) {
        //println!("Root h: {}", h.h(Pos(0, 0)));

        const PIXEL_SIZE: u32 = 10;
        const SMALL_PIXEL_BORDER: u32 = 2;

        const MATCH_COLOR: Color = Color::RGB(0, 200, 0);
        const CONTOUR_COLOR: Color = Color::RGB(0, 216, 0);
        const PATH_COLOR: Color = Color::RED;
        const H_COLOR: Color = Color::BLACK;
        const EXPANDED_COLOR: Color = Color::BLUE;
        const EXPLORED_COLOR: Color = Color::RGB(128, 0, 128);

        let low = Pos(0, 0);
        let high = target;

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        video_subsystem.gl_attr().set_double_buffer(true);
        let canvas_size = Point::new(
            min(1000, high.0 - low.0 + 1) as i32,
            min(1000, high.1 - low.1 + 1) as i32,
        );

        let pos_to_point = |Pos(x, y): Pos| -> Point {
            return Point::new(x as i32, y as i32);
            // Point::new(
            //     ((x - low.0) as usize * (canvas_size.x() + 1) as usize
            //         / (high.0 - low.0 + 1) as usize) as i32,
            //     ((y - low.1) as usize * (canvas_size.y() + 1) as usize
            //         / (high.1 - low.1 + 1) as usize) as i32,
            // )
        };

        let point_to_pos = |p: Point| -> Pos {
            //return Pos(p.x() as u32, p.y() as u32);
            Pos(
                low.0
                    + (p.x() as usize * (high.0 - low.0 + 1) as usize / canvas_size.x() as usize)
                        as u32,
                low.1
                    + (p.y() as usize * (high.1 - low.1 + 1) as usize / canvas_size.y() as usize)
                        as u32,
            )
        };

        let window = video_subsystem
            .window(
                "A*PA",
                canvas_size.x() as u32 * PIXEL_SIZE,
                canvas_size.y() as u32 * PIXEL_SIZE,
            )
            .borderless()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();

        let cell_center = |p: Point| -> Point {
            Point::new(
                p.x * PIXEL_SIZE as i32 + PIXEL_SIZE as i32 / 2,
                p.y * PIXEL_SIZE as i32 + PIXEL_SIZE as i32 / 2,
            )
        };

        let draw_pixel = |canvas: &mut Canvas<Window>, point: Point, c: Color, small: bool| {
            canvas.set_draw_color(c);
            canvas
                .fill_rect(Rect::new(
                    point.x * PIXEL_SIZE as i32 + if small { SMALL_PIXEL_BORDER as i32 } else { 0 },
                    point.y * PIXEL_SIZE as i32 + if small { SMALL_PIXEL_BORDER as i32 } else { 0 },
                    PIXEL_SIZE - if small { 2 * SMALL_PIXEL_BORDER } else { 0 },
                    PIXEL_SIZE - if small { 2 * SMALL_PIXEL_BORDER } else { 0 },
                ))
                .unwrap();
        };

        let draw_thick_line =
            |canvas: &mut Canvas<Window>, from: Point, to: Point, width: usize| {
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

        fn gradient(f: f32, c1: Color, c2: Color) -> Color {
            let frac = |a: u8, b: u8| -> u8 { ((1. - f) * a as f32 + f * b as f32) as u8 };

            return Color::RGB(frac(c1.r, c2.r), frac(c1.g, c2.g), frac(c1.b, c2.b));
        }

        let val = |pos| self.h(pos);

        let max_val = max_val.unwrap_or_else(|| max(val(Pos(0, 0)), val(Pos(0, target.1))));

        // Draw the heuristic.
        let h_gradient = |h: Cost| -> Color {
            if h <= max_val {
                gradient(h as f32 / max_val as f32, Color::WHITE, H_COLOR)
            } else {
                H_COLOR
            }
        };
        for i in 0..canvas_size.x {
            for j in 0..canvas_size.y {
                let point = Point::new(i as i32, j as i32);
                let v = val(point_to_pos(point));
                draw_pixel(&mut canvas, point, h_gradient(v), false);
            }
        }

        // Draw explored
        if let Some(explored) = explored {
            for p in explored {
                draw_pixel(&mut canvas, pos_to_point(p), EXPLORED_COLOR, false);
            }
        }

        // Draw contours
        if let Some(_) = self.contour_value(Pos(0, 0)) {
            canvas.set_draw_color(CONTOUR_COLOR);
            let draw_right_border = |canvas: &mut Canvas<Window>, point: Point| {
                canvas
                    .draw_line(
                        Point::new(
                            (point.x + 1) * PIXEL_SIZE as i32,
                            (point.y + 0) * PIXEL_SIZE as i32,
                        ),
                        Point::new(
                            (point.x + 1) * PIXEL_SIZE as i32,
                            (point.y + 1) * PIXEL_SIZE as i32,
                        ),
                    )
                    .unwrap();
            };
            let draw_bottom_border = |canvas: &mut Canvas<Window>, point: Point| {
                canvas
                    .draw_line(
                        Point::new(
                            (point.x + 0) * PIXEL_SIZE as i32,
                            (point.y + 1) * PIXEL_SIZE as i32,
                        ),
                        Point::new(
                            (point.x + 1) * PIXEL_SIZE as i32,
                            (point.y + 1) * PIXEL_SIZE as i32,
                        ),
                    )
                    .unwrap();
            };
            // Right borders
            for i in 0..canvas_size.x - 1 {
                for j in 0..canvas_size.y {
                    let point = Point::new(i as i32, j as i32);
                    let v = self.contour_value(point_to_pos(point)).unwrap();
                    let point_r = Point::new(i + 1 as i32, j as i32);
                    let v_r = self.contour_value(point_to_pos(point_r)).unwrap();
                    if v_r != v {
                        draw_right_border(&mut canvas, point);
                    }
                }
            }
            // Bottom borders
            for i in 0..canvas_size.x {
                for j in 0..canvas_size.y - 1 {
                    let point = Point::new(i as i32, j as i32);
                    let v = self.contour_value(point_to_pos(point)).unwrap();
                    // Bottom
                    let point_b = Point::new(i as i32, j + 1 as i32);
                    let v_b = self.contour_value(point_to_pos(point_b)).unwrap();
                    if v_b != v {
                        draw_bottom_border(&mut canvas, point);
                    }
                }
            }
        }

        // Draw expanded
        if let Some(expanded) = expanded {
            for p in expanded {
                draw_pixel(&mut canvas, pos_to_point(p), EXPANDED_COLOR, false);
            }
        }

        // Draw path
        // if let Some(path) = path {
        //     canvas.set_draw_color(PATH_COLOR);
        //     let mut prev = pos_to_point(Pos(0, 0));
        //     for p in path {
        //         let p = pos_to_point(p);
        //         //draw_thick_line(&mut canvas, cell_center(prev), cell_center(p), 2);
        //         draw_pixel(&mut canvas, p, PATH_COLOR, false);
        //         prev = p;
        //     }
        // }

        // Draw character matches
        if false {
            canvas.set_draw_color(MATCH_COLOR);
            for i in 0..a.len() as I {
                for j in 0..b.len() as I {
                    if a[i as usize] == b[j as usize] {
                        canvas
                            .draw_line(
                                cell_center(pos_to_point(Pos(i, j))),
                                cell_center(pos_to_point(Pos(i, j).add_diagonal(1))),
                            )
                            .unwrap();
                    }
                }
            }
        }

        // Draw matches
        if let Some(seed_matches) = self.matches() {
            canvas.set_draw_color(MATCH_COLOR);
            for m in &seed_matches.matches {
                if m.match_cost > 0 {
                    continue;
                }
                if true {
                    draw_thick_line(
                        &mut canvas,
                        cell_center(pos_to_point(m.start)),
                        cell_center(pos_to_point(m.end)),
                        4,
                    );
                } else {
                    let mut p = m.start;
                    draw_pixel(&mut canvas, pos_to_point(p), MATCH_COLOR, false);
                    draw_pixel(&mut canvas, pos_to_point(p), MATCH_COLOR, false);
                    loop {
                        p = p.add_diagonal(1);
                        draw_pixel(&mut canvas, pos_to_point(p), MATCH_COLOR, false);
                        if p == m.end {
                            break;
                        }
                    }
                }
            }
        }

        // Draw path
        if let Some(path) = path {
            canvas.set_draw_color(PATH_COLOR);
            let mut prev = pos_to_point(Pos(0, 0));
            for p in path {
                let p = pos_to_point(p);
                draw_thick_line(&mut canvas, cell_center(prev), cell_center(p), 1);
                prev = p;
            }
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
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        break 'outer;
                    }
                    _ => {}
                }
            }
            ::std::thread::sleep(Duration::from_secs_f32(0.01));
        }
    }
}
