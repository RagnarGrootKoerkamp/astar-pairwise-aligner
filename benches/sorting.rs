//! Time measurements for some simple operations, but not full matching algorithms.
#![feature(test)]
#![cfg(test)]
use pairwise_aligner::prelude::*;

extern crate test;

use test::Bencher;

#[bench]
fn n100_b_suffix_array(bench: &mut Bencher) {
    let n = 100;
    let (a, b, _, _) = setup(n, 0.0);
    bench.iter(|| {
        matches::suffix_array_bio(&a, &b, 0);
    });
}

#[bench]
fn n10000_b_suffix_array(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, _, _) = setup(n, 0.0);
    bench.iter(|| {
        matches::suffix_array_bio(&a, &b, 0);
    });
}

#[bench]
fn n100_b_suffix_array_2(bench: &mut Bencher) {
    let n = 100;
    let (a, b, _, _) = setup(n, 0.0);
    bench.iter(|| {
        matches::suffix_array_suffixtable(&a, &b, 0);
    });
}

#[bench]
fn n10000_b_suffix_array_2(bench: &mut Bencher) {
    let n = 10000;
    let (a, b, _, _) = setup(n, 0.0);
    bench.iter(|| {
        matches::suffix_array_suffixtable(&a, &b, 0);
    });
}

#[bench]
fn n100_b_suffix_array_sort(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::suffix_array_sort(&a, &b, k);
    });
}
#[bench]
fn n10000_b_suffix_array_sort(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::suffix_array_sort(&a, &b, k);
    });
}

#[bench]
fn n100_a_sort_seeds(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::sort_seeds(&a, &b, k);
    });
}
#[bench]
fn n10000_a_sort_seeds(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::sort_seeds(&a, &b, k);
    });
}

#[bench]
fn n100_b_build_trie(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::build_trie(&a, &b, k);
    });
}
#[bench]
fn n10000_b_build_trie(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::build_trie(&a, &b, k);
    });
}

#[bench]
fn n100_a_build_trie_on_seeds(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::build_trie_on_seeds(&a, &b, k);
    });
}
#[bench]
fn n10000_a_build_trie_on_seeds(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::build_trie_on_seeds(&a, &b, k);
    });
}

#[bench]
fn n100_b_qgramindex(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::suffix_qgrams(&a, &b, k);
    });
}
#[bench]
fn n10000_b_qgramindex(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::suffix_qgrams(&a, &b, k);
    });
}

#[bench]
fn n100_b_hashmap(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::suffix_hashmap(&a, &b, k);
    });
}
#[bench]
fn n10000_b_hashmap(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::suffix_hashmap(&a, &b, k);
    });
}

#[bench]
fn n100_a_hashmap(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::seed_hashmap(&a, &b, k);
    });
}
#[bench]
fn n10000_a_hashmap(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::seed_hashmap(&a, &b, k);
    });
}

#[bench]
fn n100_b_hashmap_qgrams(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::suffix_hashmap_qgrams(&a, &b, k);
    });
}
#[bench]
fn n10000_b_hashmap_qgrams(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::suffix_hashmap_qgrams(&a, &b, k);
    });
}

#[bench]
fn n100_a_hashmap_qgrams(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::seed_hashmap_qgrams(&a, &b, k);
    });
}
#[bench]
fn n10000_a_hashmap_qgrams(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let k = 8;
    let (a, b, _, _) = setup(n, e);
    bench.iter(|| {
        matches::seed_hashmap_qgrams(&a, &b, k);
    });
}
