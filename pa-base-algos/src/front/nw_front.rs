use std::cmp::min;

use pa_affine_types::{AffineCost, State};
use pa_types::{Cost, Pos, Seq, I};

use crate::edit_graph::EditGraph;

use super::*;

/// The base vector M, and one vector per affine layer.
/// TODO: Possibly switch to a Vec<Layer> instead.
pub type NwFront<const N: usize> = Front<N, Cost, I>;

pub struct NwFronts<'a, const N: usize> {
    a: Seq<'a>,
    b: Seq<'a>,
    cm: &'a AffineCost<N>,
    pub fronts: Fronts<N, Cost, I>,
}

/// NW DP only needs the cell just left and above of the current cell.
const LEFT_BUFFER: I = 2;
const RIGHT_BUFFER: I = 2;

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

impl<'a, const N: usize> NwFronts<'a, N> {
    pub fn new(
        a: Seq<'a>,
        b: Seq<'a>,
        cm: &'a AffineCost<N>,
        initial_j_range: RangeInclusive<I>,
    ) -> Self {
        Self {
            a,
            b,
            cm,
            fronts: {
                let mut fronts = Fronts::new(
                    INF,
                    0..=0 as I,
                    |_i| initial_j_range.clone(),
                    0,
                    0,
                    LEFT_BUFFER,
                    RIGHT_BUFFER,
                );
                fronts[0].m_mut()[0] = 0;
                fronts
            },
        }
    }

    /// Compute fronts `i_range.start..i_range.end` with the given `j_range`.
    /// Stores all computed states.
    pub fn push_next_fronts(
        &mut self,
        i_range: Range<I>,
        j_range: RangeInclusive<I>,
        mut expand_callback: impl FnMut(Pos, Cost),
    ) {
        for i in i_range.start..i_range.end {
            let mut next = Front::new(INF, j_range.clone(), LEFT_BUFFER, RIGHT_BUFFER);
            self.next_front(i, &self.fronts[i - 1], &mut next, &mut expand_callback);
            self.fronts.fronts.push(next);
        }
    }

    /// Compute fronts `i_range.start..i_range.end` with the given `j_range`.
    /// Stores only the last front.
    pub fn rotate_next_fronts(
        &mut self,
        i_range: Range<I>,
        j_range: RangeInclusive<I>,
        mut expand_callback: impl FnMut(Pos, Cost),
    ) {
        for i in i_range.start..i_range.end {
            self.fronts.rotate(j_range.clone());
            let mut next = std::mem::take(&mut self.fronts[i]);
            self.next_front(i, &self.fronts[i - 1], &mut next, &mut expand_callback);
            self.fronts[i] = next;
        }
    }

    /// Compute fronts `i_range.start..i_range.end` with the given `j_range`.
    pub fn update_fronts(
        &mut self,
        i_range: Range<I>,
        j_range: RangeInclusive<I>,
        mut expand_callback: impl FnMut(Pos, Cost),
    ) {
        for _ in *self.fronts.range.end() + 1..i_range.end {
            self.fronts.push_default_front(0..=0);
        }
        for i in i_range.start..i_range.end {
            let next = &mut self.fronts[i];
            next.reset(INF, j_range.clone());
            let mut next = std::mem::take(&mut self.fronts[i]);
            self.next_front(i, &self.fronts[i - 1], &mut next, &mut expand_callback);
            self.fronts[i] = next;
        }
    }
    /// Computes the next front (front `i`) from the current one.
    ///
    /// `a` and `b` must be padded at the start by the same character.
    /// `i` and `j` will always be > 0.
    fn next_front(
        &self,
        i: I,
        prev: &NwFront<N>,
        next: &mut NwFront<N>,
        expand_callback: &mut impl FnMut(Pos, Cost),
    ) {
        for j in next.range().clone() {
            EditGraph::iterate_layers(&self.cm, |layer| {
                let mut best = INF;
                EditGraph::iterate_parents(
                    &self.a,
                    &self.b,
                    &self.cm,
                    /*greedy_matching=*/ false,
                    State::new(i, j, layer),
                    |di, dj, layer, edge_cost, _cigar_ops| {
                        let parent_cost = if di == 0 {
                            next.layer(layer).get(j + dj)
                        } else {
                            prev.layer(layer).get(j + dj)
                        };
                        if let Some(cost) = parent_cost {
                            best = min(best, cost + edge_cost);
                        }
                    },
                );
                next.layer_mut(layer)[j] = best;
            });
            expand_callback(Pos(i, j), next.m()[j]);
        }
    }

    pub fn last_front(&self) -> &NwFront<N> {
        &self.fronts.fronts.last().unwrap()
    }
}
