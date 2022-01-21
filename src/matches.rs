#![allow(dead_code)]
#![allow(unused_variables)]
use aho_corasick::AhoCorasickBuilder;
use bio::data_structures::suffix_array::suffix_array;
use bio_types::sequence::Sequence;
use itertools::Itertools;
use suffix::SuffixTable;

use crate::{prelude::*, trie::Trie};

lazy_static! {
    static ref ALPH: Alphabet = Alphabet::new(b"ACTG");
    static ref TRANSFORM: RankTransform = RankTransform::new(&ALPH);
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
pub fn exact_matches(a: &Sequence, b: &Sequence, k: I) {}
pub fn sort_seeds(a: &Sequence, b: &Sequence, k: I) {
    let mut seeds = a.chunks_exact(k as usize).collect_vec();
    seeds.sort_unstable();
}
pub fn suffix_array_sort(a: &Sequence, b: &Sequence, k: I) {
    let mut suffixes = b.windows(k as usize).collect_vec();
    suffixes.sort_unstable();
}
pub fn suffix_array_bio(a: &Sequence, b: &Sequence, k: I) {
    let mut b = b.to_vec();
    b.push('$' as u8);
    suffix_array(&b);
}
pub fn suffix_array_suffixtable(a: &Sequence, b: &Sequence, k: I) {
    let st = SuffixTable::new(to_string(b));
}

pub fn aho_corasick(a: &Sequence, b: &Sequence, k: I) {
    let ac = AhoCorasickBuilder::new()
        .build_with_size::<u16, _, _>(a.chunks_exact(k as usize))
        .unwrap();
    let cnt = ac.find_overlapping_iter(b).count();
    println!("{cnt}");
}

pub fn regex(a: &Sequence, b: &Sequence, k: I) {
    let ac = AhoCorasickBuilder::new()
        .build_with_size::<u16, _, _>(a.chunks_exact(k as usize))
        .unwrap();
    let cnt = ac.find_overlapping_iter(b).count();
    println!("{cnt}");
}

pub fn build_trie(a: &Sequence, b: &Sequence, k: I) {
    Trie::new(
        b.windows(k as usize)
            .enumerate()
            .map(|(i, w)| (w, i as trie::Data)),
        &ALPH,
    );
}

pub fn build_trie_on_seeds(a: &Sequence, b: &Sequence, k: I) {
    Trie::new(
        a.chunks_exact(k as usize)
            .enumerate()
            .map(|(i, w)| (w, i as trie::Data)),
        &ALPH,
    );
}

pub fn build_trie_sorted(a: &Sequence, b: &Sequence, k: I) {
    Trie::new(
        b.windows(k as usize)
            .enumerate()
            .map(|(i, w)| (w, i as trie::Data)),
        &ALPH,
    );
}

pub fn build_trie_on_seeds_sorted(a: &Sequence, b: &Sequence, k: I) {
    Trie::new(
        a.chunks_exact(k as usize)
            .enumerate()
            .map(|(i, w)| (w, i as trie::Data)),
        &ALPH,
    );
}

pub fn seed_qgrams(a: &Sequence, b: &Sequence, k: I) {
    todo!("Manual implementation needed");
}
pub fn suffix_qgrams(a: &Sequence, b: &Sequence, k: I) {
    QGramIndex::new(k as u32, b, &ALPH);
}

pub fn seed_hashmap(a: &Sequence, b: &Sequence, k: I) {
    let mut m = HashMap::<&[u8], Vec<u32>>::default();
    m.reserve(b.len());
    for (i, w) in a.chunks_exact(k as usize).enumerate() {
        m.entry(w).or_default().push(i as u32)
    }
}
pub fn suffix_hashmap(a: &Sequence, b: &Sequence, k: I) {
    let mut m = HashMap::<&[u8], Vec<u32>>::default();
    m.reserve(b.len());
    for (i, w) in b.windows(k as usize).enumerate() {
        m.entry(w).or_default().push(i as u32)
    }
}

pub fn seed_hashmap_qgrams(a: &Sequence, b: &Sequence, k: I) {
    let mut m = HashMap::<u32, Vec<u32>>::default();
    m.reserve(b.len());
    for (i, w) in TRANSFORM.qgrams(k, a).step_by(k as usize).enumerate() {
        m.entry(w as u32).or_default().push(i as u32)
    }
}
pub fn suffix_hashmap_qgrams(a: &Sequence, b: &Sequence, k: I) {
    let mut m = HashMap::<u32, Vec<u32>>::default();
    m.reserve(b.len());
    for (i, w) in TRANSFORM.qgrams(k, b).enumerate() {
        m.entry(w as u32).or_default().push(i as u32)
    }
}
