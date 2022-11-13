use crate::{
    canvas::html::FRAMES_PRESENTED,
    interaction::Interaction,
    prelude::*,
    runner::{AlignWithHeuristic, Cli},
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::ops::ControlFlow;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::HtmlTextAreaElement;

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
fn jsstr(s: &str) -> JsValue {
    JsValue::from_str(s)
}
#[allow(unused)]
pub fn log(s: &str) {
    web_sys::console::log_1(&jsstr(s));
}

pub static mut INTERACTION: Interaction = Interaction::default();
static mut CLI: Option<Cli> = None;

pub fn run() {
    if unsafe { INTERACTION.is_done() } {
        return;
    }
    if let Some(args) = unsafe { &CLI } {
        let before = unsafe { FRAMES_PRESENTED };
        args.input.process_input_pairs(|a: Seq, b: Seq| {
            // Run the pair.
            // TODO: Show the result somewhere.
            let _r = if args.algorithm.algorithm.external() {
                unimplemented!();
            } else {
                args.heuristic
                    .run_on_heuristic(AlignWithHeuristic { a, b, args: &args })
            };
            ControlFlow::Break(())
        });
        unsafe {
            let after = FRAMES_PRESENTED;
            if before == after {
                INTERACTION.done();
            }
        }
    }
}

#[wasm_bindgen]
pub fn reset() {
    let args_json = get::<HtmlTextAreaElement>("args").value();
    unsafe {
        INTERACTION.reset(usize::MAX);
        CLI = Some(serde_json::from_str(&args_json).unwrap());
        if let Some(cli) = &mut CLI {
            // Fix the seed so that reruns for consecutive draws don't change it.
            cli.input
                .generate
                .seed
                .get_or_insert(ChaCha8Rng::from_entropy().gen_range(0..u64::MAX));
        }
    }
}

#[wasm_bindgen]
pub fn prev() {
    unsafe {
        INTERACTION.prev();
        run();
    };
}

#[wasm_bindgen]
pub fn next() {
    unsafe {
        INTERACTION.next();
        run();
    }
}
