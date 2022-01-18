use std::iter::repeat;

use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct Seed {
    pub start: I,
    pub end: I,
    // The seed_potential is 1 more than the maximal number of errors allowed in this seed.
    pub seed_potential: Cost,
    pub qgram: usize,
}

#[derive(Clone, Debug)]
pub struct Match {
    pub start: Pos,
    pub end: Pos,
    pub match_cost: Cost,
    pub seed_potential: Cost,
}

#[derive(Default)]
pub struct SeedMatches {
    // Sorted by (i, j)
    pub num_seeds: I,
    pub matches: Vec<Match>,
    // Index of the start of the rightmost seed covering the given position.
    pub start_of_seed: Vec<I>,
    potential: Vec<Cost>,
}

impl SeedMatches {
    pub fn iter(&self) -> std::slice::Iter<Match> {
        self.matches.iter()
    }

    // The potential at p is the cost of going from p to the end, without hitting any matches.
    pub fn potential(&self, Pos(i, _): Pos) -> Cost {
        self.potential[i as usize]
    }

    // TODO: Generalize this for overlapping seeds.
    pub fn is_start_of_seed(&self, Pos(i, _): Pos) -> bool {
        self.start_of_seed[i as usize] == i
    }
}

impl<'a> HeuristicInstance<'a> for SeedMatches {
    fn h(&self, _: Self::Pos) -> Cost {
        unimplemented!("SeedMatches can only be used as a distance, not as a heuristic!");
    }
}
impl<'a> DistanceInstance<'a> for SeedMatches {
    /// The minimal distance is the potential of the seeds entirely within the `[from, to)` interval.
    /// NOTE: Assumes disjoint seeds.
    fn distance(&self, from: Pos, to: Pos) -> Cost {
        assert!(from.0 <= to.0);
        self.potential[from.0 as usize]
            - (self.potential[self.start_of_seed[to.0 as usize] as usize])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MaxMatches {
    // The smallest l with at most this many matches within the band.
    pub max_matches: usize,
    // Return the band as a function of n.
    pub band: fn(I) -> I,
}

#[derive(Clone, Copy, Debug)]
pub struct MinMatches {
    // The largest l with at least this many matches within the band.
    pub min_matches: usize,
    // Return the band as a function of n.
    pub band: fn(I) -> I,
}

#[derive(Clone, Copy, Debug)]
pub enum LengthConfig {
    Fixed(I),
    Max(MaxMatches),
    Min(MinMatches),
}

impl LengthConfig {
    pub fn fixed(l: I) -> LengthConfig {
        LengthConfig::Fixed(l)
    }
    pub fn max(max_matches: usize, band: fn(I) -> I) -> LengthConfig {
        LengthConfig::Max(MaxMatches { max_matches, band })
    }
    pub fn min(min_matches: usize, band: fn(I) -> I) -> LengthConfig {
        assert!(min_matches > 0);
        LengthConfig::Min(MinMatches { min_matches, band })
    }
    pub fn l(&self) -> Option<I> {
        match *self {
            Fixed(l) => Some(l),
            _ => None,
        }
    }
}

impl Default for LengthConfig {
    fn default() -> Self {
        LengthConfig::Fixed(0)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MatchConfig {
    // TODO: Add settings for variable length matches in here.
    pub length: LengthConfig,
    // TODO: Move the max_match_cost into MatchLength.
    pub max_match_cost: Cost,
    pub mutation_config: MutationConfig,
}

pub fn find_matches<'a>(
    a: &'a Sequence,
    b: &'a Sequence,
    alph: &Alphabet,
    MatchConfig {
        length,
        max_match_cost,
        mutation_config,
    }: MatchConfig,
) -> SeedMatches {
    assert!(max_match_cost == 0 || max_match_cost == 1);

    let Pos(n, _m) = Pos::from_length(a, b);

    // Qgrams of B.
    // TODO: Profile this index and possibly use something more efficient for large l.
    let qgrams = &mut HashMap::<I, QGramIndex>::default();
    // TODO: This should return &[I] instead.
    fn get_matches<'a, 'c>(
        qgrams: &'c mut HashMap<I, QGramIndex>,
        b: &'a Sequence,
        alph: &Alphabet,
        l: I,
        qgram: usize,
    ) -> &'c [usize] {
        qgrams
            .entry(l)
            .or_insert_with_key(|l| QGramIndex::new(*l as u32, b, alph))
            .qgram_matches(qgram)
    }

    // Stops counting when max_count is reached.
    let mut count_matches = |l: I, qgram, max_count: usize, i: I, band: I| -> usize {
        let count_in_band = |matches: &[usize]| -> usize {
            // println!(
            //     "{} {} {} {} for {:?}",
            //     l,
            //     max_count,
            //     i.saturating_sub(band),
            //     i + band,
            //     matches
            // );
            if matches.len() <= 32 {
                matches
                    .iter()
                    .copied()
                    .filter(|&j| i <= j as I + band && j as I <= i + band)
                    .count()
            } else {
                let start = matches
                    .binary_search(&(i.saturating_sub(band) as usize))
                    .map_or_else(|x| x, |x| x);
                let end = matches
                    .binary_search(&((i + band) as usize))
                    .map_or_else(|x| x + 1, |x| x);
                end - start
            }
        };

        // exact matches
        let mut cnt = 0;
        cnt += count_in_band(get_matches(qgrams, b, alph, l, qgram));
        if cnt >= max_count {
            return max_count;
        }
        if max_match_cost == 1 {
            let mutations = mutations(l, qgram, mutation_config);
            for (v, l) in [
                (mutations.deletions, l - 1),
                (mutations.substitutions, l),
                (mutations.insertions, l + 1),
            ] {
                for qgram in v {
                    cnt += count_in_band(get_matches(qgrams, b, alph, l, qgram));
                    if cnt >= max_count {
                        return max_count;
                    }
                }
            }
        }
        cnt
    };

    // Convert to a binary sequences.
    let rank_transform = RankTransform::new(alph);
    let qgram = |seed: &[u8]| {
        rank_transform
            .qgrams(seed.len() as u32, seed)
            .next()
            .unwrap()
    };

    // Split a into seeds of size l and l+1 alternating, which are bit-encoded.
    let seed_qgrams = {
        // TODO: Make a dedicated struct for seeds, apart from Matches.
        // (start, end, max_match_cost, qgram)
        let mut v: Vec<Seed> = Vec::default();
        let mut a = &a[..];
        let mut long = false;
        let mut i = 0 as I;
        loop {
            // TODO: Clever seed choice, using variable l and m.
            let seed_len = {
                match length {
                    Fixed(l) => l,
                    LengthConfig::Max(MaxMatches { max_matches, band }) => {
                        let mut l = 3 as I;
                        while l <= a.len() as I && l <= 10
                                // TODO: Use band(min(a.len(), n-a.len())) or something like it.
                                && count_matches(l, qgram(&a[..l as usize]), max_matches + 1, i, band(n))
                                    > max_matches
                        {
                            l += 1;
                        }
                        l
                    }
                    LengthConfig::Min(MinMatches { min_matches, band }) => {
                        let mut l = 4 as I;
                        // TODO: Remove max length, which is only needed because of memory reasons.
                        while l <= a.len() as I && l <= 11
                                // TODO: Use band(min(a.len(), n-a.len())) or something like it.
                                && count_matches(l, qgram(&a[..l as usize]), min_matches, i, band(n))
                                    >= min_matches
                        {
                            l += 1;
                        }
                        l - 1
                    }
                }
            };
            if seed_len > a.len() as I {
                break;
            }
            //print!("{} ", seed_len);

            let (seed, tail) = a.split_at(seed_len as usize);
            a = tail;

            v.push(Seed {
                start: i,
                end: i + seed_len,
                seed_potential: max_match_cost + 1,
                qgram: qgram(seed),
            });
            i += seed_len;

            long = !long;
        }
        //println!();
        v
    };
    let num_seeds = seed_qgrams.len() as I;
    // println!(
    //     "l: {}",
    //     //length,
    //     //num_seeds,
    //     a.len() as f32 / num_seeds as f32
    // );

    let n = a.len();
    let mut potential = Vec::with_capacity(n + 1);
    let mut start_of_seed = Vec::with_capacity(n + 1);

    // Find matches of the seeds of a in b.
    // NOTE: This uses O(alphabet^l) memory.
    let mut matches = Vec::<Match>::new();

    let mut cur_potential = seed_qgrams
        .iter()
        .map(|Seed { seed_potential, .. }| seed_potential)
        .sum();
    potential.push(cur_potential);
    //println!("{:?}", seed_qgrams);
    for &Seed {
        start,
        end,
        seed_potential,
        qgram,
    } in &seed_qgrams
    {
        let len = end - start;
        cur_potential -= seed_potential;
        potential.extend(repeat(cur_potential).take(len as usize));
        start_of_seed.extend(repeat(start).take(len as usize));

        // Exact matches
        for &j in get_matches(qgrams, b, alph, len, qgram) {
            matches.push(Match {
                start: Pos(start, j as I),
                end: Pos(end, j as I + len),
                match_cost: 0,
                seed_potential,
            });
        }
        // Inexact matches.
        if seed_potential > 1 {
            let mutations = mutations(len, qgram, mutation_config);
            for mutation in mutations.deletions {
                for &j in get_matches(qgrams, b, alph, len - 1, mutation) {
                    matches.push(Match {
                        start: Pos(start, j as I),
                        end: Pos(end, j as I + len - 1),
                        match_cost: 1,
                        seed_potential,
                    });
                }
            }
            for mutation in mutations.substitutions {
                for &j in get_matches(qgrams, b, alph, len, mutation) {
                    matches.push(Match {
                        start: Pos(start, j as I),
                        end: Pos(end, j as I + len),
                        match_cost: 1,
                        seed_potential,
                    });
                }
            }
            for mutation in mutations.insertions {
                for &j in get_matches(qgrams, b, alph, len + 1, mutation) {
                    matches.push(Match {
                        start: Pos(start, j as I),
                        end: Pos(end, j as I + len + 1),
                        match_cost: 1,
                        seed_potential,
                    });
                }
            }
        }
    }
    // Backfill a potential gap after the last seed.
    potential.extend(repeat(0).take(n + 1 - potential.len()));
    start_of_seed.extend(repeat(seed_qgrams.last().unwrap().end).take(n + 1 - start_of_seed.len()));

    //println!("{:?}", potential);
    //println!("{:?}", start_of_seed);

    // TODO: This sorting could be a no-op if we generate matches in order.
    matches.sort_unstable_by_key(|&Match { start, .. }| (start.0, start.1));
    //for m in &matches {
    //println!("{:?}", m);
    //}

    SeedMatches {
        num_seeds,
        matches,
        start_of_seed,
        potential,
    }
}
