//! TODO: Feature parity with BitFront:
//! - sparse memory/traceback
//! - reuse memory between runs
//! - incremental doubling
use super::*;
use crate::edit_graph::{AffineCigarOps, EditGraph};
use std::{
    array::from_fn,
    ops::{Index, IndexMut},
};

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

/// Costs per affine layer
pub struct AffineNwFront<const N: usize> {
    /// The main layer.
    m: Vec<Cost>,
    /// The affine layers.
    affine: [Vec<Cost>; N],
    j_range: JRange,
    fixed_j_range: Option<JRange>,
}

pub struct AffineNwFronts<'a, const N: usize> {
    trace: bool,
    a: Seq<'a>,
    b: Seq<'a>,
    cm: &'a AffineCost<N>,
    fronts: Vec<AffineNwFront<N>>,
    i_range: IRange,
}

impl<'a, const N: usize> IndexMut<usize> for AffineNwFronts<'a, N> {
    fn index_mut(&mut self, _index: usize) -> &mut Self::Output {
        todo!()
    }
}

impl<'a, const N: usize> Index<usize> for AffineNwFronts<'a, N> {
    type Output = AffineNwFront<N>;

    fn index(&self, _index: usize) -> &Self::Output {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AffineNwFrontsTag<const N: usize>;

impl<const N: usize> Default for AffineNwFront<N> {
    fn default() -> Self {
        Self {
            m: vec![],
            affine: from_fn(|_| vec![]),
            j_range: JRange(-1, -1),
            fixed_j_range: Some(JRange(-1, -1)),
        }
    }
}
impl<const N: usize> NwFront for AffineNwFront<N> {
    fn j_range(&self) -> JRange {
        self.j_range
    }
    fn fixed_j_range(&self) -> Option<JRange> {
        self.fixed_j_range
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
            fixed_j_range: None,
        }
    }
    fn first_col(cm: &AffineCost<N>, j_range: JRange) -> Self {
        let mut next = Self::new(j_range);
        next.fixed_j_range = Some(j_range);
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

    fn parent(&self, st: State, _g: &mut Cost) -> Option<(State, AffineCigarOps)> {
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

impl<const N: usize> NwFrontsTag<N> for AffineNwFrontsTag<N> {
    type Fronts<'a> = AffineNwFronts<'a, N>;
    const BLOCKSIZE: I = 1;
    fn new<'a>(
        &self,
        trace: bool,
        a: Seq<'a>,
        b: Seq<'a>,
        cm: &'a AffineCost<N>,
    ) -> Self::Fronts<'a> {
        Self::Fronts {
            fronts: vec![],
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

    fn init(&mut self, initial_j_range: JRange) {
        self.fronts = if self.trace {
            // A single vector element that will grow.
            vec![AffineNwFront::first_col(self.cm, initial_j_range)]
        } else {
            // Two vector elements that will be rotated.
            vec![
                AffineNwFront::default(),
                AffineNwFront::first_col(self.cm, initial_j_range),
            ]
        };
    }

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
    fn compute_next_block(
        &mut self,
        i_range: IRange,
        j_range: JRange,
        v: &mut impl VisualizerInstance,
    ) {
        v.expand_block_simple(Pos(i_range.0, j_range.0), Pos(i_range.len(), j_range.len()));
        // assert!(i_range.0 == self.i_range.1);
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

    fn set_last_front_fixed_j_range(&mut self, fixed_j_range: Option<JRange>) {
        self.fronts.last_mut().unwrap().fixed_j_range = fixed_j_range;
    }

    // Reusable helper implementation.
    fn trace(
        &mut self,
        _a: Seq,
        _b: Seq,
        from: State,
        mut to: State,
        _viz: &mut impl VisualizerInstance,
    ) -> AffineCigar {
        let mut cigar = AffineCigar::default();

        while to != from {
            let (parent, cigar_ops) = self.parent(to, &mut 0).unwrap();
            to = parent;
            for op in cigar_ops {
                if let Some(op) = op {
                    cigar.push_op(op);
                }
            }
        }
        cigar.reverse();
        cigar
    }
}
