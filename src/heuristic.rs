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
        false
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
    fn display(
        &self,
        target: Pos,
        max_val: Option<Cost>,
        _explored: Option<Vec<Pos>>,
        _expanded: Option<Vec<Pos>>,
        path: Option<Vec<Pos>>,
        tree: Option<Vec<(Pos, Edge)>>,
    ) {
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

        const CELL_SIZE: u32 = 14;
        const SMALL_CELL_MARGIN: u32 = 4;

        const SEED_COLOR: Color = Color::RGB(0, 0, 0);
        const MATCH_COLOR: Color = Color::RGB(0, 200, 0);
        const PRUNED_MATCH_COLOR: Color = Color::RED;
        const CONTOUR_COLOR: Color = Color::RGB(0, 216, 0);
        const TREE_COLOR: Color = Color::BLUE;
        const TREE_COLOR_MATCH: Color = Color::CYAN;
        const PATH_COLOR: Color = Color::BLUE;
        const H_COLOR: Color = Color::RGB(64, 64, 64);
        const _EXPANDED_COLOR: Color = Color::BLUE;
        const _EXPLORED_COLOR: Color = Color::RGB(128, 0, 128);

        let low = Pos(0, 0);
        let high = target;

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        video_subsystem.gl_attr().set_double_buffer(true);
        let canvas_size_cells = Pos(high.0 - low.0 + 1, high.1 - low.1 + 1);

        // Conversions
        let cell_center = |Pos(i, j): Pos| -> Point {
            Point::new(
                (i * CELL_SIZE + CELL_SIZE / 2) as i32,
                (j * CELL_SIZE + CELL_SIZE / 2) as i32,
            )
        };
        let cell_begin = |Pos(i, j): Pos| -> Point {
            Point::new((i * CELL_SIZE) as i32, (j * CELL_SIZE) as i32)
        };

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
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();

        let draw_pixel = |canvas: &mut Canvas<Window>, p: Pos, c: Color, small: bool| {
            canvas.set_draw_color(c);
            let begin = cell_begin(p);
            canvas
                .fill_rect(Rect::new(
                    begin.x,
                    begin.y,
                    CELL_SIZE - if small { 2 * SMALL_CELL_MARGIN } else { 0 },
                    CELL_SIZE - if small { 2 * SMALL_CELL_MARGIN } else { 0 },
                ))
                .unwrap();
        };

        let draw_thick_line_diag =
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

        let draw_thick_line_horizontal =
            |canvas: &mut Canvas<Window>, from: Point, to: Point, width: i32, margin: i32| {
                for w in -width / 2..width - width / 2 {
                    canvas
                        .draw_line(
                            Point::new(from.x + margin, from.y + w),
                            Point::new(to.x - margin, to.y + w),
                        )
                        .unwrap();
                }
            };

        fn gradient(f: f32, c1: Color, c2: Color) -> Color {
            let frac = |a: u8, b: u8| -> u8 { ((1. - f) * a as f32 + f * b as f32) as u8 };

            return Color::RGB(frac(c1.r, c2.r), frac(c1.g, c2.g), frac(c1.b, c2.b));
        }

        let max_h_val = max_val.unwrap_or_else(|| max(self.h(Pos(0, 0)), self.h(Pos(0, target.1))));

        // Draw the heuristic.
        let h_gradient = |h: Cost| -> Color {
            if h <= max_h_val {
                gradient(h as f32 / max_h_val as f32, Color::WHITE, H_COLOR)
            } else {
                H_COLOR
            }
        };
        for i in 0..canvas_size_cells.0 {
            for j in 0..canvas_size_cells.1 {
                let pos = Pos(i, j);
                let v = self.h(pos);
                draw_pixel(&mut canvas, pos, h_gradient(v), false);
            }
        }

        // // Draw explored
        // if let Some(explored) = explored {
        //     for p in explored {
        //         draw_pixel(&mut canvas, p, EXPLORED_COLOR, false);
        //     }
        // }

        // // Draw expanded
        // if let Some(expanded) = expanded {
        //     for p in expanded {
        //         draw_pixel(&mut canvas, p, EXPANDED_COLOR, false);
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
                    draw_thick_line_diag(&mut canvas, cell_center(m.start), cell_center(m.end), 4);
                } else {
                    let mut p = m.start;
                    draw_pixel(&mut canvas, p, MATCH_COLOR, false);
                    draw_pixel(&mut canvas, p, MATCH_COLOR, false);
                    loop {
                        p = p.add_diagonal(1);
                        draw_pixel(&mut canvas, p, MATCH_COLOR, false);
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
                    draw_thick_line_diag(&mut canvas, cell_center(prev), cell_center(p), 1);
                }
            }
        }

        // Draw path
        if let Some(path) = path {
            canvas.set_draw_color(PATH_COLOR);
            let mut prev = Pos(0, 0);
            for p in path {
                draw_thick_line_diag(&mut canvas, cell_center(prev), cell_center(p), 2);
                prev = p;
            }
        }

        // Draw contours
        if let Some(_) = self.contour_value(Pos(0, 0)) {
            canvas.set_draw_color(CONTOUR_COLOR);
            let draw_right_border = |canvas: &mut Canvas<Window>, Pos(i, j): Pos| {
                canvas
                    .draw_line(cell_begin(Pos(i + 1, j)), cell_begin(Pos(i + 1, j + 1)))
                    .unwrap();
            };
            let draw_bottom_border = |canvas: &mut Canvas<Window>, Pos(i, j): Pos| {
                canvas
                    .draw_line(cell_begin(Pos(i, j + 1)), cell_begin(Pos(i + 1, j + 1)))
                    .unwrap();
            };

            // Right borders
            for i in 0..canvas_size_cells.0 - 1 {
                for j in 0..canvas_size_cells.1 {
                    let pos = Pos(i, j);
                    let v = self.contour_value(pos).unwrap();
                    let pos_r = Pos(i + 1, j);
                    let v_r = self.contour_value(pos_r).unwrap();
                    if v_r != v {
                        draw_right_border(&mut canvas, pos);
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
                        draw_bottom_border(&mut canvas, pos);
                    }
                }
            }
        }

        // Draw seeds
        if let Some(seeds) = self.seeds() {
            for s in seeds {
                canvas.set_draw_color(SEED_COLOR);
                draw_thick_line_horizontal(
                    &mut canvas,
                    cell_center(Pos(s.start, 0)),
                    cell_center(Pos(s.end, 0)),
                    5,
                    2,
                );
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
