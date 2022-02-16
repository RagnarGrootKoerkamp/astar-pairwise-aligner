pub use crate::prelude::*;

/// Initialize a counter to 0 for all seeds in a.
/// Then count these kmers in b.
/// Keep only seeds for which the counter is at most 1.
pub fn count_matches_exact<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    alph: &Alphabet,
    MatchConfig {
        length,
        max_match_cost,
        ..
    }: MatchConfig,
) -> SeedMatches {
    let k: I = match length {
        Fixed(k) => k,
        _ => unimplemented!("Counting matches only works for exact k for now."),
    };
    assert!(max_match_cost == 0);

    let rank_transform = RankTransform::new(alph);

    type Key = u64;

    // Count number of occurrences for each key, and only keep seeds that match at most once.
    // TODO: Try a bloomfilter instead.
    let mut m = HashMap::<Key, u8>::default();
    let mut seeds = Vec::<Seed>::new();

    // TODO: SLIDING_WINDOW_MATCHES
    // TODO: Instead of only counting relevant seeds, we could also just count all of them.
    m.reserve(a.len() / k as usize + 1);
    for w in rank_transform.qgrams(k, a).step_by(k as usize) {
        m.insert(w as Key, 0);
    }

    for w in rank_transform.qgrams(k, b) {
        m.get_mut(&(w as Key)).map(|x| *x += 1);
    }

    // NOTE: We don't iterate the hashmap, since future iterations may not store
    // seeds in the hashmap at all.
    for (i, w) in rank_transform.qgrams(k, a).enumerate().step_by(k as usize) {
        let num_matches = m[&(w as Key)];
        if num_matches <= 1 {
            seeds.push(Seed {
                start: i as I,
                end: i as I + k,
                seed_potential: 1,
                has_matches: num_matches > 0,
                qgram: 0,
            })
        }
    }

    SeedMatches::new(a, seeds, Vec::default())
}
