#![feature(derive_default_enum)]
use itertools::Itertools;
use pairwise_aligner::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Input {
    #[structopt(short, long, parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short, conflicts_with = "input", required_unless = "input")]
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

        for f in files {
            let data = std::fs::read(&f).unwrap();
            let pairs = data
                .split(|c| *c == '\n' as u8)
                .tuples()
                .map(|(a, b)| {
                    assert!(a[0] == '>' as u8);
                    assert!(b[0] == '<' as u8);
                    (a[1..].to_vec(), b[1..].to_vec())
                })
                .collect_vec();

            for (a, b) in pairs {
                let r = run(&a, &b, &args.params);
                if !args.silent {
                    r.print();
                }
                cnt += 1;
                sum_band += r.astar.explored as f32 / max(r.input.len_a, r.input.len_b) as f32;
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
