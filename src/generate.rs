use clap::{Parser, ValueEnum};
use itertools::Itertools;
use rand::{Rng, SeedableRng};

use crate::{aligners::Sequence, prelude::*};

#[derive(ValueEnum, Default, Debug, Clone, Copy)]
pub enum ErrorModel {
    #[default]
    Uniform,
    /// Make a single gap (insertion or deletion) of size e*n.
    Gap,
    /// Delete a region of size e*n and insert a region of size e*n.
    Move,
}

#[derive(Parser, Clone)]
pub struct GenerateArgs {
    /// The number of sequence pairs to generate
    #[clap(short = 'x', long, default_value_t = 1, display_order = 2)]
    pub cnt: usize,

    /// Length of generated sequences
    #[clap(
        short = 'n',
        long,
        group = "inputmethod",
        requires_all = &["error-rate"],
        display_order = 3,
    )]
    pub length: Option<usize>,

    /// Input error rate
    ///
    /// This is used both to generate input sequences with the given induced
    /// error rate, and to choose values for parameters r and k
    #[clap(short, long, display_order = 4, required_unless_all = &["r", "k"])]
    pub error_rate: Option<f32>,

    #[clap(
        long,
        value_enum,
        default_value_t,
        value_name = "MODEL",
        hide_short_help = true
    )]
    pub error_model: ErrorModel,

    /// Seed to initialize RNG for reproducability
    #[clap(long)]
    pub seed: Option<u64>,
}

impl GenerateArgs {
    pub fn to_generate_options(&self) -> GenerateOptions {
        GenerateOptions {
            length: self.length.unwrap(),
            error_rate: self.error_rate.unwrap(),
            error_model: self.error_model,
        }
    }
}

pub struct GenerateOptions {
    pub length: usize,
    pub error_rate: f32,
    pub error_model: ErrorModel,
}

const ALPH: [char; 4] = ['A', 'C', 'G', 'T'];

enum Mutation {
    // Replace char at pos.
    Substitution(usize, u8),
    // Insert char before pos.
    Insertion(usize, u8),
    // Delete char at pos.
    Deletion(usize),
}

fn rand_char(rng: &mut impl Rng) -> u8 {
    ALPH[rng.gen_range(0..4)] as u8
}

fn random_mutation(len_b: usize, rng: &mut impl Rng) -> Mutation {
    // Substitution / insertion / deletion all with equal probability.
    // For length 0 sequences, only generate insertions.
    match if len_b == 0 {
        1
    } else {
        rng.gen_range(0..3usize)
    } {
        0 => Mutation::Substitution(rng.gen_range(0..len_b), rand_char(rng)),
        1 => Mutation::Insertion(rng.gen_range(0..len_b + 1), rand_char(rng)),
        2 => Mutation::Deletion(rng.gen_range(0..len_b)),
        _ => unreachable!(),
    }
}

pub fn generate_pair(opt: &GenerateOptions, rng: &mut impl Rng) -> (Sequence, Sequence) {
    assert!(opt.length > 0, "-n/--length must be specified when generating sequences. Use -i <file> to align pairs in a file.");
    assert!(
        opt.length > 0,
        "-e/--error-rate must be specified when generating sequences."
    );
    let a = (0..opt.length).map(|_| rand_char(rng)).collect_vec();
    let num_mutations = (opt.error_rate * opt.length as f32).ceil() as usize;
    let mut b = ropey::Rope::from_str(std::str::from_utf8(&a).unwrap());
    match opt.error_model {
        ErrorModel::Uniform => {
            for _ in 0..num_mutations {
                let m = random_mutation(b.len_bytes(), rng);
                match m {
                    Mutation::Substitution(i, c) => {
                        b.remove(i..=i);
                        b.insert(i, std::str::from_utf8(&[c]).unwrap());
                    }
                    Mutation::Insertion(i, c) => b.insert(i, std::str::from_utf8(&[c]).unwrap()),
                    Mutation::Deletion(i) => {
                        b.remove(i..=i);
                    }
                }
            }
        }
        ErrorModel::Gap => {
            if rng.gen_bool(0.5) {
                // deletion
                let start = rng.gen_range(0..=b.len_bytes() - num_mutations);
                b.remove(start..start + num_mutations);
            } else {
                // insertion
                let start = rng.gen_range(0..=b.len_bytes());
                let text = (0..num_mutations).map(|_| rand_char(rng)).collect_vec();
                b.insert(start, std::str::from_utf8(&text).unwrap());
            }
        }
        ErrorModel::Move => {
            // deletion
            let start = rng.gen_range(0..=b.len_bytes() - num_mutations);
            let piece = b.slice(start..start + num_mutations).to_string();
            b.remove(start..start + num_mutations);
            // insertion
            let start = rng.gen_range(0..=b.len_bytes());
            b.insert(start, piece.as_str());
        }
    }
    (a, b.to_string().into_bytes())
}

// For quick testing
pub fn setup_with_seed(
    n: usize,
    e: f32,
    seed: u64,
) -> (Sequence, Sequence, Alphabet, SequenceStats) {
    let (a, b) = setup_sequences_with_seed(seed, n, e);

    let alphabet = Alphabet::new(b"ACTG");
    let sequence_stats = SequenceStats {
        len_a: a.len(),
        len_b: b.len(),
        error_rate: e,
        source: Source::Uniform,
    };
    (a, b, alphabet, sequence_stats)
}

pub fn setup_sequences(n: usize, e: f32) -> (Sequence, Sequence) {
    setup_sequences_with_seed(31415, n, e)
}

pub fn setup_sequences_with_seed(seed: u64, n: usize, e: f32) -> (Sequence, Sequence) {
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed as u64);
    let (a, b) = generate_pair(
        &GenerateOptions {
            length: n,
            error_rate: e,
            error_model: ErrorModel::Uniform,
        },
        &mut rng,
    );
    (a, b)
}

pub fn setup(n: usize, e: f32) -> (Sequence, Sequence, Alphabet, SequenceStats) {
    setup_with_seed(n, e, 31415)
}

#[cfg(test)]
mod test {
    use rand::SeedableRng;

    use super::*;

    // Baseline implementation using quadratic implementation.
    fn generate_pair_quadratic(n: usize, e: f32, rng: &mut impl Rng) -> (Sequence, Sequence) {
        let a = (0..n).map(|_| rand_char(rng)).collect_vec();
        let num_mutations = (e * n as f32).ceil() as usize;
        let mut b = a.clone();
        for _ in 0..num_mutations {
            let m = random_mutation(b.len(), rng);
            match m {
                Mutation::Substitution(i, c) => {
                    b[i] = c;
                }
                Mutation::Insertion(i, c) => b.insert(i, c),
                Mutation::Deletion(i) => {
                    b.remove(i);
                }
            }
        }
        (a, b)
    }

    #[test]
    fn test_rope() {
        let mut rng_1 = rand_chacha::ChaCha8Rng::seed_from_u64(1234);
        let mut rng_2 = rand_chacha::ChaCha8Rng::seed_from_u64(1234);

        for n in [10, 100, 1000] {
            for e in [0.01, 0.1, 0.5, 1.0] {
                let p1 = generate_pair(
                    &GenerateOptions {
                        length: n,
                        error_rate: e,
                        error_model: ErrorModel::Uniform,
                    },
                    &mut rng_1,
                );
                let p2 = generate_pair_quadratic(n, e, &mut rng_2);
                assert_eq!(p1, p2);
            }
        }
    }
}
