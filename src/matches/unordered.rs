use smallvec::SmallVec;

pub use crate::prelude::*;

type Key = usize;

fn determine_seeds<'a, F>(
    a: &'a Sequence,
    alph: &Alphabet,
    length: LengthConfig,
    mut f: F,
) -> SeedMatches
where
    // f(i, k, qgram) returns true when the qgram was used.
    F: FnMut(I, I, usize) -> Option<(Seed, Option<Match>)>,
{
    let rank_transform = RankTransform::new(alph);
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
    let ms = mutations(k, qgram, false);
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

fn unordered_matches_hash<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    alph: &Alphabet,
    MatchConfig {
        length,
        max_match_cost,
        ..
    }: MatchConfig,
) -> SeedMatches {
    let rank_transform = RankTransform::new(alph);

    // 1. Put all k-mers (and k+-1 mers) of b in a map.
    let mut m = HashMap::<Key, SmallVec<[I; 2]>>::default();
    m.reserve((1 + 2 * max_match_cost as usize) * b.len() as usize + 1);
    for k in length.kmin() - max_match_cost as I..=length.kmax() + max_match_cost as I {
        for (i, qgram) in rank_transform.qgrams(k, b).enumerate() {
            let x = m.entry(key_for_sized_qgram(k, qgram) as Key).or_default();
            if x.len() < 2 {
                x.push(i as I);
            }
        }
    }

    // 2. Find the seeds, counting the number of (inexact) matches for each qgram.
    determine_seeds(a, alph, length, |start, k, qgram| {
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

/// Initialize a counter to 0 for all seeds in a.
/// Then count these kmers in b.
/// Keep only seeds for which the counter is at most 1.
pub fn unordered_matches<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    alph: &Alphabet,
    match_config @ MatchConfig { algorithm, .. }: MatchConfig,
) -> SeedMatches {
    match algorithm {
        MatchAlgorithm::Hash => unordered_matches_hash(a, b, alph, match_config),
        _ => unimplemented!("This algorithm is not implemented."),
    }
}
