use super::{Aligner, NoVisualizer, Visualizer};
use crate::cost_model::*;
use crate::prelude::{Pos, Sequence};
use std::cmp::max;
use std::iter::zip;
use std::ops::ControlFlow;

pub struct DiagonalTransition<CM: CostModel> {
    cm: CM,
    /// We add a few buffer layers to the top of the table, to avoid the need
    /// to check that e.g. `s` is at least the substitution cost before
    /// making a substitution.
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
    left_buffer: usize,
    right_buffer: usize,
}

// The type for storing FR points.
type FR = i32;

impl<CM: CostModel> DiagonalTransition<CM> {
    pub fn new(cm: CM) -> Self {
        let top_buffer = max(cm.sub().unwrap_or(0), max(cm.ins(), cm.del())) as usize;
        assert!(top_buffer > 0);
        let left_buffer = max(
            // substitution, if allowed
            cm.sub().unwrap_or(0).div_ceil(cm.ins()),
            // number of insertions (left moves) done in range of looking one deletion (right move) backwards
            1 + cm.del().div_ceil(cm.ins()),
        ) as usize;
        // Idem.
        let right_buffer = max(
            // substitution, if allowed
            cm.sub().unwrap_or(0).div_ceil(cm.del()),
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

    fn init_fr(
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
            let f = Self::explore_diagonal(a, b, 0, &mut 0);
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

    /// Given two sequences, a diagonal and point on it, expand it to a FR point.
    #[inline]
    fn explore_diagonal(a: &Sequence, b: &Sequence, d: FR, fr: &mut FR) -> FR {
        let j = *fr - d;
        // TODO: compare 8 chars at a time using a u64, or even try SIMD.
        // TODO: The end check can be avoided by appending `#` and `$` to `a` and `b`.
        *fr += zip(&a[*fr as usize..], &b[j as usize..])
            .take_while(|(ca, cb)| ca == cb)
            .count() as FR;
        return *fr;
    }

    /// The first active diagonal for the given layer.
    #[inline]
    fn dmin_for_layer(&self, s: Cost) -> isize {
        -((s / self.cm.ins()) as isize)
    }
    /// The last active diagonal for the given layer.
    #[inline]
    fn dmax_for_layer(&self, s: Cost) -> isize {
        -((s / self.cm.del()) as isize)
    }

    #[inline]
    fn get_layer<'a>(&self, layers: &'a Vec<Vec<FR>>, s: Cost, cost: Cost) -> &'a Vec<FR> {
        &layers[self.top_buffer + s as usize - cost as usize]
    }

    #[inline]
    fn index_layer(&self, layer: &Vec<FR>, idx: isize, offset: isize) -> FR {
        layer[self.left_buffer + (idx - offset) as usize]
    }

    #[inline]
    fn index_layer_mut<'a>(&self, layer: &'a mut Vec<FR>, idx: isize, offset: isize) -> &'a mut FR {
        &mut layer[self.left_buffer + (idx - offset) as usize]
    }
}

impl DiagonalTransition<LinearCost> {
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
        // This may be a dummy layer when subs are not actually allowed.
        l_sub: &Vec<i32>,
        l_ins: &Vec<i32>,
        l_del: &Vec<i32>,
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
        let dmin_sub = self.cm.sub().map(|sub| self.dmin_for_layer(s - sub));
        let dmin_ins = self.dmin_for_layer(s - self.cm.ins());
        let dmin_del = self.dmin_for_layer(s - self.cm.del());
        // Simply loop over the entire range -- the boundaries are buffered
        // so no boundary conditions are needed.
        for d in dmin..=dmax {
            let f = max(
                // Substitution, if allowed
                if let Some(dmin_sub) = dmin_sub {
                    self.index_layer(l_sub, d, dmin_sub) + 1
                } else {
                    FR::MIN
                },
                max(
                    // Insertion
                    self.index_layer(l_ins, d + 1, dmin_ins),
                    // Deletion
                    self.index_layer(l_del, d - 1, dmin_del) + 1,
                ),
            );
            *self.index_layer_mut(next, d, dmin) = f;
            v.expand(Pos::from(f, f as isize + d));
        }
        // Stage 2: extend all points and check if we're done.
        for d in dmin..=dmax {
            let f = self.index_layer_mut(next, d, dmin);
            let f_old = *f;
            let f_new = Self::explore_diagonal(a, b, d as FR, f);
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
    type Params = ();

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
            let ref l_sub = fr[(num_layers + s as usize
                - self.cm.sub().unwrap_or_default() as usize)
                % num_layers];
            let ref l_ins = fr[(num_layers + s as usize - self.cm.ins() as usize) % num_layers];
            let ref l_del = fr[(num_layers + s as usize - self.cm.del() as usize) % num_layers];

            if self.next_layer(a, b, s, l_sub, l_ins, l_del, &mut next, &mut NoVisualizer) {
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
            let l_sub = self.get_layer(fr, s, self.cm.sub().unwrap_or(1));
            let l_ins = self.get_layer(fr, s, self.cm.ins());
            let l_del = self.get_layer(fr, s, self.cm.del());

            let mut next = vec![];

            if self.next_layer(a, b, s, l_sub, l_ins, l_del, &mut next, v) {
                // TODO: Reconstruct path.
                return s;
            }

            fr.push(next);
        }
    }
}
