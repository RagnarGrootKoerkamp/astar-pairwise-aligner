#![feature(derive_default_enum)]
use bio::io::fasta;
use itertools::Itertools;
use pairwise_aligner::{
    generate::{generate_pair, GenerateOptions},
    prelude::*,
};
use rand::thread_rng;
use std::{fs::File, io::BufReader, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Input {
    #[structopt(short, long, parse(from_os_str))]
    input: Option<PathBuf>,

    /// Options to generate an input pair.
    #[structopt(flatten)]
    generate_options: GenerateOptions,
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

    // Where to write the statistics.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    // Where to write a csv of visited states.
    // NOTE: Only works in debug mode.
    // This information is not stored in release mode.
    #[structopt(short, long, parse(from_os_str))]
    visited_states: Option<PathBuf>,

    #[structopt(flatten)]
    params: Params,

    // Do not print a new line per alignment, but instead overwrite the previous one.
    #[structopt(short, long)]
    silent: bool,

    // Only print a summary line, for benchmarking.
    #[structopt(short = "-S", long)]
    silent2: bool,
}

fn main() {
    let args = Cli::from_args();

    // Read the input
    let mut avg_result = AlignResult::default();
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
            match f.extension().expect("Unknown file extension") {
                ext if ext == "seq" => {
                    let data = std::fs::read(&f).unwrap();
                    for (a, b) in data.split(|c| *c == '\n' as u8).tuples().map(|(a, b)| {
                        assert_eq!(a[0], '>' as u8);
                        assert_eq!(b[0], '<' as u8);
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
                let n = args.input.generate_options.length;
                if n != 0 {
                    if a.len() > n {
                        a.resize(n, Default::default());
                    }
                }
            }

            if all_vs_all {
                for ab in sequences.iter().combinations(2) {
                    if let [a, b, ..] = ab[..] {
                        let r = run(&a, &b, &args.params);
                        avg_result.add_sample(&r);
                        if !args.silent2 {
                            print!("\r");
                            if !args.silent {
                                r.print();
                            }
                            avg_result.print_no_newline();
                        }
                    } else {
                        unreachable!("Bad combinations");
                    }
                }
            } else {
                // Consecutive pairs
                for (a, b) in sequences.iter().tuples() {
                    let r = run(&a, &b, &args.params);
                    avg_result.add_sample(&r);
                    if !args.silent2 {
                        print!("\r");
                        if !args.silent {
                            r.print();
                        }
                        avg_result.print_no_newline();
                    }
                }
            }
            if avg_result.sample_size > 0 {
                if !args.silent2 {
                    print!("\r");
                }
                avg_result.print();
            }
        }
    } else {
        // Generate random input.
        // TODO: Propagate stats.
        let (a, b) = generate_pair(&args.input.generate_options, &mut thread_rng());
        let r = run(&a, &b, &args.params);
        avg_result.add_sample(&r);
        if !args.silent2 {
            r.print();
        }
    }

    if avg_result.sample_size > 0 {
        if let Some(output) = args.output {
            let (header, vals) = avg_result.values();

            std::fs::write(
                output,
                format!(
                    "{}\n{}\n",
                    header.iter().map(|x| x.trim()).join("\t"),
                    vals.iter().map(|x| x.trim()).join("\t")
                ),
            )
            .unwrap();
        }

        if let Some(output) = args.visited_states {
            avg_result.write_explored_states(output);
        }
    }
}
