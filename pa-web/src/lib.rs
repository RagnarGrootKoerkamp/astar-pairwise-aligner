#![feature(duration_constants)]

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use wasm_bindgen::prelude::*;
use web_sys::HtmlTextAreaElement;

pub mod html;
pub mod interaction;
pub mod wasm;

use wasm::*;

#[wasm_bindgen]
pub fn reset() {
    let args_json = get::<HtmlTextAreaElement>("args").value();
    unsafe {
        INTERACTION.reset(usize::MAX);
    }

    let mut args = ARGS.lock().unwrap();
    let args = args.get_mut();
    *args = Some(serde_json::from_str(&args_json).unwrap());
    // Fix the seed so that reruns for consecutive draws don't change it.
    args.as_mut()
        .unwrap()
        .cli
        .generate
        .seed
        .get_or_insert(ChaCha8Rng::from_entropy().gen_range(0..u64::MAX));
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
