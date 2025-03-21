use std::path::{Path, PathBuf};

use clap::{value_parser, Parser};
use itertools::Itertools;

type SV = Vec<u8>;
pub fn read_chromosomes(cnt_max: usize, path: &Path) -> SV {
    let Ok(mut reader) = needletail::parse_fastx_file(path) else {
        panic!("Did not find human-genome.fa. Add/symlink it to test runtime on it.");
    };
    let mut seq = SV::default();
    let mut cnt = 0;
    while let Some(r) = reader.next() {
        let text = r
            .unwrap()
            .seq()
            .iter()
            .filter_map(|&b| if b == b'N' { None } else { Some(b) })
            .collect::<Vec<_>>();
        seq.extend(text);
        cnt += 1;
        if cnt == cnt_max {
            break;
        }
    }
    seq
}

#[derive(Parser)]
pub struct Cli {
    #[clap(long, value_parser = value_parser!(PathBuf))]
    pub hg: PathBuf,
    // #[clap(long, value_parser = value_parser!(PathBuf))]
    // pub dir: PathBuf,
    #[clap(long)]
    pub count: usize,
    #[clap(long)]
    pub len: usize,
    #[clap(long)]
    pub rate: f32,
}

fn main() {
    let args = Cli::parse();
    let hg = read_chromosomes(1, &args.hg);

    for _ in 0..args.count {
        let start = rand::random_range(0..hg.len() - args.len);
        let end = start + args.len;
        let read = &hg[start..end];
        let read = pa_generate::mutate(read, (args.rate * args.len as f32) as _, &mut rand::rng());
        let read = std::str::from_utf8(&read).unwrap();
        println!(">{} {}", args.len, args.rate);
        println!("{read}");
    }
}
