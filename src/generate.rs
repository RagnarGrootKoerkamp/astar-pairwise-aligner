use itertools::Itertools;
use rand::Rng;
use structopt::StructOpt;
use strum_macros::EnumString;

#[derive(EnumString, Default, Debug, Clone, Copy)]
#[strum(ascii_case_insensitive)]
pub enum ErrorModel {
    #[default]
    Uniform,
    /// Make a single gap (insertion or deletion) of size e*n.
    Gap,
    /// Delete a region of size e*n and insert a region of size e*n.
    Move,
}

#[derive(StructOpt)]
pub struct GenerateOptions {
    // Length of the sequences to generate.
    #[structopt(short = "n", long, required_unless = "input", default_value = "0")]
    pub length: usize,

    // Induced error rate.
    #[structopt(short = "e", long, default_value = "0.1")]
    pub error: f32,

    #[structopt(long, default_value = "Uniform")]
    pub model: ErrorModel,
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

pub fn generate_pair(opt: &GenerateOptions, rng: &mut impl Rng) -> (Vec<u8>, Vec<u8>) {
    let a = (0..opt.length).map(|_| rand_char(rng)).collect_vec();
    let num_mutations = (opt.error * opt.length as f32).ceil() as usize;
    let mut b = ropey::Rope::from_str(std::str::from_utf8(&a).unwrap());
    match opt.model {
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

#[cfg(test)]
mod test {
    use rand::SeedableRng;

    use super::*;

    // Baseline implementation using quadratic implementation.
    fn generate_pair_quadratic(n: usize, e: f32, rng: &mut impl Rng) -> (Vec<u8>, Vec<u8>) {
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
                        error: e,
                        model: ErrorModel::Uniform,
                    },
                    &mut rng_1,
                );
                let p2 = generate_pair_quadratic(n, e, &mut rng_2);
                assert_eq!(p1, p2);
            }
        }
    }
}
