//! Conclusions:
//! - bm is much larger than am and b, so never even consider matching a with bm.
//! - am is 2.5x larger than b, so build a datastructure over b
#![feature(test)]
#![cfg(test)]
use pa_heuristic::matches::{
    inexact::{find_matches_qgram_hash_inexact, Mutations},
    *,
};
use pa_types::*;
use rustc_hash::FxHashMap as HashMap;

#[macro_use]
extern crate lazy_static;

extern crate test;

use bio::alphabets::{Alphabet, RankTransform};
use pa_generate::uniform_fixed;
use test::Bencher;

const E: f32 = 0.20;
const K: I = 10;

lazy_static! {
    static ref TRANSFORM: RankTransform = RankTransform::new(&Alphabet::new(b"ACGT"));
}

fn mutations(k: I, kmer: usize, dels: bool, subs: bool, ins: bool, dedup: bool) -> Mutations {
    // This assumes the alphabet size is 4.
    let mut deletions = if dels {
        Vec::with_capacity(k as usize)
    } else {
        Vec::default()
    };
    let mut substitutions = if subs {
        Vec::with_capacity(4 * k as usize)
    } else {
        Vec::default()
    };
    let mut insertions = if ins {
        Vec::with_capacity(4 * (k + 1) as usize)
    } else {
        Vec::default()
    };
    // Substitutions
    if subs {
        for i in 0..k {
            let mask = !(3 << (2 * i));
            for s in 0..4 {
                // TODO: Skip the identity substitution.
                substitutions.push((kmer & mask) | s << (2 * i));
            }
        }
    }
    if ins {
        // Insertions
        // TODO: Test that excluding insertions at the start and end doesn't matter.
        // NOTE: Apparently skipping insertions at the start is fine, but skipping at the end is not.
        for i in 0..=k {
            let mask = (1 << (2 * i)) - 1;
            for s in 0..4 {
                insertions.push((kmer & mask) | (s << (2 * i)) | ((kmer & !mask) << 2));
            }
        }
    }
    if dels {
        // Deletions
        for i in 0..=k - 1 {
            let mask = (1 << (2 * i)) - 1;
            deletions.push((kmer & mask) | ((kmer & (!mask << 2)) >> 2));
        }
    }
    if dedup {
        for v in [&mut deletions, &mut substitutions, &mut insertions] {
            // TODO: This sorting is slow; maybe we can work around it.
            v.sort_unstable();
            v.dedup();
        }
        // Remove original
        substitutions.retain(|&x| x != kmer);
    }
    Mutations {
        deletions,
        substitutions,
        insertions,
    }
}

fn lookup_b_in_am_hashmap(a: Seq, b: Seq, k: I) {
    let k = k as u32;
    assert!(k <= 14);
    let key = |l: u32, w: usize| ((w as u32) << 2) + (l + 1 - k);
    // TODO: Split in 3 hashmaps?
    let mut m = HashMap::<u32, (u32, u32)>::default();
    m.reserve(a.len() * 10 * k as usize);
    for (i, w) in TRANSFORM.qgrams(k, a).step_by(k as usize).enumerate() {
        *m.entry(key(k, w)).or_default() = (i as u32, k);
        let ms = mutations(k as _, w, true, true, true, false);
        for w in ms.deletions {
            m.insert(key(k - 1, w), (i as u32, k));
        }
        for w in ms.substitutions {
            m.insert(key(k, w), (i as u32, k));
        }
        for w in ms.insertions {
            m.insert(key(k + 1, w), (i as u32, k));
        }
    }
    let mut cnt = 0;
    for (_j, w) in TRANSFORM.qgrams(k - 1, b).enumerate() {
        if m.contains_key(&key(k - 1, w)) {
            cnt += 1;
        }
    }
    for (_j, w) in TRANSFORM.qgrams(k, b).enumerate() {
        if m.contains_key(&key(k, w)) {
            cnt += 1;
        }
    }
    for (_j, w) in TRANSFORM.qgrams(k + 1, b).enumerate() {
        if m.contains_key(&key(k + 1, w)) {
            cnt += 1;
        }
    }
    println!("{} {cnt}", m.len());
}

fn lookup_am_in_b_hashmap(a: Seq, b: Seq, k: I) {
    let k = k as u32;
    assert!(k <= 14);
    let key = |l: u32, w: usize| ((w as u32) << 2) + (l + 1 - k);
    let mut m = HashMap::<u32, (u32, u32)>::default();
    m.reserve(3 * b.len());
    for (j, w) in TRANSFORM.qgrams(k - 1, b).enumerate() {
        *m.entry(key(k - 1, w)).or_default() = (j as u32, k);
    }
    for (j, w) in TRANSFORM.qgrams(k, b).enumerate() {
        *m.entry(key(k, w)).or_default() = (j as u32, k);
    }
    for (j, w) in TRANSFORM.qgrams(k + 1, b).enumerate() {
        *m.entry(key(k + 1, w)).or_default() = (j as u32, k);
    }
    let mut cnt = 0;
    for (_i, w) in TRANSFORM.qgrams(k, a).step_by(k as usize).enumerate() {
        if m.contains_key(&key(k, w)) {
            cnt += 1;
        }
        let ms = mutations(k as _, w, true, true, true, false);
        for w in ms.deletions {
            if m.contains_key(&key(k - 1, w)) {
                cnt += 1;
            }
        }
        for w in ms.substitutions {
            if m.contains_key(&key(k, w)) {
                cnt += 1;
            }
        }
        for w in ms.insertions {
            if m.contains_key(&key(k + 1, w)) {
                cnt += 1;
            }
        }
    }
    println!("{} {cnt}", m.len());
}

fn lookup_am_in_b_hashmap_dedup(a: Seq, b: Seq, k: I) {
    let k = k as u32;
    assert!(k <= 14);
    let key = |l: u32, w: usize| ((w as u32) << 2) + (l + 1 - k);
    let mut m = HashMap::<u32, (u32, u32)>::default();
    m.reserve(3 * b.len());
    for (j, w) in TRANSFORM.qgrams(k - 1, b).enumerate() {
        *m.entry(key(k - 1, w)).or_default() = (j as u32, k);
    }
    for (j, w) in TRANSFORM.qgrams(k, b).enumerate() {
        *m.entry(key(k, w)).or_default() = (j as u32, k);
    }
    for (j, w) in TRANSFORM.qgrams(k + 1, b).enumerate() {
        *m.entry(key(k + 1, w)).or_default() = (j as u32, k);
    }
    let mut cnt = 0;
    for (_i, w) in TRANSFORM.qgrams(k, a).step_by(k as usize).enumerate() {
        if m.contains_key(&key(k, w)) {
            cnt += 1;
        }
        let ms = mutations(k as _, w, true, true, true, true);
        for w in ms.deletions {
            if m.contains_key(&key(k - 1, w)) {
                cnt += 1;
            }
        }
        for w in ms.substitutions {
            if m.contains_key(&key(k, w)) {
                cnt += 1;
            }
        }
        for w in ms.insertions {
            if m.contains_key(&key(k + 1, w)) {
                cnt += 1;
            }
        }
    }
    println!("{} {cnt}", m.len());
}

fn lookup_a_in_bm_hashmap(a: Seq, b: Seq, k: I) -> usize {
    let k = k as u32;
    let key = |_k: u32, w: usize| w as u32;
    let mut m = HashMap::<u32, (u32, u32)>::default();
    m.reserve(b.len() * 10 as usize);
    for (j, w) in TRANSFORM.qgrams(k - 1, b).enumerate() {
        let ms = mutations(k as _, w, false, false, true, false);
        for w in ms.insertions {
            m.insert(key(k, w), (j as u32, k));
        }
    }
    for (j, w) in TRANSFORM.qgrams(k, b).enumerate() {
        *m.entry(key(k, w)).or_default() = (j as u32, k);
        let ms = mutations(k as _, w, false, true, false, false);
        for w in ms.substitutions {
            m.insert(key(k, w), (j as u32, k));
        }
    }
    for (j, w) in TRANSFORM.qgrams(k + 1, b).enumerate() {
        let ms = mutations(k as _, w, true, false, false, false);
        for w in ms.deletions {
            m.insert(key(k, w), (j as u32, k));
        }
    }
    let mut cnt = 0;
    for (_i, w) in TRANSFORM.qgrams(k, a).step_by(k as usize).enumerate() {
        if m.contains_key(&key(k, w)) {
            cnt += 1;
        }
    }
    cnt
}

fn lookup_bm_in_a_hashmap(a: Seq, b: Seq, k: I) -> usize {
    let k = k as u32;
    let key = |k: u32, w: usize| ((w as u32) << 8) + k;
    let mut m = HashMap::<u32, (u32, u32)>::default();
    m.reserve(a.len());
    for (j, w) in TRANSFORM.qgrams(k, b).enumerate() {
        *m.entry(key(k, w)).or_default() = (j as u32, k);
    }
    let mut cnt = 0;
    for (_j, w) in TRANSFORM.qgrams(k - 1, b).enumerate() {
        let ms = mutations(k as _, w, false, false, true, false);
        for w in ms.insertions {
            if m.contains_key(&key(k, w)) {
                cnt += 1;
            }
        }
    }
    for (_j, w) in TRANSFORM.qgrams(k, b).enumerate() {
        if m.contains_key(&key(k, w)) {
            cnt += 1;
        }
        let ms = mutations(k as _, w, false, true, false, false);
        for w in ms.substitutions {
            if m.contains_key(&key(k, w)) {
                cnt += 1;
            }
        }
    }
    for (_j, w) in TRANSFORM.qgrams(k + 1, b).enumerate() {
        let ms = mutations(k as _, w, true, false, false, false);
        for w in ms.deletions {
            if m.contains_key(&key(k, w)) {
                cnt += 1;
            }
        }
    }
    cnt
}

// #[bench]
// fn n100_inexact_qgramindex(bench: &mut Bencher) {
//     let n = 100;
//     let (a, b) = setup(n, E);
//     bench.iter(|| {
//         find_matches_qgramindex(
//             &a,
//             &b,
//             MatchConfig {
//                 length: Fixed(K),
//                 r: 2,
//                 ..Default::default()
//             },
//         )
//     });
// }

// #[bench]
// fn n10000_inexact_qgramindex(bench: &mut Bencher) {
//     let n = 10000;
//     let (a, b) = setup(n, E);
//     bench.iter(|| {
//         find_matches_qgramindex(
//             &a,
//             &b,
//             MatchConfig {
//                 length: Fixed(K),
//                 r: 2,
//                 ..Default::default()
//             },
//         )
//     });
// }

#[bench]
fn n10000_inexact_hash(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| find_matches_qgram_hash_inexact(&a, &b, MatchConfig::inexact(K), false));
}

#[bench]
fn n100000_inexact_hash(bench: &mut Bencher) {
    let n = 100000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| find_matches_qgram_hash_inexact(&a, &b, MatchConfig::inexact(K), false));
}

#[bench]
fn n100_am_in_b(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| lookup_am_in_b_hashmap(&a, &b, K));
}

#[bench]
fn n10000_am_in_b(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| lookup_am_in_b_hashmap(&a, &b, K));
}

#[bench]
fn n100000_am_in_b(bench: &mut Bencher) {
    let n = 100000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| lookup_am_in_b_hashmap(&a, &b, K));
}

#[bench]
fn n100_am_in_b_dedup(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| lookup_am_in_b_hashmap_dedup(&a, &b, K));
}

#[bench]
fn n10000_am_in_b_dedup(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| lookup_am_in_b_hashmap_dedup(&a, &b, K));
}

#[bench]
fn n100000_am_in_b_dedup(bench: &mut Bencher) {
    let n = 100000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| lookup_am_in_b_hashmap_dedup(&a, &b, K));
}

#[bench]
fn n100_b_in_am(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| lookup_b_in_am_hashmap(&a, &b, K));
}

#[bench]
fn n10000_b_in_am(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| lookup_b_in_am_hashmap(&a, &b, K));
}

// #[bench]
// fn n100000_b_in_am(bench: &mut Bencher) {
//     let n = 100000;
//     let (a, b) = setup(n, E);
//     bench.iter(|| lookup_b_in_am_hashmap(&a, &b, K));
// }

#[bench]
fn n100_a_in_bm(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| lookup_a_in_bm_hashmap(&a, &b, K));
}

#[bench]
fn n10000_a_in_bm(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| lookup_a_in_bm_hashmap(&a, &b, K));
}

// #[bench]
// fn n100000_a_in_bm(bench: &mut Bencher) {
//     let n = 100000;
//     let (a, b) = setup(n, E);
//     bench.iter(|| lookup_a_in_bm_hashmap(&a, &b, K));
// }

#[bench]
fn n100_bm_in_a(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| lookup_bm_in_a_hashmap(&a, &b, K));
}

#[bench]
fn n10000_bm_in_a(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| lookup_bm_in_a_hashmap(&a, &b, K));
}

// #[bench]
// fn n100000_bm_in_a(bench: &mut Bencher) {
//     let n = 100000;
//     let (a, b) = setup(n, E);
//     bench.iter(|| lookup_bm_in_a_hashmap(&a, &b, K));
// }
