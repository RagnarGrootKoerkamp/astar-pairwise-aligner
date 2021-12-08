use std::iter::{self, Empty};

use crate::{
    seeds::{find_matches, SeedMatches},
    util::*,
};

pub trait Heuristic {
    fn h(&self, pos: Pos) -> usize;
}

pub struct ZeroHeuristic;
impl ZeroHeuristic {
    pub fn new() -> Self {
        ZeroHeuristic
    }
}
impl Heuristic for ZeroHeuristic {
    fn h(&self, _: Pos) -> usize {
        0
    }
}

pub struct GapHeuristic {
    target: Pos,
}

impl GapHeuristic {
    pub fn new(a: &Sequence, b: &Sequence, _text_alphabet: &Alphabet) -> Self {
        GapHeuristic {
            target: Pos(a.len(), b.len()),
        }
    }
}

impl Heuristic for GapHeuristic {
    fn h(&self, Pos(i, j): Pos) -> usize {
        abs_diff(self.target.0 - i, self.target.1 - j)
    }
}

pub struct SeedHeuristic {
    seed_matches: SeedMatches,
    max_matches: HashMap<Pos, usize>,
}

impl SeedHeuristic {
    pub fn new(a: &Sequence, b: &Sequence, text_alphabet: &Alphabet, l: usize) -> Self {
        let seed_matches = find_matches(a, b, text_alphabet, l);
        // Compute heuristic at matches.
        let mut max_matches = HashMap::new();
        max_matches.insert(Pos(a.len(), b.len()), 0);
        for pos @ Pos(i, j) in seed_matches.iter().rev() {
            // Value is 1 + max over matches bottom right of this one.
            // TODO: Make this faster.
            // TODO: Make sure seeds do not overlap.
            let val = max_matches
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i + l && y >= j + l)
                .map(|(_, &val)| val)
                .max()
                .unwrap();
            max_matches.insert(pos, 1 + val);
        }
        SeedHeuristic {
            seed_matches,
            max_matches,
        }
    }
}

impl Heuristic for SeedHeuristic {
    fn h(&self, pos @ Pos(i, j): Pos) -> usize {
        // TODO: Find a datastructure for log-time lookup.
        let cnt = self
            .max_matches
            .iter()
            .filter(|&(&Pos(x, y), &_)| x >= i && y >= j)
            .map(|(_, &val)| val)
            .max()
            .unwrap();
        self.seed_matches.potential(pos) - cnt
    }
}

pub struct GappedSeedHeuristic {
    seed_matches: SeedMatches,
    h_map: HashMap<Pos, isize>,
}

impl GappedSeedHeuristic {
    pub fn new(a: &Sequence, b: &Sequence, text_alphabet: &Alphabet, l: usize) -> Self {
        let seed_matches = find_matches(a, b, text_alphabet, l);
        let skipped: &mut usize = &mut 0;

        // TODO: Faster precomputation & querying.
        // 1. Do precomputation using a right-to-left front. The front is just an increasing function.
        // 2. Store which matches are at some point neighbours on the front.
        // 3. When querying and coming from a given position linked to a given match, only consider neighbours of that match for the new position.

        let mut h_map = HashMap::new();
        h_map.insert(Pos(a.len(), b.len()), 0);
        for pos @ Pos(i, j) in seed_matches.iter().rev() {
            let update_val = h_map
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i + l && y >= j + l)
                .map(|(&Pos(x, y), &val)| val + abs_diff(x - i, y - j) as isize - 1)
                .min()
                .unwrap();
            let query_val = h_map
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i && y >= j)
                .map(|(&Pos(x, y), &val)| val + abs_diff(x - i, y - j) as isize)
                .min()
                .unwrap();

            if update_val < query_val {
                h_map.insert(pos, update_val);
            } else {
                *skipped += 1;
            }
            //println!("{:?} => {}", pos, val);
        }
        println!("Skipped matches: {}", skipped);
        GappedSeedHeuristic {
            seed_matches,
            h_map,
        }
    }
}
impl Heuristic for GappedSeedHeuristic {
    fn h(&self, pos @ Pos(i, j): Pos) -> usize {
        (self.seed_matches.potential(pos) as isize
            + self
                .h_map
                .iter()
                .filter(|&(&Pos(x, y), &_)| x >= i && y >= j)
                .map(|(&Pos(x, y), &val)| {
                    // TODO: Should there be a +- 1 here? Or take into account
                    // whether the current position/column is a match?
                    val + abs_diff(x - i, y - j) as isize
                })
                .min()
                .unwrap()) as usize
    }
}
