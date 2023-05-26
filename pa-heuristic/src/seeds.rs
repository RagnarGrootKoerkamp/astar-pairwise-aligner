use itertools::Itertools;

use crate::prelude::*;

/// Type for the cost of a single match/mutation.
pub type MatchCost = u8;

#[derive(Clone, Debug)]
pub struct Seed {
    pub start: I,
    pub end: I,
    /// The seed_potential is 1 more than the maximal number of errors allowed in this seed.
    pub seed_potential: MatchCost,
    /// A lower bound on the cost of crossing this seed.
    /// For unordered matches, if this is < seed_potential there must be exactly one such seed.
    pub seed_cost: MatchCost,
}

#[derive(Default)]
pub struct Seeds {
    /// Sorted by start.
    pub seeds: Vec<Seed>,
    /// The index of the seed covering position I.
    /// Seeds cover [start, end) here.
    pub seed_at: Vec<Option<I>>,
    /// The sum of seed potentials of all seeds not starting before each position.
    pub potential: Vec<Cost>,
    /// The largest i with given potential.
    pub start_of_potential: Vec<I>,
}

impl Seeds {
    /// Seeds must be sorted by start.
    pub fn new(a: Seq, seeds: Vec<Seed>) -> Self {
        // Check that seeds are sorted and non-overlapping.
        assert!(seeds
            .iter()
            .tuple_windows()
            .all(|(seed1, seed2)| seed1.start <= seed1.end && seed1.end <= seed2.start));

        let n = a.len();
        let mut potential = vec![0; n + 1];
        let mut seed_at = vec![None; n + 1];
        let mut cur_potential = 0;
        let mut next_seed = seeds.iter().enumerate().rev().peekable();
        let mut start_of_potential = vec![n as I];
        for i in (0..n + 1).rev() {
            if let Some((seed_idx, ns)) = next_seed.peek() {
                if i < ns.end as usize {
                    seed_at[i] = Some(*seed_idx as I);
                }

                if i == ns.start as usize {
                    cur_potential += ns.seed_potential as Cost;
                    for _ in 0..ns.seed_potential {
                        start_of_potential.push(i as I);
                    }
                    next_seed.next();
                }
            }
            potential[i] = cur_potential;
        }
        Seeds {
            seeds,
            seed_at,
            potential,
            start_of_potential,
        }
    }

    pub fn n(&self) -> usize {
        self.potential.len() - 1
    }

    /// The potential at p is the cost of going from p to the end, without hitting any matches.
    #[inline]
    pub fn potential(&self, Pos(i, _): Pos) -> Cost {
        self.potential[i as usize]
    }

    #[inline]
    pub fn potential_distance(&self, from: Pos, to: Pos) -> Cost {
        assert!(from.0 <= to.0);
        let end_i = self.seed_at(to).map_or(to.0, |s| s.start);
        self.potential[from.0 as usize] - self.potential[end_i as usize]
    }

    /// The seed covering a given position.
    #[inline]
    pub fn seed_at(&self, Pos(i, _): Pos) -> Option<&Seed> {
        match self.seed_at[i as usize] {
            Some(idx) => Some(&self.seeds[idx as usize]),
            None => None,
        }
    }

    /// The seed covering a given position.
    #[inline]
    pub fn seed_at_mut(&mut self, Pos(i, _): Pos) -> Option<&mut Seed> {
        match self.seed_at[i as usize] {
            Some(idx) => Some(&mut self.seeds[idx as usize]),
            None => None,
        }
    }

    /// The seed ending in the given position.
    #[inline]
    pub fn seed_ending_at(&self, Pos(i, _): Pos) -> Option<&Seed> {
        if i == 0 {
            None
        } else {
            match self.seed_at[i as usize - 1] {
                Some(idx) => Some(&self.seeds[idx as usize]),
                None => None,
            }
        }
    }

    #[inline]
    pub fn is_seed_start(&self, pos: Pos) -> bool {
        self.seed_at(pos).map_or(false, |s| pos.0 == s.start)
    }

    #[inline]
    pub fn is_seed_end(&self, pos: Pos) -> bool {
        self.seed_ending_at(pos).map_or(false, |s| pos.0 == s.end)
    }

    #[inline]
    pub fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        self.is_seed_start(pos) || self.is_seed_end(pos)
    }

    /// Apply the transformation for GCSH.
    // Units here are a lie. The output should have `Cost` instead of `Position`
    // units really.
    #[inline]
    pub fn transform(&self, pos @ Pos(i, j): Pos) -> Pos {
        let p = self.potential(pos);
        Pos(i - j - p, j - i - p)
    }

    /// Invert the transformation for GCSH.
    pub fn transform_back(&self, pos @ Pos(x, y): Pos) -> Pos {
        if pos == Pos(I::MAX, I::MAX) {
            return pos;
        }
        let p = -(x + y) / 2;
        let i = self.start_of_potential[p as usize];
        let diff = (x - y) / 2;
        let j = i - diff;
        debug_assert_eq!(pos, self.transform(Pos(i, j)));
        Pos(i, j)
    }
}
