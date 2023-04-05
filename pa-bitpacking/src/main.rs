use std::time::Instant;

use bio::alphabets::{Alphabet, RankTransform};
use clap::Parser;
use pa_bitpacking::*;

#[derive(Parser)]
pub struct Cli {
    pub id: Option<usize>,
}

fn test(id: usize) {
    let (b, a) = pa_generate::uniform_fixed(4096, 0.1);
    let a = &RankTransform::new(&Alphabet::new(b"ACGT")).transform(a);
    let b = &RankTransform::new(&Alphabet::new(b"ACGT")).transform(b);

    let f = [
        nw_1, nw_2, nw_3, nw_4, nw_5, nw_6, nw_7, nw_8, nw_9, nw_10, nw_11,
    ][id - 1];
    let s = Instant::now();
    for _ in 0..2000 {
        f(a, b);
    }
    eprintln!("{id}: {}", s.elapsed().as_secs_f32());
}

fn main() {
    if let Some(id) = Cli::parse().id {
        test(id);
    } else {
        for id in 1..=8 {
            test(id);
        }
    }
}
