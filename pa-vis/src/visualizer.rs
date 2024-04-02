//! To turn images into a video, use this:
//!
//! ```sh
//! ffmpeg -framerate 20 -i %d.bmp output.mp4
//! ```
//! or when that gives errors:
//! ```sh
//! ffmpeg -framerate 20 -i %d.bmp -vf "pad=ceil(iw/2)*2:ceil(ih/2)*2" output.mp4
//! ```

use super::{canvas::*, *};
use clap::ValueEnum;
use itertools::Itertools;
use pa_affine_types::*;
use pa_heuristic::matches::MatchStatus;
use pa_types::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::exit;
use std::{
    cell::{RefCell, RefMut},
    cmp::{max, min},
    collections::HashMap,
    ops::Range,
    time::Duration,
};

#[derive(Debug, PartialEq, Default, Clone, Copy, ValueEnum, Serialize, Deserialize)]
pub enum VisualizerStyle {
    #[default]
    Default,
    Large,
    Detailed,
    Test,
    Debug,
}

#[derive(Debug, PartialEq, Eq, Clone, ValueEnum, Serialize, Deserialize)]
pub enum When {
    None,
    // Translates to Frames([0])
    First,
    Last,
    All,
    Layers,
    // Show/save each Nth frame.
    #[clap(skip)]
    StepBy(usize),
    // Show/save each Nth layer.
    #[clap(skip)]
    LayersStepBy(usize),
    #[clap(skip)]
    Frames(Vec<usize>),
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Type {
    Expanded,
    Explored,
    Extended,
}
use Type::*;

type CanvasRC = RefCell<CanvasBox>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExpandPos {
    Single(Pos),
    Block(Pos, Pos),
    Blocks(Vec<(Pos, Pos)>),
}

impl ExpandPos {
    fn pos(&self) -> Pos {
        match self {
            Self::Single(p) => *p,
            Self::Block(p, _) => *p,
            _ => panic!(),
        }
    }
}

pub struct Visualizer {
    config: Config,

    // Name of the algorithm
    title: Option<String>,
    // Heuristic / algorithm parameters. List of (key, value).
    params: Option<String>,
    // An optional comment explaining the algorithm.
    comment: Option<String>,

    canvas: Option<CanvasRC>,

    // The size in pixels of the entire canvas.
    canvas_size: (i32, i32),
    // The region of the NW states.
    nw: Region,
    // The region of the DT states.
    dt: Region,
    // The region of the transformed states.
    // FIXME: USE THIS.
    _tr: Region,

    // The last DP state (a.len(), b.len()).
    target: Pos,

    // Number of calls to draw().
    frame_number: usize,
    // Number of calls to draw() for a new layer.
    layer_number: usize,
    // Number of saved frames.
    file_number: usize,
    // Number of times config.draw triggers.
    drawn_frame_number: usize,

    // Type, Pos, g, f
    pub expanded: Vec<(Type, ExpandPos, Cost, Cost)>,
    pub trace: Vec<ExpandPos>,
    pub preprune: Vec<Pos>,
    // Calls to the heuristic.
    h_calls: Vec<Pos>,
    f_calls: Vec<(Pos, bool, bool)>,
    j_ranges: Vec<(Pos, Pos)>,
    fixed_j_ranges: Vec<(Pos, Pos)>,
    fixed_h: Vec<(Pos, Pos)>,
    next_fixed_h: Option<(Pos, Pos)>,
    // The current layer
    layer: Option<usize>,
    // Index in expanded where each layer stars.
    expanded_layers: Vec<usize>,
    // Partial path for divide-and-conquer.
    meeting_points: Vec<Pos>,
}

impl VisualizerInstance for Visualizer {
    fn explore<'a, H: HeuristicInstance<'a>>(&mut self, pos: Pos, g: Cost, f: Cost, h: Option<&H>) {
        if !(pos <= self.target) {
            return;
        }
        self.expanded.push((Explored, ExpandPos::Single(pos), g, f));
        // Only draw a new frame if explored states are actually shown.
        if self.config.style.explored.is_some() {
            self.draw(false, None, false, h, None);
        }
    }

    fn expand<'a, H: HeuristicInstance<'a>>(&mut self, pos: Pos, g: Cost, f: Cost, h: Option<&H>) {
        if !(pos <= self.target) {
            return;
        }
        self.expanded.push((Expanded, ExpandPos::Single(pos), g, f));
        self.draw(false, None, false, h, None);
    }

    fn extend<'a, H: HeuristicInstance<'a>>(&mut self, pos: Pos, g: Cost, f: Cost, h: Option<&H>) {
        if !(pos <= self.target) {
            return;
        }
        self.expanded.push((Extended, ExpandPos::Single(pos), g, f));
        self.draw(false, None, false, h, None);
    }

    fn expand_preprune(&mut self, pos: Pos) {
        if self.config.style.preprune.is_some() {
            self.preprune.push(pos);
            self.draw::<!>(false, None, false, None, None);
        }
    }
    fn extend_preprune(&mut self, pos: Pos) {
        if self.config.style.preprune.is_some() {
            self.preprune.push(pos);
        }
    }

    fn expand_trace(&mut self, pos: Pos) {
        if self.config.style.trace.is_some() {
            self.trace.push(ExpandPos::Single(pos));
            self.draw::<!>(false, None, false, None, None);
        }
    }
    fn extend_trace(&mut self, pos: Pos) {
        if self.config.style.trace.is_some() {
            self.trace.push(ExpandPos::Single(pos));
        }
    }

    fn h_call(&mut self, pos: Pos) {
        if pos <= self.target && self.config.style.draw_h_calls {
            self.h_calls.push(pos);
            self.draw::<!>(false, None, false, None, None);
        }
    }
    fn f_call(&mut self, pos: Pos, in_bounds: bool, fixed: bool) {
        if self.config.style.draw_f_calls {
            self.f_calls.push((pos, in_bounds, fixed));
            self.draw::<!>(false, None, false, None, None);
        }
    }
    fn j_range(&mut self, start: Pos, end: Pos) {
        if self.config.style.draw_ranges {
            if let Some(r) = self.j_ranges.iter_mut().find(|(s, _)| s.0 == start.0) {
                *r = (start, end);
            } else {
                self.j_ranges.push((start, end));
            }
            self.draw::<!>(false, None, false, None, None);
        }
    }
    fn fixed_j_range(&mut self, start: Pos, end: Pos) {
        if self.config.style.draw_ranges {
            if let Some(r) = self.fixed_j_ranges.iter_mut().find(|(s, _)| s.0 == start.0) {
                *r = (start, end);
            } else {
                self.fixed_j_ranges.push((start, end));
            }
            self.draw::<!>(false, None, false, None, None);
        }
    }
    fn fixed_h(&mut self, start: Pos, end: Pos) {
        if self.config.style.draw_ranges {
            self.next_fixed_h = None;
            self.fixed_h.push((start, end));
            self.draw::<!>(false, None, false, None, None);
            if let Some(r) = self
                .fixed_h
                .iter_mut()
                .rev()
                .skip(1)
                .find(|(s, _)| s.0 == start.0)
            {
                *r = (start, end);
                self.draw::<!>(false, None, false, None, None);
                self.fixed_h.pop();
            }
        }
    }
    fn next_fixed_h(&mut self, start: Pos, end: Pos) {
        if self.config.style.draw_ranges {
            self.next_fixed_h = Some((start, end));
            self.draw::<!>(false, None, false, None, None);
        }
    }

    fn new_layer<'a, H: HeuristicInstance<'a>>(&mut self, h: Option<&H>) {
        if let Some(layer) = self.layer {
            self.layer = Some(layer + 1);
            self.expanded_layers.push(self.expanded.len());
        }
        self.draw(false, None, true, h, None);
        self.f_calls.clear();
        self.j_ranges.clear();
    }

    fn add_meeting_point<'a, HI: HeuristicInstance<'a>>(&mut self, pos: Pos) {
        self.meeting_points.push(pos);
        if self.config.clear_after_meeting_point {
            self.expanded.clear();
        }
        self.draw::<HI>(false, None, true, None, None);
    }

    fn last_frame<'a, H: HeuristicInstance<'a>>(
        &mut self,
        cigar: Option<&AffineCigar>,
        parent: ParentFn<'_>,
        h: Option<&H>,
    ) {
        self.draw(true, cigar, false, h, parent);
    }

    fn expand_block<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        pos: Pos,
        size: Pos,
        g: Cost,
        f: Cost,
        h: Option<&HI>,
    ) {
        let maxsize = self.target - pos + Pos(1, 1);
        let size = Pos(min(size.0, maxsize.0), min(size.1, maxsize.1));
        self.expanded
            .push((Expanded, ExpandPos::Block(pos, size), g, f));
        self.draw(false, None, false, h, None);
    }

    fn expand_block_trace(&mut self, pos: Pos, size: Pos) {
        let maxsize = self.target - pos + Pos(1, 1);
        let size = Pos(min(size.0, maxsize.0), min(size.1, maxsize.1));
        self.trace.push(ExpandPos::Block(pos, size));
        if self.config.style.trace.is_some() {
            self.draw::<!>(false, None, false, None, None);
        }
    }

    fn expand_blocks<'a, HI: HeuristicInstance<'a>>(
        &mut self,
        poss: [Pos; 4],
        sizes: [Pos; 4],
        g: Cost,
        f: Cost,
        h: Option<&HI>,
    ) {
        self.expanded.push((
            Expanded,
            ExpandPos::Blocks(poss.into_iter().zip(sizes).collect_vec()),
            g,
            f,
        ));
        self.draw(false, None, false, h, None);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Gradient {
    Fixed(Color),
    Gradient(Range<Color>),
    // 0 <= start < end <= 1
    TurboGradient(Range<f64>),
    BoundedGradient(Range<Color>, usize),
    // 0 <= start < end <= 1
    BoundedTurboGradient(Range<f64>, usize),
}

impl Gradient {
    fn color(&self, i: usize, cnt: usize) -> Color {
        match self {
            Gradient::Fixed(color) => *color,
            Gradient::Gradient(_) | Gradient::TurboGradient(_) => {
                self.color_f(i as f64 / cnt as f64)
            }
            Gradient::BoundedGradient(_, max) | Gradient::BoundedTurboGradient(_, max) => {
                self.color_f(i as f64 / *max as f64)
            }
        }
    }
    fn color_f(&self, f: f64) -> Color {
        match self {
            Gradient::Fixed(color) => *color,
            Gradient::Gradient(range) | Gradient::BoundedGradient(range, _) => {
                let frac =
                    |a: u8, b: u8| -> u8 { (a as f64 + f * (b as f64 - a as f64)).ceil() as u8 };
                (
                    frac(range.start.0, range.end.0),
                    frac(range.start.1, range.end.1),
                    frac(range.start.2, range.end.2),
                    frac(range.start.3, range.end.3),
                )
            }
            Gradient::TurboGradient(range) | Gradient::BoundedTurboGradient(range, _) => {
                let f = range.start + f * (range.end - range.start);
                let c = colorgrad::turbo().at(f).to_rgba8();
                (c[0], c[1], c[2], c[3])
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub expanded: Gradient,
    pub explored: Option<Color>,
    pub extended: Option<Color>,
    pub trace: Option<(Color, Color)>,
    pub fixed: Option<Color>,
    pub preprune: Option<Color>,
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
    pub draw_layers: bool,
    pub draw_matches: bool,
    pub draw_parents: bool,
    pub draw_dt: bool,
    pub draw_f: bool,
    pub draw_h_calls: bool,
    pub draw_f_calls: bool,
    pub draw_ranges: bool,
    pub draw_fixed_h: bool,
    pub h_call: Color,
    pub draw_labels: bool,
    pub heuristic: Gradient,
    pub layer: Gradient,
    pub max_heuristic: Option<I>,
    pub max_layer: Option<I>,
    pub active_match: Color,
    pub pruned_match: Color,
    pub pre_pruned_match: Color,
    pub filtered_match: Color,
    pub match_shrink: usize,
    pub match_width: usize,
    pub contour: Color,
}

impl When {
    fn is_active(&self, frame: usize, layer: usize, is_last: bool, new_layer: bool) -> bool {
        match &self {
            When::None => false,
            When::First => frame == 1,
            When::Last => is_last,
            When::All => is_last || !new_layer,
            When::Layers => is_last || new_layer,
            When::Frames(v) => v.contains(&frame) || (is_last && v.contains(&usize::MAX)),
            When::StepBy(step) => is_last || frame % step == 0,
            When::LayersStepBy(step) => is_last || (new_layer && layer % step == 0),
        }
    }
}

const CANVAS_HEIGHT: I = 1000;

#[derive(Clone, PartialEq, Debug)]
pub struct Config {
    /// 0 to infer automatically.
    pub cell_size: I,
    /// Divide all input coordinates by this for large inputs.
    /// 0 to infer automatically.
    pub downscaler: I,
    pub filepath: PathBuf,
    pub draw: When,
    /// Used in wasm rendering: the entire alignment is run and only this
    /// single frame is drawn.
    pub draw_single_frame: Option<usize>,
    pub delay: Duration,
    pub paused: bool,
    pub save: When,
    pub save_last: bool,
    pub style: Style,
    pub transparent_bmp: bool,
    pub draw_old_on_top: bool,
    pub layer_drawing: bool,
    pub num_layers: Option<usize>,
    pub clear_after_meeting_point: bool,
}

impl Config {
    pub fn new(style: VisualizerStyle) -> Self {
        let mut config = Self {
            cell_size: 8,
            downscaler: 1,
            save: When::None,
            save_last: false,
            filepath: PathBuf::default(),
            draw: When::None,
            draw_single_frame: None,
            delay: Duration::from_secs_f32(0.1),
            paused: false,
            style: Style {
                expanded: Gradient::TurboGradient(0.2..0.95),
                explored: None,
                extended: None,
                trace: None,
                fixed: None,
                preprune: None,
                bg_color: WHITE,
                path: Some(BLACK),
                path_width: Some(2),
                tree: None,
                tree_substitution: None,
                tree_match: None,
                tree_width: 1,
                tree_fr_only: false,
                tree_direction_change: None,
                tree_affine_open: None,
                draw_heuristic: false,
                draw_contours: false,
                draw_layers: false,
                draw_matches: false,
                draw_parents: false,
                draw_dt: true,
                draw_f: false,
                draw_h_calls: false,
                draw_f_calls: false,
                draw_ranges: false,
                draw_fixed_h: false,
                h_call: RED,
                draw_labels: true,
                heuristic: Gradient::Gradient((250, 250, 250, 0)..(180, 180, 180, 0)),
                layer: Gradient::Gradient((250, 250, 250, 0)..(100, 100, 100, 0)),
                max_heuristic: None,
                max_layer: None,
                active_match: BLACK,
                pruned_match: RED,
                pre_pruned_match: PURPLE,
                filtered_match: RED,
                match_shrink: 2,
                match_width: 2,
                contour: BLACK,
            },
            draw_old_on_top: true,
            layer_drawing: false,
            num_layers: None,
            transparent_bmp: true,
            clear_after_meeting_point: true,
        };

        match style {
            VisualizerStyle::Default => {}
            VisualizerStyle::Large => {
                config.transparent_bmp = false;
                config.downscaler = 100;
                config.cell_size = 1;
                config.style.path = None;
                config.style.draw_matches = true;
                config.style.match_width = 1;
                config.style.match_shrink = 0;
                config.style.expanded = Gradient::TurboGradient(0.25..0.90)
            }
            VisualizerStyle::Detailed => {
                config.paused = false;
                config.delay = Duration::from_secs_f32(0.2);
                config.cell_size = 6;
                config.style.bg_color = WHITE;
                config.style.tree = Some(GRAY);
                config.style.expanded = Gradient::Fixed((130, 179, 102, 0));
                config.style.preprune = Some((130, 179, 102, 0));
                config.style.explored = Some((0, 102, 204, 0));
                config.style.max_heuristic = Some(10);
                config.style.pruned_match = RED;
                config.style.path = None;
                config.style.match_width = 3;
                config.style.draw_heuristic = true;
                config.style.draw_contours = true;
                config.style.draw_matches = true;
                config.draw_old_on_top = true;
                config.layer_drawing = false;
            }
            VisualizerStyle::Test => {
                config.draw = When::All;
                config.paused = true;
                config.cell_size = 0;
                config.style.explored = Some((0, 102, 204, 0));
                config.style.max_heuristic = Some(100);
                config.style.pruned_match = RED;
                config.style.match_width = 3;
                config.style.path_width = Some(4);
                config.style.draw_heuristic = true;
                config.style.draw_contours = true;
                config.style.draw_matches = true;
                config.style.draw_f = true;
                config.style.draw_dt = true;
            }
            VisualizerStyle::Debug => {
                config.paused = true;
                config.cell_size = 0;
                config.style.explored = Some((0, 102, 204, 0));
                config.style.pruned_match = RED;
                config.style.match_width = 3;
                config.style.path_width = Some(4);
                config.style.draw_heuristic = true;
                config.style.draw_contours = true;
                config.style.draw_layers = false;
                config.style.draw_matches = true;
                config.style.draw_f = false;
                config.style.draw_dt = true;
            }
        }

        config
    }

    pub fn with_filename(&self, filename: &str) -> Self {
        let mut config = self.clone();
        config.filepath = config.filepath.join(filename);
        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new(VisualizerStyle::Default)
    }
}

impl VisualizerT for Config {
    type Instance = Visualizer;

    #[cfg(feature = "sdl")]
    fn build(&self, a: Seq, b: Seq) -> Self::Instance {
        Visualizer::new::<crate::sdl::SdlCanvasFactory>(self.clone(), a, b)
    }
    #[cfg(not(feature = "sdl"))]
    fn build(&self, _a: Seq, _b: Seq) -> Self::Instance {
        unimplemented!("Enable the pa_vis:sdl feature to use the default sdl canvas.");
    }

    fn build_from_factory<CF: CanvasFactory>(&self, a: Seq, b: Seq) -> Self::Instance {
        Visualizer::new::<CF>(self.clone(), a, b)
    }
}

struct Region {
    /// Start position on canvas
    start: CPos,
    /// Size on canvas
    size: CPos,
    /// Cell size: state is cs by cs pixels.
    // FIXME: USE THIS
    _cs: I,
    /// Downscaler: each state on the canvas represents ds by ds actual states.
    // FIXME: USE THIS
    _ds: I,
}

impl Visualizer {
    /// This sets the title and parameters based on the CLI arguments.
    /// FIXME: Add algorithm and heuristic args or title/params/comment args.
    pub fn new<CF: CanvasFactory>(mut config: Config, a: Seq, b: Seq) -> Self {
        // layout:
        //
        // ---------------
        // |      |  DT  |
        // |  NW  |------|
        // |      |  T   |
        // ---------------
        // | fmax | fmax |
        // ---------------
        //
        // NW follows the cell size if given.
        // Otherwise, the cell size and downscaler are chosen to give a height around 500 pixels.
        // The DT window is chosen with the same height, but half the width.

        let grid_width = a.len() as I + 1;
        let grid_height = b.len() as I + 1;

        if config.cell_size != 0 {
            if config.downscaler == 0 {
                config.downscaler = 1;
            }
        } else {
            if config.downscaler == 0 {
                config.downscaler = max(1, grid_height.div_ceil(CANVAS_HEIGHT));
            }
            let ds = config.downscaler;
            config.cell_size = max(1, CANVAS_HEIGHT / (grid_height.div_ceil(ds)));
        }
        let nw = Region {
            start: CPos(0, 0),
            _cs: config.cell_size,
            _ds: config.downscaler,
            size: CPos(
                (grid_width.div_ceil(config.downscaler) * config.cell_size) as i32,
                (grid_height.div_ceil(config.downscaler) * config.cell_size) as i32,
            ),
        };
        let dt = Region {
            start: nw.start.right(nw.size.0),
            size: nw.size / 2,
            _cs: 0,
            _ds: 0,
        };
        let tr = Region {
            start: dt.start.down(dt.size.1),
            size: nw.size / 2,
            _cs: 0,
            _ds: 0,
        };
        let canvas_size = (
            nw.size.0 + if config.style.draw_dt { dt.size.0 } else { 0 },
            nw.size.1,
        );

        Visualizer {
            title: None,
            params: None,
            comment: None,
            canvas: {
                (config.draw != When::None || config.save != When::None || config.save_last).then(
                    || {
                        RefCell::new(CF::new(
                            canvas_size.0 as usize,
                            canvas_size.1 as usize,
                            &config.filepath.to_str().unwrap(),
                        ))
                    },
                )
            },
            config: config.clone(),
            expanded: vec![],
            preprune: vec![],
            trace: vec![],
            h_calls: vec![],
            f_calls: vec![],
            j_ranges: vec![],
            fixed_j_ranges: vec![],
            fixed_h: vec![],
            next_fixed_h: None,
            target: Pos::target(a, b),
            frame_number: 0,
            layer_number: 0,
            file_number: 0,
            drawn_frame_number: 0,
            layer: if config.layer_drawing { Some(0) } else { None },
            expanded_layers: vec![],
            meeting_points: vec![],

            canvas_size,
            nw,
            dt,
            _tr: tr,
        }
    }

    fn cell_begin(&self, Pos(i, j): Pos) -> CPos {
        CPos(
            (i / self.config.downscaler * self.config.cell_size) as i32,
            (j / self.config.downscaler * self.config.cell_size) as i32,
        )
    }

    fn cell_center(&self, Pos(i, j): Pos) -> CPos {
        CPos(
            (i / self.config.downscaler * self.config.cell_size + self.config.cell_size / 2) as i32,
            (j / self.config.downscaler * self.config.cell_size + self.config.cell_size / 2) as i32,
        )
    }

    fn cell_end(&self, Pos(i, j): Pos) -> CPos {
        CPos(
            (i / self.config.downscaler * self.config.cell_size + self.config.cell_size) as i32,
            (j / self.config.downscaler * self.config.cell_size + self.config.cell_size) as i32,
        )
    }

    fn draw_pixel(&self, canvas: &mut CanvasBox, pos: Pos, color: Color) {
        if self.config.cell_size == 1 {
            canvas.draw_point(self.cell_begin(pos), color);
        } else {
            canvas.fill_rect(
                self.cell_begin(pos),
                self.config.cell_size,
                self.config.cell_size,
                color,
            );
        }
    }

    fn draw_pixels(&self, canvas: &mut CanvasBox, pos: &Vec<Pos>, color: Color) {
        let rects = pos
            .iter()
            .map(|p| {
                (
                    self.cell_begin(*p),
                    self.config.cell_size,
                    self.config.cell_size,
                )
            })
            .collect_vec();
        canvas.fill_rects(&rects, color);
    }

    fn draw_box(&self, canvas: &mut CanvasBox, mut start: Pos, mut size: Pos, color: Color) {
        start += Pos(1, 0);
        size += Pos(0, 1);
        let end = start + size;
        let end = self.cell_begin(Pos(end.0, end.1.min(self.target.1 + 1)));
        let start = self.cell_begin(start);
        canvas.fill_rect(start, end.0 - start.0, end.1 - start.1, color);
    }

    fn draw_boxes(&self, canvas: &mut CanvasBox, boxes: &Vec<(Pos, Pos)>, color: Color) {
        let rects = boxes
            .iter()
            .map(|(pos, size)| {
                let end = self.cell_end(*pos + *size - Pos(1, 1));
                let start = self.cell_begin(*pos);
                (start, end.0 - start.0, end.1 - start.1)
            })
            .collect_vec();
        canvas.fill_rects(&rects, color);
    }

    // TODO: Does this work with html canvas? maybe there is a simpler API there.
    fn draw_diag_line(canvas: &mut CanvasBox, from: CPos, to: CPos, color: Color, width: usize) {
        if from == to {
            // NOTE: We skip the line width in this case.
            canvas.draw_point(from, color);
            return;
        }
        canvas.draw_line(from, to, color);
        for mut w in 1..width as i32 {
            if w % 2 == 1 {
                w = (w + 1) / 2;
                canvas.draw_line(
                    CPos(from.0 + w, from.1 - w + 1),
                    CPos(to.0 + w - 1, to.1 - w),
                    color,
                );
                canvas.draw_line(
                    CPos(from.0 - w, from.1 + w - 1),
                    CPos(to.0 - w + 1, to.1 + w),
                    color,
                );
                canvas.draw_line(
                    CPos(from.0 + w - 1, from.1 - w),
                    CPos(to.0 + w, to.1 - w + 1),
                    color,
                );
                canvas.draw_line(
                    CPos(from.0 - w + 1, from.1 + w),
                    CPos(to.0 - w, to.1 + w - 1),
                    color,
                );
            } else {
                w /= 2;
                canvas.draw_line(
                    CPos(from.0 + w, from.1 - w),
                    CPos(to.0 + w, to.1 - w),
                    color,
                );
                canvas.draw_line(
                    CPos(from.0 - w, from.1 + w),
                    CPos(to.0 - w, to.1 + w),
                    color,
                );
            }
        }
    }

    #[allow(unused)]
    fn draw_thick_line_horizontal(
        canvas: &mut CanvasBox,
        from: CPos,
        to: CPos,
        width: i32,
        margin: i32,
        color: Color,
    ) {
        for w in -width / 2..width - width / 2 {
            canvas.draw_line(
                CPos(from.0 + margin, from.1 + w),
                CPos(to.0 - margin, to.1 + w),
                color,
            );
        }
    }

    //Saves canvas to bmp file
    fn save_canvas(&self, canvas: &mut CanvasBox, last: bool, suffix: Option<&str>) {
        let extension = suffix.map_or("bmp".to_string(), |s| s.to_string() + ".bmp");
        let path = if last {
            if let Some(parent) = self.config.filepath.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            self.config.filepath.with_extension(extension).to_owned()
        } else {
            // Make sure the directory exists.
            let mut dir = self.config.filepath.clone();
            std::fs::create_dir_all(&dir).unwrap();
            dir.push(self.file_number.to_string());
            dir.set_extension(extension);
            dir
        };
        if self.config.transparent_bmp {
            canvas.save_transparent(&path, self.config.style.bg_color);
        } else {
            canvas.save(&path);
        }
    }

    fn draw<'a, H: HeuristicInstance<'a>>(
        &mut self,
        is_last: bool,
        cigar: Option<&AffineCigar>,
        is_new_layer: bool,
        h: Option<&H>,
        parent: ParentFn,
    ) {
        self.frame_number += 1;
        if is_new_layer {
            self.layer_number += 1;
        }
        if !self
            .config
            .draw
            .is_active(self.frame_number, self.layer_number, is_last, is_new_layer)
            && !self.config.save.is_active(
                self.frame_number,
                self.layer_number,
                is_last,
                is_new_layer,
            )
            && !(is_last && self.config.save_last)
        {
            return;
        }

        // Filter out non-target frames if only drawing a single frame.
        if let Some(target_frame) = self.config.draw_single_frame
            && self.drawn_frame_number != target_frame
        {
            self.drawn_frame_number += 1;
            return;
        }
        self.drawn_frame_number += 1;

        // DRAW
        {
            // Draw background.
            let canvas = if let Some(canvas) = &self.canvas {
                canvas
            } else {
                return;
            };
            let mut canvas = canvas.borrow_mut();
            canvas.fill_rect(
                CPos(0, 0),
                self.canvas_size.0 as I,
                self.canvas_size.1 as I,
                self.config.style.bg_color,
            );

            // Draw heuristic values.
            if self.config.style.draw_heuristic
                && let Some(h) = h
            {
                let mut hint = Default::default();
                let h_max = self.config.style.max_heuristic.unwrap_or(h.h(Pos(0, 0)));
                let mut value_pos_map = HashMap::<I, Vec<Pos>>::default();
                for i in 0..=self.target.0 {
                    hint = h.h_with_hint(Pos(i, 0), hint).1;
                    let mut hint = hint;
                    for j in 0..=self.target.1 {
                        let pos = Pos(i, j);
                        let (h, new_hint) = h.h_with_hint(pos, hint);
                        hint = new_hint;
                        value_pos_map.entry(h).or_default().push(pos);
                    }
                }
                for (h, poss) in value_pos_map {
                    self.draw_pixels(
                        &mut canvas,
                        &poss,
                        self.config
                            .style
                            .heuristic
                            .color(h as usize, h_max as usize),
                    );
                }
            }

            // Draw layer values.
            if self.config.style.draw_layers
                && let Some(h) = h
            {
                if let Some((mut l_max, mut hint)) =
                    h.layer_with_hint(Pos(0, 0), Default::default())
                {
                    if let Some(m) = self.config.style.max_layer {
                        l_max = m;
                    }
                    let mut value_pos_map = HashMap::<I, Vec<Pos>>::default();
                    for i in 0..=self.target.0 {
                        hint = h.layer_with_hint(Pos(i, 0), hint).unwrap().1;
                        let mut hint = hint;
                        for j in 0..=self.target.1 {
                            let pos = Pos(i, j);
                            let (l, new_hint) = h.layer_with_hint(pos, hint).unwrap();
                            hint = new_hint;
                            value_pos_map.entry(l).or_default().push(pos);
                        }
                    }
                    for (l, poss) in value_pos_map {
                        self.draw_pixels(
                            &mut canvas,
                            &poss,
                            self.config
                                .style
                                .layer
                                .color(l as usize, max(l_max as usize, 1)),
                        );
                    }
                }
            }

            // Draw prepruning states
            if let Some(c) = self.config.style.preprune {
                for pos in &self.preprune {
                    self.draw_pixel(&mut canvas, *pos, c);
                }
            }

            let color_for_pos = |p: &Pos| -> Color {
                let mut rng = StdRng::seed_from_u64(
                    (self.target.0.saturating_mul(p.0).saturating_add(p.1)) as _,
                );
                let r = rng.gen_range(0..=255);
                let g = rng.gen_range(0..=255);
                let b = rng.gen_range(0..=255);
                (r, g, b, 0)
            };

            // Draw parents
            if self.config.style.draw_parents
                && let Some(h) = h
            {
                let mut parent_pos_map = HashMap::<Pos, Vec<Pos>>::default();
                for i in 0..=self.target.0 {
                    for j in 0..=self.target.1 {
                        let pos = Pos(i, j);
                        let (_h, parent) = h.h_with_parent(pos);
                        parent_pos_map.entry(parent).or_default().push(pos);
                    }
                }

                for (_i, (parent, poss)) in parent_pos_map.iter().enumerate() {
                    self.draw_pixels(
                        &mut canvas,
                        poss,
                        color_for_pos(parent), //self.config.style.heuristic.color(i as f64 / parent_pos_map.len() as f64),
                    );
                }
            }

            let mut draw_pos = |pos: &ExpandPos, color: Color| match pos {
                ExpandPos::Single(pos) => self.draw_pixel(&mut canvas, *pos, color),
                ExpandPos::Block(s, t) => self.draw_box(&mut canvas, *s, *t, color),
                ExpandPos::Blocks(blocks) => self.draw_boxes(&mut canvas, blocks, color),
            };

            if self.config.draw_old_on_top {
                // Explored
                if let Some(color) = self.config.style.explored {
                    for (t, pos, _, _) in &self.expanded {
                        if *t == Type::Explored {
                            draw_pos(pos, color);
                        }
                    }
                }
                // Expanded
                let mut current_layer = self.layer.unwrap_or(0);
                for (i, (t, pos, _, _)) in self.expanded.iter().enumerate().rev() {
                    match *t {
                        Type::Explored => continue,
                        Type::Extended => {
                            if let Some(c) = self.config.style.extended {
                                draw_pos(pos, c);
                            }
                        }
                        Type::Expanded => {
                            let color = if let Some(layer) = self.layer
                                && layer != 0
                            {
                                if current_layer > 0 && i < self.expanded_layers[current_layer - 1]
                                {
                                    current_layer -= 1;
                                }
                                self.config
                                    .style
                                    .expanded
                                    .color(current_layer, self.config.num_layers.unwrap_or(layer))
                            } else {
                                self.config.style.expanded.color(i, self.expanded.len())
                            };
                            draw_pos(pos, color);
                        }
                    }
                }
            } else {
                // Explored
                if let Some(color) = self.config.style.explored {
                    for (t, pos, _, _) in &self.expanded {
                        if *t == Type::Explored {
                            draw_pos(pos, color);
                        }
                    }
                }
                // Expanded
                let mut current_layer = 0;
                for (i, (t, pos, _, _)) in self.expanded.iter().enumerate() {
                    match *t {
                        Type::Explored => continue,
                        Type::Extended => {
                            if let Some(color) = self.config.style.extended {
                                draw_pos(pos, color);
                            }
                        }
                        Type::Expanded => {
                            let color = if let Some(layer) = self.layer
                                && layer != 0
                            {
                                if current_layer < layer && i >= self.expanded_layers[current_layer]
                                {
                                    current_layer += 1;
                                }
                                self.config
                                    .style
                                    .expanded
                                    .color(current_layer, self.config.num_layers.unwrap_or(layer))
                            } else {
                                self.config.style.expanded.color(i, self.expanded.len())
                            };
                            draw_pos(pos, color);
                        }
                    }
                }
            }
            if let Some((dt_color, box_color)) = self.config.style.trace {
                for pos in &self.trace {
                    if matches!(pos, ExpandPos::Block(_, _)) {
                        draw_pos(pos, box_color);
                    }
                }
                for pos in &self.trace {
                    if matches!(pos, ExpandPos::Single(_)) {
                        draw_pos(pos, dt_color);
                    }
                }
            }

            // Draw meeting points.
            if let Some(path_color) = self.config.style.path {
                for &p in &self.meeting_points {
                    self.draw_pixel(&mut canvas, p, path_color)
                }
            }

            // Draw matches.
            if self.config.style.draw_matches
                && let Some(h) = h
                && let Some(matches) = h.matches()
            {
                // first draw inexact matches, then exact ones on top.
                for exact in [false, true] {
                    for m in &matches {
                        if (m.match_cost == 0) != exact {
                            continue;
                        }
                        let mut b = self.cell_center(m.start);
                        let shrink = min(
                            self.config.style.match_shrink as i32,
                            self.config.cell_size - 1,
                        );
                        b.0 += shrink;
                        b.1 += shrink;
                        let mut e = self.cell_center(m.end);
                        e.0 -= shrink;
                        e.1 -= shrink;
                        let mut color = match m.pruned {
                            MatchStatus::Active => self.config.style.active_match,
                            MatchStatus::Pruned => self.config.style.pruned_match,
                            MatchStatus::PrePruned => self.config.style.pre_pruned_match,
                            MatchStatus::Filtered => self.config.style.filtered_match,
                        };
                        let width = self.config.style.match_width;
                        if m.match_cost > 0 {
                            if m.pruned == MatchStatus::Active {
                                color = GRAY;
                            }
                        }
                        if self.config.style.draw_parents {
                            if m.pruned == MatchStatus::Active {
                                Self::draw_diag_line(&mut canvas, b, e, color, width + 1);
                                color = color_for_pos(&m.start);
                                Self::draw_diag_line(&mut canvas, b, e, color, width - 1);
                            }
                        } else {
                            Self::draw_diag_line(&mut canvas, b, e, color, width);
                        }
                    }
                }
            }

            // Draw h calls.
            if self.config.style.draw_h_calls {
                for &p in &self.h_calls {
                    self.draw_pixel(&mut canvas, p, self.config.style.h_call);
                }
            }

            if self.config.style.draw_f_calls {
                // Draw f calls.
                for &(p, in_bounds, fixed) in &self.f_calls {
                    self.draw_pixel(
                        &mut canvas,
                        p,
                        match (in_bounds, fixed) {
                            (true, true) => GREEN,
                            (true, false) => BLUE,
                            (false, _) => RED,
                        },
                    );
                }
            }

            if self.config.style.draw_ranges {
                // Draw j_range.
                for &(start, end) in &self.j_ranges {
                    let tl = self.cell_begin(start + Pos(1, 0));
                    let wh = self.cell_end(end) - tl;
                    canvas.draw_rect(tl, wh.0, wh.1, BLUE);
                    canvas.draw_rect(tl + CPos(1, 1), wh.0 - 2, wh.1 - 2, BLUE);
                }

                // Draw fixed ranges.
                for &(start, end) in &self.fixed_j_ranges {
                    let tl = self.cell_begin(start);
                    let wh = self.cell_end(end) - tl;
                    if let Some(fixed) = self.config.style.fixed {
                        canvas.fill_rect(tl, wh.0, wh.1, fixed);
                    }
                    canvas.draw_rect(tl, wh.0, wh.1, BLACK);
                    // canvas.draw_rect(tl - CPos(1, 1), wh.0 + 2, wh.1 + 2, BLACK);
                }
            }

            if self.config.style.draw_fixed_h {
                // Draw fixed h.
                for &(start, end) in &self.fixed_h {
                    let tl = self.cell_begin(start);
                    let wh = self.cell_end(end) - tl;
                    if let Some(fixed) = self.config.style.fixed {
                        canvas.fill_rect(tl, wh.0, wh.1, fixed);
                    }
                    canvas.draw_rect(tl, wh.0, wh.1, BLACK);
                    // canvas.draw_rect(tl - CPos(1, 1), wh.0 + 2, wh.1 + 2, BLACK);
                }

                // Draw fixed h.
                if let Some((start, end)) = self.next_fixed_h {
                    let tl = self.cell_begin(start);
                    let wh = self.cell_end(end) - tl;
                    canvas.draw_rect(tl, wh.0, wh.1, BLACK);
                    // canvas.draw_rect(tl - CPos(1, 1), wh.0 + 2, wh.1 + 2, BLACK);
                }
            }

            // Draw path.
            if let Some(cigar) = cigar
                && let Some(path_color) = self.config.style.path
            {
                if let Some(path_width) = self.config.style.path_width {
                    for (from, to) in cigar.to_path().iter().tuple_windows() {
                        Self::draw_diag_line(
                            &mut canvas,
                            self.cell_center(*from),
                            self.cell_center(*to),
                            path_color,
                            path_width,
                        );
                    }
                } else {
                    for p in cigar.to_path() {
                        self.draw_pixel(&mut canvas, p, path_color)
                    }
                }
            }

            // Draw contours.
            if self.config.style.draw_contours
                && let Some(h) = h
                && h.layer(Pos(0, 0)).is_some()
            {
                let draw_right_border = |canvas: &mut CanvasBox, Pos(i, j): Pos| {
                    canvas.draw_line(
                        self.cell_begin(Pos(i + 1, j)),
                        self.cell_begin(Pos(i + 1, j + 1)),
                        self.config.style.contour,
                    );
                };
                let draw_bottom_border = |canvas: &mut CanvasBox, Pos(i, j): Pos| {
                    canvas.draw_line(
                        self.cell_begin(Pos(i, j + 1)),
                        self.cell_begin(Pos(i + 1, j + 1)),
                        self.config.style.contour,
                    );
                };

                // Right borders
                let mut hint = Default::default();
                let mut top_borders = vec![(0, h.layer(Pos(0, 0)).unwrap())];
                for i in 0..self.target.0 {
                    hint = h.layer_with_hint(Pos(i, 0), hint).unwrap().1;
                    let mut hint = hint;
                    for j in 0..=self.target.1 {
                        let pos = Pos(i, j);
                        let (v, new_hint) = h.layer_with_hint(pos, hint).unwrap();
                        hint = new_hint;
                        let pos_r = Pos(i + 1, j);
                        let (v_r, new_hint) = h.layer_with_hint(pos_r, hint).unwrap();
                        hint = new_hint;
                        if v_r != v {
                            draw_right_border(&mut canvas, pos);

                            if j == 0 {
                                top_borders.push((i + 1, v_r));
                            }
                        }
                    }
                }
                top_borders.push((self.target.0 + 1, 0));

                // Bottom borders
                let mut hint = Default::default();
                let mut left_borders = vec![(0, h.layer(Pos(0, 0)).unwrap())];
                for i in 0..=self.target.0 {
                    hint = h.layer_with_hint(Pos(i, 0), hint).unwrap().1;
                    let mut hint = hint;
                    for j in 0..self.target.1 {
                        let pos = Pos(i, j);
                        let (v, new_hint) = h.layer_with_hint(pos, hint).unwrap();
                        hint = new_hint;
                        let pos_l = Pos(i, j + 1);
                        let (v_l, new_hint) = h.layer_with_hint(pos_l, hint).unwrap();
                        hint = new_hint;
                        if v_l != v {
                            draw_bottom_border(&mut canvas, pos);

                            if i == 0 {
                                left_borders.push((j + 1, v_l));
                            }
                        }
                    }
                }
                left_borders.push((self.target.1, 0));

                // Draw numbers at the top and left.
                for (&(_left, layer), &(right, _)) in top_borders.iter().tuple_windows() {
                    if right < 3 {
                        continue;
                    }
                    let x = (right * self.config.cell_size - 1).saturating_sub(1);
                    canvas.write_text(
                        CPos(x as i32, -6),
                        HAlign::Right,
                        VAlign::Top,
                        &layer.to_string(),
                        BLACK,
                    );
                }
                for (&(_top, layer), &(bottom, _)) in left_borders.iter().tuple_windows() {
                    if bottom < 3 || bottom == self.target.1 {
                        continue;
                    }
                    let y = bottom * self.config.cell_size + 5;
                    canvas.write_text(
                        CPos(3, y as i32),
                        HAlign::Left,
                        VAlign::Bottom,
                        &layer.to_string(),
                        BLACK,
                    );
                }
            }

            // Draw tree.
            if let Some(parent) = parent
                && let Some(tree_color) = self.config.style.tree
            {
                for (_t, u, _, _) in &self.expanded {
                    let u = u.pos();
                    if self.config.style.tree_fr_only {
                        // Only trace if u is the furthest point on this diagonal.
                        let mut v = u;
                        let mut skip = false;
                        loop {
                            v = v + Pos(1, 1);
                            if !(v <= self.target) {
                                break;
                            }
                            if self
                                .expanded
                                .iter()
                                .filter(|(_, u, _, _)| u.pos() == v)
                                .count()
                                > 0
                            {
                                skip = true;
                                break;
                            }
                        }
                        if skip {
                            continue;
                        }
                    }
                    let mut st = State {
                        i: u.0,
                        j: u.1,
                        layer: None,
                    };
                    let mut path = vec![];
                    while let Some((p, op)) = parent(st) {
                        path.push((st, p, op));
                        let color = if let Some(AffineCigarOp::AffineOpen(_)) = op[1]
                            && let Some(c) = self.config.style.tree_affine_open
                        {
                            c
                        } else {
                            match op[0].unwrap() {
                                AffineCigarOp::Match => self.config.style.tree_match,
                                AffineCigarOp::Sub => self.config.style.tree_substitution,
                                _ => None,
                            }
                            .unwrap_or(tree_color)
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
                        let mut last = AffineCigarOp::Match;
                        for &(u, p, op) in path.iter().rev() {
                            let op = op[0].unwrap();
                            match op {
                                AffineCigarOp::Ins => {
                                    if last == AffineCigarOp::Del {
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
                                AffineCigarOp::Del => {
                                    if last == AffineCigarOp::Ins {
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
                                AffineCigarOp::Sub => {
                                    last = op;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            } // draw tree

            // Draw labels
            if self.config.style.draw_labels {
                let mut row = 0;
                if let Some(title) = &self.title {
                    canvas.write_text(
                        self.nw.start.right(self.nw.size.0 / 2).down(30 * row),
                        HAlign::Center,
                        VAlign::Top,
                        title,
                        BLACK,
                    );
                    row += 1;
                }
                if let Some(params) = &self.params
                    && !params.is_empty()
                {
                    canvas.write_text(
                        self.nw.start.right(self.nw.size.0 / 2).down(30 * row),
                        HAlign::Center,
                        VAlign::Top,
                        params,
                        (50, 50, 50, 0),
                    );
                    row += 1;
                }
                if let Some(comment) = &self.comment
                    && !comment.is_empty()
                {
                    canvas.write_text(
                        self.nw.start.right(self.nw.size.0 / 2).down(30 * row),
                        HAlign::Center,
                        VAlign::Top,
                        comment,
                        (50, 50, 50, 0),
                    );
                    row += 1;
                }
                canvas.write_text(
                    self.nw.start.right(self.nw.size.0),
                    HAlign::Right,
                    VAlign::Top,
                    &make_label("i = ", self.target.0),
                    GRAY,
                );
                canvas.write_text(
                    self.nw.start.down(self.nw.size.1),
                    HAlign::Left,
                    VAlign::Bottom,
                    &make_label("j = ", self.target.1),
                    GRAY,
                );

                canvas.write_text(
                    self.nw.start.right(self.nw.size.0 / 2).down(30 * row),
                    HAlign::Center,
                    VAlign::Top,
                    "DP states (i,j)",
                    GRAY,
                );
                canvas.write_text(
                    self.nw.start.right(self.nw.size.0 / 2).down(30 * (row + 1)),
                    HAlign::Center,
                    VAlign::Top,
                    &make_label(
                        "expanded: ",
                        self.expanded
                            .iter()
                            .filter(|&(t, ..)| *t == Expanded)
                            .count(),
                    ),
                    GRAY,
                );
            }
        }

        self.draw_dt(cigar);
        self.draw_f(cigar, h);

        let Some(canvas) = &self.canvas else {
            return;
        };
        let mut canvas = canvas.borrow_mut();

        // SAVE

        if self
            .config
            .save
            .is_active(self.frame_number, self.layer_number, is_last, is_new_layer)
        {
            self.save_canvas(&mut canvas, false, None);
            self.file_number += 1;
        }

        if self.config.save == When::First && self.config.draw == When::None {
            eprintln!("Exiting after saving first frame.");
            std::process::exit(0);
        }

        // Save the final frame separately if needed.
        if is_last && self.config.save_last {
            self.save_canvas(&mut canvas, true, None);
        }

        // SHOW

        if !self
            .config
            .draw
            .is_active(self.frame_number, self.layer_number, is_last, is_new_layer)
        {
            return;
        }

        //Keyboard events
        canvas.present();
        let key = canvas.wait(if self.config.paused || is_last {
            Duration::MAX
        } else {
            self.config.delay
        });
        match key {
            KeyboardAction::Next => {}
            KeyboardAction::Prev => {
                unimplemented!()
            }
            KeyboardAction::PausePlay => {
                self.config.paused = !self.config.paused;
            }
            KeyboardAction::Faster => {
                self.config.delay = self.config.delay.mul_f32(0.8);
            }
            KeyboardAction::Slower => {
                self.config.delay = self.config.delay.div_f32(0.8);
            }
            KeyboardAction::ToEnd => {
                self.config.draw = When::Last;
            }
            KeyboardAction::Exit => {
                eprintln!("Running aborted by user!");
                exit(1);
            }
            KeyboardAction::None => {}
        }
    }

    // Draw DT states to the top-right 1/3rd of the canvas.
    fn draw_dt(&mut self, cigar: Option<&AffineCigar>) {
        if !self.config.style.draw_dt || self.expanded.is_empty() {
            return;
        }
        let Some(canvas) = &self.canvas else {
            return;
        };
        let mut canvas = canvas.borrow_mut();

        let offset = self.dt.start.down(self.dt.size.0 / 2);
        // Cell_size goes down in powers of 2.
        let front_max = self.expanded.iter().map(|st| st.2).max().unwrap();
        let diagonal_min = self
            .expanded
            .iter()
            .map(|st| st.1.pos().diag())
            .min()
            .unwrap();
        let diagonal_max = self
            .expanded
            .iter()
            .map(|st| st.1.pos().diag())
            .max()
            .unwrap();
        let dt_cell_size = min(
            self.dt.size.0 as I / (front_max + 1),
            min(
                self.dt.size.1 as I / 2 / max(-diagonal_min + 1, diagonal_max + 1) as I,
                10,
            ),
        );

        // Draw grid

        // Divider
        canvas.draw_line(
            self.nw.start.right(self.nw.size.0),
            self.nw.start + self.nw.size,
            BLACK,
        );

        // Horizontal d lines
        let dy = |d: i32| offset.1 - d * dt_cell_size as i32 - dt_cell_size as i32 / 2;

        let mut draw_d_line = |d: i32, y: i32| {
            canvas.draw_line(
                self.dt.start.down(y),
                self.dt.start.down(y).right(self.dt.size.0),
                GRAY,
            );
            canvas.write_text(
                self.dt.start.down(y),
                HAlign::Right,
                VAlign::Center,
                &make_label("d = ", d),
                GRAY,
            );
        };
        // d=0
        draw_d_line(0, offset.1);
        // d=min
        if diagonal_min != 0 {
            draw_d_line(diagonal_min, dy(diagonal_min - 1));
        }
        // d=max
        if diagonal_max != 0 {
            draw_d_line(diagonal_max, dy(diagonal_max));
        }

        // Vertical g lines
        let mut draw_g_line = |g: i32| {
            let line_g = if g == 0 { 0 } else { g + 1 };
            let x = self.nw.size.0 as i32 + line_g * dt_cell_size as i32;
            canvas.draw_line(CPos(x, 0), CPos(x, self.canvas_size.1 as i32), GRAY);
            canvas.write_text(
                CPos(x, dy(diagonal_min - 1)),
                if g == 0 { HAlign::Left } else { HAlign::Right },
                VAlign::Top,
                &make_label("g = ", g),
                GRAY,
            );
        };
        // g=0
        draw_g_line(0);
        // g=min
        if front_max > 2 {
            draw_g_line(front_max as i32);
        }

        let state_coords = |st: (Pos, Cost)| -> CPos {
            CPos(offset.0 + (dt_cell_size * st.1) as i32, dy(st.0.diag()))
        };

        let draw_state =
            |canvas: &mut RefMut<CanvasBox>, st: &(Type, ExpandPos, Cost, Cost), color: Color| {
                canvas.fill_rect(
                    state_coords((st.1.pos(), st.2)),
                    dt_cell_size,
                    dt_cell_size,
                    color,
                );
            };

        if self.config.draw_old_on_top {
            // Expanded
            let mut current_layer = self.layer.unwrap_or(0);
            for (i, st) in self.expanded.iter().enumerate().rev() {
                let color = if let Some(layer) = self.layer
                    && layer != 0
                {
                    if current_layer > 0 && i < self.expanded_layers[current_layer - 1] {
                        current_layer -= 1;
                    }
                    self.config
                        .style
                        .expanded
                        .color(current_layer, self.config.num_layers.unwrap_or(layer))
                } else {
                    self.config.style.expanded.color(i, self.expanded.len())
                };
                draw_state(&mut canvas, st, color);
            }
        } else {
            // Expanded
            let mut current_layer = 0;
            for (i, st) in self.expanded.iter().enumerate() {
                let color = if let Some(layer) = self.layer
                    && layer != 0
                {
                    if current_layer < layer && i >= self.expanded_layers[current_layer] {
                        current_layer += 1;
                    }
                    self.config
                        .style
                        .expanded
                        .color(current_layer, self.config.num_layers.unwrap_or(layer))
                } else {
                    self.config.style.expanded.color(i, self.expanded.len())
                };
                draw_state(&mut canvas, st, color);
            }
        }

        // Title
        canvas.write_text(
            CPos(self.nw.size.0 as i32 + self.dt.size.0 as i32 / 2, 0),
            HAlign::Center,
            VAlign::Top,
            "Diagonal Transition states (g, d) = (s, k)",
            GRAY,
        );

        if let Some(cigar) = cigar {
            if let Some(path_color) = self.config.style.path {
                for (from, to) in cigar
                    .to_path_with_costs(AffineCost::unit())
                    .iter()
                    .tuple_windows()
                {
                    let from_coords = state_coords(*from);
                    let to_coords = state_coords(*to);
                    if from_coords == to_coords {
                        continue;
                    }
                    if let Some(path_width) = self.config.style.path_width {
                        Self::draw_diag_line(
                            &mut canvas,
                            CPos(
                                from_coords.0 + dt_cell_size as i32 / 2,
                                from_coords.1 + dt_cell_size as i32 / 2,
                            ),
                            CPos(
                                to_coords.0 + dt_cell_size as i32 / 2,
                                to_coords.1 + dt_cell_size as i32 / 2,
                            ),
                            path_color,
                            path_width,
                        );
                    } else {
                        draw_state(
                            &mut canvas,
                            &(Type::Expanded, ExpandPos::Single(from.0), from.1, 0),
                            path_color,
                        );
                    }
                }
            }
        }
    }

    fn draw_f<'a, H: HeuristicInstance<'a>>(&mut self, cigar: Option<&AffineCigar>, h: Option<&H>) {
        if !self.config.style.draw_f || self.expanded.is_empty() {
            return;
        }
        let Some(canvas) = &self.canvas else {
            return;
        };
        let mut canvas = canvas.borrow_mut();

        // Soft red
        const SOFT_RED: Color = (244, 113, 116, 0);
        const _SOFT_GREEN: Color = (111, 194, 118, 0);

        // Cell size from DT
        // Cell_size goes down in powers of 2.
        let front_max = self.expanded.iter().map(|st| st.2).max().unwrap();
        let diagonal_min = self
            .expanded
            .iter()
            .map(|st| st.1.pos().diag())
            .min()
            .unwrap();
        let diagonal_max = self
            .expanded
            .iter()
            .map(|st| st.1.pos().diag())
            .max()
            .unwrap();
        let dt_cell_size = min(
            self.dt.size.0 as I / (front_max + 1),
            min(
                self.dt.size.1 as I / 2 / max(-diagonal_min + 1, diagonal_max + 1) as I,
                10,
            ),
        );

        // f is plotted with f_min at y=height-30, and f_max at y=3/4*height
        let f_min = self
            .expanded
            .iter()
            .filter(|st| st.0 == Expanded)
            .map(|st| st.3)
            .min()
            .unwrap();
        let f_max = self
            .expanded
            .iter()
            .filter(|st| st.0 == Expanded)
            .map(|st| st.3)
            .max()
            .unwrap();
        let f_y = |f| {
            (self.canvas_size.1 as i32).saturating_sub(
                ((f as f32 - f_min as f32) / max(f_max - f_min, 1) as f32
                    * self.canvas_size.1 as f32
                    / 4.) as i32
                    + 30,
            )
        };

        // Draw shifted states after pruning.
        if let Some(h) = h {
            for (t, pos, g, _) in self.expanded.iter() {
                if *t == Explored {
                    continue;
                }
                let f = g + h.h(pos.pos());
                let rel_f = (f as f64 - f_min as f64) / max(f_max - f_min, 1) as f64;
                if rel_f > 1.5 {
                    continue;
                }
                let color = Gradient::Gradient(GRAY..WHITE).color_f(2. * rel_f - 2.);
                let y = f_y(f);
                canvas.fill_rect(
                    CPos((pos.pos().0 * self.config.cell_size) as i32, y),
                    self.config.cell_size,
                    1,
                    color,
                );
                canvas.fill_rect(
                    CPos(self.nw.size.0 as i32 + (g * dt_cell_size) as i32, y),
                    dt_cell_size,
                    1,
                    color,
                );
            }
        }

        for (i, (t, pos, g, f)) in self.expanded.iter().enumerate() {
            if *t == Explored {
                continue;
            }
            let color = Gradient::TurboGradient(0.2..0.95).color(i, self.expanded.len());
            canvas.fill_rect(
                CPos((pos.pos().0 * self.config.cell_size) as i32, f_y(*f)),
                self.config.cell_size,
                2,
                color,
            );
            if self.config.style.draw_dt {
                canvas.fill_rect(
                    CPos(self.nw.size.0 as i32 + (g * dt_cell_size) as i32, f_y(*f)),
                    dt_cell_size,
                    2,
                    color,
                );
            }
        }

        // Horizontal line at final cost when path is given.
        let mut cost = None;
        if let Some(cigar) = cigar {
            let c = cigar
                .to_path_with_costs(AffineCost::unit())
                .last()
                .unwrap()
                .1;
            cost = Some(c);
            let y = f_y(c);
            canvas.draw_line(CPos(0, y), CPos(self.canvas_size.0 as i32, y), SOFT_RED);

            canvas.write_text(
                CPos(self.nw.size.0 as i32, y),
                HAlign::Left,
                VAlign::Center,
                &make_label("g* = ", c),
                SOFT_RED,
            );
        };

        canvas.write_text(
            CPos(
                self.nw.size.0 as i32 + self.dt.size.0 as i32 / 2,
                self.dt.size.1 as i32,
            ),
            HAlign::Center,
            VAlign::Bottom,
            "max f per front g",
            SOFT_RED,
        );
        for f in [f_min, f_max] {
            if Some(f) == cost {
                continue;
            }
            canvas.write_text(
                CPos(self.nw.size.0 as i32, f_y(f)),
                HAlign::Left,
                VAlign::Center,
                &make_label("f = ", f),
                SOFT_RED,
            );
        }

        canvas.write_text(
            CPos(self.nw.size.0 as i32 / 2, self.nw.size.1 as i32),
            HAlign::Center,
            VAlign::Bottom,
            "max f per column i",
            SOFT_RED,
        );
    }
}
