use itertools::izip;

use super::nw::Layers;
use super::{Aligner, NoVisualizer, Visualizer};
use crate::cost_model::*;
use crate::prelude::{Pos, Sequence};
use std::cmp::{max, min};
use std::iter::zip;

pub struct DiagonalTransition<CM: CostModel> {
    cm: CM,
    /// We add a few buffer layers to the top of the table, to avoid the need
    /// to check that e.g. `s` is at least the substitution cost before
    /// making a substitution.
    ///
    /// The value is the max of the substitution cost and all (affine) costs of a gap of size 1.
    top_buffer: usize,
    /// We also add a buffer to the left and right of each wavefront to reduce the need for if-statements.
    /// The size of the left buffer is the number of insertions that can be done for the cost of one deletion.
    /// We also account for high substitution costs.
    ///
    /// Example:
    /// ins = 2
    /// del = 3
    /// sub = 5
    ///
    /// moving right: deletion (we skip a character of a)
    /// moving left: insertion (we insert between two characters of a)
    ///
    ///  --> d
    /// |      x
    /// v   *..x.*    <- left buffer: ceil(sub/ins) = ceil(5/2) = 3, right buffer: ceil(sub/del) = ceil(5/3) = 2
    /// s     xx
    ///    *..xxx     <- 1 + ceil(del/ins) = 1 + ceil(3/2) = 3 buffer
    ///      xxxx.*   <- 1 + ceil(ins/del) = 1 + ceil(2/3) = 2 buffer
    ///      xxxx
    ///     XxxxxX    <- when computing these cells.
    ///
    /// For affine costs, we replace the numerator by the maximum open+extend cost, and the numerator by the minimum extend cost.
    left_buffer: usize,
    right_buffer: usize,
}

// The type for storing FR points.
type FR = i32;
type DTLayers<const N: usize> = Layers<N, Vec<FR>>;

#[derive(Clone, Copy)]
struct DTLayerRef<'a> {
    /// The (affine) layer to use.
    l: &'a Vec<FR>,
    /// The minimum value of `d` used in this layer.
    /// When `dmin=-5`, diagonal `0` is at index `left_buffer + 5.
    dmin: isize,
}

struct MutDTLayerRef<'a> {
    /// The (affine) layer to use.
    l: &'a mut Vec<FR>,
    /// The minimum value of `d` used in this layer.
    /// When `dmin=-5`, diagonal `0` is at index `left_buffer + 5.
    dmin: isize,
}

impl<CM: CostModel> DiagonalTransition<CM> {
    pub fn new(cm: CM) -> Self {
        // The maximum cost we look back:
        // max(substitution, indel, affine indel of size 1)
        let top_buffer = max(
            cm.sub().unwrap_or(0),
            max(cm.max_ins_cost(), cm.max_del_cost()),
        ) as usize;

        let left_buffer = max(
            // substitution, if allowed
            cm.sub()
                .unwrap_or(0)
                .div_ceil(cm.ins().unwrap_or(Cost::MAX)),
            // number of insertions (left moves) done in range of looking one deletion (right move) backwards
            1 + cm.max_del_cost().div_ceil(cm.min_ins_extend_cost()),
        ) as usize;
        // Idem.
        let right_buffer = max(
            // substitution, if allowed
            cm.sub()
                .unwrap_or(0)
                .div_ceil(cm.del().unwrap_or(Cost::MAX)),
            // number of deletions (right moves) done in range of looking one insertion (left move) backwards
            1 + cm.max_ins_cost().div_ceil(cm.min_del_extend_cost()),
        ) as usize;
        Self {
            cm,
            top_buffer,
            left_buffer,
            right_buffer,
        }
    }

    /// Given two sequences, a diagonal and point on it, expand it to a FR point.
    #[inline]
    fn extend_diagonal(a: &Sequence, b: &Sequence, d: FR, fr: &mut FR) -> FR {
        let j = *fr - d;

        // TODO: The end check can be avoided by appending `#` and `$` to `a` and `b`.
        *fr += zip(&a[*fr as usize..], &b[j as usize..])
            .take_while(|(ca, cb)| ca == cb)
            .count() as FR;
        return *fr;
    }

    /// Given two sequences, a diagonal and point on it, expand it to a FR point.
    ///
    /// This version compares one usize at a time.
    /// FIXME: This needs sentinels at the ends of the sequences to finish correctly.
    #[allow(unused)]
    #[inline]
    fn extend_diagonal_packed(a: &Sequence, b: &Sequence, d: FR, fr: &mut FR) -> FR {
        let j = *fr - d;

        // cast [u8] to *const usize
        let mut a_ptr = a[*fr as usize..].as_ptr() as *const usize;
        let mut b_ptr = b[j as usize..].as_ptr() as *const usize;
        let a_ptr_original = a_ptr;
        let cmp = loop {
            let cmp = unsafe { *a_ptr ^ *b_ptr };
            // TODO: Make the break the `likely` case?
            if cmp != 0 {
                break cmp;
            }
            unsafe {
                a_ptr = a_ptr.offset(1);
                b_ptr = b_ptr.offset(1);
            }
        };
        *fr += unsafe { a_ptr.offset_from(a_ptr_original) } as FR
            + (if cfg!(target_endian = "little") {
                cmp.trailing_zeros()
            } else {
                cmp.leading_zeros()
            } / u8::BITS) as FR;

        return *fr;
    }

    /// The first active diagonal for the given layer.
    #[inline]
    fn dmin_for_layer(&self, s: Cost) -> isize {
        let mut x = -(self.cm.ins_or(0, |ins| s / ins) as isize);
        for cm in self.cm.affine() {
            match cm.affine_type {
                InsertLayer => x = min(x, -((s.saturating_sub(cm.open) / cm.extend) as isize)),
                DeleteLayer => {}
                _ => todo!(),
            };
        }
        x
    }
    /// The last active diagonal for the given layer.
    #[inline]
    fn dmax_for_layer(&self, s: Cost) -> isize {
        let mut x = -(self.cm.del_or(0, |del| s / del) as isize);
        for cm in self.cm.affine() {
            match cm.affine_type {
                DeleteLayer => x = min(x, -((s.saturating_sub(cm.open) / cm.extend) as isize)),
                InsertLayer => {}
                _ => todo!(),
            };
        }
        x
    }

    #[inline]
    fn index_layer(&self, l: DTLayerRef, idx: isize) -> FR {
        l.l[self.left_buffer + (idx - l.dmin) as usize]
    }

    #[inline]
    fn index_layer_mut<'a, 'b>(&self, l: &'a mut MutDTLayerRef<'b>, idx: isize) -> &'a mut FR {
        &mut l.l[self.left_buffer + (idx - l.dmin) as usize]
    }
}

impl<const N: usize> DiagonalTransition<AffineCost<N>> {
    /// Returns None when the distance is 0.
    fn init_layers(
        &self,
        a: &Vec<u8>,
        b: &Vec<u8>,
        v: &mut impl Visualizer,
    ) -> Option<Vec<DTLayers<N>>> {
        let num_layers = self.top_buffer + 1;
        assert!(num_layers > self.cm.sub().unwrap_or_default() as usize);
        assert!(num_layers > self.cm.max_ins_cost() as usize);
        assert!(num_layers > self.cm.max_del_cost() as usize);
        let mut layers =
            vec![Layers::new(vec![FR::MIN; self.left_buffer + 1 + self.right_buffer]); num_layers];
        layers[num_layers - 1].m[self.left_buffer] = {
            // Find the first FR point, and return 0 if it already covers both sequences.
            let f = Self::extend_diagonal(a, b, 0, &mut 0);
            if f >= a.len() as FR && f >= b.len() as FR {
                return None;
            }

            // Expand points on the first run.
            let mut p = Pos::from(0, 0);
            for _ in 0..=f {
                v.expand(p);
                p = p.add_diagonal(1);
            }
            f
        };
        Some(layers)
    }

    /// Computes the next layer from the current one.
    /// `ca` is the `i`th character of sequence `a`.
    ///
    /// Returns `true` when the search completes.
    #[inline]
    fn next_layer(
        &self,
        a: &Vec<u8>,
        b: &Vec<u8>,
        s: u32,
        prev: &[DTLayers<N>],
        next: &mut DTLayers<N>,
        v: &mut impl Visualizer,
    ) -> bool {
        // The first and last active diagonal for the current layer.
        let dmin = self.dmin_for_layer(s);
        let dmax = self.dmax_for_layer(s);

        // Resize all affine layers.
        next.into_iter().for_each(|l| {
            l.fill(FR::MIN);
            l.resize(
                self.left_buffer + (dmax - dmin) as usize + 1 + self.right_buffer,
                FR::MIN,
            );
        });

        // Wrap the m layer in a mutable reference that contains the offset, for easier indexing.
        let ref mut next_m = MutDTLayerRef {
            l: &mut next.m,
            dmin,
        };

        // Get the layers `cost` from the top/last one.
        let get_layer = |cost| &prev[prev.len() - cost as usize];

        // The layer dependencies for the sub/ins/del operations, with offsets.
        let linear_layers = [self.cm.sub(), self.cm.ins(), self.cm.del()].map(|cost| {
            cost.map(|cost| DTLayerRef {
                l: &get_layer(cost).m,
                dmin: self.dmin_for_layer(s - cost),
            })
        });

        // The layer dependencies for each of the affine layers, for both gap extend and gap open operations.
        let affine_layers = {
            // array::map does not support enumeration, so we count manually instead.
            let mut affine_layer = -1 as isize;
            self.cm.affine.each_ref().map(|cm| {
                affine_layer += 1;
                [
                    // Gap extend dependency.
                    // Depends on the affine layer itself, at cost `cm.extend` back.
                    DTLayerRef {
                        l: &get_layer(cm.extend).affine[affine_layer as usize],
                        dmin: self.dmin_for_layer(s - cm.extend),
                    },
                    // Gap open dependency.
                    // Depends on the `m` layer, at cost `cm.open + cm.extend` back.
                    DTLayerRef {
                        l: &get_layer(cm.open + cm.extend).m,
                        dmin: self.dmin_for_layer(s - cm.open - cm.extend),
                    },
                ]
            })
        };

        // Loop over the entire dmin..=dmax range.
        // The boundaries are buffered so no boundary checks are needed.
        // TODO: Vectorize this loop.
        for d in dmin..=dmax {
            // The new value of next.m[d].
            let mut f = FR::MIN;
            // Substitution
            if let Some(l) = linear_layers[0] {
                f = max(f, self.index_layer(l, d) + 1);
            }
            // Insertion
            if let Some(l) = linear_layers[1] {
                f = max(f, self.index_layer(l, d + 1));
            }
            // Deletion
            if let Some(l) = linear_layers[2] {
                f = max(f, self.index_layer(l, d - 1) + 1);
            }
            // Affine layers
            for (cm, l, open_extend) in izip!(&self.cm.affine, &mut next.affine, &affine_layers) {
                // The new value of next.affine[..][d] = l[d].
                let mut affine_f = FR::MIN;
                // Handle insertion and deletion similar to before.
                match cm.affine_type {
                    InsertLayer => {
                        for l in *open_extend {
                            affine_f = max(affine_f, self.index_layer(l, d + 1));
                        }
                    }
                    DeleteLayer => {
                        for l in *open_extend {
                            affine_f = max(affine_f, self.index_layer(l, d - 1) + 1);
                        }
                    }
                };
                *self.index_layer_mut(&mut MutDTLayerRef { l, dmin }, d) = affine_f;
                f = max(f, affine_f);
            }
            *self.index_layer_mut(next_m, d) = f;

            v.expand(Pos::from(f, f as isize + d));
        }
        // Stage 2: extend all points in the m layer and check if we're done.
        for d in dmin..=dmax {
            let f = self.index_layer_mut(next_m, d);
            let f_old = *f;
            let f_new = Self::extend_diagonal(a, b, d as FR, f);
            let mut p = Pos::from(f_old, f_old as isize + d);
            for _ in f_old..f_new {
                p = p.add_diagonal(1);
                v.expand(p);
            }
        }

        if *self.index_layer_mut(next_m, a.len() as isize - b.len() as isize) >= a.len() as FR {
            return true;
        }
        false
    }
}

impl<const N: usize> Aligner for DiagonalTransition<AffineCost<N>> {
    /// The cost-only version uses linear memory.
    ///
    /// In particular, the number of layers is max(sub, ins, del)+1.
    fn cost(&self, a: &Sequence, b: &Sequence) -> Cost {
        let Some(ref mut layers) =
            self.init_layers(a, b, &mut NoVisualizer) else {return 0;};

        let mut s = 0;
        loop {
            s += 1;
            // Rotate all layers back by one, so that we can fill the new last layer.
            layers.rotate_left(1);
            let (next, layers) = layers.split_last_mut().unwrap();
            if self.next_layer(a, b, s, layers, next, &mut NoVisualizer) {
                return s;
            }
        }
    }

        let Some(ref mut layers) = self.init_layers(a, b, v) else {
    /// NOTE: DT does not explore states; it only expands them.
    fn visualize(&self, a: &Sequence, b: &Sequence, v: &mut impl Visualizer) -> Cost {
            return 0;
        };

        v.expand(Pos(0, 0));

        let mut s = 0;
        loop {
            s += 1;

            let mut next = DTLayers::<N>::new(vec![]);

            if self.next_layer(a, b, s, layers, &mut next, v) {
                // FIXME: Reconstruct path.
                return s;
            }

            layers.push(next);
        }
    }
}
