use crate::{html::FRAMES_PRESENTED, interaction::Interaction};
use astarpa::{cli::Cli, make_aligner_with_visualizer};
use pa_types::*;
use pa_vis::cli::{VisualizerArgs, VisualizerType};
use std::ops::ControlFlow;
use wasm_bindgen::{prelude::*, JsCast};

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
pub static mut ARGS: Option<Args> = None;

pub fn run() {
    if unsafe { INTERACTION.is_done() } {
        return;
    }
    if let Some(args) = unsafe { &ARGS } {
        let before = unsafe { FRAMES_PRESENTED };

        let VisualizerType::Visualizer(visualizer) = args.visualizer.make_visualizer() else {
            panic!();
        };
        let aligner = make_aligner_with_visualizer(
            args.cli.diagonal_transition,
            &args.cli.heuristic,
            visualizer,
        );
        args.cli.input.process_input_pairs(|a: Seq, b: Seq| {
            // Run the pair.
            // TODO: Show the result somewhere.
            let _r = aligner.align(a, b);
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
