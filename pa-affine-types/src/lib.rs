use pa_types::{Cost, Pos, Seq, I};

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

    pub fn pos(&self) -> Pos {
        Pos(self.i, self.j)
    }
}

/// Generic pairwise global alignment interface.
pub trait AffineAligner: std::fmt::Debug {
    /// An alignment of sequences `a` and `b`.
    /// The returned cost is the *non-negative* cost of the alignment.
    /// Costmodel and traceback parameters must be specified on construction of the aligner.
    fn align_affine(&mut self, a: Seq, b: Seq) -> (Cost, Option<AffineCigar>);
}
