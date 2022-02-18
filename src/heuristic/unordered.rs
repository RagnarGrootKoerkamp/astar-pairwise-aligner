use std::cmp::Ordering;

use crate::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct UnorderedHeuristic {
    pub match_config: MatchConfig,
    pub pruning: bool,
}

impl Heuristic for UnorderedHeuristic {
    type Instance<'a> = UnorderedHeuristicI;

    fn build<'a>(
        &self,
        a: &'a Sequence,
        b: &'a Sequence,
        alphabet: &Alphabet,
    ) -> Self::Instance<'a> {
        UnorderedHeuristicI::new(a, b, alphabet, *self)
    }

    fn name(&self) -> String {
        "Unordrd".into()
    }

    fn params(&self) -> HeuristicParams {
        // TODO
        HeuristicParams {
            name: self.name(),
            ..Default::default()
        }
    }
}

pub struct UnorderedHeuristicI {
    params: UnorderedHeuristic,
    target: Pos,
    seeds: Seeds,
    /// Starts of the remaining matches, in reverse order.
    /// Pruning will happen mostly from back to the front.
    remaining_matches: SplitVec<I>,

    // TODO: Put statistics into a separate struct.
    num_pruned: usize,
}

type Hint = Cost;

impl UnorderedHeuristicI {
    fn new(a: &Sequence, b: &Sequence, alph: &Alphabet, params: UnorderedHeuristic) -> Self {
        let mut seeds = unordered_matches(a, b, alph, params.match_config);
        // Delete unused match data.
        seeds.matches.clear();
        // Contains start positions of all matches.
        let mut remaining_matches = SplitVec::default();
        {
            let mut seeds_with_matches = seeds
                .seeds
                .iter()
                .rev()
                .filter(|seed| seed.seed_cost < seed.seed_potential);
            let num_seeds_with_matches = seeds_with_matches.clone().count();
            remaining_matches.resize_with(1, || I::MAX);
            // TODO: Add sentinel value at the start.
            remaining_matches.resize_with(num_seeds_with_matches + 1, || {
                seeds_with_matches.next().unwrap().start
            });
        }

        if print() {
            println!("{:?}\n{remaining_matches:?}", seeds.seeds);
        }

        let h = UnorderedHeuristicI {
            params,
            target: Pos::from_length(a, b),
            seeds,
            remaining_matches,
            num_pruned: 0,
        };
        if print() {
            h.print(false, false);
        }
        h
    }

    /// The number of matches starting at or after q.
    fn value(&self, q: Pos) -> Cost {
        (self
            .remaining_matches
            .binary_search_by(|start| {
                if *start >= q.0 {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .unwrap_err() as Cost
            - 1)
            * (self.params.match_config.max_match_cost as Cost + 1)
    }

    /// Hint is the index from the end in self.remaining_matches.
    fn value_with_hint(&self, pos: Pos, hint: Hint) -> (Cost, Hint) {
        let v = (self.remaining_matches.len() as Cost).saturating_sub(max(hint, 1));

        const SEARCH_RANGE: Cost = 8;

        // Do a linear search for some steps, starting at contour v.
        let w = 'outer: {
            if self.remaining_matches[v as usize] >= pos.0 {
                // Go up.
                for v in v + 1..min(v + 1 + SEARCH_RANGE, self.remaining_matches.len() as Cost) {
                    if self.remaining_matches[v as usize] < pos.0 {
                        break 'outer v - 1;
                    }
                }
            } else {
                // Go down.
                for v in (v.saturating_sub(SEARCH_RANGE)..v).rev() {
                    if self.remaining_matches[v as usize] >= pos.0 {
                        break 'outer v;
                    }
                }
            }

            // Fall back to binary search if not found close to the hint.
            self.remaining_matches
                .binary_search_by(|start| {
                    if *start >= pos.0 {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                })
                .unwrap_err() as Cost
                - 1
        };
        //println!("{pos} : {v}, {w}, {:?}", self.remaining_matches);
        assert!(pos.0 <= self.remaining_matches[w as usize]);
        if w as usize + 1 < self.remaining_matches.len() {
            assert!(pos.0 > self.remaining_matches[w as usize + 1]);
        }
        (
            w * (self.params.match_config.max_match_cost as Cost + 1),
            self.remaining_matches.len() as Cost - w,
        )
    }
}

impl<'a> HeuristicInstance<'a> for UnorderedHeuristicI {
    /// The index of the next match, from the end of the splitvec.
    type Hint = Hint;

    fn h(&self, pos: Pos) -> Cost {
        let p = self.seeds.potential(pos);
        let m = self.value(pos);
        p - m
    }

    fn h_with_hint(&self, pos: Pos, hint: Self::Hint) -> (Cost, Self::Hint) {
        let p = self.seeds.potential(pos);
        let (m, h) = self.value_with_hint(pos, hint);
        (p - m, h)
    }

    fn root_potential(&self) -> Cost {
        self.seeds.potential[0]
    }

    fn is_seed_start_or_end(&self, pos: Pos) -> bool {
        self.seeds.is_seed_start_or_end(pos)
    }

    // Prune the match ending in `pos`.
    fn prune(&mut self, pos: Pos, hint: Self::Hint, seed_cost: MatchCost) -> Cost {
        if !self.params.pruning {
            return 0;
        }
        if pos.0 == 0 {
            return 0;
        }
        if seed_cost > self.params.match_config.max_match_cost {
            // The path through this seed is too expensive for there to be a match here.
            //println!("Skip {pos} / {seed_cost}");
            return 0;
        }
        // If there is no seed ending here, there is nothing to prune.
        // This can happen if a seed is not present because it matches multiple time.
        let s = if let Some(s) = self.seeds.seed_ending_at(pos) {
            s
        } else {
            return 0;
        };
        assert!(s.seed_cost < s.seed_potential);

        //println!("Prune {pos} / {seed_cost}");
        // +1 because pos is at the end of the match.
        let idx = self.remaining_matches.len() - self.value_with_hint(pos, hint).1 as usize + 1;
        // println!(
        //     "{pos} v {v}   values {:?} {:?} {:?}",
        //     self.remaining_matches.get(v.saturating_sub(1) as usize),
        //     self.remaining_matches.get(v as usize),
        //     self.remaining_matches.get(v as usize + 1)
        // );
        // println!("V: {v}");
        // println!("seed: {:?}", self.seeds.seed_ending_at(pos));
        // println!("seed: {:?}", self.seeds.seed_at(pos));
        // Check that we found the correct match, starting k before the current pos.
        if !self
            .remaining_matches
            .get(idx)
            .map_or(false, |&x| x == s.start)
        {
            // Match was already pruned, since it's not in remaining matches anymore.
            // This happens when greedy matching tries to prune multiple times.
            return 0;
        }
        // Remove the match.
        self.remaining_matches.remove(idx);
        self.num_pruned += 1;

        self.print(false, false);

        // TODO: Add Shifting.
        0
    }

    // All below here is using just the default implementation.

    fn explore(&mut self, _pos: Pos) {}

    fn stats(&self) -> HeuristicStats {
        let num_matches = self
            .seeds
            .seeds
            .iter()
            .filter(|seed| seed.seed_cost < seed.seed_potential)
            .count();
        HeuristicStats {
            num_seeds: self.seeds.seeds.len() as I,
            num_matches,
            num_filtered_matches: num_matches,
            matches: Default::default(),
            pruning_duration: Default::default(),
            num_prunes: self.num_pruned,
        }
    }

    fn print(&self, _transform: bool, wait_for_user: bool) {
        super::print::print(self, self.target, wait_for_user);
    }
}