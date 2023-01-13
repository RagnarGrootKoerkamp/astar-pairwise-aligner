use crate::aligners::Sequence;

pub use pa_generate::*;

pub fn setup_sequences_with_seed(seed: u64, n: usize, e: f32) -> (Sequence, Sequence) {
    uniform_seeded(n, e, seed)
}
