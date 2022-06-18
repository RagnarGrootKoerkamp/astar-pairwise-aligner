use super::cigar::Cigar;
use super::layer::Layers;
use super::nw::{NW, PATH};
use super::NoVisualizer;
use super::{Aligner, Visualizer};
use crate::cost_model::*;
use crate::prelude::{Pos, Sequence};
use std::cmp::max;

pub struct Exponential<CostModel> {
    pub cm: CostModel,
    /// When false, it's like Dijkstra and all states of distance < s are covered.
    /// When true, a band of s/2 is sufficient to prove cost s.
    pub use_gapcost: bool,
}

type I = usize;

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

type Front<const N: usize> = super::front::Front<N, Cost, I>;

/// Settings for the algorithm, and derived constants.
///
/// TODO: Split into two classes: A static user supplied config, and an instance
/// to use for a specific alignment. Similar to Heuristic vs HeuristicInstance.
/// The latter can contain the sequences, direction, and other specifics.
pub struct ExpBand<CostModel> {
    /// The CostModel to use, possibly affine.
    cm: CostModel,

    /// When false, the band covers all states with distance <=s.
    /// When true, we only cover states with distance <=s/2.
    use_gap_cost_heuristic: bool,
}

impl<const N: usize> ExpBand<AffineCost<N>> {
    /// The first active row in column `i`, when searching up to distance `s`.
    #[inline]
    fn jmin(&self, a: &Sequence, b: &Sequence, i: I, s: Cost) -> I {
        if self.use_gap_cost_heuristic {
            let d = a.len() as isize - b.len() as isize;
            // FIXME: Get the rounding right here.
            i - ((s - d as Cost * self.cm.min_ins_extend)
                / (self.cm.min_del_extend + self.cm.min_ins_extend)) as I
        } else {
            i.saturating_sub((s / self.cm.min_ins_extend) as I)
        }
    }
    /// The last active column for the given front.
    #[inline]
    fn jmax(&self, a: &Sequence, b: &Sequence, i: I, s: Cost) -> I {
        if self.use_gap_cost_heuristic {
            let d = a.len() as isize - b.len() as isize;
            // FIXME: Get the rounding right here.
            i + ((s + d as Cost * self.cm.min_del_extend)
                / (self.cm.min_del_extend + self.cm.min_ins_extend)) as I
        } else {
            i + (s / self.cm.min_del_extend) as I
        }
    }
}

impl<const N: usize> ExpBand<AffineCost<N>> {
    /// Test whether the cost is at most s.
    /// Returns None or cost > s, or the actual cost otherwise.
    fn cost_for_band(&self, a: &Sequence, b: &Sequence, s: Cost) -> Option<Cost> {
        let range = self.jmin(a, b, 0, s)..=self.jmax(a, b, 0, s);
        let ref mut prev = Front {
            layers: Layers::new(vec![INF; range.end() - range.start() + 1]),
            range,
            offset: self.jmin(a, b, 0, s),
        };
        let ref mut next = prev.clone();
        next.m()[0] = 0;

        for (i0, &ca) in a.iter().enumerate() {
            // Convert to 1 based index.
            let i = i0 + 1;
            std::mem::swap(prev, next);
            // Update front parameters.
            next.range = self.jmin(a, b, i, s)..=self.jmax(a, b, i, s);
            NW { cm: self.cm }.next_front(i, ca, b, prev, next, &mut NoVisualizer);
        }

        if let Some(&dist) = next.m().get(b.len()) {
            if dist <= s {
                return Some(dist);
            }
        }
        None
    }
}

impl<const N: usize> Aligner for ExpBand<AffineCost<N>> {
    fn cost(&self, a: &Sequence, b: &Sequence) -> Cost {
        // Really, when self.gap_heuristic is false we should start at 0 instead of the difference.
        let mut s = if a.len() >= b.len() {
            (a.len() - b.len()) as Cost / self.cm.min_del_extend
        } else {
            (b.len() - a.len()) as Cost / self.cm.min_ins_extend
        };
        loop {
            if let Some(d) = self.cost_for_band(a, b, s) {
                break d;
            }
            s = max(2 * s, 1);
        }
    }

    fn visualize(
        &self,
        a: &Sequence,
        b: &Sequence,
        v: &mut impl Visualizer,
    ) -> (Cost, PATH, Cigar) {
        let Some(ref mut fronts) = self.init_fronts(a, b, v) else {
            return (0,vec![],Cigar::default());
        };

        v.expand(Pos(0, 0));

        let mut s = 0;
        loop {
            s += 1;

            // A temporary front without any content.
            let mut next = Front::<N> {
                layers: Layers::<N, Vec<Fr>>::new(vec![]),
                range: self.jmin(s)..=self.jmax(s),
                offset: self.left_buffer as Fr - self.jmin(s),
            };

            if self.next_front(a, b, fronts, &mut next, v) {
                // FIXME: Reconstruct path.
                return (s, vec![], Cigar::default());
            }

            fronts.push(next);
        }
    }
}
