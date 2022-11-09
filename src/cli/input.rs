use crate::{
    generate::{generate_pair, GenerateArgs, GenerateOptions},
    prelude::Seq,
};
use bio::io::fasta;
use clap::{ArgGroup, Parser};
use itertools::Itertools;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    ops::ControlFlow,
    path::PathBuf,
};

#[derive(Parser, Deserialize)]
#[clap(help_heading = "INPUT", group = ArgGroup::new("inputmethod").required(true))]
pub struct Input {
    /// The .seq, .txt, or Fasta file with sequence pairs to align.
    #[clap(
        short,
        long,
        parse(from_os_str),
        display_order = 1,
        group = "inputmethod"
    )]
    input: Option<PathBuf>,

    /// Options to generate an input pair.
    #[clap(flatten)]
    generate: GenerateArgs,
}

impl Input {
    pub fn process_input_pairs(&self, mut run_pair: impl FnMut(Seq, Seq) -> ControlFlow<()>) {
        let mut run_cropped_pair = |mut a: Seq, mut b: Seq| -> ControlFlow<()> {
            // Shrink if needed.
            if let Some(n) = self.generate.length && n > 0 {
                if a.len() > n {
                    a = &a[..n];
                }
                if b.len() > n {
                    b = &b[..n];
                }
            }
            run_pair(a, b)
        };

        if let Some(input) = &self.input {
            let files = if input.is_file() {
                vec![input.clone()]
            } else {
                input
                    .read_dir()
                    .unwrap()
                    .map(|x| x.unwrap().path())
                    .collect_vec()
            };

            'outer: for f in files {
                match f.extension().expect("Unknown file extension") {
                    ext if ext == "seq" || ext == "txt" => {
                        let f = std::fs::File::open(&f).unwrap();
                        let f = BufReader::new(f);
                        for (mut a, mut b) in f.lines().map(|l| l.unwrap().into_bytes()).tuples() {
                            if ext == "seq" {
                                assert_eq!(a.remove(0), '>' as u8);
                                assert_eq!(b.remove(0), '<' as u8);
                            }
                            if let ControlFlow::Break(()) = run_cropped_pair(&a, &b) {
                                break 'outer;
                            }
                        }
                    }
                    ext if ext == "fna" || ext == "fa" || ext == "fasta" => {
                        for (a, b) in fasta::Reader::new(BufReader::new(File::open(&f).unwrap()))
                            .records()
                            .tuples()
                        {
                            if let ControlFlow::Break(()) =
                                run_cropped_pair(a.unwrap().seq(), b.unwrap().seq())
                            {
                                break 'outer;
                            }
                        }
                    }
                    ext => {
                        unreachable!(
                            "Unknown file extension {ext:?}. Must be in {{seq,txt,fna,fa,fasta}}."
                        )
                    }
                };
            }
        } else {
            // Generate random input.
            let seed = self.generate.seed.unwrap_or_else(|| {
                let seed = ChaCha8Rng::from_entropy().gen_range(0..u64::MAX);
                eprintln!("Seed: {seed}");
                seed
            });
            let ref mut rng = ChaCha8Rng::seed_from_u64(seed);
            let generate_options = GenerateOptions {
                length: self.generate.length.unwrap(),
                error_rate: self.generate.error_rate.unwrap(),
                error_model: self.generate.error_model,
                pattern_length: self.generate.pattern_length,
                m: self.generate.m,
            };
            for _ in 0..self.generate.cnt {
                let (a, b) = generate_pair(&generate_options, rng);
                if let ControlFlow::Break(()) = run_pair(&a, &b) {
                    break;
                }
            }
        }
    }
}
