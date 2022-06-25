use itertools::chain;

use super::cigar::Cigar;
use super::edit_graph::{EditGraph, State};
use super::{Aligner, VisualizerT};
use super::{Seq, Sequence};
use crate::cost_model::*;
use crate::prelude::Pos;
use std::cmp::{max, min};
use std::ops::RangeInclusive;

pub type Path = Vec<Pos>;
pub struct NW<CostModel, V: VisualizerT> {
    /// The cost model to use.
    pub cm: CostModel,

    /// When false, the band covers all states with distance <=s.
    /// When true, we only cover states with distance <=s/2.
    pub use_gap_cost_heuristic: bool,

    /// The visualizer to use.
    pub v: V,
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
/// After padding `a` and `b` at the front, an extra buffer isn't needed anymore.
const TOP_BUFFER: Idx = 0;

impl<const N: usize, V: VisualizerT> NW<AffineCost<N>, V> {
    fn track_path(&self, fronts: &Fronts<N>, a: Seq, b: Seq) -> (Path, Cigar) {
        let mut path: Path = vec![];
        let mut cigar = Cigar::default();

        let mut st = State::target(a, b);
        // Remove the last appended character.
        st.i -= 1;
        st.j -= 1;

        path.push(st.pos());

        let mut save = |st: State| {
            if let Some(last) = path.last() {
                if *last == st.pos() {
                    return;
                }
            }
            path.push(st.pos());
        };

        while st.i > 1 || st.j > 1 || st.layer.is_some() {
            let cur_cost = fronts[st.i].layer(st.layer)[st.j];
            let mut parent = None;
            EditGraph::iterate_parents(
                a,
                b,
                &self.cm,
                /*greedy_matching=*/ false,
                st,
                |di, dj, new_layer, cost, ops| {
                    if parent.is_none()
                        && cur_cost == fronts[st.i + di].layer(new_layer)[st.j + dj] + cost
                    {
                        parent = Some(State::new(st.i + di, st.j + dj, new_layer));
                        save(st);
                        for op in ops {
                            if let Some(op) = op {
                                cigar.push(op);
                            }
                        }
                    }
                },
            );

            if let Some(parent) = parent {
                st = parent
            } else {
                let State { i, j, layer } = st;
                panic!("Did not find parent on path!\nIn ({i}, {j}) at layer {layer:?} with cost ",);
            }
        }
        path.reverse();
        cigar.reverse();
        (path, cigar)
    }

    /// Computes the next front (front `i`) from the current one.
    ///
    /// `a` and `b` must be padded at the start by the same character.
    /// `i` and `j` will always be > 0.
    fn next_front(&mut self, i: Idx, a: Seq, b: Seq, prev: &Front<N>, next: &mut Front<N>) {
        for j in next.range().clone() {
            self.v.expand(Pos::from(i - 1, j - 1));
            EditGraph::iterate_layers(&self.cm, |layer| {
                let mut best = INF;
                EditGraph::iterate_parents(
                    a,
                    b,
                    &self.cm,
                    /*greedy_matching=*/ false,
                    State::new(i, j, layer),
                    |di, dj, layer, edge_cost, _cigar_ops| {
                        best = min(
                            best,
                            if di == 0 {
                                next.layer(layer)[j + dj] + edge_cost
                            } else {
                                prev.layer(layer)[j + dj] + edge_cost
                            },
                        );
                    },
                );
                next.layer_mut(layer)[j] = best;
            });
        }
    }

    /// The range of rows `j` to consider in column `i`, when the cost is bounded by `s_bound`.
    fn j_range(&self, a: Seq, b: Seq, i: Idx, s_bound: Option<Cost>) -> RangeInclusive<Idx> {
        let Some(mut s) = s_bound else {
            return 1..=b.len() as Idx;
        };
        let range = if self.use_gap_cost_heuristic {
            let d = b.len() as Idx - a.len() as Idx;
            // We subtract the cost needed to bridge the gap from the start to the end.
            s -= self.cm.gap_cost(Pos(0, 0), Pos::from_lengths(a, b));
            // Each extra diagonal costs one insertion and one deletion.
            let extra_diagonals = s / (self.cm.min_ins_extend + self.cm.min_del_extend);
            // NOTE: The range could be reduced slightly further by considering gap open costs.
            min(d, 0) - extra_diagonals as Idx..=max(d, 0) + extra_diagonals as Idx
        } else {
            -(self.cm.max_del_for_cost(s) as Idx)..=self.cm.max_ins_for_cost(s) as Idx
        };

        // crop
        max(i + *range.start(), 1)..=min(i + *range.end(), b.len() as Idx)
    }
}

fn pad(a: Seq) -> Sequence {
    chain!(b"^", a, b"$").copied().collect()
}

impl<const N: usize, V: VisualizerT> Aligner for NW<AffineCost<N>, V> {
    type CostModel = AffineCost<N>;

    fn cost_model(&self) -> &Self::CostModel {
        &self.cm
    }

    /// Test whether the cost is at most s.
    /// Returns None if cost > s, or the actual cost otherwise.
    fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, s_bound: Option<Cost>) -> Option<Cost> {
        // Pad both sequences.
        let ref a = pad(a);
        let ref b = pad(b);

        let ref mut prev = Front::default();
        let ref mut next = Front::new(
            INF,
            self.j_range(a, b, 0, s_bound),
            LEFT_BUFFER,
            RIGHT_BUFFER,
        );
        next.m_mut()[0] = 0;
        for i in 1..=a.len() as Idx {
            std::mem::swap(prev, next);
            // Update front size.
            next.reset(
                INF,
                self.j_range(a, b, i, s_bound),
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

        let mut fronts = Fronts::new(
            INF,
            // The fronts to create.
            0..=a.len() as Idx,
            // The range for each front.
            |i| self.j_range(a, b, i, s_bound),
            TOP_BUFFER,
            0,
            LEFT_BUFFER,
            RIGHT_BUFFER,
        );
        fronts[0].m_mut()[0] = 0;

        for i in 1..=a.len() {
            let i = i as Idx;
            let [prev, next] = &mut fronts[i-1..=i] else {unreachable!();};
            self.next_front(i, a, b, prev, next);
        }

        if let Some(&dist) = fronts[a.len() as Idx].m().get(b.len() as Idx) {
            // We only track the actual path if `s` is small enough.
            if dist <= s_bound.unwrap_or(INF) {
                let (path, cigar) = self.track_path(&fronts, a, b);
                return Some((dist, path, cigar));
            }
        }
        None
    }
}
