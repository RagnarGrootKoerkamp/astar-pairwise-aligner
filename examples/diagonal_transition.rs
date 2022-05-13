use std::cmp::max;

use pairwise_aligner::{diagonal_transition::diagonal_transition, prelude::*};

fn main() {
    let n = 100000;
    let e = 0.01;

    let _m = 0;
    let _k = 3;

    let (ref a, ref b, ref _alphabet, _stats) = setup(n, e);

    let start = std::time::Instant::now();

    let r = diagonal_transition(a, b);

    let duration = start.elapsed().as_secs_f32();

    println!("DTM says that edit distance is {}", r);

    println!("DTM has needed for this {duration} seconds");
}
