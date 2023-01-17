use pa_types::I;

pub mod cigar;
pub mod cost_model;

// Re-export types for convenience of `use pa_affine_types::*;`.
pub use cigar::*;
pub use cost_model::*;

pub type Layer = Option<usize>;

/// State in the edit graph during an affine alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub i: I,
    pub j: I,
    pub layer: Layer,
}
