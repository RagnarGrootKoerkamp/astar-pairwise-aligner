use crate::canvas::Canvas;
use crate::canvas::Color;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;

use super::CPos;

pub struct HtmlCanvas {
    context: CanvasRenderingContext2d,
}

fn jscol((r, g, b, _): Color) -> JsValue {
    JsValue::from_str(&format!("rgb({r},{g},{b})"))
}
fn jsstr(s: &str) -> JsValue {
    JsValue::from_str(s)
}
#[allow(unused)]
fn log(s: &str) {
    web_sys::console::log_1(&jsstr(s));
}

fn document() -> web_sys::Document {
    let window = web_sys::window().expect("no global `window` exists");
    window.document().expect("should have a document on window")
}

fn get<T: wasm_bindgen::JsCast>(id: &str) -> T {
    document()
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<T>()
        .unwrap()
}

impl Canvas for HtmlCanvas {
    fn fill_background(&mut self, _color: crate::canvas::Color) {
        //self.context.set_fill_style(&jscol(color));
        self.context.clear_rect(
            0.,
            0.,
            self.context.canvas().unwrap().width() as f64,
            self.context.canvas().unwrap().height() as f64,
        );
    }

    fn fill_rect(&mut self, CPos(x, y): CPos, w: u32, h: u32, color: crate::canvas::Color) {
        self.context.set_fill_style(&jscol(color));
        self.context
            .fill_rect(x as f64, y as f64, w as f64, h as f64);
    }

    fn draw_rect(&mut self, CPos(x, y): CPos, w: u32, h: u32, color: crate::canvas::Color) {
        self.context.begin_path();
        self.context.set_stroke_style(&jscol(color));
        self.context
            .stroke_rect(x as f64, y as f64, w as f64, h as f64);
    }

    fn write_text(
        &mut self,
        CPos(x, y): CPos,
        ha: crate::canvas::HAlign,
        _va: crate::canvas::VAlign,
        text: &str,
        color: Color,
    ) {
        self.context.set_fill_style(&jscol(color));
        self.context.set_font("20px Arial");
        self.context.set_text_baseline("middle");
        self.context.set_text_align(match ha {
            crate::canvas::HAlign::Left => "left",
            crate::canvas::HAlign::Center => "center",
            crate::canvas::HAlign::Right => "right",
        });
        self.context.fill_text(text, x as f64, y as f64).unwrap();
    }

    // no-op
    fn save(&mut self, _path: &std::path::Path) {}

    fn draw_line(&mut self, _p: CPos, _q: CPos, _color: Color) {
        todo!();
    }

    fn wait(&mut self, _timeout: std::time::Duration) -> super::KeyboardAction {
        todo!()
    }
}

pub fn new_canvas() -> HtmlCanvas {
    let element = get::<HtmlCanvasElement>("canvas");
    element.set_height(1200 as u32);
    element.set_width(800 as u32);
    let context = element
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    HtmlCanvas { context }
}
