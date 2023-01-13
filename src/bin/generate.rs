//! Copied from pa-generate for convenience.

use clap::Parser;
use pa_generate::DatasetGenerator;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Location of the output `.seq` file.
    output: PathBuf,

    #[clap(flatten)]
    generate_args: DatasetGenerator,
}

fn main() {
    let args = Cli::parse();
    args.generate_args.generate_file(&args.output);
}
