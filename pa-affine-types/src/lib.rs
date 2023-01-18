use pa_types::{Cost, Seq, I};

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

impl State {
    pub fn new(i: I, j: I, layer: Layer) -> Self {
        Self { i, j, layer }
    }
}

/// Generic pairwise global alignment interface.
pub trait AffineAligner {
    /// An alignment of sequences `a` and `b`.
    /// The returned cost is the *non-negative* cost of the alignment.
    /// Returns a trace when specified on construction.
    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Option<AffineCigar>);
}
