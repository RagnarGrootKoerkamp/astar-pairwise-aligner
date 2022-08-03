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
    /// Takes a region of size e*n/2 and insert it
    Insert,
    /// Takes a region of size e*n/2 and inserts it twice in a row next to
    /// each other
    Doubleinsert,
    /// Construct the sequence of e*n repeating subsequences B and mutates each
    /// of them randomly
    RepeatedPattern,
    /// Construct the sequence of e*n repeating subsequences for sequence A
    /// and adds sequence_length*error_rate mutations for sequence B
    Repeat,
    /// Construct the sequence of e*n repeating subsequences for sequence and adds
    /// sequence_length*error_rate mutations for sequence A, and then adds
    /// sequence_length*error_rate mutations for sequence B
    MutatedRepeat,
    /// Construct the sequence of e*n repeating subsequences for sequence and adds
    /// sequence_length*error_rate mutations for sequences A and B individually
    DoubleMutatedRepeat,
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
    /// The length of b for the case MutatedRepetativePattern3
    #[clap(short, hide_short_help = true)]
    pub m: Option<usize>,

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

    /// The number of sequence pairs to generate
    #[clap(long, default_value_t = 0, hide_short_help = true)]
    pub pattern_length: usize,
}

impl GenerateArgs {
    pub fn to_generate_options(&self) -> GenerateOptions {
        GenerateOptions {
            length: self.length.unwrap(),
            error_rate: self.error_rate.unwrap(),
            error_model: self.error_model,
            pattern_length: self.pattern_length,
            m: self.m,
        }
    }
}

pub struct GenerateOptions {
    pub length: usize,
    pub error_rate: f32,
    pub error_model: ErrorModel,
    pub pattern_length: usize,
    pub m: Option<usize>,
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
    let mut a = (0..opt.length).map(|_| rand_char(rng)).collect_vec();
    let num_mutations = (opt.error_rate * opt.length as f32).ceil() as usize;
    let mut b = ropey::Rope::from_str(std::str::from_utf8(&a).unwrap());
    match opt.error_model {
        ErrorModel::Uniform => {
            for _ in 0..num_mutations {
                make_mutation(&mut b, rng);
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
        ErrorModel::Insert => {
            let start = rng.gen_range(0..b.len_bytes() - num_mutations);
            let piece = b.slice(start..start + num_mutations / 2).to_string();
            b.insert(start, piece.as_str());
        }
        ErrorModel::Doubleinsert => {
            let start = rng.gen_range(0..b.len_bytes() - num_mutations);
            let piece = b.slice(start..start + num_mutations).to_string();
            b.insert(start, piece.as_str());
            b.insert(start + piece.len(), piece.as_str());
        }
        ErrorModel::RepeatedPattern => {
            let pattern = ropey::Rope::from_str(
                std::str::from_utf8(
                    &(0..opt.pattern_length)
                        .map(|_| rand_char(rng))
                        .collect_vec(),
                )
                .unwrap(),
            );
            a = Vec::new();
            let mut bb: Vec<u8> = Vec::new();
            // fill a
            for _ in 0..opt.length / opt.pattern_length {
                let mut mutated_pattern = pattern.clone();
                for _ in 0..(opt.error_rate * opt.pattern_length as f32).ceil() as usize {
                    make_mutation(&mut mutated_pattern, rng);
                }
                a.append(&mut mutated_pattern.to_string().into_bytes());
            }
            // fill b
            for _ in 0..opt.length / opt.pattern_length {
                let mut mutated_pattern = pattern.clone();
                for _ in 0..(opt.error_rate * opt.pattern_length as f32).ceil() as usize {
                    make_mutation(&mut mutated_pattern, rng);
                }
                bb.append(&mut mutated_pattern.to_string().into_bytes());
            }
            b = ropey::Rope::from_str(std::str::from_utf8(&bb).unwrap());
        }
        ErrorModel::Repeat => {
            let pattern = ropey::Rope::from_str(
                std::str::from_utf8(
                    &(0..opt.pattern_length)
                        .map(|_| rand_char(rng))
                        .collect_vec(),
                )
                .unwrap(),
            );
            a = Vec::new();
            // fill a
            for _ in 0..opt.length / opt.pattern_length {
                a.append(&mut pattern.to_string().into_bytes());
            }
            let mut bb = ropey::Rope::from_str(std::str::from_utf8(&a).unwrap());
            for _ in 0..(opt.length as f32 * opt.error_rate) as usize {
                make_mutation(&mut bb, rng);
            }
            b = bb;
        }
        ErrorModel::MutatedRepeat => {
            let pattern = ropey::Rope::from_str(
                std::str::from_utf8(
                    &(0..opt.pattern_length)
                        .map(|_| rand_char(rng))
                        .collect_vec(),
                )
                .unwrap(),
            );
            let mut aa = ropey::Rope::from_str(std::str::from_utf8(&a).unwrap());
            // fill a
            for _ in 0..opt.length / opt.pattern_length {
                a.append(&mut pattern.to_string().into_bytes());
            }
            for _ in 0..(opt.length as f32 * opt.error_rate) as usize {
                make_mutation(&mut aa, rng);
            }
            // fill b
            for _ in 0..(opt.length as f32 * opt.error_rate) as usize {
                make_mutation(&mut b, rng);
            }
            a = aa.to_string().into_bytes();
        }
        ErrorModel::DoubleMutatedRepeat => {
            let pattern = ropey::Rope::from_str(
                std::str::from_utf8(
                    &(0..opt.pattern_length)
                        .map(|_| rand_char(rng))
                        .collect_vec(),
                )
                .unwrap(),
            );
            a = Vec::new();
            // fill a
            for _ in 0..opt.length / opt.pattern_length {
                a.append(&mut pattern.to_string().into_bytes());
            }
            let mut aa = ropey::Rope::from_str(std::str::from_utf8(&a).unwrap());
            for _ in 0..(opt.length as f32 * opt.error_rate / 2 as f32) as usize {
                make_mutation(&mut aa, rng);
            }
            b = ropey::Rope::new();
            // fill b
            for _ in 0..opt.m.unwrap_or(opt.length) / opt.pattern_length {
                b.append(pattern.clone());
            }
            for _ in 0..(opt.m.unwrap_or(opt.length) as f32 * opt.error_rate / 2 as f32) as usize {
                make_mutation(&mut b, rng);
            }
            a = aa.to_string().into_bytes();
        }
    }
    println!("{}\n\n{}\n", to_string(&a), b);
    (a, b.to_string().into_bytes())
}

fn make_mutation(b: &mut ropey::Rope, rng: &mut impl Rng) {
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
            pattern_length: 0,
            m: Some(n),
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
                        pattern_length: 0,
                        m: Some(n),
                    },
                    &mut rng_1,
                );
                let p2 = generate_pair_quadratic(n, e, &mut rng_2);
                assert_eq!(p1, p2);
            }
        }
    }
}
