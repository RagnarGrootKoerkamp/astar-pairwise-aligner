// Deps for HTML canvas rendering
#[cfg(feature = "wasm")]
pub mod html;

#[cfg(feature = "bin")]
pub mod sdl;

use std::ops::{Add, Sub};

pub type Color = (u8, u8, u8, u8);
pub const BLACK: Color = (0, 0, 0, 0);
pub const GRAY: Color = (128, 128, 128, 0);
pub const WHITE: Color = (255, 255, 255, 0);
pub const RED: Color = (255, 0, 0, 0);
pub const GREEN: Color = (0, 255, 0, 0);
pub const BLUE: Color = (0, 0, 255, 0);
pub const CYAN: Color = (0, 255, 255, 0);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HAlign {
    Left,
    Center,
    Right,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VAlign {
    Top,
    Center,
    Bottom,
}

pub trait Canvas {
    fn fill_background(&mut self, color: Color);
    fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color);
    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color);

    fn write_text(&mut self, x: i32, y: i32, ha: HAlign, va: VAlign, text: &str);

    fn save(&mut self);
    fn present(&mut self) {}
}

pub type CanvasBox = Box<dyn Canvas>;

// Utility functions.

pub fn to_label(c: u8) -> String {
    String::from_utf8(vec![c]).unwrap()
}
pub fn make_label(text: &str, val: impl ToString) -> String {
    text.to_string() + &val.to_string()
}

/// Position of a cell in the grid.
#[derive(Copy, Clone)]
pub struct CPos(pub usize, pub usize);
impl Add<CPos> for CPos {
    type Output = CPos;

    fn add(self, rhs: CPos) -> Self::Output {
        CPos(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl Sub<CPos> for CPos {
    type Output = CPos;

    fn sub(self, rhs: CPos) -> Self::Output {
        CPos(self.0 - rhs.0, self.1 - rhs.1)
    }
}
impl CPos {
    pub fn left(self, d: usize) -> Self {
        let CPos(x, y) = self;
        Self(x - d, y)
    }
    pub fn right(self, d: usize) -> Self {
        let CPos(x, y) = self;
        Self(x + d, y)
    }
    pub fn up(self, d: usize) -> Self {
        let CPos(x, y) = self;
        Self(x, y - d)
    }
    pub fn down(self, d: usize) -> Self {
        let CPos(x, y) = self;
        Self(x, y + d)
    }
}

// Cell size
const CS: u32 = 30;
const BACKGROUND: Color = (255, 255, 255, 0);

pub fn canvas_size(w: usize, h: usize) -> (usize, usize) {
    (w * CS as usize, h * CS as usize)
}

pub fn draw_background(canvas: &mut CanvasBox) {
    canvas.fill_background(BACKGROUND);
}

fn write_label(x: i32, y: i32, ha: HAlign, va: VAlign, text: &str, canvas: &mut CanvasBox) {
    canvas.write_text(x, y, ha, va, text);
}

pub fn draw_label(CPos(x, y): CPos, label: &str, canvas: &mut CanvasBox) {
    canvas.write_text(
        x as i32 * CS as i32 + CS as i32 / 2,
        y as i32 * CS as i32 + CS as i32 / 2,
        HAlign::Center,
        VAlign::Center,
        label,
    );
}

pub fn draw_text(CPos(x, y): CPos, label: &str, canvas: &mut CanvasBox) {
    let x = x as i32 * CS as i32;
    let y = y as i32 * CS as i32;
    write_label(
        x,
        y + CS as i32 / 2,
        HAlign::Left,
        VAlign::Center,
        label,
        canvas,
    );
}

pub fn draw_char_box(CPos(x, y): CPos, c: u8, color: Color, canvas: &mut CanvasBox) {
    let x = x as i32 * CS as i32;
    let y = y as i32 * CS as i32;
    canvas.fill_rect(x, y, CS, CS, color);
    canvas.draw_rect(x, y, CS, CS, BLACK);
    // letter
    canvas.write_text(
        x + CS as i32 / 2,
        y + CS as i32 / 2,
        HAlign::Center,
        VAlign::Center,
        &to_label(c),
    );
}

pub fn draw_highlight_box(
    CPos(x, y): CPos,
    w: usize,
    h: usize,
    color: Color,
    canvas: &mut CanvasBox,
) {
    let x = x as i32 * CS as i32;
    let y = y as i32 * CS as i32;
    if w == 0 {
        for margin in 0..=2 {
            canvas.draw_rect(
                x - margin as i32,
                y as i32,
                2 * margin,
                h as u32 * CS,
                color,
            );
        }
    } else if h == 0 {
        for margin in 0..=2 {
            canvas.draw_rect(
                x as i32,
                y - margin as i32,
                w as u32 * CS,
                2 * margin,
                color,
            );
        }
    } else {
        for margin in 1..=3 {
            canvas.draw_rect(
                x + margin as i32,
                y + margin as i32,
                w as u32 * CS - 2 * margin,
                h as u32 * CS - 2 * margin,
                color,
            );
        }
    }
}

// Draw a box around a cell.
pub fn draw_highlight(p: CPos, color: Color, canvas: &mut CanvasBox) {
    draw_highlight_box(p, 1, 1, color, canvas);
}

pub fn draw_string(
    CPos(x, y): CPos,
    s: &[u8],
    color: impl Fn(usize) -> Color,
    canvas: &mut CanvasBox,
) {
    for (i, &c) in s.iter().enumerate() {
        draw_char_box(CPos(x + i, y), c, color(i), canvas);
    }
}

pub fn draw_string_with_labels(
    CPos(x, y): CPos,
    s: &[u8],
    color: impl Fn(usize) -> Color,
    canvas: &mut CanvasBox,
) {
    draw_label(CPos(x - 1, y - 1), "i", canvas);
    for i in 0..s.len() {
        draw_label(CPos(x + i, y - 1), &i.to_string(), canvas);
    }
    draw_label(CPos(x - 1, y), "S", canvas);
    draw_string(CPos(x, y), s, color, canvas);
}
