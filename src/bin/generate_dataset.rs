use astar_pairwise_aligner::generate::{generate_pair, GenerateArgs};
use clap::Parser;
use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

#[derive(Parser)]
#[clap(next_line_help = false)]
struct Cli {
    /// Location of the output file
    #[clap(parse(from_os_str))]
    output: PathBuf,

    #[clap(flatten)]
    generate_args: GenerateArgs,
}

fn main() {
    let args = Cli::from_args();

    assert_eq!(args.output.extension().unwrap_or_default(), "seq");

    let mut f = BufWriter::new(
        std::fs::File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(args.output)
            .unwrap(),
    );
    for _ in 0..args.generate_args.cnt {
        let (a, b) = generate_pair(
            &args.generate_args.to_generate_options(),
            &mut rand::thread_rng(),
        );
        f.write_all(">".as_bytes()).unwrap();
        f.write_all(&a).unwrap();
        f.write_all("\n".as_bytes()).unwrap();
        f.write_all("<".as_bytes()).unwrap();
        f.write_all(&b).unwrap();
        f.write_all("\n".as_bytes()).unwrap();
    }
}
