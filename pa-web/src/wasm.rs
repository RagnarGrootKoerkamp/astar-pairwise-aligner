use crate::{html::FRAMES_PRESENTED, interaction::Interaction};
use astarpa::{make_aligner_with_visualizer, HeuristicParams};
use pa_bin::Cli;
use pa_types::*;
use pa_vis::cli::{VisualizerArgs, VisualizerType};
use std::{cell::Cell, ops::ControlFlow, sync::Mutex};
use wasm_bindgen::prelude::*;

fn document() -> web_sys::Document {
    let window = web_sys::window().expect("no global `window` exists");
    window.document().expect("should have a document on window")
}

pub fn get<T: wasm_bindgen::JsCast>(id: &str) -> T {
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Args {
    pub cli: Cli,
    pub visualizer: VisualizerArgs,
}

pub static mut INTERACTION: Interaction = Interaction::default();
pub static ARGS: Mutex<Cell<Option<Args>>> = Mutex::new(Cell::new(None));

pub fn run() {
    if unsafe { INTERACTION.is_done() } {
        return;
    }
    let mut args = ARGS.lock().unwrap();
    if let Some(args) = args.get_mut() {
        let before = unsafe { FRAMES_PRESENTED };

        let VisualizerType::Visualizer(visualizer) = args.visualizer.make_visualizer() else {
            panic!();
        };
        let aligner = make_aligner_with_visualizer(true, &HeuristicParams::default(), visualizer);
        args.cli.process_input_pairs(|a: Seq, b: Seq| {
            // Run the pair.
            // TODO: Show the result somewhere.
            aligner.align(a, b);
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
