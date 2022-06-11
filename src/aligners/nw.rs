use itertools::izip;

use super::NoVisualizer;
use super::{nw::NW, Aligner, Visualizer};
use crate::cost_model::*;
use crate::prelude::{Pos, Sequence, I};
use std::cmp::min;

pub struct NW<CM: CostModel> {
    pub cm: CM,
}

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

struct Layers<const N: usize, T> {
    m: T,
    affine: [T; N],
}

impl<const N: usize, T> Layers<N, T> {
    fn new(m: T) -> Self
    where
        T: Clone,
    {
        let affine = [m; N];
        Self { m, affine }
    }
}

/// The base vector M, and one vector per affine layer.
/// TODO: Possibly switch to a Vec<Layer> instead.
type NWLayers<const N: usize> = Layers<N, Vec<Cost>>;

impl<const N: usize> NW<AffineCost<N>> {
    /// Computes the next layer from the current one.
    /// `ca` is the `i`th character of sequence `a`.
    fn next_layer_affine(
        &self,
        i: usize,
        ca: u8,
        b: &Sequence,
        prev: &NWLayers<N>,
        next: &mut NWLayers<N>,
        v: &mut impl Visualizer,
    ) {
        v.expand(Pos(i as I + 1, 0));
        // Initialize the first state by linear insertion.
        next.m[0] = self.cm.ins_or(INF, |ins| (i + 1) as Cost * ins);
        // Initialize the first state by affine insertion.
        for (costs, layer) in std::iter::zip(&self.cm.layers, &mut next.affine) {
            match costs.affine_type {
                Insert => {
                    layer[0] = costs.open + (i + 1) as Cost * costs.extend;
                    next.m[0] = min(next.m[0], layer[0]);
                }
                Delete => {
                    layer[0] = INF;
                }
            };
        }
        for (j, &cb) in b.iter().enumerate() {
            v.expand(Pos(i as I + 1, j as I + 1));

            // Main layer: substitution and linear indels
            next.m[j + 1] = min(
                self.cm.sub_cost_or(ca, cb, INF, |sub| prev.m[j] + sub),
                min(
                    self.cm.ins_or(INF, |ins| prev.m[j + 1] + ins),
                    self.cm.del_or(INF, |del| next.m[j] + del),
                ),
            );

            // Affine layers
            for (costs, prev_layer, next_layer) in
                izip!(&self.cm.layers, &prev.affine, &mut next.affine)
            {
                match costs.affine_type {
                    Insert => {
                        next_layer[j + 1] = min(
                            prev_layer[j + 1] + costs.extend,
                            prev.m[j + 1] + costs.open + costs.extend,
                        )
                    }
                    Delete => {
                        next_layer[j + 1] = min(
                            next_layer[j] + costs.extend,
                            next.m[j] + costs.open + costs.extend,
                        )
                    }
                };
                next.m[j + 1] = min(next.m[j + 1], next_layer[j + 1]);
            }
        }
    }
}

impl<const N: usize> Aligner for NW<AffineCost<N>> {
    /// The cost-only version uses linear memory.
    fn cost(&self, a: &Sequence, b: &Sequence, _params: Self::Params) -> Cost {
        let ref mut prev = NWLayers::new(vec![INF; b.len() + 1]);
        let ref mut next = NWLayers::new(vec![INF; b.len() + 1]);

        next.m[0] = 0;
        for j in 1..=b.len() {
            // Initialize the main layer with linear deletions.
            next.m[j] = self.cm.del_or(INF, |del| j as Cost * del);

            // Initialize the affine deletion layers.
            for (costs, next_layer) in izip!(&self.cm.layers, &mut next.affine) {
                match costs.affine_type {
                    Delete => {
                        next_layer[j] = costs.open + j as Cost * costs.extend;
                    }
                    Insert => {}
                };
                next.m[j] = min(next.m[j], next_layer[j]);
            }
        }

        for (i, &ca) in a.iter().enumerate() {
            std::mem::swap(prev, next);
            self.next_layer_affine(i, ca, b, prev, next, &mut NoVisualizer);
        }

        return next.m[b.len()];
    }

    fn visualize(
        &self,
        a: &Sequence,
        b: &Sequence,
        _params: Self::Params,
        v: &mut impl Visualizer,
    ) -> Cost {
        let ref mut layers = vec![NWLayers::new(vec![INF; b.len() + 1]); a.len() + 1];

        v.expand(Pos(0, 0));
        layers[0].m[0] = 0;
        for j in 1..=b.len() {
            v.expand(Pos(0, j as I));
            // Initialize the main layer with linear deletions.
            layers[0].m[j] = self.cm.del_or(INF, |del| j as Cost * del);

            // Initialize the affine deletion layers.
            for (costs, next_layer) in izip!(&self.cm.layers, &mut layers[0].affine) {
                match costs.affine_type {
                    Delete => {
                        next_layer[j] = costs.open + j as Cost * costs.extend;
                    }
                    Insert => {}
                };
                layers[0].m[j] = min(layers[0].m[j], next_layer[j]);
            }
        }

        for (i, &ca) in a.iter().enumerate() {
            let [prev, next] = &mut layers[i..i+2] else {unreachable!();};
            self.next_layer_affine(i, ca, b, prev, next, v);
        }

        // FIXME: Backtrack the optimal path.

        return layers[a.len()].m[b.len()];
    }
}
