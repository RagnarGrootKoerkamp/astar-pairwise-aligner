pub use pa_generate::*;
use pa_types::Sequence;

/// FIXME(inline this)
pub fn setup_sequences_with_seed(seed: u64, n: usize, e: f32) -> (Sequence, Sequence) {
    uniform_seeded(n, e, seed)
}
