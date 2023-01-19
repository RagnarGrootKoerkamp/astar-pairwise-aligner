#![feature(let_chains, int_roundings)]

pub mod canvas;
pub mod cli;
#[cfg(feature = "sdl")]
mod sdl;
pub mod visualizer;
