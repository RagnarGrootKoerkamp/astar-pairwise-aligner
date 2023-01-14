use smallvec::SmallVec;

use super::*;
use crate::prelude::*;

type Key = usize;

fn determine_seeds<'a, F>(a: Seq<'a>, length: LengthConfig, mut f: F) -> SeedMatches
where
    // f(i, k, qgram) returns true when the qgram was used.
    F: FnMut(I, I, usize) -> Option<(Seed, Option<Match>)>,
{
    let rank_transform = RankTransform::new(&Alphabet::new(b"ACGT"));
    match length {
        Fixed(k) => {
            let mut seeds = Vec::<Seed>::new();
            let mut matches = Vec::<Match>::default();
            for (i, qgram) in iterate_fixed_qgrams(&rank_transform, a, k) {
                if let Some((seed, m)) = f(i, k, qgram) {
                    seeds.push(seed);
                    if let Some(m) = m {
                        matches.push(m);
                    }
                }
            }

            SeedMatches::new(a, seeds, matches)
        }
        LengthConfig::Max(MaxMatches {
            max_matches,
            k_min,
            k_max,
        }) => {
            assert_eq!(
                max_matches, 1,
                "Zero or more than 1 max matches does not make sense!"
            );
            let mut seeds = Vec::<Seed>::default();
            let mut matches = Vec::<Match>::default();
            let width = rank_transform.get_width();
            let mut start = 0 as I;
            let mut end = k_min;
            'outer: while (end as usize) <= a.len() {
                // Find the minimal end that gives at most 1 match.
                let mut qgram = to_qgram(&rank_transform, width, &a[start as usize..end as usize]);
                loop {
                    if let Some((seed, m)) = f(start, end - start, qgram) {
                        seeds.push(seed);
                        if let Some(m) = m {
                            matches.push(m);
                        }
                        start = end;
                        end = start + k_min;
                        continue 'outer;
                    } else {
                        if end as usize >= a.len() {
                            break 'outer;
                        }
                        qgram <<= width;
                        qgram |= rank_transform.get(a[end as usize]) as usize;
                        end += 1;
                        if end - start > k_max {
                            start += 1;
                            continue 'outer;
                        }
                    }
                }
            }

            SeedMatches::new(a, seeds, matches)
        }
    }
}

/// Counts the number of times a mutation of qgram occurs in m.
/// Returns the seed_cost if at most one match was found.
fn count_inexact_matches(
    max_match_cost: MatchCost,
    m: &HashMap<Key, SmallVec<[I; 2]>>,
    k: I,
    qgram: usize,
) -> Option<MatchCost> {
    let mut seed_cost = max_match_cost + 1;
    let mut num_matches = 0usize;
    let mut matching_k = 0;
    let mut matching_q = 0;
    let mut add = |cur_k, q| -> usize {
        let cnt = m
            .get(&key_for_sized_qgram(cur_k, q as Key))
            .map_or(0, |x| x.len());
        match cnt {
            1 => {
                if num_matches == 0 {
                    num_matches = cnt as usize;
                    matching_k = cur_k;
                    matching_q = q;
                } else {
                    // In case we have multiple kmers with matches, we must
                    // be careful to not double count the same match.
                    if qgrams_overlap(cur_k, q, matching_k, matching_q) {
                        // Keep the match of length k.
                        if cur_k == k {
                            matching_k = cur_k;
                            matching_q = q;
                        }
                    } else {
                        // Non overlapping qgrams imply at least two matches, so break.
                        num_matches = 2;
                    }
                }
            }
            cnt => num_matches += cnt as usize,
        }
        num_matches
    };
    match add(k, qgram) {
        0 => {}
        1 => seed_cost = 0,
        _ => return None,
    }
    let ms = mutations(k, qgram, false, false);
    for qgram in ms.deletions {
        if add(k - 1, qgram) > 1 {
            return None;
        }
    }
    for qgram in ms.substitutions {
        if add(k, qgram) > 1 {
            return None;
        }
    }
    for qgram in ms.insertions {
        if add(k + 1, qgram) > 1 {
            return None;
        }
    }
    if num_matches > 0 && seed_cost == 2 {
        seed_cost = 1;
    }
    Some(seed_cost)
}

/// Build a hashset of the seeds in a, and query all kmers in b.
pub fn find_matches_qgram_hash_exact_unordered<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    MatchConfig {
        length,
        max_match_cost,
        ..
    }: MatchConfig,
) -> SeedMatches {
    let k: I = match length {
        Fixed(k) => k,
        _ => unimplemented!("QGram Hashing only works for fixed k for now."),
    };
    assert!(max_match_cost == 0);

    let rank_transform = RankTransform::new(&Alphabet::new(b"ACGT"));

    let mut seeds = fixed_seeds(&rank_transform, max_match_cost, a, k);
    let mut num_matches = vec![0; seeds.len()];
    let mut matches = vec![];

    type Key = u64;

    // TODO: See if we can get rid of the Vec alltogether.
    let mut m = HashMap::<Key, SmallVec<[I; 4]>>::default();
    //let mut matches = Vec::<Match>::new();

    m.reserve(a.len() / k as usize + 1);
    for (i, w) in rank_transform
        .qgrams(k as _, a)
        .enumerate()
        .step_by(k as usize)
    {
        m.entry(w as Key).or_default().push(i as I);
    }

    for (j, w) in rank_transform.qgrams(k as _, b).enumerate() {
        if let Some(is) = m.get(&(w as Key)) {
            for &i in is {
                seeds[(i / k) as usize].seed_cost = 0;
                num_matches[(i / k) as usize] += 1;
                // Only make one match per seed.
                if num_matches[(i / k) as usize] > 1 {
                    continue;
                }
                matches.push(Match {
                    start: Pos(i, j as I),
                    end: Pos(i + k, j as I + k),
                    match_cost: 0,
                    seed_potential: 1,
                    pruned: MatchStatus::Active,
                });
            }
        }
    }
    // Only keep seeds with one match.
    seeds
        .drain_filter(|s| num_matches[(s.start / k) as usize] > 1)
        .count();

    SeedMatches::new(a, seeds, matches)
}

/// Build a hashset of the kmers in b, and query all mutations of seeds in a.
/// TODO MAKE THIS UNORDERED:
/// Store at most 6 matches.
pub fn find_matches_qgram_hash_inexact_unordered<'a>(
    a: Seq<'a>,
    b: Seq<'a>,
    MatchConfig {
        length,
        max_match_cost,
        ..
    }: MatchConfig,
) -> SeedMatches {
    let k: I = match length {
        Fixed(k) => k,
        _ => unimplemented!("QGram Hashing only works for fixed k for now."),
    };
    assert!(max_match_cost == 1);

    let rank_transform = RankTransform::new(&Alphabet::new(b"ACGT"));

    let mut seeds = fixed_seeds(&rank_transform, max_match_cost, a, k);

    // type of Qgrams
    type Q = usize;
    assert!(k <= 31);

    // TODO: See if we can get rid of the Vec alltogether.
    let mut m = HashMap::<Q, SmallVec<[Cost; 4]>>::default();
    m.reserve(3 * b.len());
    for k in k - 1..=k + 1 {
        for (j, w) in rank_transform.qgrams(k as _, b).enumerate() {
            m.entry(key_for_sized_qgram(k, w))
                .or_default()
                .push(j as Cost);
        }
    }
    let mut matches = Vec::<Match>::new();
    for seed @ &mut Seed { start, qgram, .. } in &mut seeds {
        if let Some(js) = m.get(&key_for_sized_qgram(k, qgram)) {
            for &j in js {
                seed.seed_cost = 0;
                matches.push(Match {
                    start: Pos(start, j),
                    end: Pos(start + k, j + k),
                    match_cost: 0,
                    seed_potential: 2,
                    pruned: MatchStatus::Active,
                });
            }
        }
        // We don't dedup here, since we'll be sorting and deduplicating the list of all matches anyway.
        let ms = mutations(k, qgram, false, false);
        for w in ms.deletions {
            if let Some(js) = m.get(&key_for_sized_qgram(k - 1, w)) {
                for &j in js {
                    seed.seed_cost = min(seed.seed_cost, 1);
                    matches.push(Match {
                        start: Pos(start, j),
                        end: Pos(start + k, j + k - 1),
                        match_cost: 1,
                        seed_potential: 2,
                        pruned: MatchStatus::Active,
                    });
                }
            }
        }
        for w in ms.substitutions {
            if let Some(js) = m.get(&key_for_sized_qgram(k, w)) {
                for &j in js {
                    seed.seed_cost = min(seed.seed_cost, 1);
                    matches.push(Match {
                        start: Pos(start, j),
                        end: Pos(start + k, j + k),
                        match_cost: 1,
                        seed_potential: 2,
                        pruned: MatchStatus::Active,
                    });
                }
            }
        }
        for w in ms.insertions {
            if let Some(js) = m.get(&key_for_sized_qgram(k + 1, w)) {
                for &j in js {
                    seed.seed_cost = min(seed.seed_cost, 1);
                    matches.push(Match {
                        start: Pos(start, j),
                        end: Pos(start + k, j + k + 1),
                        match_cost: 1,
                        seed_potential: 2,
                        pruned: MatchStatus::Active,
                    });
                }
            }
        }

        // If there are non-overlapping matches, remove the seed and all matches.
        //
    }

    SeedMatches::new(a, seeds, matches)
}

/// Initialize a counter to 0 for all seeds in a.
/// Then count these kmers in b.
/// Keep only seeds for which the counter is at most 1.
pub fn unordered_matches<'a>(a: Seq<'a>, b: Seq<'a>, match_config: MatchConfig) -> SeedMatches {
    let match_config @ MatchConfig {
        length,
        max_match_cost,
        ..
    } = match_config;
    let rank_transform = RankTransform::new(&Alphabet::new(b"ACGT"));

    if let Fixed(_) = length {
        if max_match_cost == 0 {
            return find_matches_qgram_hash_exact_unordered(a, b, match_config);
        } else {
            return find_matches_qgram_hash_inexact_unordered(a, b, match_config);
        }
    }
    assert!(
        max_match_cost == 0,
        "The code below is broken somehow for inexact matches!"
    );

    // 1. Put all k-mers (and k+-1 mers) of b in a map.
    let mut m = HashMap::<Key, SmallVec<[I; 2]>>::default();
    m.reserve((1 + 2 * max_match_cost as usize) * b.len() as usize + 1);
    for k in length.kmin() - max_match_cost as I..=length.kmax() + max_match_cost as I {
        for (i, qgram) in rank_transform.qgrams(k as _, b).enumerate() {
            let x = m.entry(key_for_sized_qgram(k, qgram) as Key).or_default();
            if x.len() < 2 {
                x.push(i as I);
            }
        }
    }

    // 2. Find the seeds, counting the number of (inexact) matches for each qgram.
    determine_seeds(a, length, |start, k, qgram| {
        let (seed_cost, m) = if max_match_cost == 0 {
            match m.get(&key_for_sized_qgram(k, qgram as Key)) {
                None => (1, None),
                Some(x) if x.len() == 1 => {
                    let m = Match {
                        start: Pos(start, x[0]),
                        end: Pos(start + k, x[0] + k),
                        match_cost: 0,
                        seed_potential: 1,
                        pruned: MatchStatus::Active,
                    };
                    (0, Some(m))
                }
                _ => return None,
            }
        } else {
            match count_inexact_matches(max_match_cost, &m, k, qgram) {
                Some(value) => (value, None),
                None => return None,
            }
        };
        Some((
            Seed {
                start,
                end: start as I + k,
                seed_potential: max_match_cost + 1,
                seed_cost,
                qgram,
            },
            m,
        ))
    })
}
