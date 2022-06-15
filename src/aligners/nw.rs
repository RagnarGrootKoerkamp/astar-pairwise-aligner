use itertools::izip;

use super::layer::Layers;
use super::NoVisualizer;
use super::{Aligner, Visualizer};
use crate::cost_model::*;
use crate::prelude::{Pos, Sequence, I};
use std::cmp::min;
use std::iter::zip;

pub type PATH = Vec<(usize, usize)>;
pub struct NW<CostModel> {
    pub cm: CostModel,
}

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

/// The base vector M, and one vector per affine layer.
/// TODO: Possibly switch to a Vec<Layer> instead.
type NWLayers<const N: usize> = Layers<N, Vec<Cost>>;

impl<const N: usize> NW<AffineCost<N>> {
    fn track_path(&self, layers: &mut Vec<NWLayers<N>>, a: &Sequence, b: &Sequence) -> PATH {
        let mut path: PATH = vec![];
        let mut save = |x: &usize, y: &usize| path.push((*x, *y));
        let mut i = a.len();
        let mut j = b.len();
        save(&i, &j);
        let mut layer: isize = -1; //shows in what layer we currently are. -1 - main layer; any other positive or zero index - index of affine layer

        while i > 0 || j > 0 {
            //i + j > 0 == i > 0 || j > 0
            if layer == -1 {
                if i > 0 && j > 0 {
                    //x*y > 0 == (x > 0 && y > 0)
                    if a[i - 1] == b[j - 1] && layers[i].m[j] == layers[i - 1].m[j - 1] {
                        //match
                        i -= 1;
                        j -= 1;
                        save(&i, &j);
                        continue;
                    }
                    if let Some(sub) = self.cm.sub {
                        if layers[i].m[j] == layers[i - 1].m[j - 1] + sub {
                            //mismatch
                            i -= 1;
                            j -= 1;
                            save(&i, &j);
                            continue;
                        }
                    }
                }
                if i > 0 {
                    if let Some(ins) = self.cm.ins {
                        if layers[i].m[j] == layers[i - 1].m[j] + ins {
                            //insertion
                            i -= 1;
                            save(&i, &j);
                            continue;
                        }
                    }
                }
                if j > 0 {
                    if let Some(del) = self.cm.del {
                        if layers[i].m[j] == layers[i].m[j - 1] + del {
                            //deletion
                            j -= 1;
                            save(&i, &j);
                            continue;
                        }
                    }
                }
                //affine layers check
                let mut tmp = 0;
                for affine_layer in &layers[i].affine {
                    if layers[i].m[j] == affine_layer[j] {
                        layer = tmp;
                        break;
                    }
                    tmp += 1;
                }
            } else {
                match self.cm.affine[layer as usize].affine_type {
                    InsertLayer => {
                        if layers[i].affine[layer as usize][j]
                            == layers[i - 1].affine[layer as usize][j]
                                + self.cm.affine[layer as usize].extend
                        {
                            //insertion gap extention from current affine layer
                            i -= 1;
                            save(&i, &j);
                            break;
                        } else {
                            // layers[i].affine[layer as usize][j]
                            // == layers[i - 1].m[j]
                            // + self.cm.affine[layer as usize].open + self.cm.affine[layer as usize].extend
                            //oppening new insertion gap from main layer
                            i -= 1;
                            save(&i, &j);
                            layer = -1;
                            break;
                        }
                    }
                    DeleteLayer => {
                        if layers[i].affine[layer as usize][j]
                            == layers[i].affine[layer as usize][j - 1]
                                + self.cm.affine[layer as usize].extend
                        {
                            //deletion gap extention from current affine layer
                            j -= 1;
                            save(&i, &j);
                            break;
                        } else {
                            //oppening new deletion gap from main layer
                            j -= 1;
                            save(&i, &j);
                            layer = -1;
                            break;
                        }
                    }
                    _ => todo!(),
                };
            }
        }

        path.reverse();
        path
    }

    /// Computes the next layer (layer `i`) from the current one.
    /// `ca` is the `i-1`th character of sequence `a`.
    fn next_layer(
        &self,
        i: usize,
        ca: u8,
        b: &Sequence,
        prev: &NWLayers<N>,
        next: &mut NWLayers<N>,
        v: &mut impl Visualizer,
    ) {
        v.expand(Pos(i as I, 0));
        // Initialize the first state by linear insertion.
        next.m[0] = self.cm.ins_or(INF, |ins| i as Cost * ins);
        // Initialize the first state by affine insertion.
        for (cm, layer) in zip(&self.cm.affine, &mut next.affine) {
            match cm.affine_type {
                InsertLayer => {
                    layer[0] = cm.open + i as Cost * cm.extend;
                    next.m[0] = min(next.m[0], layer[0]);
                }
                DeleteLayer => {
                    layer[0] = INF;
                }
                _ => todo!(),
            };
        }
        for (j0, &cb) in b.iter().enumerate() {
            // Change from 0 to 1 based indexing.
            let j = j0 + 1;

            // Compute all layers at (i, j).
            v.expand(Pos(i as I, j as I));

            // Main layer: substitutions and linear indels.
            let mut f = INF;
            // NOTE: When sub/ins/del is not allowed, we have to skip them.
            if ca == cb {
                f = prev.m[j - 1];
            } else {
                if let Some(sub) = self.cm.sub {
                    f = min(f, prev.m[j - 1] + sub);
                }
            }
            if let Some(ins) = self.cm.ins {
                f = min(f, prev.m[j] + ins);
            }
            if let Some(del) = self.cm.del {
                f = min(f, next.m[j - 1] + del);
            }

            // Affine layers
            // TODO: Swap the order of this for loop and the loop over j?
            for (cm, prev_layer, next_layer) in
                izip!(&self.cm.affine, &prev.affine, &mut next.affine)
            {
                match cm.affine_type {
                    InsertLayer => {
                        next_layer[j] =
                            min(prev_layer[j] + cm.extend, prev.m[j] + cm.open + cm.extend)
                    }
                    DeleteLayer => {
                        next_layer[j] = min(
                            next_layer[j - 1] + cm.extend,
                            next.m[j - 1] + cm.open + cm.extend,
                        )
                    }
                    _ => todo!(),
                };
                f = min(f, next_layer[j]);
            }

            next.m[j] = f;
        }
    }
}

impl<const N: usize> Aligner for NW<AffineCost<N>> {
    /// The cost-only version uses linear memory.
    fn cost(&self, a: &Sequence, b: &Sequence) -> Cost {
        let ref mut prev = NWLayers::new(vec![INF; b.len() + 1]);
        let ref mut next = NWLayers::new(vec![INF; b.len() + 1]);

        next.m[0] = 0;
        for j in 1..=b.len() {
            // Initialize the main layer with linear deletions.
            next.m[j] = self.cm.del_or(INF, |del| j as Cost * del);

            // Initialize the affine deletion layers.
            for (cm, next_layer) in zip(&self.cm.affine, &mut next.affine) {
                match cm.affine_type {
                    DeleteLayer => {
                        next_layer[j] = cm.open + j as Cost * cm.extend;
                    }
                    InsertLayer => {}
                    _ => todo!(),
                };
                next.m[j] = min(next.m[j], next_layer[j]);
            }
        }

        for (i0, &ca) in a.iter().enumerate() {
            // Convert to 1 based index.
            let i = i0 + 1;
            std::mem::swap(prev, next);
            self.next_layer(i, ca, b, prev, next, &mut NoVisualizer);
        }

        return next.m[b.len()];
    }

    fn visualize(&self, a: &Sequence, b: &Sequence, v: &mut impl Visualizer) -> (Cost, PATH) {
        let ref mut layers = vec![NWLayers::<N>::new(vec![INF; b.len() + 1]); a.len() + 1];

        v.expand(Pos(0, 0));
        layers[0].m[0] = 0;
        for j in 1..=b.len() {
            v.expand(Pos(0, j as I));
            // Initialize the main layer with linear deletions.
            layers[0].m[j] = self.cm.del_or(INF, |del| j as Cost * del);

            // Initialize the affine deletion layers.
            let Layers { m, affine } = &mut layers[0];
            for (costs, next_layer) in izip!(&self.cm.affine, affine) {
                match costs.affine_type {
                    DeleteLayer => {
                        next_layer[j] = costs.open + j as Cost * costs.extend;
                    }
                    InsertLayer => {}
                    _ => todo!(),
                };
                m[j] = min(m[j], next_layer[j]);
            }
        }

        for (i0, &ca) in a.iter().enumerate() {
            // Change from 0-based to 1-based indexing.
            let i = i0 + 1;
            let [prev, next] = &mut layers[i-1..=i] else {unreachable!();};
            self.next_layer(i, ca, b, &*prev, next, v);
        }

        // FIXME: Backtrack the optimal path.

        let d = layers[a.len()].m[b.len()];
        return (d, self.track_path(layers, a, b));
    }
}
