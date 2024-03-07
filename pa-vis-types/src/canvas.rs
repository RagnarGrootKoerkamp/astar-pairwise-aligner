use std::{
    ops::{Add, Div, Sub},
    path::Path,
    time::Duration,
};

use pa_types::I;

pub fn to_label(c: u8) -> String {
    String::from_utf8(vec![c]).unwrap()
}
pub fn make_label(text: &str, val: impl ToString) -> String {
    text.to_string() + &val.to_string()
}

/// Position of a cell in the grid.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct CPos(pub i32, pub i32);
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
impl Div<i32> for CPos {
    type Output = CPos;

    fn div(self, rhs: i32) -> Self::Output {
        CPos(self.0 / rhs, self.1 / rhs)
    }
}
impl CPos {
    pub fn left(self, d: i32) -> Self {
        let CPos(x, y) = self;
        Self(x - d, y)
    }
    pub fn right(self, d: i32) -> Self {
        let CPos(x, y) = self;
        Self(x + d, y)
    }
    pub fn up(self, d: i32) -> Self {
        let CPos(x, y) = self;
        Self(x, y - d)
    }
    pub fn down(self, d: i32) -> Self {
        let CPos(x, y) = self;
        Self(x, y + d)
    }
}

pub type Color = (u8, u8, u8, u8);
pub const BLACK: Color = (0, 0, 0, 0);
pub const GRAY: Color = (128, 128, 128, 0);
pub const WHITE: Color = (255, 255, 255, 0);
pub const RED: Color = (255, 0, 0, 0);
pub const PURPLE: Color = (158, 50, 158, 0);
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

#[derive(PartialEq, Eq)]
pub enum KeyboardAction {
    Next,
    Prev,
    PausePlay,
    Faster,
    Slower,
    ToEnd,
    Exit,
    None,
}

pub trait Canvas {
    fn fill_background(&mut self, color: Color);
    fn fill_rect(&mut self, p: CPos, w: I, h: I, color: Color);
    fn fill_rects(&mut self, rects: &[(CPos, I, I)], color: Color) {
        for &(p, w, h) in rects {
            self.fill_rect(p, w, h, color);
        }
    }
    fn draw_rect(&mut self, p: CPos, w: I, h: I, color: Color);
    fn draw_point(&mut self, p: CPos, color: Color) {
        self.draw_rect(p, 1, 1, color);
    }
    fn draw_line(&mut self, p: CPos, q: CPos, color: Color);

    fn write_text(&mut self, p: CPos, ha: HAlign, va: VAlign, text: &str, color: Color);

    fn save(&mut self, _path: &Path) {}
    fn save_transparent(&mut self, _path: &Path, _bg_color: Color) {}
    fn present(&mut self) {}

    fn wait(&mut self, timeout: Duration) -> KeyboardAction;
}

pub type CanvasBox = Box<dyn Canvas>;
