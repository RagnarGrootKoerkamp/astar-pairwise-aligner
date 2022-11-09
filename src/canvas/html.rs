use crate::alg::Viz;
use crate::alg::{bibwt::BiBWT, bwt::BWT, suffix_array::SA};
use crate::canvas::Canvas;
use crate::canvas::CanvasBox;
use crate::canvas::Color;
use crate::canvas::BLACK;
use crate::interaction::Interaction;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlInputElement;
use web_sys::HtmlSelectElement;

struct HtmlCanvas {
    context: CanvasRenderingContext2d,
}

fn jscol((r, g, b): Color) -> JsValue {
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

    fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: crate::canvas::Color) {
        self.context.set_fill_style(&jscol(color));
        self.context
            .fill_rect(x as f64, y as f64, w as f64, h as f64);
    }

    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: crate::canvas::Color) {
        self.context.begin_path();
        self.context.set_stroke_style(&jscol(color));
        self.context
            .stroke_rect(x as f64, y as f64, w as f64, h as f64);
    }

    fn write_text(
        &mut self,
        x: i32,
        y: i32,
        ha: crate::canvas::HAlign,
        _va: crate::canvas::VAlign,
        text: &str,
    ) {
        self.context.set_fill_style(&jscol(BLACK));
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
    fn save(&mut self) {}
}

thread_local! {
    static ALG: RefCell<Box<dyn Viz>> = RefCell::new(Box::new(SA::new("GTCCCGATGTCATGTCAGGA$".as_bytes().to_vec())));
}
static mut INTERACTION: Interaction = Interaction::default();

#[wasm_bindgen]
pub fn reset() {
    let alg_name = get::<HtmlSelectElement>("algorithm").value();
    let mut string = get::<HtmlInputElement>("string").value().into_bytes();
    if string.is_empty() {
        string = "GTCCCGATGTCATGTCAGGA$".as_bytes().to_vec()
    };
    let mut query = get::<HtmlInputElement>("query").value().into_bytes();
    if query.is_empty() {
        query = "GTCC".as_bytes().to_vec()
    };
    let new_alg = match alg_name.as_str() {
        "suffix-array" => Box::new(SA::new(string)) as Box<dyn Viz>,
        "bwt" => Box::new(BWT::new(string, query)) as Box<dyn Viz>,
        "bibwt" => Box::new(BiBWT::new(string, query)) as Box<dyn Viz>,
        _ => panic!(),
    };
    unsafe {
        INTERACTION.reset(new_alg.num_states());
    }
    ALG.set(new_alg);
    let element = get::<HtmlCanvasElement>("canvas");
    let (w, h) = ALG.with(|alg| alg.borrow().canvas_size());
    element.set_height(h as u32);
    element.set_width(w as u32);

    draw();
}

#[wasm_bindgen]
pub fn draw() {
    let element = get::<HtmlCanvasElement>("canvas");
    let context = element
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    let ref mut canvas = Box::new(HtmlCanvas { context }) as CanvasBox;
    unsafe {
        loop {
            if ALG.with(|alg| alg.borrow_mut().draw(INTERACTION.get(), canvas)) {
                break;
            }
            INTERACTION.step();
        }
    }
}

#[wasm_bindgen]
pub fn prev() {
    unsafe {
        INTERACTION.prev();
        draw();
    };
}

#[wasm_bindgen]
pub fn next() {
    unsafe {
        INTERACTION.next();
        draw();
    }
}
