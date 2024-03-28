use pa_types::I;
use pa_vis::canvas::{self, *};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// A canvas element and context.
/// Note that these are used for double-buffered drawing calls only.
/// present() copies the contents to the on-screen canvas.
pub struct HtmlCanvas {
    element: HtmlCanvasElement,
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

pub static mut FRAMES_PRESENTED: usize = 0;

impl Canvas for HtmlCanvas {
    fn fill_background(&mut self, _color: canvas::Color) {
        //self.context.set_fill_style(&jscol(color));
        self.context.clear_rect(
            0.,
            0.,
            self.context.canvas().unwrap().width() as f64,
            self.context.canvas().unwrap().height() as f64,
        );
    }

    fn fill_rect(&mut self, CPos(x, y): CPos, w: I, h: I, color: canvas::Color) {
        self.context.set_fill_style(&jscol(color));
        self.context
            .fill_rect(x as f64, y as f64, w as f64, h as f64);
    }

    fn draw_rect(&mut self, CPos(x, y): CPos, w: I, h: I, color: canvas::Color) {
        self.context.begin_path();
        self.context.set_stroke_style(&jscol(color));
        self.context
            .stroke_rect(x as f64, y as f64, w as f64, h as f64);
    }

    fn draw_line(&mut self, p: CPos, q: CPos, color: Color) {
        self.context.begin_path();
        self.context.set_stroke_style(&jscol(color));
        self.context.set_line_width(0.0);
        self.context.move_to(p.0 as f64 + 0.5, p.1 as f64 + 0.5);
        self.context.line_to(q.0 as f64 + 0.5, q.1 as f64 + 0.5);
        self.context.stroke();
    }

    fn write_text(
        &mut self,
        CPos(x, y): CPos,
        ha: canvas::HAlign,
        va: canvas::VAlign,
        text: &str,
        color: Color,
    ) {
        self.context.set_fill_style(&jscol(color));
        self.context.set_font("24px Arial");
        self.context.set_text_baseline("middle");
        self.context.set_text_align(match ha {
            canvas::HAlign::Left => "left",
            canvas::HAlign::Center => "center",
            canvas::HAlign::Right => "right",
        });
        self.context.set_text_baseline(match va {
            canvas::VAlign::Top => "top",
            canvas::VAlign::Center => "middle",
            canvas::VAlign::Bottom => "bottom",
        });
        self.context.fill_text(text, x as f64, y as f64).unwrap();
    }

    fn present(&mut self) {
        // Copy the internal image to the on-screen canvas.
        let element = get::<HtmlCanvasElement>("canvas");
        element.set_width(1200 as u32);
        element.set_height(800 as u32);
        let context = element
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        log("Present");
        context
            .draw_image_with_html_canvas_element(&self.element, 0., 0.)
            .unwrap();
        unsafe {
            FRAMES_PRESENTED += 1;
        }
    }

    fn wait(&mut self, _timeout: std::time::Duration) -> canvas::KeyboardAction {
        canvas::KeyboardAction::None
    }
}

pub fn new_canvas(_w: usize, _h: usize, _title: &str) -> HtmlCanvas {
    console_error_panic_hook::set_once();
    let element = document()
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();
    element.set_width(1200 as u32);
    element.set_height(800 as u32);
    let context = element
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    HtmlCanvas { element, context }
}
