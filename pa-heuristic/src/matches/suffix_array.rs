use std::ops::Range;

use bio::{
    alphabets::Alphabet,
    data_structures::{
        bwt::{bwt, less, Less, Occ, BWT},
        suffix_array::{suffix_array, RawSuffixArray},
    },
};
use pa_types::{Base, Pos, Seq};

use crate::{
    matches::{qgrams::QGrams, Match, MatchBuilder, MatchStatus, Matches, MaxMatches, Seed},
    LengthConfig, MatchConfig,
};

type SaRange = Range<usize>;

pub struct FmIndex {
    sa: RawSuffixArray,
    bwt: BWT,
    less: Less,
    occ: Occ,
}

impl FmIndex {
    fn new(text: Seq) -> Self {
        let mut text = text.to_owned();
        text.push(b'$');
        let alphabet = Alphabet::new(b"ACGT");
        let sa = suffix_array(&text);
        let bwt = bwt(&text, &sa);
        let less = less(&bwt, &alphabet);
        // TODO: Experiment with sample frequency.
        let occ = Occ::new(&bwt, 1, &alphabet);
        Self { sa, bwt, less, occ }
    }
    fn full_range(&self) -> SaRange {
        0..self.bwt.len()
    }
    fn prepend(&self, range: &SaRange, c: Base) -> SaRange {
        let l = range.start;
        let r = range.end;
        let less = self.less[c as usize];
        let nl = less
            + if l > 0 {
                self.occ.get(&self.bwt, l - 1, c)
            } else {
                0
            };
        let nr = less + self.occ.get(&self.bwt, r - 1, c);
        nl..nr
    }
}

/// Determine seeds as minimal unique matches:
/// For each seed, start from the end and keep prepending chars until
/// the seed has at most `x` matches with cost less than `r`.
///
/// Implementation:
/// - Build a suffix array, BWT, and FM-index on `b`.
/// - Keep prepending chars.
/// - Keep a list of 'current' intervals, and prepend each char to each interval.
/// - Also allow mutations for intervals that still have leftover cost.

pub fn minimal_unique_matches(
    a: Seq,
    b: Seq,
    config @ MatchConfig { length, r, .. }: MatchConfig,
) -> Matches {
    // TODO: Use `k_min` and `k_max`.
    let LengthConfig::Max(MaxMatches { max_matches, ..}) = length else {
        panic!("This function requires a maximum number of matches.")
    };

    let fm = FmIndex::new(b);
    assert!(
        r == 1 || r == 2,
        "Matches with more than 1 error are not supported."
    );

    let mut seeds = vec![];
    let qgrams = QGrams::new(a, b);
    let mut matches = MatchBuilder::new(&qgrams, config, false);

    let chars = b"ACGT";

    // The end of the current seed.
    let mut seed_end = a.len();
    // The range of the current seed.
    let mut ranges: Vec<(SaRange, u8, usize)>;

    // Initial ranges include a range for:
    // - the full interval, ie the exact match of 0 chars
    // - an interval corresponding to inserting a first char.
    let init_ranges = |fm: &FmIndex| {
        let mut ranges = vec![(fm.full_range(), 0, 0)];
        if 1 < r {
            for &c in chars {
                ranges.push((fm.prepend(&fm.full_range(), c), 1, 1));
            }
        }
        ranges
    };

    ranges = init_ranges(&fm);

    for i in (0..a.len()).rev() {
        // Keep prepending chars until the number of matches is sufficiently low.
        let mut new_ranges = vec![];
        for (range, cost, len) in ranges {
            let match_range;
            // match
            {
                match_range = fm.prepend(&range, a[i]);
                if !match_range.is_empty() {
                    new_ranges.push((match_range.clone(), cost, len + 1));
                }
            }
            if cost + 1 >= r {
                continue;
            }
            // delete
            new_ranges.push((range.clone(), cost + 1, len));

            // subs
            for &c in chars {
                if c != a[i] {
                    let range = fm.prepend(&range, c);
                    if !range.is_empty() {
                        new_ranges.push((range, cost + 1, len + 1));
                    }
                }
            }

            // insertion after match
            // NOTE: This does not work for repeated insertions when r>2.
            if !match_range.is_empty() {
                for &c in chars {
                    let range = fm.prepend(&match_range, c);
                    if !range.is_empty() {
                        new_ranges.push((range, cost + 1, len + 2));
                    }
                }
            }
        }
        // Sort and dedup
        new_ranges.sort_by_key(|(range, r, l): &_| (range.start, range.end, *r, *l));
        new_ranges.dedup();
        ranges = new_ranges;

        let total_matches: usize = ranges.iter().map(|(range, ..)| range.len()).sum();

        if total_matches <= max_matches {
            let seed_start = i;
            seeds.push(Seed {
                start: seed_start as _,
                end: seed_end as _,
                seed_potential: r as _,
                seed_cost: 0,
            });
            for (range, cost, l) in ranges {
                for sa_idx in range {
                    let match_start = fm.sa[sa_idx];
                    let match_end = match_start + l;
                    matches.push(Match {
                        start: Pos(seed_start as _, match_start as _),
                        end: Pos(seed_end as _, match_end as _),
                        match_cost: cost,
                        seed_potential: r as _,
                        pruned: MatchStatus::Active,
                    });
                }
            }

            // Reset state.
            seed_end = i;
            ranges = init_ranges(&fm);
        }
    }
    matches.seeds.seeds.reverse();
    matches.matches.reverse();

    // for s in &seeds {
    //     eprintln!("Seed {s:?}");
    // }
    // for m in &matches {
    //     eprintln!("Match {m:?}");
    // }

    matches.finish()
}
