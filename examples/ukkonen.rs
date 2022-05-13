use core::time;
use std::cmp::{max, min};

use num_traits::abs;
use pairwise_aligner::{
    align,
    diagonal_transition::{
        diagonal_transition, diagonal_transition_a, diagonal_transition_a_oxy,
        diagonal_transition_a_oxy_linear, diagonal_transition_linear,
        diagonal_transition_linear_fast, diagonal_transition_short,
    },
    prelude::{setup, to_string, LengthConfig::Fixed, MatchConfig, Sequence, ZeroCost, CSH, SH},
};

fn ukkonen<'a>(mut s1: &'a Sequence, mut s2: &'a Sequence, d: usize) -> usize {
    /*println!("String1 is {}", to_string(s1));
    println!("String2 is {}", to_string(s2));*/
    let mut len1 = s1.len();
    let mut len2 = s2.len();
    if len1 > len2 {
        (s1, s2) = (s2, s1);
        (len1, len2) = (len2, len1);
    }
    let p: usize = ((d - abs(len2 as i32 - len1 as i32) as usize) / 2) as usize;
    let t: usize = (len2 - len1) + 2 * p;
    if t < 2 {
        return d + 1;
    }
    let mut A = vec![vec![0usize; (t + 1) as usize]; len1 + 1];
    let mut j;

    for i in p..=t {
        A[0][i] = i - p;
    }

    for i in 0..=p {
        A[i][p - i] = i;
    }

    for i in 1..=len1 {
        if i > p {
            if s1[i - 1] == s2[i - p - 1] {
                A[i][0] = A[i - 1][0];
            } else {
                A[i][0] = min(A[i - 1][0], A[i - 1][1]) + 1;
            }
        } else if i == p {
            A[i][0] = A[i - 1][1] + 1;
        }

        for jj in 1..t {
            if i + jj < p + 1 {
                continue;
            }

            if let Some(&c2) = s2.get(i + jj - p - 1) {
                if s1[i - 1] == c2 {
                    A[i][jj] = A[i - 1][jj];
                } else {
                    A[i][jj] = min(min(A[i - 1][jj], A[i - 1][jj + 1]), A[i][jj - 1]) + 1;
                }
            }
        }
        j = t;
        if i + j >= p + 1 && i + j - p - 1 < len2 {
            if s1[i - 1] == s2[i + j - p - 1] {
                A[i][j] = A[i - 1][j];
            } else {
                A[i][j] = min(A[i - 1][j], A[i][j - 1]) + 1;
            }
        } else {
            A[i][j] = min(A[i - 1][j], A[i][j - 1]) + 1;
        }
    }

    /*print!("\n\n");
    for i in 0..(p + 1) {
        print!(" ___ ");
    }
    for i in 0..(len2 - 1) {
        print!("{:5}", s2[i]);
    }
    print!("\n");
    for i in 0..len1 {
        if (i > 0) {
            print!("{:5}", s1[i - 1]);
        } else {
            print!(" ___ ");
        }
        for k in 0..max(0, (i as i32 - 1) as usize) {
            print!(" ___ ");
        }
        for k in 0..t {
            print!("{:5}", A[i][k]);
        }
        print!("\n");
    }*/
    /*print!("\nt is {t}");
    print!("\np is {p}");
    print!("\nlen1 is {len1}\nlen2 is {len2}");
    print!("\n\n");*/

    A[len1][t - p]
}

fn main() {
    let n = 50000;
    let e = 0.05;

    let _m = 0;
    let _k = 3;

    let (ref a, ref b, ref _alphabet, _stats) = setup(n, e);

    /*println!("First string: {}", to_string(a));
    println!("Second string: {}", to_string(b));*/

    let start = std::time::Instant::now();
    let r = align::align(
        a,
        b,
        _alphabet,
        _stats,
        SH {
            match_config: MatchConfig {
                length: Fixed(15),
                max_match_cost: 0,
                ..Default::default()
            },
            pruning: true,
        },
    );

    let duration = start.elapsed().as_secs_f32();

    println!("Ragnar says that edit distance is {}", r.edit_distance);

    println!("Ragnar has needed for this {duration} seconds");

    let start = std::time::Instant::now();

    let r = diagonal_transition_linear(a, b);

    let duration = start.elapsed().as_secs_f32();

    println!("DTM with linear memory says that edit distance is {}", r);

    println!("DTM with linear memory has needed for this {duration} seconds");

    let start = std::time::Instant::now();

    let r = diagonal_transition_linear_fast(a, b);

    let duration = start.elapsed().as_secs_f32();

    println!("DTM with linear memory and some speed-up says that edit distance is {r}");

    println!("DTM with linear memory and some speed-up has needed for this {duration} seconds");

    let start = std::time::Instant::now();

    let r = diagonal_transition_short(a, b);

    let duration = start.elapsed().as_secs_f32();

    println!("DTM with u32 says that edit distance is {}", r);

    println!("DTM with u32 has needed for this {duration} seconds");

    let start = std::time::Instant::now();

    let r = diagonal_transition_a(a, b);

    let duration = start.elapsed().as_secs_f32();

    println!("DTM_oxy says that edit distance is {}", r);

    println!("DTM_oxy has needed for this {duration} seconds");

    let start = std::time::Instant::now();

    let r = diagonal_transition_a_oxy(a, b);

    let duration = start.elapsed().as_secs_f32();

    println!("DTM_oxy2 says that edit distance is {}", r);

    println!("DTM_oxy2 has needed for this {duration} seconds");

    let start = std::time::Instant::now();

    let r = diagonal_transition_a_oxy_linear(a, b);

    let duration = start.elapsed().as_secs_f32();

    println!(
        "DTM_oxy2 with linear memory says that edit distance is {}",
        r
    );

    println!("DTM_oxy2 with linear memory has needed for this {duration} seconds");

    let start = std::time::Instant::now();

    let r = diagonal_transition(a, b);

    let duration = start.elapsed().as_secs_f32();

    println!("DTM says that edit distance is {}", r);

    println!("DTM has needed for this {duration} seconds");

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
