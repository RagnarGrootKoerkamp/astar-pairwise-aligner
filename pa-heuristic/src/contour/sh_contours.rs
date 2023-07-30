use std::cmp::Ordering;

use super::*;
use crate::{prelude::*, seeds::Seeds, split_vec::SplitVec};

/// An arrow for SH implies f(start) >= f(end) + score.
#[derive(Clone, PartialEq, Debug)]
pub struct Arrow {
    pub start: I,
    pub end: I,
    pub score: MatchCost,
}

/// A Contours implementation based on Contour layers with queries in O(log(r)^2).
#[derive(Default, Debug)]
pub struct ShContours {
    /// index `[l][seed_idx]`: number of arrows for the seed with given `seed_idx` length `l`.
    num_arrows_per_length: Vec<Vec<usize>>,

    /// For each score `s`, this is the largest index `i` where total score `s` is still available.
    /// ```text
    /// layer_start[0] = n
    /// layer_start[1] = start of first seed with a match
    /// ...
    /// ```
    /// scores in this vector are decreasing, and the layer of position `i` is the
    /// largest index that has a score at least `i`.
    layer_starts: SplitVec<I>,
}

pub type Hint = Layer;

impl ShContours {
    // NOTE: Arrows must satisfy the following 'consistency' properties:
    // - If there is an arrow A->B of cost c>1, there is also an arrow A'->B of cost c-1, where A' is an indel away from A.
    pub fn new(seeds: &Seeds, arrows: impl IntoIterator<Item = Arrow>, max_len: I) -> Self {
        let mut layer_starts = SplitVec::default();
        // Layer 0 starts at the end of A.
        layer_starts.push(seeds.n() as _);
        for seed in seeds.seeds.iter().rev() {
            let seed_score = seed.seed_potential - seed.seed_cost;
            for _ in 0..seed_score {
                layer_starts.push(seed.start);
            }
        }

        // Count number of matches per seed of each length.
        // The max length is r.
        let mut num_arrows_per_length = vec![vec![0; seeds.seeds.len()]; max_len as usize + 1];

        for a in arrows {
            num_arrows_per_length[a.score as usize]
                [seeds.seed_at[a.start as usize].unwrap() as usize] += 1;
        }

        ShContours {
            num_arrows_per_length,
            layer_starts,
        }
    }

    /// The layer of position i is the largest index that has a score at least i.
    pub fn score(&self, pos: I) -> Cost {
        // FIXME: Make sure this is still up-to-date!
        self.layer_starts
            .binary_search_by(|start| {
                if *start >= pos {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .unwrap_err() as Cost
            - 1
    }

    /// Hint is the total number of layers _before_ the position, since this will change
    /// less than the number of layers _after_ the position (where more pruning typically happens).
    pub fn score_with_hint(&self, pos: I, hint: Hint) -> (Cost, Hint) {
        let hint_layer = (self.layer_starts.len() as Layer).saturating_sub(max(hint, 1));

        const SEARCH_RANGE: Layer = 5;

        // Do a linear search for some steps, starting at contour v.
        let layer = 'outer: {
            if self.layer_starts[hint_layer as usize] >= pos {
                // Go up.
                for layer in hint_layer + 1
                    ..min(
                        hint_layer + 1 + SEARCH_RANGE,
                        self.layer_starts.len() as Layer,
                    )
                {
                    if self.layer_starts[layer as usize] < pos {
                        break 'outer layer - 1;
                    }
                }
            } else {
                // Go down.
                for layer in (hint_layer.saturating_sub(SEARCH_RANGE)..hint_layer).rev() {
                    if self.layer_starts[layer as usize] >= pos {
                        break 'outer layer;
                    }
                }
            }

            // Fall back to binary search if not found close to the hint.
            self.score(pos) as Layer
        };
        assert!(pos <= self.layer_starts[layer as usize]);
        if layer as usize + 1 < self.layer_starts.len() {
            assert!(pos > self.layer_starts[layer as usize + 1]);
        }
        let hint = self.layer_starts.len() as Layer - layer;
        (layer as Cost, hint)
    }

    pub fn prune_with_hint(&mut self, seeds: &Seeds, a: Arrow, hint: Hint) -> Cost {
        let seed_idx = seeds.seed_at[a.start as usize].unwrap() as usize;
        let cnt = &mut self.num_arrows_per_length[a.score as usize][seed_idx];
        assert!(*cnt > 0, "Count of matches is already 0!");
        *cnt -= 1;
        if *cnt > 0 {
            // Remaining matches; nothing to prune.
            return 0;
        }
        // Make sure all larger lengths are also 0.
        for l in a.score as usize + 1..self.num_arrows_per_length.len() {
            if self.num_arrows_per_length[l][seed_idx] > 0 {
                return 0;
            }
        }
        // No seeds of length a.len remain, so we remove the layer.
        let mut removed = 0;
        let mut score = self.score_with_hint(a.start, hint).0;
        // NOTE: we don't actually have arrows of length 0.
        for l in (1..=a.score).rev() {
            if self.num_arrows_per_length[l as usize][seed_idx] > 0 {
                break;
            }
            assert_eq!(self.layer_starts[score as usize], a.start);
            self.layer_starts.remove(score as usize);
            removed += 1;
            score -= 1;
        }

        removed
    }
}
