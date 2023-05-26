#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(test)]
#![cfg(test)]
use lazy_static::*;
use pa_generate::uniform_fixed;
use pa_heuristic::matches::{
    exact::{find_matches_qgramindex, hash_a},
    MatchConfig,
};
use pa_types::*;
use rustc_hash::FxHashMap as HashMap;

extern crate test;

use test::Bencher;

mod matches {

    use super::*;
    use aho_corasick::AhoCorasickBuilder;
    use bio::{
        alphabets::{Alphabet, RankTransform},
        data_structures::{qgram_index::QGramIndex, suffix_array::suffix_array},
    };
    use itertools::Itertools;
    use suffix::SuffixTable;

    lazy_static! {
        static ref TRANSFORM: RankTransform = RankTransform::new(&Alphabet::new(b"ACGT"));
    }

    /// Some options for finding all matches between a[i*k, (i+1)*k) and b:
    /// - qgramindex on b, query all seeds
    /// - qgramindex on a, query all b
    /// - suffixarray on b, sort/suffixarray a, sliding window
    /// - suffixarray on b, query all a in log time each
    /// - sort all a, query all b in log time each
    /// - suffix automaton on full a, do incremental queries from b using back edges
    /// - trie on b/a, query a/b.
    /// - suffixarray on b, then build trie from sorted input and query all of a.
    pub fn exact_matches(a: Seq, b: Seq, k: I) {}
    pub fn sort_seeds(a: Seq, b: Seq, k: I) {
        let mut seeds = a.chunks_exact(k as usize).collect_vec();
        seeds.sort_unstable();
    }
    pub fn suffix_array_sort(a: Seq, b: Seq, k: I) {
        let mut suffixes = b.windows(k as usize).collect_vec();
        suffixes.sort_unstable();
    }
    pub fn suffix_array_bio(a: Seq, b: Seq, k: I) {
        let mut b = b.to_vec();
        b.push('$' as u8);
        suffix_array(&b);
    }
    pub fn suffix_array_suffixtable(a: Seq, b: Seq, k: I) {
        let st = SuffixTable::new(seq_to_string(b));
    }

    pub fn aho_corasick(a: Seq, b: Seq, k: I) {
        let ac = AhoCorasickBuilder::new()
            .build_with_size::<u16, _, _>(a.chunks_exact(k as usize))
            .unwrap();
        let cnt = ac.find_overlapping_iter(b).count();
        println!("{cnt}");
    }

    pub fn seed_qgrams(a: Seq, b: Seq, k: I) {
        todo!("Manual implementation needed");
    }
    pub fn suffix_qgrams(a: Seq, b: Seq, k: I) {
        QGramIndex::new(k as u32, b, &Alphabet::new(b"ACGT"));
    }

    pub fn seed_hashmap(a: Seq, b: Seq, k: I) {
        let mut m = HashMap::<&[u8], u32>::default();
        m.reserve(a.len());
        for (i, w) in a.chunks_exact(k as usize).enumerate() {
            *m.entry(w).or_default() = i as u32
        }
    }
    pub fn suffix_hashmap(a: Seq, b: Seq, k: I) {
        let mut m = HashMap::<&[u8], u32>::default();
        m.reserve(b.len());
        for (i, w) in b.windows(k as usize).enumerate() {
            *m.entry(w).or_default() = i as u32
        }
    }

    pub fn seed_hashmap_qgrams(a: Seq, b: Seq, k: I) {
        let mut m = HashMap::<u32, u32>::default();
        m.reserve(a.len());
        for (i, w) in TRANSFORM.qgrams(k as _, a).step_by(k as usize).enumerate() {
            *m.entry(w as u32).or_default() = i as u32;
        }
    }
    pub fn suffix_hashmap_qgrams(a: Seq, b: Seq, k: I) {
        let mut m = HashMap::<u32, u32>::default();
        m.reserve(b.len());
        for (i, w) in TRANSFORM.qgrams(k as _, b).enumerate() {
            *m.entry(w as u32).or_default() = i as u32;
        }
    }

    pub fn lookup_seeds_in_qgram_hashmap(a: Seq, b: Seq, k: I) {
        let mut m = HashMap::<u32, u32>::default();
        m.reserve(b.len());
        for (i, w) in TRANSFORM.qgrams(k as _, b).enumerate() {
            *m.entry(w as u32).or_default() = i as u32;
        }
        let mut cnt = 0;
        for (j, w) in TRANSFORM.qgrams(k as _, a).step_by(k as usize).enumerate() {
            if m.contains_key(&(w as u32)) {
                cnt += 1;
            }
        }
    }

    pub fn lookup_suffixes_in_qgram_hashmap(a: Seq, b: Seq, k: I) {
        let mut m = HashMap::<u32, u32>::default();
        m.reserve(a.len());
        for (i, w) in TRANSFORM.qgrams(k as _, a).step_by(k as usize).enumerate() {
            *m.entry(w as u32).or_default() = i as u32;
        }
        let mut cnt = 0;
        for (j, w) in TRANSFORM.qgrams(k as _, b).enumerate() {
            if m.contains_key(&(w as u32)) {
                cnt += 1;
            }
        }
    }
}

const N: usize = 100000;
const E: f32 = 0.02;
const K: I = 12;

#[bench]
fn exact_qgramindex(bench: &mut Bencher) {
    let (a, b) = uniform_fixed(N, E);
    bench.iter(|| find_matches_qgramindex(&a, &b, MatchConfig::exact(K), false));
}

#[bench]
fn exact_hash(bench: &mut Bencher) {
    let (a, b) = uniform_fixed(N, E);
    bench.iter(|| hash_a(&a, &b, MatchConfig::exact(K), false));
}

#[bench]
fn aho_corasick(bench: &mut Bencher) {
    let (a, b) = uniform_fixed(N, E);
    bench.iter(|| {
        matches::aho_corasick(&a, &b, K);
    });
}

#[bench]
fn lookup_seeds_in_qgram_hashmap(bench: &mut Bencher) {
    let (a, b) = uniform_fixed(N, E);
    bench.iter(|| {
        matches::lookup_seeds_in_qgram_hashmap(&a, &b, K);
    });
}

#[bench]
fn lookup_suffixes_in_qgram_hashmap(bench: &mut Bencher) {
    let (a, b) = uniform_fixed(N, E);
    bench.iter(|| {
        matches::lookup_suffixes_in_qgram_hashmap(&a, &b, K);
    });
}
#[bench]
fn b_suffix_array(bench: &mut Bencher) {
    let (a, b) = uniform_fixed(N, 0.0);
    bench.iter(|| {
        matches::suffix_array_bio(&a, &b, 0);
    });
}

#[bench]
fn b_suffix_array_2(bench: &mut Bencher) {
    let (a, b) = uniform_fixed(N, 0.0);
    bench.iter(|| {
        matches::suffix_array_suffixtable(&a, &b, 0);
    });
}

#[bench]
fn b_suffix_array_sort(bench: &mut Bencher) {
    let e = 0.01;
    let (a, b) = uniform_fixed(N, e);
    bench.iter(|| {
        matches::suffix_array_sort(&a, &b, K);
    });
}

#[bench]
fn a_sort_seeds(bench: &mut Bencher) {
    let e = 0.01;
    let (a, b) = uniform_fixed(N, e);
    bench.iter(|| {
        matches::sort_seeds(&a, &b, K);
    });
}

#[bench]
fn b_qgramindex(bench: &mut Bencher) {
    let e = 0.01;
    let (a, b) = uniform_fixed(N, e);
    bench.iter(|| {
        matches::suffix_qgrams(&a, &b, K);
    });
}

#[bench]
fn b_hashmap(bench: &mut Bencher) {
    let e = 0.01;
    let (a, b) = uniform_fixed(N, e);
    bench.iter(|| {
        matches::suffix_hashmap(&a, &b, K);
    });
}

#[bench]
fn a_hashmap(bench: &mut Bencher) {
    let e = 0.01;
    let (a, b) = uniform_fixed(N, e);
    bench.iter(|| {
        matches::seed_hashmap(&a, &b, K);
    });
}

#[bench]
fn b_hashmap_qgrams(bench: &mut Bencher) {
    let e = 0.01;
    let (a, b) = uniform_fixed(N, e);
    bench.iter(|| {
        matches::suffix_hashmap_qgrams(&a, &b, K);
    });
}

#[bench]
fn a_hashmap_qgrams(bench: &mut Bencher) {
    let e = 0.01;
    let (a, b) = uniform_fixed(N, e);
    bench.iter(|| {
        matches::seed_hashmap_qgrams(&a, &b, K);
    });
}
