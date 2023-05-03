use std::{array::from_fn, cmp::min, ops::IndexMut};

use pa_affine_types::{AffineCost, State};
use pa_types::{Cost, Seq, I};

use crate::{
    edit_graph::{AffineCigarOps, EditGraph},
    nw::front::JRange,
};

use super::*;

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

/// Costs per affine layer
pub struct AffineNwFront<const N: usize> {
    /// The main layer.
    m: Vec<Cost>,
    /// The affine layers.
    affine: [Vec<Cost>; N],
    j_range: JRange,
}

pub struct AffineNwFronts<'a, const N: usize> {
    trace: bool,
    a: Seq<'a>,
    b: Seq<'a>,
    cm: &'a AffineCost<N>,
    fronts: Vec<AffineNwFront<N>>,
    i_range: IRange,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AffineNwFrontsTag<const N: usize>;

impl<const N: usize> Default for AffineNwFront<N> {
    fn default() -> Self {
        Self {
            m: vec![],
            affine: from_fn(|_| vec![]),
            j_range: JRange(0, 0),
        }
    }
}
impl<const N: usize> NwFront for AffineNwFront<N> {
    fn j_range(&self) -> JRange {
        self.j_range
    }

    fn index(&self, j: I) -> Cost {
        self.m[(j - self.j_range.0) as usize]
    }

    fn get(&self, j: I) -> Option<Cost> {
        self.m.get((j - self.j_range.0) as usize).copied()
    }
}
impl<const N: usize> AffineNwFront<N> {
    fn new(j_range: JRange) -> Self {
        Self {
            m: vec![INF; j_range.len() as usize],
            affine: from_fn(|_| vec![INF; j_range.len() as usize]),
            j_range,
        }
    }
    fn first_col(cm: &AffineCost<N>, j_range: JRange) -> Self {
        let mut next = Self::new(j_range);
        next.m[0] = 0;
        for j in next.j_range.0..=next.j_range.1 {
            EditGraph::iterate_layers(cm, |layer| {
                let mut best = INF;
                EditGraph::iterate_parents(
                    b"",
                    b"",
                    cm,
                    /*greedy_matching=*/ false,
                    State::new(0, j, layer),
                    |di, dj, layer, edge_cost, _cigar_ops| {
                        if di == 0 {
                            if let Some(cost) = next.get(layer, j + dj) {
                                best = min(best, cost + edge_cost);
                            }
                        }
                    },
                );
                if (layer, j) != (None, 0) {
                    *next.index_mut(layer, j) = best;
                }
            });
        }
        next
    }
    fn index(&self, layer: Option<usize>, j: I) -> Cost {
        let l = match layer {
            None => &self.m,
            Some(layer) => &self.affine[layer],
        };
        l[(j - self.j_range.0) as usize]
    }
    fn get(&self, layer: Option<usize>, j: I) -> Option<Cost> {
        let l = match layer {
            None => &self.m,
            Some(layer) => &self.affine[layer],
        };
        l.get((j - self.j_range.0) as usize).copied()
    }
    fn index_mut(&mut self, layer: Option<usize>, j: I) -> &mut Cost {
        let l = match layer {
            None => &mut self.m,
            Some(layer) => &mut self.affine[layer],
        };
        l.index_mut((j - self.j_range.0) as usize)
    }
}

impl<'a, const N: usize> AffineNwFronts<'a, N> {
    /// Computes the next front (front `i`) from the current one.
    fn next_front(&self, i: I, prev: &AffineNwFront<N>, next: &mut AffineNwFront<N>) {
        for j in next.j_range.0..=next.j_range.1 {
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
                            next.get(layer, j + dj)
                        } else {
                            prev.get(layer, j + dj)
                        };
                        if let Some(cost) = parent_cost {
                            best = min(best, cost + edge_cost);
                        }
                    },
                );
                *next.index_mut(layer, j) = best;
            });
        }
    }
}

impl<const N: usize> NwFrontsTag<N> for AffineNwFrontsTag<N> {
    type Fronts<'a> = AffineNwFronts<'a, N>;
    const BLOCKSIZE: I = 1;
    fn new<'a>(
        &self,
        trace: bool,
        a: Seq<'a>,
        b: Seq<'a>,
        cm: &'a AffineCost<N>,
        initial_j_range: JRange,
    ) -> Self::Fronts<'a> {
        Self::Fronts {
            fronts: if trace {
                // A single vector element that will grow.
                vec![AffineNwFront::first_col(cm, initial_j_range)]
            } else {
                // Two vector elements that will be rotated.
                vec![
                    AffineNwFront::default(),
                    AffineNwFront::first_col(cm, initial_j_range),
                ]
            },
            trace,
            a,
            b,
            cm,
            i_range: IRange(-1, 0),
        }
    }
}

impl<'a, const N: usize> NwFronts<N> for AffineNwFronts<'a, N> {
    type Front = AffineNwFront<N>;

    fn last_front(&self) -> &AffineNwFront<N> {
        &self.fronts.last().unwrap()
    }

    fn cm(&self) -> &AffineCost<N> {
        self.cm
    }

    fn last_i(&self) -> I {
        self.i_range.1
    }

    // TODO: Allow updating/overwriting as well.
    fn compute_next_block(&mut self, i_range: IRange, j_range: JRange) {
        assert!(i_range.0 == self.i_range.1);
        self.i_range.1 = i_range.1;

        for i in i_range.0..i_range.1 {
            if self.trace {
                let mut next = AffineNwFront::new(j_range);
                self.next_front(i + 1, &self.fronts[i as usize], &mut next);
                self.fronts.push(next);
            } else {
                let mut next = std::mem::take(&mut self.fronts[0]);
                let mut prev = std::mem::take(&mut self.fronts[1]);
                self.next_front(i + 1, &mut prev, &mut next);
                self.fronts[0] = prev;
                self.fronts[1] = next;
            }
        }
    }

    // TODO: Add `update_fronts` for local doubling.
    /// Compute fronts `i_range.start..i_range.end` with the given `j_range`.
    #[cfg(any())]
    fn update_fronts(&mut self, i_range: Range<I>, j_range: RangeInclusive<I>) {
        for _ in *self.fronts.range.end() + 1..i_range.end {
            self.fronts.push_default_front(0..=0);
        }
        for i in i_range.start..i_range.end {
            let next = &mut self.fronts[i];
            next.reset(INF, j_range.clone());
            let mut next = std::mem::take(&mut self.fronts[i]);
            self.next_front(i + 1, &self.fronts[i], &mut next);
            self.fronts[i] = next;
        }
    }

    fn parent(&self, st: State) -> Option<(State, AffineCigarOps)> {
        let cur_cost = self.fronts[st.i as usize].index(st.layer, st.j);
        let mut parent = None;
        let mut cigar_ops: AffineCigarOps = [None, None];
        EditGraph::iterate_parents(
            &self.a,
            &self.b,
            &self.cm,
            /*greedy_matching=*/ false,
            st,
            |di, dj, new_layer, cost, ops| {
                if parent.is_none()
                        // We use `get` to handle possible out-of-bound lookups.
                        && let Some(parent_cost) =
                            self.fronts[(st.i + di) as usize].get(new_layer, st.j + dj)
                        && cur_cost == parent_cost + cost
                    {
                        parent = Some(State::new(st.i + di, st.j + dj, new_layer));
                        cigar_ops = ops;
                    }
            },
        );
        Some((parent?, cigar_ops))
    }
}
