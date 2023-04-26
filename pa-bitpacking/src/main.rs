#![feature(portable_simd)]
use std::time::Instant;

use bio::alphabets::{Alphabet, RankTransform};
use clap::Parser;
use pa_bitpacking::*;
use pa_generate::{ErrorModel, SeqPairGenerator};
use pa_types::Seq;

#[derive(Parser)]
pub struct Cli {
    pub id: Option<usize>,
    #[arg(short)]
    pub lanes: Option<isize>,
    #[arg(short)]
    pub n: Option<usize>,
}

fn test(f: fn(Seq, Seq) -> i64) -> f32 {
    let (b, a) = SeqPairGenerator {
        length: 4096,
        error_rate: 0.,
        error_model: ErrorModel::Independent,
        pattern_length: None,
    }
    .seeded(31415);

    let a = &RankTransform::new(&Alphabet::new(b"ACGT")).transform(a);
    let b = &RankTransform::new(&Alphabet::new(b"ACGT")).transform(b);
    let s = Instant::now();
    for _ in 0..2000 {
        f(a, b);
    }
    s.elapsed().as_secs_f32()
}

fn main() {
    let args = Cli::parse();
    if let Some(id) = args.id {
        let f = [
            nw_edlib,
            nw_better,
            nw_bool,
            nw_edlib_h,
            nw_better_h,
            nw_bool_h,
        ][id - 1];
        eprintln!("t: {:.2}", test(f));
    } else {
        if args.lanes.is_none() && args.n.is_none() {
            eprintln!("      \tcopies");
            eprintln!("      \t1\t2\t3\t4");
            eprint!("scalar\t");
            eprint!("{:.2}\t", test(nw_scalar::<1>));
            eprint!("{:.2}\t", test(nw_scalar::<2>));
            eprint!("{:.2}\t", test(nw_scalar::<3>));
            eprint!("{:.2}", test(nw_scalar::<4>));
            eprintln!();
            eprint!("simd 4\t");
            eprint!("{:.2}\t", test(nw_simd::<1>));
            eprint!("{:.2}\t", test(nw_simd::<2>));
            eprint!("{:.2}\t", test(nw_simd::<3>));
            eprint!("{:.2}\t", test(nw_simd::<4>));
            eprintln!();
        } else {
            let f = match (args.lanes, args.n) {
                (None, Some(1)) => nw_scalar::<1>,
                (None, Some(2)) => nw_scalar::<2>,
                (None, Some(3)) => nw_scalar::<3>,
                (None, Some(4)) => nw_scalar::<4>,
                (Some(4), Some(1)) => nw_simd::<1>,
                (Some(4), Some(2)) => nw_simd::<2>,
                (Some(4), Some(3)) => nw_simd::<3>,
                (Some(4), Some(4)) => nw_simd::<4>,
                _ => unimplemented!(),
            };
            eprintln!("t: {:.2}", test(f));
        }
    }
}
