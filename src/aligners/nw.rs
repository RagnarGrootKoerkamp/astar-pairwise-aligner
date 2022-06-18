use itertools::izip;

use super::cigar::{Cigar, CigarOp};
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
type Front<const N: usize> = Layers<N, Vec<Cost>>;

impl<const N: usize> NW<AffineCost<N>> {
    fn track_path(&self, layers: &mut Vec<Front<N>>, a: &Sequence, b: &Sequence) -> (PATH, Cigar) {
        let mut path: PATH = vec![];
        let mut cigar = Cigar::default();

        // The current position and affine layer.
        let mut i = a.len();
        let mut j = b.len();
        // None for main layer.
        let mut layer: Option<usize> = None;

        path.push((i, j));

        let mut save = |x: usize, y: usize, op: CigarOp| {
            cigar.push(op);
            if let Some(last) = path.last() {
                if *last == (x, y) {
                    return;
                }
            }
            path.push((x, y));
        };
        'path_loop: while i > 0 || j > 0 {
            if let Some(layer_idx) = layer {
                match self.cm.affine[layer_idx].affine_type {
                    InsertLayer => {
                        if layers[i].affine[layer_idx][j]
                            == layers[i - 1].affine[layer_idx][j] + self.cm.affine[layer_idx].extend
                        {
                            // insertion gap extention from current affine layer
                            i -= 1;
                            save(i, j, CigarOp::AffineInsertion(layer_idx));
                            continue 'path_loop;
                        } else {
                            assert_eq!(
                                layers[i].affine[layer_idx][j], layers[i - 1].m[j]
                                        + self.cm.affine[layer_idx].open
                                        + self.cm.affine[layer_idx].extend,"Path tracking error! No trace from insertion layer number {layer_idx}, coordinates {i}, {j}"
                            );
                            // opening new insertion gap from main layer
                            i -= 1;
                            save(i, j, CigarOp::AffineInsertion(layer_idx));
                            save(i, j, CigarOp::AffineOpen(layer_idx));
                            layer = None;
                            continue 'path_loop;
                        }
                    }
                    DeleteLayer => {
                        if layers[i].affine[layer_idx][j]
                            == layers[i].affine[layer_idx][j - 1] + self.cm.affine[layer_idx].extend
                        {
                            // deletion gap extention from current affine layer
                            j -= 1;
                            save(i, j, CigarOp::AffineDeletion(layer_idx));
                            continue 'path_loop;
                        } else {
                            assert_eq!(
                                layers[i].affine[layer_idx][j], layers[i].m[j-1]
                                        + self.cm.affine[layer_idx].open
                                        + self.cm.affine[layer_idx].extend,"Path tracking error! No trace from deletion layer number {layer_idx}, coordinates {i}, {j}"
                            );
                            // Open new deletion gap from main layer
                            j -= 1;
                            save(i, j, CigarOp::AffineDeletion(layer_idx));
                            save(i, j, CigarOp::AffineOpen(layer_idx));
                            layer = None;
                            continue 'path_loop;
                        }
                    }
                    _ => todo!(),
                };
            } else {
                if i > 0 && j > 0 {
                    if a[i - 1] == b[j - 1] && layers[i].m[j] == layers[i - 1].m[j - 1] {
                        //match
                        i -= 1;
                        j -= 1;
                        save(i, j, CigarOp::Match);
                        continue 'path_loop;
                    }
                    if let Some(sub) = self.cm.sub {
                        if layers[i].m[j] == layers[i - 1].m[j - 1] + sub {
                            //mismatch
                            i -= 1;
                            j -= 1;
                            save(i, j, CigarOp::Mismatch);
                            continue 'path_loop;
                        }
                    }
                }
                if i > 0 {
                    if let Some(ins) = self.cm.ins {
                        if layers[i].m[j] == layers[i - 1].m[j] + ins {
                            //insertion
                            i -= 1;
                            save(i, j, CigarOp::Insertion);
                            continue 'path_loop;
                        }
                    }
                }
                if j > 0 {
                    if let Some(del) = self.cm.del {
                        if layers[i].m[j] == layers[i].m[j - 1] + del {
                            //deletion
                            j -= 1;
                            save(i, j, CigarOp::Deletion);
                            continue 'path_loop;
                        }
                    }
                }
                // Affine layers check
                // NOTE: This loop does not change the position, only the layer.
                for (parent_layer, affine_layer) in layers[i].affine.iter().enumerate() {
                    if layers[i].m[j] == affine_layer[j] {
                        layer = Some(parent_layer);
                        save(i, j, CigarOp::AffineClose(parent_layer));
                        continue 'path_loop;
                    }
                }
            }
            panic!("Did not find parent on path!");
        }
        path.reverse();
        cigar.reverse();
        (path, cigar)
    }

    /// Computes the next layer (layer `i`) from the current one.
    /// `ca` is the `i-1`th character of sequence `a`.
    fn next_front(
        &self,
        i: usize,
        ca: u8,
        b: &Sequence,
        prev: &Front<N>,
        next: &mut Front<N>,
        v: &mut impl Visualizer,
    ) {
        v.expand(Pos(i as I, 0));
        // TODO: Instead of manually doing the first state, it is also possible
        // to simply add a buffer layer around the DP. The issue with that
        // however, is that we would need to prefix both sequences with the same
        // unique character to have a place to look at.

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
        let ref mut prev = Front::new(vec![INF; b.len() + 1]);
        let ref mut next = prev.clone();

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
            self.next_front(i, ca, b, prev, next, &mut NoVisualizer);
        }

        return next.m[b.len()];
    }

    fn visualize(
        &self,
        a: &Sequence,
        b: &Sequence,
        v: &mut impl Visualizer,
    ) -> (Cost, PATH, Cigar) {
        let ref mut fronts = vec![Front::<N>::new(vec![INF; b.len() + 1]); a.len() + 1];

        v.expand(Pos(0, 0));
        fronts[0].m[0] = 0;
        for j in 1..=b.len() {
            v.expand(Pos(0, j as I));
            // Initialize the main layer with linear deletions.
            fronts[0].m[j] = self.cm.del_or(INF, |del| j as Cost * del);

            // Initialize the affine deletion layers.
            let Layers { m, affine } = &mut fronts[0];
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
            let [prev, next] = &mut fronts[i-1..=i] else {unreachable!();};
            self.next_front(i, ca, b, &*prev, next, v);
        }

        let d = fronts[a.len()].m[b.len()];
        let tmp = self.track_path(fronts, a, b);
        return (d, tmp.0, tmp.1);
    }
}
