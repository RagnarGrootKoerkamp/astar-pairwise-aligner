#![feature(derive_default_enum)]
use itertools::Itertools;
use rand::Rng;
use std::{io::Write, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "Generate Dataset",
    about = "Generate pairs of sequences with given edit distance. Mirror of WFA/generate_dataset.",
    author = "Ragnar Groot Koerkamp, Pesho Ivanov"
)]
struct Cli {
    // Where to write the file.
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    // Number of patterns (pairs of sequences) to generate.
    #[structopt(short, long)]
    num_patterns: usize,

    // Length of the sequences to generate.
    #[structopt(short, long)]
    length: usize,

    // Induced error rate.
    #[structopt(short, long)]
    error: f32,
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

fn generate_pair(n: usize, e: f32, rng: &mut impl Rng) -> (Vec<u8>, Vec<u8>) {
    let a = (0..n).map(|_| rand_char(rng)).collect_vec();
    let num_mutations = (e * n as f32).ceil() as usize;
    let mut b = ropey::Rope::from_str(std::str::from_utf8(&a).unwrap());
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
    (a, b.to_string().into_bytes())
}

fn main() {
    let args = Cli::from_args();

    let mut f = std::fs::File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(args.output)
        .unwrap();
    for _ in 0..args.num_patterns {
        let (a, b) = {
            let n = args.length;
            let e = args.error;
            generate_pair(n, e, &mut rand::thread_rng())
        };
        f.write(">".as_bytes()).unwrap();
        f.write(&a).unwrap();
        f.write("\n".as_bytes()).unwrap();
        f.write("<".as_bytes()).unwrap();
        f.write(&b).unwrap();
        f.write("\n".as_bytes()).unwrap();
    }
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
                let p1 = generate_pair(n, e, &mut rng_1);
                let p2 = generate_pair_quadratic(n, e, &mut rng_2);
                assert_eq!(p1, p2);
            }
        }
    }
}
