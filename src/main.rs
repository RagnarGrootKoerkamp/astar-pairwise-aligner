use astar_pairwise_aligner::{
    generate::{generate_pair, GenerateOptions},
    prelude::*,
};
use bio::io::fasta;
use clap::{ArgGroup, Parser};
use itertools::Itertools;
use rand::SeedableRng;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    time::{self, Duration},
};

#[derive(Parser)]
#[clap(help_heading = "INPUT", group = ArgGroup::new("inputmethod").required(true))]
struct Input {
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

#[derive(Parser)]
#[clap(author, about)]
struct Cli {
    #[clap(flatten)]
    input: Input,

    /// Where to write optional statistics.
    #[clap(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Parameters and settings for the algorithm.
    #[clap(flatten, help_heading = "PARAMETERS")]
    params: Params,

    /// Print less. Pass twice for summary line only.
    ///
    /// Do not print a new line per alignment, but instead overwrite the previous one.
    /// Pass twice to only print a summary line and avoid all terminal clutter, e.g. for benchmarking.
    #[clap(short, long, parse(from_occurrences))]
    silent: u8,

    /// Stop aligning new pairs after this timeout.
    #[clap(long, parse(try_from_str = parse_duration::parse))]
    timeout: Option<Duration>,
}

fn main() {
    let mut args = Cli::parse();
    // Hacky, but needed for now.
    args.params.error_rate = args.input.generate.error_rate;

    // Read the input
    let mut avg_result = AlignResult::default();
    let mut run_pair = |mut a: Seq, mut b: Seq| {
        // Shrink if needed.
        if let Some(n) = args.input.generate.length {
            if n != 0 {
                if a.len() > n {
                    a = &a[..n];
                }
                if b.len() > n {
                    b = &b[..n];
                }
            }
        }

        // Run the pair.
        let r = run(&a, &b, &args.params);

        // Record and print stats.
        avg_result.add_sample(&r);
        if args.silent <= 1 {
            print!("\r");
            if args.silent == 0 {
                r.print();
            }
            avg_result.print_no_newline();
        }
    };
    let start = time::Instant::now();
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
                        run_pair(&a, &b);
                        if let Some(d) = args.timeout {
                            if start.elapsed() > d {
                                break 'outer;
                            }
                        }
                    }
                }
                ext if ext == "fna" || ext == "fa" || ext == "fasta" => {
                    for (a, b) in fasta::Reader::new(BufReader::new(File::open(&f).unwrap()))
                        .records()
                        .tuples()
                    {
                        run_pair(a.unwrap().seq(), b.unwrap().seq());
                        if let Some(d) = args.timeout {
                            if start.elapsed() > d {
                                break 'outer;
                            }
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
        let args = &args.input.generate;
        let ref mut rng = if let Some(seed) = args.seed {
            rand_chacha::ChaCha8Rng::seed_from_u64(seed)
        } else {
            rand_chacha::ChaCha8Rng::from_entropy()
        };
        let generate_options = GenerateOptions {
            length: args.length.unwrap(),
            error_rate: args.error_rate.unwrap(),
            error_model: args.error_model,
        };
        for _ in 0..args.cnt {
            let (a, b) = generate_pair(&generate_options, rng);
            run_pair(&a, &b);
        }
    }

    if avg_result.sample_size > 0 {
        print!("\r");
        avg_result.print();

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
    }
}

#[cfg(test)]
mod test {
    use super::Cli;

    #[test]
    fn cli_test() {
        <Cli as clap::CommandFactory>::command().debug_assert();
    }
}
