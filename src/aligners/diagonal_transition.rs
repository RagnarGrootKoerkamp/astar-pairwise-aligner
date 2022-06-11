use super::{Aligner, NoVisualizer, Visualizer};
use crate::cost_model::*;
use crate::prelude::{Pos, Sequence};
use std::cmp::max;
use std::iter::zip;
use std::ops::ControlFlow;

pub struct DiagonalTransition<CM: CostModel> {
    pub cm: CM,
    /// We add a few buffer layers to the top of the table, to avoid the need
    /// to check that e.g. `s` is at least the substitution cost before
    /// making a substitution.
    ///
    /// The value is the max of the substitution cost and all (affine) costs of a gap of size 1.
    pub top_buffer: usize,
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
    pub left_buffer: usize,
    pub right_buffer: usize,
}

// The type for storing FR points.
pub type FR = i32;

impl<CM: CostModel> DiagonalTransition<CM> {
    pub fn new(cm: CM) -> Self {
        // The maximum cost we look back:
        // max(substitution, indel, affine indel of size 1)
        let top_buffer = {
            let x = max(
                cm.sub().unwrap_or(0),
                max(cm.ins().unwrap_or(0), cm.del().unwrap_or(0)),
            );

            for cm in cm.layers {
                x = max(x, cm.open + cm.extend);
            }

            x as usize
        };

        let left_buffer = max(
            // substitution, if allowed
            cm.sub()
                .unwrap_or(0)
                .div_ceil(cm.ins().unwrap_or(Cost::MAX)),
            // number of insertions (left moves) done in range of looking one deletion (right move) backwards
            1 + cm.del().div_ceil(cm.ins()),
        ) as usize;
        // Idem.
        let right_buffer = max(
            // substitution, if allowed
            cm.sub()
                .unwrap_or(0)
                .div_ceil(cm.del().unwrap_or(Cost::MAX)),
            // number of deletions (right moves) done in range of looking one insertion (left move) backwards
            1 + cm.ins().div_ceil(cm.del()),
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
    pub fn extend_diagonal(a: &Sequence, b: &Sequence, d: FR, fr: &mut FR) -> FR {
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
    pub fn extend_diagonal_packed(a: &Sequence, b: &Sequence, d: FR, fr: &mut FR) -> FR {
        let j = *fr - d;

        // cast [u8] to [u64]
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
        // TODO: Leading or trailing zeros?
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
    pub fn dmin_for_layer(&self, s: Cost) -> isize {
        -((s.saturating_sub(self.cm.ins_open()) / self.cm.ins()) as isize)
    }
    /// The last active diagonal for the given layer.
    #[inline]
    pub fn dmax_for_layer(&self, s: Cost) -> isize {
        -((s.saturating_sub(self.cm.del_open()) / self.cm.del()) as isize)
    }

    #[inline]
    pub fn get_layer<'a>(&self, layers: &'a Vec<Vec<FR>>, s: Cost, cost: Cost) -> &'a Vec<FR> {
        &layers[self.top_buffer + s as usize - cost as usize]
    }

    #[inline]
    pub fn index_layer(&self, layer: &Vec<FR>, idx: isize, offset: isize) -> FR {
        layer[self.left_buffer + (idx - offset) as usize]
    }

    #[inline]
    pub fn index_layer_mut<'a>(
        &self,
        layer: &'a mut Vec<FR>,
        idx: isize,
        offset: isize,
    ) -> &'a mut FR {
        &mut layer[self.left_buffer + (idx - offset) as usize]
    }
}

impl DiagonalTransition<LinearCost> {
    pub fn init_fr(
        &self,
        a: &Vec<u8>,
        b: &Vec<u8>,
        v: &mut impl Visualizer,
    ) -> ControlFlow<(), Vec<Vec<i32>>> {
        let num_layers = self.top_buffer + 1;
        assert!(num_layers > self.cm.sub().unwrap_or_default() as usize);
        assert!(num_layers > self.cm.ins() as usize);
        assert!(num_layers > self.cm.del() as usize);
        let mut fr = vec![vec![FR::MIN; self.left_buffer + 1 + self.right_buffer]; num_layers];
        fr[num_layers - 1][self.left_buffer] = {
            // Find the first FR point, and return 0 if it already covers both sequences.
            let f = Self::extend_diagonal(a, b, 0, &mut 0);
            if f >= a.len() as FR && f >= b.len() as FR {
                return ControlFlow::Break(());
            }

            let mut p = Pos::from(0, 0);
            for _ in 0..=f {
                v.expand(p);
                p = p.add_diagonal(1);
            }
            f
        };
        ControlFlow::Continue(fr)
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
        l_ins: Option<&Vec<i32>>,
        l_del: Option<&Vec<i32>>,
        l_sub: Option<&Vec<i32>>,
        next: &mut Vec<i32>,
        v: &mut impl Visualizer,
    ) -> bool {
        // The first and last active diagonal for the current layer.
        let dmin = self.dmin_for_layer(s);
        let dmax = self.dmax_for_layer(s);
        next.resize(
            self.left_buffer + (dmax - dmin) as usize + 1 + self.right_buffer,
            FR::MIN,
        );
        // Offsets for each layer.
        let dmin_ins = self.cm.ins().map(|ins| self.dmin_for_layer(s - ins));
        let dmin_del = self.cm.del().map(|del| self.dmin_for_layer(s - del));
        let dmin_sub = self.cm.sub().map(|sub| self.dmin_for_layer(s - sub));
        // Simply loop over the entire range -- the boundaries are buffered
        // so no boundary conditions are needed.
        for d in dmin..=dmax {
            let f = max(
                // Substitution, if allowed
                l_sub.map_or(FR::MIN, |l| self.index_layer(l, d, dmin_sub.unwrap()) + 1),
                max(
                    // Insertion
                    l_ins.map_or(FR::MIN, |l| self.index_layer(l, d + 1, dmin_ins.unwrap())),
                    // Deletion
                    l_del.map_or(FR::MIN, |l| {
                        self.index_layer(l, d - 1, dmin_del.unwrap()) + 1
                    }),
                ),
            );
            *self.index_layer_mut(next, d, dmin) = f;
            v.expand(Pos::from(f, f as isize + d));
        }
        // Stage 2: extend all points and check if we're done.
        for d in dmin..=dmax {
            let f = self.index_layer_mut(next, d, dmin);
            let f_old = *f;
            let f_new = Self::extend_diagonal(a, b, d as FR, f);
            let mut p = Pos::from(f_old, f_old as isize + d);
            for _ in f_old..f_new {
                p = p.add_diagonal(1);
                v.expand(p);
            }
        }

        if self.index_layer(next, a.len() as isize - b.len() as isize, dmin) >= a.len() as FR {
            return true;
        }
        false
    }
}

impl Aligner for DiagonalTransition<LinearCost> {
    /// The cost-only version uses linear memory.
    ///
    /// In particular, the number of layers is max(sub, ins, del)+1.
    fn cost(&self, a: &Sequence, b: &Sequence, _params: Self::Params) -> Cost {
        let ref mut fr = match self.init_fr(a, b, &mut NoVisualizer) {
            ControlFlow::Continue(fr) => fr,
            ControlFlow::Break(_) => return 0,
        };
        let num_layers = fr.len();

        let mut s = 0;
        loop {
            s += 1;
            // Take the next layer out, and put it back later.
            // This is needed to avoid borrow problems.
            let mut next = std::mem::take(&mut fr[(num_layers + s as usize) % num_layers]);
            let l_ins = self
                .cm
                .ins()
                .map(|ins| &fr[(num_layers + s as usize - ins as usize) % num_layers]);
            let l_del = self
                .cm
                .del()
                .map(|del| &fr[(num_layers + s as usize - del as usize) % num_layers]);
            let l_sub = self
                .cm
                .sub()
                .map(|sub| &fr[(num_layers + s as usize - sub as usize) % num_layers]);

            if self.next_layer(a, b, s, l_ins, l_del, l_sub, &mut next, &mut NoVisualizer) {
                return s;
            }
            fr[(num_layers + s as usize) % num_layers] = next;
        }
    }

    // NOTE: DT does not explore states; it only expands them.
    fn visualize(
        &self,
        a: &Sequence,
        b: &Sequence,
        _params: Self::Params,
        v: &mut impl Visualizer,
    ) -> Cost {
        let ref mut fr = match self.init_fr(a, b, v) {
            ControlFlow::Continue(fr) => fr,
            ControlFlow::Break(_) => return 0,
        };

        v.expand(Pos(0, 0));

        let mut s = 0;
        loop {
            s += 1;
            let l_ins = self.cm.ins().map(|ins| self.get_layer(fr, s, ins));
            let l_del = self.cm.del().map(|del| self.get_layer(fr, s, del));
            let l_sub = self.cm.sub().map(|sub| self.get_layer(fr, s, sub));

            let mut next = vec![];

            if self.next_layer(a, b, s, l_ins, l_del, l_sub, &mut next, v) {
                // TODO: Reconstruct path.
                return s;
            }

            fr.push(next);
        }
    }
}
