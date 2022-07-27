use astar_pairwise_aligner::{
    generate::{generate_pair, GenerateOptions},
    prelude::*,
};
use bio::io::fasta;
use itertools::Itertools;
use rand::SeedableRng;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    time::{self, Duration},
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Input {
    #[structopt(short, long, parse(from_os_str))]
    input: Option<PathBuf>,

    /// Length of the sequences to generate.
    #[structopt(short = "x", long, default_value = "1")]
    pub cnt: usize,

    /// Seed to initialize RNG.
    #[structopt(long)]
    pub seed: Option<u64>,

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

    #[structopt(flatten)]
    params: Params,

    // Do not print a new line per alignment, but instead overwrite the previous one.
    #[structopt(short, long)]
    silent: bool,

    // Only print a summary line, for benchmarking.
    #[structopt(short = "S", long)]
    silent2: bool,

    /// Maximum duration to run for.
    #[structopt(long, parse(try_from_str = parse_duration::parse))]
    timeout: Option<Duration>,
}

fn main() {
    let args = Cli::from_args();

    // Read the input
    let mut avg_result = AlignResult::default();
    let mut run_pair = |mut a: Seq, mut b: Seq| {
        // Shrink if needed.
        let n = args.input.generate_options.length;
        if n != 0 {
            if a.len() > n {
                a = &a[..n];
            }
            if b.len() > n {
                b = &b[..n];
            }
        }
        // Run the pair.
        let r = run(&a, &b, &args.params);

        // Record and print stats.
        avg_result.add_sample(&r);
        if !args.silent2 {
            print!("\r");
            if !args.silent {
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
                ext if ext == "fna" || ext == "fa" => {
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
                _ => unreachable!("Unknown file extension"),
            };
        }
    } else {
        // Generate random input.
        let ref mut rng = if let Some(seed) = args.input.seed {
            rand_chacha::ChaCha8Rng::seed_from_u64(seed)
        } else {
            rand_chacha::ChaCha8Rng::from_entropy()
        };
        for _ in 0..args.input.cnt {
            let (a, b) = generate_pair(&args.input.generate_options, rng);
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
