#![feature(derive_default_enum)]
use bio::io::fasta;
use itertools::Itertools;
use pairwise_aligner::prelude::*;
use std::{fs::File, io::BufReader, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Input {
    #[structopt(short, long, parse(from_os_str))]
    input: Option<PathBuf>,

    /// If input is also given, sequences will be cropped to this length.
    /// Otherwise, a pair of sequences of length `n` and relative distance `e` is generated.
    #[structopt(short, required_unless = "input")]
    n: Option<usize>,

    #[structopt(short, default_value = "0.2")]
    e: f32,
}

#[derive(StructOpt)]
#[structopt(
    name = "A* Pairwise Aligner",
    about = "Exact pairwise alignment using A*",
    author = "Ragnar Groot Koerkamp, Pesho Ivanov"
)]
struct Cli {
    #[structopt(flatten)]
    input: Input,

    // Where to write the average bandwith
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(flatten)]
    params: Params,

    // Do not print anything, for benchmarking.
    #[structopt(short, long)]
    silent: bool,
}

fn main() {
    let args = Cli::from_args();

    // Read the input
    let mut sum_band = 0.0;
    let mut cnt = 0;
    if let Some(input) = &args.input.input {
        let files = if input.is_file() {
            vec![input.clone()]
        } else {
            input
                .read_dir()
                .unwrap()
                .map(|x| x.unwrap().path())
                .collect_vec()
        };

        // True: all-vs-all comparison.
        // False: only consecutive pairs.
        let mut all_vs_all = false;
        let mut sequences = Vec::<Sequence>::default();

        for f in files {
            match f.extension().unwrap() {
                ext if ext == "seq" => {
                    let data = std::fs::read(&f).unwrap();
                    for (a, b) in data.split(|c| *c == '\n' as u8).tuples().map(|(a, b)| {
                        assert!(a[0] == '>' as u8);
                        assert!(b[0] == '<' as u8);
                        (a[1..].to_vec(), b[1..].to_vec())
                    }) {
                        sequences.push(a);
                        sequences.push(b);
                    }
                }
                ext if ext == "fna" => {
                    all_vs_all = true;
                    for record in
                        fasta::Reader::new(BufReader::new(File::open(&f).unwrap())).records()
                    {
                        sequences.push(record.unwrap().seq().to_vec());
                    }
                }
                _ => unreachable!("Unknown file extension"),
            };

            for a in &mut sequences {
                if let Some(n) = args.input.n {
                    if a.len() > n {
                        a.resize(n, Default::default());
                    }
                }
            }

            if all_vs_all {
                for ab in sequences.iter().combinations(2) {
                    if let [a, b, ..] = ab[..] {
                        let r = run(&a, &b, &args.params);
                        if !args.silent {
                            r.print();
                        }
                        cnt += 1;
                        sum_band +=
                            r.astar.explored as f32 / max(r.input.len_a, r.input.len_b) as f32;
                    } else {
                        unreachable!("Bad combinations");
                    }
                }
            } else {
                // Consecutive pairs
                for (a, b) in sequences.iter().tuples() {
                    let r = run(&a, &b, &args.params);
                    if !args.silent {
                        r.print();
                    }
                    cnt += 1;
                    sum_band += r.astar.explored as f32 / max(r.input.len_a, r.input.len_b) as f32;
                }
            }
        }
    } else {
        // Generate random input.
        // TODO: Propagate stats.
        let (a, b, _, _) = setup(args.input.n.unwrap(), args.input.e);
        let r = run(&a, &b, &args.params);
        if !args.silent {
            r.print();
            cnt += 1;
            sum_band += r.astar.explored as f32 / max(r.input.len_a, r.input.len_b) as f32;
        }
    }
    if let Some(output) = args.output {
        let avg_band = sum_band / cnt as f32;
        std::fs::write(output, format!("{}\n", avg_band)).unwrap();
    }
}
