use super::cigar::Cigar;
use super::diagonal_transition::Direction;
use super::edit_graph::{CigarOps, EditGraph, State};
use super::{exponential_search, Aligner, Path};
use super::{Seq, Sequence};
use crate::cost_model::*;
use crate::heuristic::{Heuristic, HeuristicInstance, ZeroCost};
use crate::prelude::Pos;
use crate::visualizer::{NoVisualizer, VisualizerT};
use itertools::chain;
use std::cmp::{max, min};
use std::ops::RangeInclusive;

/// Needleman-Wunsch aligner.
///
/// NOTE: Heuristics only support unit cost graph for now.
pub struct NW<CostModel, V: VisualizerT, H: Heuristic> {
    /// The cost model to use.
    pub cm: CostModel,

    /// When false, the band covers all states with distance <=s.
    /// When true, we only cover states with distance <=s/2.
    pub use_gap_cost_heuristic: bool,

    /// The heuristic to use.
    pub h: H,

    /// The visualizer to use.
    pub v: V,
}

impl<CostModel, V: VisualizerT, H: Heuristic> std::fmt::Debug for NW<CostModel, V, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NW")
            .field("use_gap_cost_heuristic", &self.use_gap_cost_heuristic)
            .field("h", &self.h)
            .finish()
    }
}

/// Type used for indexing sequences.
type Idx = isize;

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

/// The base vector M, and one vector per affine layer.
/// TODO: Possibly switch to a Vec<Layer> instead.
type Front<const N: usize> = super::front::Front<N, Cost, Idx>;
type Fronts<const N: usize> = super::front::Fronts<N, Cost, Idx>;

/// NW DP only needs the cell just left and above of the current cell.
const LEFT_BUFFER: Idx = 1;
const RIGHT_BUFFER: Idx = 1;

impl<const N: usize> NW<AffineCost<N>, NoVisualizer, ZeroCost> {
    pub fn new(cm: AffineCost<N>, use_gap_cost_heuristic: bool) -> Self {
        Self {
            cm,
            use_gap_cost_heuristic,
            h: ZeroCost,
            v: NoVisualizer,
        }
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic> NW<AffineCost<N>, V, H> {
    /// Computes the next front (front `i`) from the current one.
    ///
    /// `a` and `b` must be padded at the start by the same character.
    /// `i` and `j` will always be > 0.
    fn next_front(&mut self, i: Idx, a: Seq, b: Seq, prev: &Front<N>, next: &mut Front<N>) {
        for j in next.range().clone() {
            self.v.expand(Pos::from(i - 1, j - 1));
            EditGraph::iterate_parent_layers(&self.cm, |layer| {
                let mut best = INF;
                EditGraph::iterate_parents(
                    a,
                    b,
                    &self.cm,
                    /*greedy_matching=*/ false,
                    State::new(i, j, layer),
                    |di, dj, layer, edge_cost, _cigar_ops| {
                        if H::IS_DEFAULT {
                            best = min(
                                best,
                                if di == 0 {
                                    next.layer(layer)[j + dj] + edge_cost
                                } else {
                                    prev.layer(layer)[j + dj] + edge_cost
                                },
                            );
                        } else {
                            let parent_cost = if di == 0 {
                                next.layer(layer).get(j + dj)
                            } else {
                                prev.layer(layer).get(j + dj)
                            };
                            if let Some(cost) = parent_cost {
                                best = min(best, cost + edge_cost);
                            }
                        }
                    },
                );
                next.layer_mut(layer)[j] = best;
            });
        }
        self.v.new_layer();
    }

    /// The range of rows `j` to consider in column `i`, when the cost is bounded by `s_bound`.
    fn j_range(
        &self,
        a: Seq,
        b: Seq,
        h: &H::Instance<'_>,
        i: Idx,
        s_bound: Option<Cost>,
        prev: &Front<N>,
    ) -> RangeInclusive<Idx> {
        // Without a bound on the distance, we can notuse any heuristic.
        let Some(s) = s_bound else {
            return 1..=b.len() as Idx;
        };
        if H::IS_DEFAULT {
            // For the default heuristic, either use the full range of diagonals
            // covered by distance `s`, or do only the gap-cost to the end when
            // needed.
            let range = if self.use_gap_cost_heuristic {
                let d = b.len() as Idx - a.len() as Idx;
                // We subtract the cost needed to bridge the gap from the start to the end.
                let s = s - self.cm.gap_cost(Pos(0, 0), Pos::from_lengths(a, b));
                // Each extra diagonal costs one insertion and one deletion.
                let extra_diagonals = s / (self.cm.min_ins_extend + self.cm.min_del_extend);
                // NOTE: The range could be reduced slightly further by considering gap open costs.
                min(d, 0) - extra_diagonals as Idx..=max(d, 0) + extra_diagonals as Idx
            } else {
                -(self.cm.max_del_for_cost(s) as Idx)..=self.cm.max_ins_for_cost(s) as Idx
            };
            // crop
            max(i + *range.start(), 1)..=min(i + *range.end(), b.len() as Idx)
        } else {
            if i == 0 {
                0..=0
            } else {
                // Start with the range of the previous front.
                // Then:
                // Keep increasing the start while prev[start]+h() > s_bound.
                // Keep decreasing the end while prev[end]+h() > s_bound.
                // Keep increasing the end while prev[prev_end]+extend_cost*(end-prev_end)+h() > s_bound.
                let mut start = *prev.range().start();

                // To fix the padded character, we do max(start, 1) and max(i,1).
                // TODO: include the cost needed to transition from column `prev`/`i-1` to the current column.
                // h.h has (-1, -1) to offset the padding.
                while start < b.len() as Idx
                    && start <= prev.range().end() + 1
                    && prev.m()[start] + h.h(Pos::from(max(i, 1) - 1, max(start, 1) - 1)) > s
                {
                    start += 1;
                }
                start = max(start, 1);
                if start > prev.range().end() + 1 {
                    return start..=start - 1;
                }
                let prev_end = *prev.range().end();
                let prev_end_cost = prev.m()[prev_end];
                let mut end = prev_end;

                // Decrease end as needed.
                while end >= start && prev.m()[end] + h.h(Pos::from(max(i, 1) - 1, end - 1)) > s {
                    end -= 1;
                }

                // Increase end as needed.

                // We use the cheapest possible way to extend vertically.
                // h.h has (-1, -1) to offset the padding.
                while end < b.len() as Idx
                    && prev_end_cost
                        + self
                            .cm
                            .extend_cost(Pos::from(i - 1, prev_end), Pos::from(i, end + 1))
                        + h.h(Pos::from(i - 1, end + 1 - 1))
                        <= s
                {
                    end += 1;
                }
                start..=end
            }
        }
    }
}

fn pad(a: Seq) -> Sequence {
    chain!(b"^", a).copied().collect()
}

impl<const N: usize, V: VisualizerT, H: Heuristic> Aligner for NW<AffineCost<N>, V, H> {
    type CostModel = AffineCost<N>;

    type Fronts = Fronts<N>;

    type State = State;

    fn cost_model(&self) -> &Self::CostModel {
        &self.cm
    }

    fn parent(
        &self,
        a: Seq,
        b: Seq,
        fronts: &Self::Fronts,
        st: State,
        direction: Direction,
    ) -> Option<(State, CigarOps)> {
        assert!(direction == Direction::Forward);
        let cur_cost = fronts[st.i].layer(st.layer)[st.j];
        let mut parent = None;
        let mut cigar_ops: CigarOps = [None, None];
        EditGraph::iterate_parents(
            a,
            b,
            &self.cm,
            /*greedy_matching=*/ false,
            st,
            |di, dj, new_layer, cost, ops| {
                if parent.is_none()
                        // We use `get` to handle possible out-of-bound lookups.
                        && let Some(parent_cost) =
                            fronts[st.i + di].layer(new_layer).get(st.j + dj)
                        && cur_cost == parent_cost + cost
                    {
                        parent = Some(State::new(st.i + di, st.j + dj, new_layer));
                        cigar_ops = ops;
                    }
            },
        );
        Some((parent?, cigar_ops))
    }

    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        let cost = if self.use_gap_cost_heuristic || !H::IS_DEFAULT {
            exponential_search(
                self.cm.gap_cost(Pos(0, 0), Pos::from_lengths(a, b)),
                2.,
                |s| self.cost_for_bounded_dist(a, b, Some(s)).map(|c| (c, c)),
            )
            .1
        } else {
            self.cost_for_bounded_dist(a, b, None).unwrap()
        };
        self.v.last_frame(None);
        cost
    }

    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Path, Cigar) {
        let (cost, path, cigar) = if self.use_gap_cost_heuristic || !H::IS_DEFAULT {
            exponential_search(
                self.cm.gap_cost(Pos(0, 0), Pos::from_lengths(a, b)),
                2.,
                |s| {
                    self.align_for_bounded_dist(a, b, Some(s))
                        .map(|x @ (c, _, _)| (c, x))
                },
            )
            .1
        } else {
            self.align_for_bounded_dist(a, b, None).unwrap()
        };
        self.v.last_frame(Some(&path));
        (cost, path, cigar)
    }

    /// Test whether the cost is at most s.
    /// Returns None if cost > s, or the actual cost otherwise.
    fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, s_bound: Option<Cost>) -> Option<Cost> {
        // Pad both sequences.
        let ref a = pad(a);
        let ref b = pad(b);

        let ref mut h = self.h.build(
            &a[1..a.len()],
            &b[1..b.len()],
            &bio::alphabets::dna::alphabet(),
        );

        let ref mut prev = Front::default();
        let ref mut next = Front::new(
            INF,
            self.j_range(a, b, h, 0, s_bound, prev),
            LEFT_BUFFER,
            RIGHT_BUFFER,
        );
        next.m_mut()[0] = 0;
        for i in 1..=a.len() as Idx {
            std::mem::swap(prev, next);
            // Update front size.
            next.reset(
                INF,
                self.j_range(a, b, h, i, s_bound, prev),
                LEFT_BUFFER,
                RIGHT_BUFFER,
            );
            self.next_front(i, a, b, prev, next);
        }

        if let Some(&dist) = next.m().get(b.len() as Idx) {
            Some(dist)
        } else {
            None
        }
    }

    /// Tries to find a path with cost <= s.
    /// Returns None if cost > s, or the actual cost otherwise.
    fn align_for_bounded_dist(
        &mut self,
        a: Seq,
        b: Seq,
        s_bound: Option<Cost>,
    ) -> Option<(Cost, Path, Cigar)> {
        // Pad both sequences.
        let ref a = pad(a);
        let ref b = pad(b);

        // Build `h` for the original, unpadded strings.
        let ref mut h = self.h.build(
            &a[1..a.len()],
            &b[1..b.len()],
            &bio::alphabets::dna::alphabet(),
        );

        let mut fronts = Fronts::new(
            INF,
            // The fronts to create.
            0..=0 as Idx,
            // The range for each front.
            |i| self.j_range(a, b, h, i, s_bound, &Front::default()),
            0,
            0,
            LEFT_BUFFER,
            RIGHT_BUFFER,
        );
        fronts[0].m_mut()[0] = 0;

        for i in 1..=a.len() as Idx {
            let prev = &fronts[i - 1];
            let mut next = Front::new(
                INF,
                self.j_range(a, b, h, i, s_bound, prev),
                LEFT_BUFFER,
                RIGHT_BUFFER,
            );
            self.next_front(i, a, b, prev, &mut next);
            fronts.fronts.push(next);
        }

        if let Some(&dist) = fronts[a.len() as Idx].m().get(b.len() as Idx) {
            // We only track the actual path if `s` is small enough.
            if dist <= s_bound.unwrap_or(INF) {
                let (mut path, cigar) = self.trace(
                    a,
                    b,
                    &fronts,
                    State {
                        i: 1,
                        j: 1,
                        layer: None,
                    },
                    State {
                        i: a.len() as Idx,
                        j: b.len() as Idx,
                        layer: None,
                    },
                    Direction::Forward,
                );
                path.iter_mut().for_each(|pos| *pos = *pos - Pos(1, 1));
                return Some((dist, path, cigar));
            }
        }
        None
    }
}
