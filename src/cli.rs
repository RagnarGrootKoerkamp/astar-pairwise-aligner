use crate::{prelude::Seq, HeuristicArgs};
use bio::io::fasta;
use clap::{value_parser, Parser};
use itertools::Itertools;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    ops::ControlFlow,
    path::PathBuf,
    time::Duration,
};

#[derive(Parser, Serialize, Deserialize)]
#[clap(author, about)]
pub struct Cli {
    #[clap(flatten)]
    pub input: Input,

    /// Where to write optional statistics.
    #[arg(short, long, value_parser = value_parser!(PathBuf))]
    pub output: Option<PathBuf>,

    /// Use diagonal-transition based A*.
    #[clap(long, hide_short_help = true)]
    pub dt: bool,

    /// Parameters and settings for the heuristic.
    #[clap(flatten)]
    pub heuristic: HeuristicArgs,

    /// Print less. Pass twice for summary line only.
    ///
    /// Do not print a new line per alignment, but instead overwrite the previous one.
    /// Pass twice to only print a summary line and avoid all terminal clutter, e.g. for benchmarking.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub silent: u8,

    /// Stop aligning new pairs after this timeout.
    #[arg(long, value_parser = parse_duration::parse, hide_short_help = true)]
    pub timeout: Option<Duration>,
}

#[derive(Parser, Serialize, Deserialize)]
#[clap(next_help_heading = "Input")]
pub struct Input {
    /// The .seq, .txt, or Fasta file with sequence pairs to align.
    #[clap(short, long, value_parser = value_parser!(PathBuf), display_order = 1)]
    pub input: Option<PathBuf>,

    /// Options to generate an input pair.
    #[clap(flatten)]
    pub generate: pa_generate::DatasetGenerator,
}

impl Input {
    /// Call the given function for each pair in the input.
    pub fn process_input_pairs(&self, mut run_pair: impl FnMut(Seq, Seq) -> ControlFlow<()>) {
        let mut run_cropped_pair = |mut a: Seq, mut b: Seq| -> ControlFlow<()> {
            // Shrink if needed.
            let n = self.generate.settings.length;
            if n > 0 {
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
            for _ in 0..self.generate.cnt.unwrap() {
                let (a, b) = self.generate.settings.generate(rng);
                if let ControlFlow::Break(()) = run_pair(&a, &b) {
                    break;
                }
            }
        }
    }
}
