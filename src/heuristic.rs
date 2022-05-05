pub mod bruteforce_csh;
pub mod chained_seed_heuristic;
pub mod distance;
pub mod equal_heuristic;
pub mod seed_heuristic;

use std::time::Duration;

pub use bruteforce_csh::*;
pub use chained_seed_heuristic::*;
pub use distance::*;
pub use equal_heuristic::*;
use rand::{prelude::Distribution, SeedableRng};
use rand_chacha::ChaCha8Rng;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Point};
pub use seed_heuristic::*;

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

pub enum DisplayType {
    Heuristic,
    Contours,
}

/// An instantiation of a heuristic for a specific pair of sequences.
pub trait HeuristicInstance<'a> {
    fn h(&self, pos: Pos) -> Cost;

    /// The internal contour value at the given position, if available.
    fn contour_value(&self, _pos: Pos) -> Cost {
        unimplemented!();
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

    // hmax is used to cap the color gradient.
    fn display(&self, target: Pos, display_type: DisplayType, max_val: Option<Cost>) {
        //println!("Root h: {}", h.h(Pos(0, 0)));

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
            //return Point::new(x as i32, y as i32);
            Point::new(
                ((x - low.0) as usize * (canvas_size.x() + 1) as usize
                    / (high.0 - low.0 + 1) as usize) as i32,
                ((y - low.1) as usize * (canvas_size.y() + 1) as usize
                    / (high.1 - low.1 + 1) as usize) as i32,
            )
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
                "rust-sdl2 demo",
                canvas_size.x() as u32,
                canvas_size.y() as u32,
            )
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();

        fn from_to_gradient(f: f32, c1: Color, c2: Color) -> Color {
            let frac = |a: u8, b: u8| -> u8 { ((1. - f) * a as f32 + f * b as f32) as u8 };

            return Color::RGB(frac(c1.r, c2.r), frac(c1.g, c2.g), frac(c1.b, c2.b));
        }

        let val = |pos| match display_type {
            DisplayType::Heuristic => self.h(pos),
            DisplayType::Contours => self.contour_value(pos),
        };

        let max_val = max_val.unwrap_or_else(|| max(val(Pos(0, 0)), val(Pos(0, target.1))));

        // Draw background in color, matches in black.
        let gradient = |h: Cost| -> Color {
            if h <= max_val {
                from_to_gradient(h as f32 / max_val as f32, Color::WHITE, Color::BLACK)
            } else {
                Color::GRAY
            }
        };
        for i in 0..canvas_size.x {
            for j in 0..canvas_size.y {
                let point = Point::new(i as i32, j as i32);
                let v = val(point_to_pos(point));
                canvas.set_draw_color(gradient(v));
                canvas.draw_point(point).unwrap();
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
                        todo!();
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
