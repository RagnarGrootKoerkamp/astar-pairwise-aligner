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
        let f = [nw_1, nw_2, nw_3, nw_4, nw_5, nw_6, nw_7][id - 1];
        test(f);
    } else {
        if args.lanes.is_none() && args.n.is_none() {
            eprintln!("      \tcopies");
            eprintln!("      \t1\t2\t3\t4");
            eprint!("scalar\t");
            eprint!("{:.2}\t", test(nw_scalar_copies::<1>));
            eprint!("{:.2}\t", test(nw_scalar_copies::<2>));
            eprint!("{:.2}\t", test(nw_scalar_copies::<3>));
            eprint!("{:.2}", test(nw_scalar_copies::<4>));
            eprintln!();
            eprint!("simd 1\t");
            eprint!("{:.2}\t", test(nw_simd_copies::<1, 1>));
            eprint!("{:.2}\t", test(nw_simd_copies::<1, 2>));
            eprint!("{:.2}\t", test(nw_simd_copies::<1, 3>));
            eprint!("{:.2}\t", test(nw_simd_copies::<1, 4>));
            eprintln!();
            eprint!("simd 2\t");
            eprint!("{:.2}\t", test(nw_simd_copies::<2, 1>));
            eprint!("{:.2}\t", test(nw_simd_copies::<2, 2>));
            eprint!("{:.2}\t", test(nw_simd_copies::<2, 3>));
            eprint!("{:.2}\t", test(nw_simd_copies::<2, 4>));
            eprintln!();
            eprint!("simd 4\t");
            eprint!("{:.2}\t", test(nw_simd_copies::<4, 1>));
            eprint!("{:.2}\t", test(nw_simd_copies::<4, 2>));
            eprint!("{:.2}\t", test(nw_simd_copies::<4, 3>));
            eprint!("{:.2}\t", test(nw_simd_copies::<4, 4>));
            eprintln!();
        } else {
            let f = match (args.lanes, args.n) {
                (None, Some(1)) => nw_scalar_copies::<1>,
                (None, Some(2)) => nw_scalar_copies::<2>,
                (None, Some(3)) => nw_scalar_copies::<3>,
                (None, Some(4)) => nw_scalar_copies::<4>,
                (Some(1), Some(1)) => nw_simd_copies::<1, 1>,
                (Some(1), Some(2)) => nw_simd_copies::<1, 2>,
                (Some(1), Some(3)) => nw_simd_copies::<1, 3>,
                (Some(1), Some(4)) => nw_simd_copies::<1, 4>,
                (Some(2), Some(1)) => nw_simd_copies::<2, 1>,
                (Some(2), Some(2)) => nw_simd_copies::<2, 2>,
                (Some(2), Some(3)) => nw_simd_copies::<2, 3>,
                (Some(2), Some(4)) => nw_simd_copies::<2, 4>,
                (Some(4), Some(1)) => nw_simd_copies::<4, 1>,
                (Some(4), Some(2)) => nw_simd_copies::<4, 2>,
                (Some(4), Some(3)) => nw_simd_copies::<4, 3>,
                (Some(4), Some(4)) => nw_simd_copies::<4, 4>,
                _ => unimplemented!(),
            };
            eprintln!("t: {:.2}", test(f));
        }
    }
}
