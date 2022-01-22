#![allow(dead_code)]
#![allow(unused_variables)]
use bio_types::sequence::Sequence;

use crate::prelude::*;

lazy_static! {
    static ref ALPH: Alphabet = Alphabet::new(b"ACTG");
    static ref TRANSFORM: RankTransform = RankTransform::new(&ALPH);
}

pub fn mutations(k: I, kmer: usize, dels: bool, subs: bool, ins: bool, dedup: bool) -> Mutations {
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

pub fn lookup_b_in_am_hashmap(a: &Sequence, b: &Sequence, k: I) {
    assert!(k <= 14);
    let key = |l: u32, w: usize| ((w as u32) << 2) + (l + 1 - k);
    // TODO: Split in 3 hashmaps?
    let mut m = HashMap::<u32, (u32, u32)>::default();
    m.reserve(a.len() * 10 * k as usize);
    for (i, w) in TRANSFORM.qgrams(k, a).step_by(k as usize).enumerate() {
        *m.entry(key(k, w)).or_default() = (i as u32, k);
        let ms = mutations(k, w, true, true, true, false);
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
    for (j, w) in TRANSFORM.qgrams(k - 1, b).enumerate() {
        if m.contains_key(&key(k - 1, w)) {
            cnt += 1;
        }
    }
    for (j, w) in TRANSFORM.qgrams(k, b).enumerate() {
        if m.contains_key(&key(k, w)) {
            cnt += 1;
        }
    }
    for (j, w) in TRANSFORM.qgrams(k + 1, b).enumerate() {
        if m.contains_key(&key(k + 1, w)) {
            cnt += 1;
        }
    }
    println!("{} {cnt}", m.len());
}

pub fn lookup_am_in_b_hashmap(a: &Sequence, b: &Sequence, k: I) {
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
    for (i, w) in TRANSFORM.qgrams(k, a).step_by(k as usize).enumerate() {
        if m.contains_key(&key(k, w)) {
            cnt += 1;
        }
        let ms = mutations(k, w, true, true, true, false);
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

pub fn lookup_am_in_b_hashmap_dedup(a: &Sequence, b: &Sequence, k: I) {
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
    for (i, w) in TRANSFORM.qgrams(k, a).step_by(k as usize).enumerate() {
        if m.contains_key(&key(k, w)) {
            cnt += 1;
        }
        let ms = mutations(k, w, true, true, true, true);
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

pub fn lookup_a_in_bm_hashmap(a: &Sequence, b: &Sequence, k: I) {
    let key = |k: u32, w: usize| w as u32;
    let mut m = HashMap::<u32, (u32, u32)>::default();
    m.reserve(b.len() * 10 as usize);
    for (j, w) in TRANSFORM.qgrams(k - 1, b).enumerate() {
        let ms = mutations(k, w, false, false, true, false);
        for w in ms.insertions {
            m.insert(key(k, w), (j as u32, k));
        }
    }
    for (j, w) in TRANSFORM.qgrams(k, b).enumerate() {
        *m.entry(key(k, w)).or_default() = (j as u32, k);
        let ms = mutations(k, w, false, true, false, false);
        for w in ms.substitutions {
            m.insert(key(k, w), (j as u32, k));
        }
    }
    for (j, w) in TRANSFORM.qgrams(k + 1, b).enumerate() {
        let ms = mutations(k, w, true, false, false, false);
        for w in ms.deletions {
            m.insert(key(k, w), (j as u32, k));
        }
    }
    let mut cnt = 0;
    for (i, w) in TRANSFORM.qgrams(k, a).step_by(k as usize).enumerate() {
        if m.contains_key(&key(k, w)) {
            cnt += 1;
        }
    }
}

pub fn lookup_bm_in_a_hashmap(a: &Sequence, b: &Sequence, k: I) {
    let key = |k: u32, w: usize| ((w as u32) << 8) + k;
    let mut m = HashMap::<u32, (u32, u32)>::default();
    m.reserve(a.len());
    for (j, w) in TRANSFORM.qgrams(k, b).enumerate() {
        *m.entry(key(k, w)).or_default() = (j as u32, k);
    }
    let mut cnt = 0;
    for (j, w) in TRANSFORM.qgrams(k - 1, b).enumerate() {
        let ms = mutations(k, w, false, false, true, false);
        for w in ms.insertions {
            if m.contains_key(&key(k, w)) {
                cnt += 1;
            }
        }
    }
    for (j, w) in TRANSFORM.qgrams(k, b).enumerate() {
        if m.contains_key(&key(k, w)) {
            cnt += 1;
        }
        let ms = mutations(k, w, false, true, false, false);
        for w in ms.substitutions {
            if m.contains_key(&key(k, w)) {
                cnt += 1;
            }
        }
    }
    for (j, w) in TRANSFORM.qgrams(k + 1, b).enumerate() {
        let ms = mutations(k, w, true, false, false, false);
        for w in ms.deletions {
            if m.contains_key(&key(k, w)) {
                cnt += 1;
            }
        }
    }
}
