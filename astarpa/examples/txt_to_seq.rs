use clap::Parser;
use itertools::Itertools;
use std::{io::Write, path::PathBuf};

#[derive(Parser)]
#[clap(
    name = "Txt to .seq",
    about = "Convert a plaint text file containing pairs of sequences on consecutive lines to a .seq file with > and < prefixes.",
    author = "Ragnar Groot Koerkamp, Pesho Ivanov"
)]
struct Cli {
    #[clap(value_parser = clap::value_parser!(PathBuf))]
    file: PathBuf,
}

fn main() {
    let args = Cli::parse();

    assert_eq!(args.file.extension().unwrap_or_default(), "txt");
    let mut out_path = args.file.clone();
    out_path.set_extension("seq");

    let mut f = std::fs::File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_path)
        .unwrap();

    let data = std::fs::read(&args.file).unwrap();

    for (a, b) in data.split(|c| *c == '\n' as u8).tuples() {
        f.write(">".as_bytes()).unwrap();
        f.write(a).unwrap();
        f.write("\n".as_bytes()).unwrap();
        f.write("<".as_bytes()).unwrap();
        f.write(b).unwrap();
        f.write("\n".as_bytes()).unwrap();
    }
}
