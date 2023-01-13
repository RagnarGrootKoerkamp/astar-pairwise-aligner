#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(test)]
#![cfg(test)]
use astar_pairwise_aligner::{
    matches::{find_matches_qgram_hash_exact, find_matches_qgramindex, find_matches_trie},
    prelude::*,
};

#[macro_use]
extern crate lazy_static;
extern crate test;

use test::Bencher;

const E: f32 = 0.02;
const K: I = 8;

mod matches {

    use aho_corasick::AhoCorasickBuilder;
    use bio::data_structures::suffix_array::suffix_array;
    use itertools::Itertools;
    use suffix::SuffixTable;

    use crate::{prelude::*, trie::Trie};

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
        let st = SuffixTable::new(to_string(b));
    }

    pub fn aho_corasick(a: Seq, b: Seq, k: I) {
        let ac = AhoCorasickBuilder::new()
            .build_with_size::<u16, _, _>(a.chunks_exact(k as usize))
            .unwrap();
        let cnt = ac.find_overlapping_iter(b).count();
        println!("{cnt}");
    }

    pub fn regex(a: Seq, b: Seq, k: I) {
        let ac = AhoCorasickBuilder::new()
            .build_with_size::<u16, _, _>(a.chunks_exact(k as usize))
            .unwrap();
        let cnt = ac.find_overlapping_iter(b).count();
        println!("{cnt}");
    }

    pub fn build_trie(a: Seq, b: Seq, k: I) {
        Trie::new(
            b.windows(k as usize)
                .enumerate()
                .map(|(i, w)| (w, i as trie::Data)),
        );
    }

    pub fn build_trie_on_seeds(a: Seq, b: Seq, k: I) {
        Trie::new(
            a.chunks_exact(k as usize)
                .enumerate()
                .map(|(i, w)| (w, i as trie::Data)),
        );
    }

    pub fn build_trie_sorted(a: Seq, b: Seq, k: I) {
        Trie::new(
            b.windows(k as usize)
                .enumerate()
                .map(|(i, w)| (w, i as trie::Data)),
        );
    }

    pub fn build_trie_on_seeds_sorted(a: Seq, b: Seq, k: I) {
        Trie::new(
            a.chunks_exact(k as usize)
                .enumerate()
                .map(|(i, w)| (w, i as trie::Data)),
        );
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
        for (i, w) in TRANSFORM.qgrams(k, a).step_by(k as usize).enumerate() {
            *m.entry(w as u32).or_default() = i as u32;
        }
    }
    pub fn suffix_hashmap_qgrams(a: Seq, b: Seq, k: I) {
        let mut m = HashMap::<u32, u32>::default();
        m.reserve(b.len());
        for (i, w) in TRANSFORM.qgrams(k, b).enumerate() {
            *m.entry(w as u32).or_default() = i as u32;
        }
    }

    pub fn lookup_seeds_in_qgram_hashmap(a: Seq, b: Seq, k: I) {
        let mut m = HashMap::<u32, u32>::default();
        m.reserve(b.len());
        for (i, w) in TRANSFORM.qgrams(k, b).enumerate() {
            *m.entry(w as u32).or_default() = i as u32;
        }
        let mut cnt = 0;
        for (j, w) in TRANSFORM.qgrams(k, a).step_by(k as usize).enumerate() {
            if m.contains_key(&(w as u32)) {
                cnt += 1;
            }
        }
    }

    pub fn lookup_suffixes_in_qgram_hashmap(a: Seq, b: Seq, k: I) {
        let mut m = HashMap::<u32, u32>::default();
        m.reserve(a.len());
        for (i, w) in TRANSFORM.qgrams(k, a).step_by(k as usize).enumerate() {
            *m.entry(w as u32).or_default() = i as u32;
        }
        let mut cnt = 0;
        for (j, w) in TRANSFORM.qgrams(k, b).enumerate() {
            if m.contains_key(&(w as u32)) {
                cnt += 1;
            }
        }
    }
}

#[bench]
fn n100_exact_qgramindex(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| find_matches_qgramindex(&a, &b, MatchConfig::exact(K), false));
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
//                 length: Fixed(6),
//                 max_match_cost: 1,
//                 ..Default::default()
//             },
//         )
//     });
// }

#[bench]
fn n10000_exact_qgramindex(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| find_matches_qgramindex(&a, &b, MatchConfig::exact(K), false));
}

// #[bench]
// fn n10000_inexact_qgramindex(bench: &mut Bencher) {
//     let n = 10000;
//     let e = 0.20;
//     let (a, b) = setup(n, e);
//     bench.iter(|| {
//         find_matches_qgramindex(
//             &a,
//             &b,
//             MatchConfig {
//                 length: Fixed(9),
//                 max_match_cost: 1,
//                 ..Default::default()
//             },
//         )
//     });
// }

#[bench]
fn n100_exact_trie(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| find_matches_trie(&a, &b, MatchConfig::exact(K)));
}

// #[bench]
// fn n100_inexact_trie(bench: &mut Bencher) {
//     let n = 100;
//     let e = 0.10;
//     let (a, b) = setup(n, e);
//     bench.iter(|| {
//         find_matches_trie(
//             &a,
//             &b,
//             MatchConfig {
//                 length: Fixed(6),
//                 max_match_cost: 1,
//                 ..Default::default()
//             },
//         )
//     });
// }

#[bench]
fn n10000_exact_trie(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| find_matches_trie(&a, &b, MatchConfig::exact(K)));
}

// #[bench]
// fn n10000_inexact_trie(bench: &mut Bencher) {
//     let n = 10000;
//     let e = 0.20;
//     let (a, b) = setup(n, e);
//     bench.iter(|| {
//         find_matches_trie(
//             &a,
//             &b,
//             MatchConfig {
//                 length: Fixed(9),
//                 max_match_cost: 1,
//                 ..Default::default()
//             },
//         )
//     });
// }

#[bench]
fn n100_exact_hash(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| find_matches_qgram_hash_exact(&a, &b, MatchConfig::exact(K)));
}

#[bench]
fn n10000_exact_hash(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| find_matches_qgram_hash_exact(&a, &b, MatchConfig::exact(K)));
}

#[bench]
fn n100_aho_corasick(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| {
        matches::aho_corasick(&a, &b, K);
    });
}
#[bench]
fn n10000_aho_corasick(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| {
        matches::aho_corasick(&a, &b, K);
    });
}

#[bench]
fn n100_lookup_seeds_in_qgram_hashmap(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| {
        matches::lookup_seeds_in_qgram_hashmap(&a, &b, K);
    });
}
#[bench]
fn n10000_lookup_seeds_in_qgram_hashmap(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| {
        matches::lookup_seeds_in_qgram_hashmap(&a, &b, K);
    });
}

#[bench]
fn n100_lookup_suffixes_in_qgram_hashmap(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| {
        matches::lookup_suffixes_in_qgram_hashmap(&a, &b, K);
    });
}
#[bench]
fn n10000_lookup_suffixes_in_qgram_hashmap(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, E);
    bench.iter(|| {
        matches::lookup_suffixes_in_qgram_hashmap(&a, &b, K);
    });
}

#[bench]
fn n100_b_suffix_array(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, 0.0);
    bench.iter(|| {
        matches::suffix_array_bio(&a, &b, 0);
    });
}

#[bench]
fn n10000_b_suffix_array(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, 0.0);
    bench.iter(|| {
        matches::suffix_array_bio(&a, &b, 0);
    });
}

#[bench]
fn n100_b_suffix_array_2(bench: &mut Bencher) {
    let n = 100;
    let (a, b) = uniform_fixed(n, 0.0);
    bench.iter(|| {
        matches::suffix_array_suffixtable(&a, &b, 0);
    });
}

#[bench]
fn n10000_b_suffix_array_2(bench: &mut Bencher) {
    let n = 10000;
    let (a, b) = uniform_fixed(n, 0.0);
    bench.iter(|| {
        matches::suffix_array_suffixtable(&a, &b, 0);
    });
}

#[bench]
fn n100_b_suffix_array_sort(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::suffix_array_sort(&a, &b, K);
    });
}
#[bench]
fn n10000_b_suffix_array_sort(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::suffix_array_sort(&a, &b, K);
    });
}

#[bench]
fn n100_a_sort_seeds(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::sort_seeds(&a, &b, K);
    });
}
#[bench]
fn n10000_a_sort_seeds(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::sort_seeds(&a, &b, K);
    });
}

#[bench]
fn n100_b_build_trie(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::build_trie(&a, &b, K);
    });
}
#[bench]
fn n10000_b_build_trie(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::build_trie(&a, &b, K);
    });
}

#[bench]
fn n100_a_build_trie_on_seeds(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::build_trie_on_seeds(&a, &b, K);
    });
}
#[bench]
fn n10000_a_build_trie_on_seeds(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::build_trie_on_seeds(&a, &b, K);
    });
}

#[bench]
fn n100_b_qgramindex(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::suffix_qgrams(&a, &b, K);
    });
}
#[bench]
fn n10000_b_qgramindex(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::suffix_qgrams(&a, &b, K);
    });
}

#[bench]
fn n100_b_hashmap(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::suffix_hashmap(&a, &b, K);
    });
}
#[bench]
fn n10000_b_hashmap(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::suffix_hashmap(&a, &b, K);
    });
}

#[bench]
fn n100_a_hashmap(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::seed_hashmap(&a, &b, K);
    });
}
#[bench]
fn n10000_a_hashmap(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::seed_hashmap(&a, &b, K);
    });
}

#[bench]
fn n100_b_hashmap_qgrams(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::suffix_hashmap_qgrams(&a, &b, K);
    });
}
#[bench]
fn n10000_b_hashmap_qgrams(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::suffix_hashmap_qgrams(&a, &b, K);
    });
}

#[bench]
fn n100_a_hashmap_qgrams(bench: &mut Bencher) {
    let n = 100;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::seed_hashmap_qgrams(&a, &b, K);
    });
}
#[bench]
fn n10000_a_hashmap_qgrams(bench: &mut Bencher) {
    let n = 10000;
    let e = 0.01;
    let (a, b) = uniform_fixed(n, e);
    bench.iter(|| {
        matches::seed_hashmap_qgrams(&a, &b, K);
    });
}
