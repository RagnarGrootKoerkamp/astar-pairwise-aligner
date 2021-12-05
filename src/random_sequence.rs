use bio::alphabets::Alphabet;
use bio_types::sequence::Sequence;
use block_aligner::simulate;
use rand::Rng;

/// Generate a random sequence of length `n` using the given alphabet.
pub fn random_sequence<R: Rng>(n: usize, alphabet: &Alphabet, rng: &mut R) -> Sequence {
    assert!(!alphabet.is_empty(), "Alphabet may not be empty.");
    let symbols = alphabet.symbols.iter().map(|x| x as u8).collect::<Vec<_>>();
    simulate::rand_str(n, &symbols, rng)
}

/// Given a sequence, generate a sequence with `e` edits.
pub fn random_mutate<R: Rng>(
    sequence: &Sequence,
    alphabet: &Alphabet,
    e: usize,
    rng: &mut R,
) -> Sequence {
    assert!(!alphabet.is_empty(), "Alphabet may not be empty.");
    let symbols = alphabet.symbols.iter().map(|x| x as u8).collect::<Vec<_>>();
    simulate::rand_mutate(sequence, e, &symbols, rng)
}
