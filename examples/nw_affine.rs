use std::cmp::max;

use num_traits::abs;
use pairwise_aligner::{
    prelude::{setup, to_string, Sequence},
    ukkonen::ukkonen,
};

fn main() {
    let c = String::from("CG").as_bytes();
    let s1 = Vec::from(String::from("CG").as_bytes());
    let s2 = Vec::from(String::from("TGTC").as_bytes());
    let (ref a, ref b, ref _alphabet, _stats) = setup(500, 0.6);
    print!("s1 == {}\ns2 == {}\n", to_string(&a), to_string(&b));

    let start = std::time::Instant::now();

    let r = pairwise_aligner::nw_affine::nw_affine(a, b);

    let duration = start.elapsed().as_secs_f32();

    println!(
        "Needleman-Wunsch with affine gap penalty with u32 says that edit distance is {}",
        r
    );

    println!(
        "Needleman-Wunsch with affine gap penalty with u32 has needed for this {duration} seconds"
    );

    let mut d = max(2, abs(a.len() as i32 - b.len() as i32) as usize);
    let mut r = d + 1;
    let start = std::time::Instant::now();
    while r > d {
        r = ukkonen(a, b, d);

        println!("d = {} r = {}", d, r);
        d *= 2;
        r *= 2;
    }
    let duration = start.elapsed().as_secs_f32();

    println!("Ukkonen says that edit distance is {}", r / 2);

    println!("Ukkonen has needed for this {duration} seconds");
}
