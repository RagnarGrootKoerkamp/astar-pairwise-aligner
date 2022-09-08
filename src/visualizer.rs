//! To turn images into a video, use this:
//!
//! ```sh
//! ffmpeg -framerate 20 -i %d.bmp output.mp4
//! ```
//! or when that gives errors:
//! ```sh
//! ffmpeg -framerate 20 -i %d.bmp -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" output.mp4
//! ```

use crate::{
    aligners::{cigar::CigarOp, edit_graph::State, Path},
    heuristic::{HeuristicInstance, ZeroCostI},
    prelude::Pos,
};

type ParentFn<'a> = Option<&'a dyn Fn(State) -> Option<(State, [Option<CigarOp>; 2])>>;

/// A visualizer can be used to visualize progress of an implementation.
pub trait VisualizerT {
    fn explore(&mut self, pos: Pos) {
        self.explore_with_h::<ZeroCostI>(pos, None);
    }
    fn expand(&mut self, pos: Pos) {
        self.expand_with_h::<ZeroCostI>(pos, None);
    }
    fn extend(&mut self, pos: Pos) {
        self.extend_with_h::<ZeroCostI>(pos, None);
    }
    fn explore_with_h<'a, H: HeuristicInstance<'a>>(&mut self, _pos: Pos, _h: Option<&H>) {}
    fn expand_with_h<'a, H: HeuristicInstance<'a>>(&mut self, _pos: Pos, _h: Option<&H>) {}
    fn extend_with_h<'a, H: HeuristicInstance<'a>>(&mut self, _pos: Pos, _h: Option<&H>) {}

    /// This function should be called after completing each layer
    fn new_layer(&mut self) {
        self.new_layer_with_h::<ZeroCostI>(None);
    }
    fn new_layer_with_h<'a, H: HeuristicInstance<'a>>(&mut self, _h: Option<&H>) {}

    /// This function may be called after the main loop to display final image.
    fn last_frame(&mut self, path: Option<&Path>) {
        self.last_frame_with_h::<ZeroCostI>(path, None, None);
    }
    fn last_frame_with_tree(&mut self, path: Option<&Path>, parent: ParentFn) {
        self.last_frame_with_h::<ZeroCostI>(path, parent, None);
    }
    fn last_frame_with_h<'a, H: HeuristicInstance<'a>>(
        &mut self,
        _path: Option<&Path>,
        _parent: ParentFn<'_>,
        _h: Option<&H>,
    ) {
    }
}

/// A trivial visualizer that does not do anything.
pub struct NoVisualizer;
impl VisualizerT for NoVisualizer {}

#[cfg(feature = "sdl2")]
pub use with_sdl2::*;

#[cfg(feature = "sdl2")]
mod with_sdl2 {
    use super::*;
    use crate::{
        aligners::{edit_graph::State, StateT},
        matches::MatchStatus,
        prelude::Seq,
    };
    use itertools::Itertools;
    #[cfg(feature = "sdl2_ttf")]
    use sdl2::ttf::{Font, Sdl2TtfContext};
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
        collections::HashMap,
        ops::Range,
        path,
        time::{Duration, Instant},
    };

    #[cfg(feature = "sdl2_ttf")]
    lazy_static! {
        static ref TTF_CONTEXT: Sdl2TtfContext = sdl2::ttf::init().unwrap();
    }

    #[derive(PartialEq, Eq, Clone, Copy)]
    pub enum Type {
        Expanded,
        Explored,
        Extended,
    }
    use Type::*;

    pub struct Visualizer {
        canvas: Option<RefCell<Canvas<Window>>>,
        sdl_context: Sdl,
        #[cfg(feature = "sdl2_ttf")]
        font: Font<'static, 'static>,
        config: Config,
        pub expanded: Vec<(Type, Pos)>,
        width: u32,
        height: u32,
        frame_number: usize,
        file_number: usize,
        layer: Option<usize>,
        expanded_layers: Vec<usize>,
    }

    impl VisualizerT for Visualizer {
        fn expand_with_h<'a, H: HeuristicInstance<'a>>(&mut self, pos: Pos, h: Option<&H>) {
            if !(pos <= Pos(self.width - 1, self.height - 1)) {
                return;
            }
            self.expanded.push((Expanded, pos));
            self.draw(false, None, false, h, None);
        }

        fn explore_with_h<'a, H: HeuristicInstance<'a>>(&mut self, pos: Pos, h: Option<&H>) {
            if !(pos <= Pos(self.width - 1, self.height - 1)) {
                return;
            }
            self.expanded.push((Explored, pos));
            self.draw(false, None, false, h, None);
        }

        fn extend_with_h<'a, H: HeuristicInstance<'a>>(&mut self, pos: Pos, h: Option<&H>) {
            if !(pos <= Pos(self.width - 1, self.height - 1)) {
                return;
            }
            self.expanded.push((Extended, pos));
            self.draw(false, None, false, h, None);
        }

        fn new_layer_with_h<'a, H: HeuristicInstance<'a>>(&mut self, h: Option<&H>) {
            if let Some(layer) = self.layer {
                self.layer = Some(layer + 1);
                self.expanded_layers.push(self.expanded.len());
            }
            self.draw(false, None, true, h, None);
        }

        fn last_frame_with_h<'a, H: HeuristicInstance<'a>>(
            &mut self,
            path: Option<&Path>,
            parent: ParentFn<'_>,
            h: Option<&H>,
        ) {
            self.draw(true, path, false, h, parent);
        }
    }

    #[derive(Clone)]
    pub enum Gradient {
        Fixed(Color),
        Gradient(Range<Color>),
        // 0 <= start < end <= 1
        TurboGradient(Range<f64>),
    }

    impl Gradient {
        fn color(&self, f: f64) -> Color {
            match self {
                Gradient::Fixed(color) => *color,
                Gradient::Gradient(range) => {
                    let frac =
                        |a: u8, b: u8| -> u8 { ((1. - f) * a as f64 + f * b as f64).ceil() as u8 };
                    Color::RGB(
                        frac(range.start.r, range.end.r),
                        frac(range.start.g, range.end.g),
                        frac(range.start.b, range.end.b),
                    )
                }
                Gradient::TurboGradient(range) => {
                    let f = range.start + f * (range.end - range.start);
                    let c = colorgrad::turbo().at(f).to_rgba8();
                    Color::RGBA(c[0], c[1], c[2], c[3])
                }
            }
        }
    }

    #[derive(Clone)]
    pub struct Style {
        pub expanded: Gradient,
        pub explored: Option<Color>,
        pub extended: Option<Color>,
        pub bg_color: Color,
        /// None to disable
        pub path: Option<Color>,
        /// None to draw cells.
        pub path_width: Option<usize>,

        /// None to disable
        pub tree: Option<Color>,
        pub tree_substitution: Option<Color>,
        pub tree_match: Option<Color>,
        pub tree_width: usize,
        pub tree_fr_only: bool,
        pub tree_direction_change: Option<Color>,
        pub tree_affine_open: Option<Color>,

        // Options to draw heuristics
        pub draw_heuristic: bool,
        pub draw_contours: bool,
        pub draw_matches: bool,
        pub heuristic: Gradient,
        pub max_heuristic: Option<u32>,
        pub active_match: Color,
        pub pruned_match: Color,
        pub match_shrink: usize,
        pub match_width: usize,
        pub contour: Color,
        pub layer_label: Color,
    }

    #[derive(PartialEq, Eq, Clone)]
    pub enum When {
        None,
        Last,
        All,
        Layers,
        Frames(Vec<usize>),
    }

    impl When {
        fn is_active(&self, frame: usize, is_last: bool, new_layer: bool) -> bool {
            match &self {
                When::None => false,
                When::Last => is_last,
                When::All => is_last || !new_layer,
                When::Layers => is_last || new_layer,
                When::Frames(v) => v.contains(&frame) || (is_last && v.contains(&usize::MAX)),
            }
        }
    }

    #[derive(Clone)]
    pub struct Config {
        pub cell_size: usize,
        pub prescaler: u32,
        /// Divide all input coordinates by this for large inputs.
        pub downscaler: u32,
        pub filepath: String,
        pub draw: When,
        pub delay: f32,
        pub paused: bool,
        pub save: When,
        pub save_last: bool,
        pub style: Style,
        pub transparent_bmp: bool,
        pub draw_old_on_top: bool,
        pub layer_drawing: bool,
        pub num_layers: Option<usize>,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                cell_size: 8,
                prescaler: 1,
                downscaler: 1,
                save: When::None,
                save_last: false,
                filepath: String::from(""),
                draw: When::None,
                delay: 0.2,
                paused: false,
                style: Style {
                    expanded: Gradient::Fixed(Color::BLUE),
                    explored: None,
                    extended: None,
                    bg_color: Color::WHITE,
                    path: Some(Color::BLACK),
                    path_width: Some(2),
                    tree: Some(Color::GRAY),
                    tree_substitution: None,
                    tree_match: None,
                    tree_width: 1,
                    tree_fr_only: false,
                    tree_direction_change: None,
                    tree_affine_open: None,
                    draw_heuristic: false,
                    draw_contours: false,
                    draw_matches: false,
                    heuristic: Gradient::Gradient(Color::WHITE..Color::RGB(128, 128, 128)),
                    max_heuristic: None,
                    active_match: Color::BLACK,
                    pruned_match: Color::RED,
                    match_shrink: 2,
                    match_width: 2,
                    contour: Color::GREEN,
                    layer_label: Color::BLACK,
                },
                draw_old_on_top: true,
                layer_drawing: false,
                num_layers: None,
                transparent_bmp: true,
            }
        }
    }

    impl Visualizer {
        pub fn new(config: Config, a: Seq, b: Seq) -> Self {
            let sdl_context = sdl2::init().unwrap();

            // Draw layer numbers
            #[cfg(feature = "sdl2_ttf")]
            let font = TTF_CONTEXT
                .load_font("/usr/share/fonts/TTF/OpenSans-Regular.ttf", 24)
                .unwrap();

            Visualizer {
                canvas: {
                    let canvas_size_cells = Pos::from(a.len() + 1, b.len() + 1);
                    let video_subsystem = sdl_context.video().unwrap();
                    video_subsystem.gl_attr().set_double_buffer(true);
                    if config.draw != When::None || config.save != When::None || config.save_last {
                        Some(RefCell::new(
                            video_subsystem
                                .window(
                                    &config.filepath,
                                    (canvas_size_cells.0 as u32).div_ceil(config.downscaler)
                                        * config.cell_size as u32
                                        * config.prescaler as u32,
                                    (canvas_size_cells.1 as u32).div_ceil(config.downscaler)
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
                #[cfg(feature = "sdl2_ttf")]
                font,
                config: config.clone(),
                expanded: vec![],
                width: a.len() as u32 + 1,
                height: b.len() as u32 + 1,
                frame_number: 0,
                file_number: 0,
                layer: if config.layer_drawing { Some(0) } else { None },
                expanded_layers: vec![],
            }
        }

        fn cell_begin(&self, Pos(i, j): Pos) -> Point {
            Point::new(
                (i / self.config.downscaler * self.config.prescaler * self.config.cell_size as u32)
                    as i32,
                (j / self.config.downscaler * self.config.prescaler * self.config.cell_size as u32)
                    as i32,
            )
        }
        fn cell_center(&self, Pos(i, j): Pos) -> Point {
            Point::new(
                (i / self.config.downscaler * self.config.prescaler * self.config.cell_size as u32
                    + self.config.cell_size as u32 / 2) as i32,
                (j / self.config.downscaler * self.config.prescaler * self.config.cell_size as u32
                    + self.config.cell_size as u32 / 2) as i32,
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
                    self.config.cell_size as u32 * self.config.prescaler,
                    self.config.cell_size as u32 * self.config.prescaler,
                ))
                .unwrap();
        }

        fn draw_pixels(&self, canvas: &mut Canvas<Window>, p: Vec<Pos>, c: Color) {
            canvas.set_draw_color(c);
            let rects = p
                .iter()
                .map(|p| {
                    let mut begin = self.cell_begin(*p);
                    begin *= self.config.prescaler as i32;
                    Rect::new(
                        begin.x,
                        begin.y,
                        self.config.cell_size as u32 * self.config.prescaler,
                        self.config.cell_size as u32 * self.config.prescaler,
                    )
                })
                .collect_vec();
            canvas.fill_rects(&rects).unwrap();
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
            if self.config.transparent_bmp {
                surf.set_color_key(true, self.config.style.bg_color)
                    .unwrap();
            }

            surf.save_bmp(path).unwrap_or_else(|error| {
                print!("Problem saving the file: {:?}", error);
            });
        }

        fn draw<'a, H: HeuristicInstance<'a>>(
            &mut self,
            is_last: bool,
            path: Option<&Path>,
            is_new_layer: bool,
            h: Option<&H>,
            parent: ParentFn,
        ) {
            let current_frame = self.frame_number;
            self.frame_number += 1;
            if !self
                .config
                .draw
                .is_active(current_frame, is_last, is_new_layer)
                && !self
                    .config
                    .save
                    .is_active(current_frame, is_last, is_new_layer)
                && !(is_last && self.config.save_last)
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

            // Draw heuristic values.
            if self.config.style.draw_heuristic && let Some(h) = h {
                let mut hint = Default::default();
                let h_max = self.config.style.max_heuristic.unwrap_or(h.h(Pos(0,0)));
                let mut value_pos_map = HashMap::<u32, Vec<Pos>>::default();
                for i in 0..self.width {
                    hint = h.h_with_hint(Pos(i,0), hint).1;
                    let mut hint = hint;
                    for j in 0..self.height {
                        let pos = Pos(i, j);
                        let (h, new_hint) = h.h_with_hint(pos, hint);
                        hint = new_hint;
                        value_pos_map.entry(h).or_default().push(pos);
                    }
                }
                for (h, poss) in value_pos_map {
                    self.draw_pixels(
                        &mut canvas,
                        poss,
                        self.config.style.heuristic.color(h as f64 / h_max as f64),
                    );
                }
            }

            // Draw layers and contours.
            if self.config.style.draw_contours && let Some(h) = h && h.layer(Pos(0,0)).is_some() {
                canvas.set_draw_color(self.config.style.contour);
                let draw_right_border = |canvas: &mut Canvas<Window>, Pos(i, j): Pos| {
                    canvas
                        .draw_line(self.cell_begin(Pos(i + 1, j)), self.cell_begin(Pos(i + 1, j + 1)))
                        .unwrap();
                };
                let draw_bottom_border = |canvas: &mut Canvas<Window>, Pos(i, j): Pos| {
                    canvas
                        .draw_line(self.cell_begin(Pos(i, j + 1)), self.cell_begin(Pos(i + 1, j + 1)))
                        .unwrap();
                };


                // Right borders
                let mut hint = Default::default();
                let mut top_borders = vec![(0, h.layer(Pos(0,0)).unwrap())];
                for i in 0..self.width-1 {
                    hint = h.layer_with_hint(Pos(i, 0), hint).unwrap().1;
                    let mut hint = hint;
                    for j in 0..self.height {
                        let pos = Pos(i, j);
                        let (v, new_hint) = h.layer_with_hint(pos, hint).unwrap();
                        hint = new_hint;
                        let pos_r = Pos(i + 1, j);
                        let (v_r, new_hint) = h.layer_with_hint(pos_r, hint).unwrap();
                        hint = new_hint;
                        if v_r != v {
                            draw_right_border(&mut canvas, pos);

                            if j == 0 {
                                top_borders.push((i+1, v_r));
                            }
                        }
                    }
                }
                top_borders.push((self.width, 0));

                // Bottom borders
                let mut hint = Default::default();
                let mut left_borders = vec![(0, h.layer(Pos(0,0)).unwrap())];
                for i in 0..self.width {
                    hint = h.layer_with_hint(Pos(i, 0), hint).unwrap().1;
                    let mut hint = hint;
                    for j in 0..self.height-1 {
                        let pos = Pos(i, j);
                        let (v, new_hint) = h.layer_with_hint(pos, hint).unwrap();
                        hint = new_hint;
                        let pos_l = Pos(i, j + 1);
                        let (v_l, new_hint) = h.layer_with_hint(pos_l, hint).unwrap();
                        hint = new_hint;
                        if v_l != v {
                            draw_bottom_border(&mut canvas, pos);

                            if i == 0 {
                                left_borders.push((j+1, v_l));
                            }
                        }
                    }
                }
                left_borders.push((self.height, 0));

                // Draw numbers at the top and left.
                #[cfg(feature = "sdl2_ttf")]
                {
                    let texture_creator = canvas.texture_creator();
                    for (&(_left, layer), &(right, _)) in top_borders.iter().tuple_windows() {
                        if right < 10 { continue; }
                        let surface = self.font.render(&layer.to_string()).blended(self.config.style.layer_label).unwrap();
                        let w = surface.width();
                        let h = surface.height();
                        //let x = ((left*self.config.cell_size as u32+right*self.config.cell_size as u32)/2).saturating_sub(w/2);
                        let x = (right * self.config.cell_size as u32).saturating_sub(w + 1);
                        let y = -6;
                        canvas.copy(&surface.as_texture(&texture_creator).unwrap(),
                                    None, Some(Rect::new(x as i32,y,w,h))).unwrap();
                    }
                    for (&(_top, layer), &(bottom, _)) in left_borders.iter().tuple_windows(){
                        if bottom < 10 { continue; }
                        let surface = self.font.render(&layer.to_string()).blended(self.config.style.layer_label).unwrap();
                        let w = surface.width();
                        let h = surface.height();
                        //let y = ((top*self.config.cell_size as u32+bottom*self.config.cell_size as u32)/2).saturating_sub(h/2);
                        let x = 3;
                        let y = (bottom * self.config.cell_size as u32).saturating_sub(h)+5;
                        canvas.copy(&surface.as_texture(&texture_creator).unwrap(),
                                    None, Some(Rect::new(x, y as i32,w,h))).unwrap();
                    }
                }
            }

            if self.config.draw_old_on_top {
                // Explored
                if let Some(color) = self.config.style.explored {
                    for &(t, pos) in &self.expanded {
                        if t == Type::Explored {
                            self.draw_pixel(&mut canvas, pos, color);
                        }
                    }
                }
                // Expanded
                let mut current_layer = self.layer.unwrap_or(0);
                for (i, &(t, pos)) in self.expanded.iter().enumerate().rev() {
                    if t == Type::Explored {
                        continue;
                    }
                    if t == Type::Extended && let Some(c) = self.config.style.extended {
                        self.draw_pixel(&mut canvas, pos, c);
                        continue;
                    }
                    self.draw_pixel(
                        &mut canvas,
                        pos,
                        self.config.style.expanded.color(
                            if let Some(layer) = self.layer && layer != 0 {
                                if current_layer > 0
                                    && i < self.expanded_layers[current_layer - 1]
                                {
                                    current_layer -= 1;
                                }
                                current_layer as f64 / self.config.num_layers.unwrap_or(layer) as f64
                            } else {
                                    i as f64 / self.expanded.len() as f64
                            },
                        ),
                    );
                }
            } else {
                // Explored
                if let Some(color) = self.config.style.explored {
                    for &(t, pos) in &self.expanded {
                        if t == Type::Explored {
                            self.draw_pixel(&mut canvas, pos, color);
                        }
                    }
                }
                // Expanded
                let mut current_layer = 0;
                for (i, &(t, pos)) in self.expanded.iter().enumerate() {
                    if t == Type::Explored {
                        continue;
                    }
                    if t == Type::Extended && let Some(c) = self.config.style.extended {
                        self.draw_pixel(&mut canvas, pos, c);
                        continue;
                    }
                    self.draw_pixel(
                        &mut canvas,
                        pos,
                        self.config.style.expanded.color(
                            if let Some(layer) = self.layer && layer != 0 {
                                if current_layer < layer && i >= self.expanded_layers[current_layer] {
                                    current_layer += 1;
                                }
                                current_layer as f64 / self.config.num_layers.unwrap_or(layer) as f64
                            } else {
                                    i as f64 / self.expanded.len() as f64
                            },
                        ),
                    );
                }
            }

            // Draw matches.
            if self.config.style.draw_matches && let  Some(h) = h && let Some(matches) = h.matches() {
                for m in &matches {
                    if m.match_cost > 0 {
                        continue;
                    }
                    let mut b = self.cell_center(m.start);
                    b.x += self.config.style.match_shrink as i32;
                    b.y += self.config.style.match_shrink as i32;
                    let mut e = self.cell_center(m.end);
                    e.x -= self.config.style.match_shrink as i32;
                    e.y -= self.config.style.match_shrink as i32;
                    Self::draw_diag_line(
                        &mut canvas,
                        b, e,
                        match m.pruned {
                            MatchStatus::Active => self.config.style.active_match,
                            MatchStatus::Pruned => self.config.style.pruned_match,
                        },
                        self.config.style.match_width,
                    );
                }
            }

            // Draw path.
            if let Some(path) = path &&
               let Some(path_color) = self.config.style.path {
                if let Some(path_width) = self.config.style.path_width {
                    for (from, to) in path.iter().tuple_windows() {
                        Self::draw_diag_line(
                            &mut canvas,
                            self.cell_center(*from),
                            self.cell_center(*to),
                            path_color,
                            path_width,
                        );
                    }
                } else {
                    for p in path {
                        self.draw_pixel(&mut canvas, *p, path_color)
                    }
                }
            }

            // Draw tree.
            if let Some(parent) = parent && let Some(tree_color) = self.config.style.tree {
                for &(_t, u) in &self.expanded {
                    if self.config.style.tree_fr_only {
                        // Only trace if u is the furthest point on this diagonal.
                        let mut v = u;
                        let mut skip = false;
                        loop {
                            v = v + Pos(1,1);
                            if !(v <= Pos(self.width, self.height)) {
                                break;
                            }
                            if self.expanded.iter().filter(|(_, u)| *u == v).count() > 0 {
                                skip = true;
                                break;
                            }
                        }
                        if skip {
                            continue;
                        }
                    }
                    let mut st = State{i: u.0 as isize, j: u.1 as isize, layer: None};
                    let mut path = vec![];
                    while let Some((p, op)) = parent(st){
                        path.push((st, p, op));
                        let color = if let Some(CigarOp::AffineOpen(_)) = op[1]
                            && let Some(c) = self.config.style.tree_affine_open {
                                c
                            } else {
                                match op[0].unwrap() {
                                    CigarOp::Match => self.config.style.tree_match,
                                    CigarOp::Mismatch => self.config.style.tree_substitution,
                                    _ => None,
                                }.unwrap_or(tree_color)
                            };
                        Self::draw_diag_line(
                            &mut canvas,
                            self.cell_center(p.pos()),
                            self.cell_center(st.pos()),
                            color,
                            self.config.style.tree_width,
                        );

                        st = p;
                    }
                    if let Some(c) = self.config.style.tree_direction_change {
                        let mut last = CigarOp::Match;
                        for &(u, p, op)  in path.iter().rev() {
                            let op = op[0].unwrap();
                            match op {
                                CigarOp::Insertion => {
                                    if last == CigarOp::Deletion {
                                        Self::draw_diag_line(
                                            &mut canvas,
                                            self.cell_center(p.pos()),
                                            self.cell_center(u.pos()),
                                            c,
                                            self.config.style.tree_width,
                                        );
                                    }
                                    last = op;
                                }
                                CigarOp::Deletion => {
                                    if last == CigarOp::Insertion {
                                        Self::draw_diag_line(
                                            &mut canvas,
                                            self.cell_center(p.pos()),
                                            self.cell_center(u.pos()),
                                            c,
                                            self.config.style.tree_width,
                                        );
                                    }
                                    last = op;
                                }
                                CigarOp::Mismatch => {
                                    last = op;
                                }
                                _ => {
                                }
                            }
                        }
                    }

                }
            }

            // SAVE

            if self
                .config
                .save
                .is_active(current_frame, is_last, is_new_layer)
            {
                self.save_canvas(&mut canvas, false);
                self.file_number += 1;
            }

            // Save the final frame separately if needed.
            if is_last && self.config.save_last {
                self.save_canvas(&mut canvas, true);
            }

            // SHOW

            if !self
                .config
                .draw
                .is_active(current_frame, is_last, is_new_layer)
            {
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
                                self.config.draw = When::Last;
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
