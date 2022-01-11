pub use crate::prelude::*;

// For quick testing
pub fn setup_with_seed(
    n: usize,
    e: f32,
    seed: u64,
) -> (Sequence, Sequence, Alphabet, SequenceStats) {
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    let alphabet = Alphabet::new(b"ACTG");
    let a = random_sequence(n, &alphabet, &mut rng);
    let b = random_mutate(&a, &alphabet, (n as f32 * e) as usize, &mut rng);

    let sequence_stats = SequenceStats {
        len_a: a.len(),
        len_b: b.len(),
        error_rate: e,
        source: Source::Uniform,
    };
    (a, b, alphabet, sequence_stats)
}

pub fn setup(n: usize, e: f32) -> (Sequence, Sequence, Alphabet, SequenceStats) {
    setup_with_seed(n, e, 31415)
}
