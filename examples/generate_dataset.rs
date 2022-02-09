#![feature(derive_default_enum)]
use pairwise_aligner::generate::{generate_pair, GenerateOptions};
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
    #[structopt(short = "x", long, default_value = "1")]
    num_patterns: usize,

    #[structopt(flatten)]
    generate_options: GenerateOptions,
}

fn main() {
    let args = Cli::from_args();

    assert_eq!(args.output.extension().unwrap_or_default(), "seq");

    let mut f = std::fs::File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(args.output)
        .unwrap();
    for _ in 0..args.num_patterns {
        let (a, b) = generate_pair(&args.generate_options, &mut rand::thread_rng());
        f.write(">".as_bytes()).unwrap();
        f.write(&a).unwrap();
        f.write("\n".as_bytes()).unwrap();
        f.write("<".as_bytes()).unwrap();
        f.write(&b).unwrap();
        f.write("\n".as_bytes()).unwrap();
    }
}
