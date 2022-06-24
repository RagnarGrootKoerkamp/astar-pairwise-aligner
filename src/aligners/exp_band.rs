use super::cigar::Cigar;
use super::nw::{Path, NW};
use super::{Aligner, VisualizerT};
use super::{NoVisualizer, Seq};
use crate::cost_model::*;
use crate::prelude::Pos;
use std::cmp::{max, min};
use std::ops::RangeInclusive;

pub struct Exponential<CostModel> {
    pub cm: CostModel,
    /// When false, it's like Dijkstra and all states of distance < s are covered.
    /// When true, a band of s/2 is sufficient to prove cost s.
    pub use_gapcost: bool,
}

/// Type used for indexing sequences.
type Idx = usize;

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

type Front<const N: usize> = super::front::Front<N, Cost, Idx>;

/// NW DP only needs the cell just left and above of the current cell.
const LEFT_BUFFER: Idx = 1;
const RIGHT_BUFFER: Idx = 1;

/// Settings for the algorithm, and derived constants.
///
/// TODO: Split into two classes: A static user supplied config, and an instance
/// to use for a specific alignment. Similar to Heuristic vs HeuristicInstance.
/// The latter can contain the sequences, direction, and other specifics.
pub struct ExpBand<CostModel> {
    /// The CostModel to use, possibly affine.
    pub cm: CostModel,

    /// When false, the band covers all states with distance <=s.
    /// When true, we only cover states with distance <=s/2.
    pub use_gap_cost_heuristic: bool,
}

impl<const N: usize> ExpBand<AffineCost<N>> {
    /// The first active row in column `i`, when searching up to distance `s`.
    fn j_range(&self, a: Seq, b: Seq, i: Idx, s: Cost) -> RangeInclusive<Idx> {
        let i = i as isize;
        let s = s as isize;
        let range = if self.use_gap_cost_heuristic {
            let d = b.len() as isize - a.len() as isize;
            let per_band_cost = (self.cm.min_ins_extend + self.cm.min_del_extend) as isize;
            if d > 0 {
                let reduced_s = s - d * self.cm.min_ins_extend as isize;
                -(reduced_s / per_band_cost)..=d + reduced_s / per_band_cost
            } else {
                let reduced_s = s - d * self.cm.min_del_extend as isize;
                d - (reduced_s / per_band_cost)..=reduced_s / per_band_cost
            }
        } else {
            -(s / self.cm.min_del_extend as isize)..=(s / self.cm.min_ins_extend as isize)
        };
        // crop
        max(i + *range.start(), 0) as Idx..=min(i + *range.end(), b.len() as isize) as Idx
    }
}

impl<const N: usize> ExpBand<AffineCost<N>> {
    /// Test whether the cost is at most s.
    /// Returns None if cost > s, or the actual cost otherwise.
    fn cost_for_band(&self, a: Seq, b: Seq, s: Cost) -> Option<Cost> {
        let range = self.j_range(a, b, 0, s);
        let ref mut prev = Front::new_with_buffer(INF, range, LEFT_BUFFER, RIGHT_BUFFER);
        let ref mut next = prev.clone();

        // TODO: Find a way to not have to manually process the first layer.
        // TODO: Reuse from NW.
        next.m_mut()[0] = 0;
        for j in next.range().clone() {
            // Initialize the main layer with linear insertions.
            if j == 0 {
                continue;
            }
            next.m_mut()[j] = self.cm.ins_or(INF, |ins| j as Cost * ins);

            // Initialize the affine insertion layers.
            for (layer_idx, cm) in self.cm.affine.iter().enumerate() {
                let (mut next_m, mut next_layer) = next.m_affine_mut(layer_idx);
                match cm.affine_type {
                    DeleteLayer => {}
                    InsertLayer => {
                        next_layer[j] = cm.open + j as Cost * cm.extend;
                    }
                    _ => todo!(),
                };
                next_m[j] = min(next_m[j], next_layer[j]);
            }
        }

        for (i0, &ca) in a.iter().enumerate() {
            // Convert to 1 based index.
            let i = i0 + 1;
            std::mem::swap(prev, next);
            // Update front size.
            next.reset_with_buffer(INF, self.j_range(a, b, i, s), LEFT_BUFFER, RIGHT_BUFFER);
            NW {
                cm: self.cm.clone(),
            }
            .next_front(i, ca, b, prev, next, &mut NoVisualizer);
        }

        if let Some(&dist) = next.m().get(b.len()) {
            if dist <= s {
                return Some(dist);
            }
        }
        None
    }

    /// Tries to find a path with cost <= s.
    /// Returns None if cost > s, or the actual cost otherwise.
    fn path_for_band(
        &self,
        a: Seq,
        b: Seq,
        s: Cost,
        v: &mut impl VisualizerT,
    ) -> Option<(Cost, Path, Cigar)> {
        let ref mut fronts: Vec<Front<N>> = (0..=a.len())
            .map(|i| {
                Front::new_with_buffer(INF, self.j_range(a, b, i, s), LEFT_BUFFER, RIGHT_BUFFER)
            })
            .collect();

        // TODO: Find a way to not have to manually process the first layer.
        v.expand(Pos(0, 0));
        fronts[0].m_mut()[0] = 0;
        for j in fronts[0].range().clone() {
            if j == 0 {
                continue;
            }
            v.expand(Pos(0, j as crate::prelude::I));
            // Initialize the main layer with linear deletions.
            fronts[0].m_mut()[j] = self.cm.ins_or(INF, |ins| j as Cost * ins);

            // Initialize the affine deletion layers.
            for (layer_idx, cm) in self.cm.affine.iter().enumerate() {
                let (mut next_m, mut next_layer) = fronts[0].m_affine_mut(layer_idx);
                match cm.affine_type {
                    DeleteLayer => {}
                    InsertLayer => {
                        next_layer[j] = cm.open + j as Cost * cm.extend;
                    }
                    _ => todo!(),
                };
                next_m[j] = min(next_m[j], next_layer[j]);
            }
        }

        for (i0, &ca) in a.iter().enumerate() {
            // Convert to 1 based index.
            let i = i0 + 1;
            let [prev, next] = &mut fronts[i-1..=i] else {unreachable!();};
            // FIXME: Take a ref instead of clone.
            NW {
                cm: self.cm.clone(),
            }
            .next_front(i, ca, b, prev, next, v);
        }

        if let Some(&dist) = fronts[a.len()].m().get(b.len()) {
            if dist <= s {
                let (path, cigar) = NW {
                    cm: self.cm.clone(),
                }
                .track_path(fronts, a, b);
                return Some((dist, path, cigar));
            }
        }
        None
    }

    fn exponential_search_s<T>(&self, a: Seq, b: Seq, mut f: impl FnMut(Cost) -> Option<T>) -> T {
        let mut s = self.cm.gap_cost(Pos(0, 0), Pos::from_lengths(a, b));
        // TODO: Fix the potential infinite loop here.
        loop {
            if let Some(d) = f(s) {
                return d;
            }
            s = max(2 * s, 1);
        }
    }
}

impl<const N: usize> Aligner for ExpBand<AffineCost<N>> {
    fn cost(&self, a: Seq, b: Seq) -> Cost {
        self.exponential_search_s(a, b, |s| self.cost_for_band(a, b, s))
    }

    fn visualize(&self, a: Seq, b: Seq, v: &mut impl VisualizerT) -> (Cost, Path, Cigar) {
        self.exponential_search_s(a, b, |s| self.path_for_band(a, b, s, v))
    }
}
