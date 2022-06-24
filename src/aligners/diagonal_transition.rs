//! Diagonal transition algorithm, with support for affine costs.
//!
//! This uses a more symmetric version of the recursion, where furthest reaching
//! (f.r.) points are stored by the sum of coordinates $i+j$ instead of the
//! usual $i$.
//! See here: https://research.curiouscoding.nl/posts/affine-gap-close-cost/#even-more-symmetry
//!
//! Terminology and notation:
//! - Front: the furthest reaching points for a fixed distance s.
//! - Layer: the extra I/D matrices needed for each affine indel.
//! - Run: a sequence of states on the same diagonal with matching characters in
//!   between, along which we greedy extend.
//! - Feather: a suboptimal branch of visited states growing off the main path.
//! - `s`: iterator over fronts; `s=0` is the first front at the top left.
//! - `idx`: iterator over the `N` affine layers.
//! - `d`: iterator over diagonals; `d=0` is the diagonal through the top left.
//!      `d=1` is above `d=0`. From `d=0` to `d=1` is a deletion.
//! - `dmin`/`dmax`: the inclusive range of diagonals processed for a given front.
//! - `{top,left,right}_buffer`: additional allocated fronts/diagonals that remove
//!   the need for boundary checks.
//! - `offset`: the index of diagonal `0` in a layer. `offset = left_buffer - dmin`.
//!
//!
use super::cigar::Cigar;
use super::nw::Path;
use super::{Aligner, Seq, VisualizerT};
use crate::cost_model::*;
use crate::prelude::Pos;
use std::cmp::{max, min};
use std::iter::zip;
use std::ops::RangeInclusive;

/// The type for storing furthest reaching points.
/// Sized, so that we can default them to -INF.
type Fr = i32;

type Front<const N: usize> = super::front::Front<N, Fr, Fr>;

/// GapOpen costs can be processed either when entering of leaving the gap.
pub enum GapVariant {
    GapOpen,
    GapClose,
}
use GapVariant::*;

/// The direction to run in.
pub enum Direction {
    Forward,
    Backward,
}
use Direction::*;

/// Settings for the algorithm, and derived constants.
///
/// TODO: Split into two classes: A static user supplied config, and an instance
/// to use for a specific alignment. Similar to Heuristic vs HeuristicInstance.
/// The latter can contain the sequences, direction, and other specifics.
pub struct DiagonalTransition<'a, CostModel, V: VisualizerT> {
    /// The CostModel to use, possibly affine.
    cm: CostModel,

    /// Whether to use gap-open or gap-close costs.
    /// https://research.curiouscoding.nl/notes/affine-gap-close-cost/
    gap_variant: GapVariant,

    v: &'a mut V,

    /// Whether to run the wavefronts forward or backward.
    /// Will be used for BiWFA.
    /// TODO: Move this setting elsewhere.
    /// TODO: Should this be a compile time setting instead?
    direction: Direction,

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
    /// For affine GapOpen costs, we replace the numerator by the maximum open+extend cost, and the numerator by the minimum extend cost.
    /// FIXME: For affine GapClose costs, we add the max open cost to the substitution cost.
    left_buffer: Fr,
    right_buffer: Fr,
}

/// Converts a pair of (diagonal index, furthest reaching) to a position.
/// TODO: Return Pos or usize instead?
fn fr_to_coords(d: Fr, f: Fr) -> (Fr, Fr) {
    ((f + d) / 2, (f - d) / 2)
}
fn fr_to_pos(d: Fr, f: Fr) -> Pos {
    Pos(
        ((f + d) / 2) as crate::prelude::I,
        ((f - d) / 2) as crate::prelude::I,
    )
}

impl<'a, const N: usize, V: VisualizerT> DiagonalTransition<'a, AffineCost<N>, V> {
    pub fn new_variant(
        cm: AffineCost<N>,
        gap_variant: GapVariant,
        direction: Direction,
        v: &'a mut V,
    ) -> Self {
        // The maximum cost we look back:
        // max(substitution, indel, affine indel of size 1)
        let top_buffer = max(
            max(
                cm.sub.unwrap_or(0),
                match gap_variant {
                    GapOpen => 0,
                    GapClose => max(cm.max_del_open, cm.max_ins_open),
                },
            ),
            match gap_variant {
                GapOpen => max(cm.max_ins_open_extend, cm.max_del_open_extend),
                GapClose => max(cm.max_ins_extend, cm.max_del_extend),
            },
        ) as usize;

        let left_buffer = max(
            // substitution, if allowed
            cm.sub
                .unwrap_or(match gap_variant {
                    GapOpen => 0,
                    GapClose => max(cm.max_del_open, cm.max_ins_open),
                })
                .div_ceil(cm.ins.unwrap_or(Cost::MAX)),
            // number of insertions (left moves) done in range of looking one deletion (right move) backwards
            1 + dbg!(match gap_variant {
                GapOpen => cm.max_del_open_extend,
                GapClose => cm.max_del_extend,
            })
            .div_ceil(dbg!(cm.min_ins_extend)),
        ) as Fr;
        // Idem.
        let right_buffer = max(
            // substitution, if allowed
            cm.sub
                .unwrap_or(match gap_variant {
                    GapOpen => 0,
                    GapClose => max(cm.max_del_open, cm.max_ins_open),
                })
                .div_ceil(cm.del.unwrap_or(Cost::MAX)),
            // number of deletions (right moves) done in range of looking one insertion (left move) backwards
            1 + match gap_variant {
                GapOpen => cm.max_ins_open_extend,
                GapClose => cm.max_ins_extend,
            }
            .div_ceil(cm.min_del_extend),
        ) as Fr;
        Self {
            cm,
            gap_variant,
            v,
            top_buffer,
            left_buffer,
            right_buffer,
            direction,
        }
    }

    pub fn new(cm: AffineCost<N>, v: &'a mut V) -> Self {
        Self::new_variant(cm, GapOpen, Forward, v)
    }

    /// Given two sequences, a diagonal and point on it, expand it to a FR point.
    fn extend_diagonal(&mut self, a: Seq, b: Seq, d: Fr, fr: &mut Fr) -> Fr {
        let (i, j) = fr_to_coords(d, *fr);
        //println!("FR to pos d {d} fr {fr} => pos({i}, {j})");
        if i as usize >= a.len() || j as usize >= b.len() {
            return *fr;
        }

        // TODO: The end check can be avoided by appending `#` and `$` to `a` and `b`.
        *fr += 2 * match self.direction {
            Forward => zip(a[i as usize..].iter(), b[j as usize..].iter())
                .take_while(|(ca, cb)| ca == cb)
                .count(),
            Backward => zip(
                a[..a.len() - i as usize].iter().rev(),
                b[..b.len() - j as usize].iter().rev(),
            )
            .take_while(|(ca, cb)| ca == cb)
            .count(),
        } as Fr;
        *fr
    }

    /// Given two sequences, a diagonal and point on it, expand it to a FR point.
    ///
    /// This version compares one usize at a time.
    /// FIXME: This needs sentinels at the starts/ends of the sequences to finish correctly.
    #[allow(unused)]
    fn extend_diagonal_packed(&mut self, a: Seq, b: Seq, d: Fr, fr: &mut Fr) -> Fr {
        let i = (*fr + d) / 2;
        let j = (*fr - d) / 2;

        // cast [u8] to *const usize, to compare 8 bytes at a time.
        let mut a_ptr = a[i as usize..].as_ptr() as *const usize;
        let mut b_ptr = b[j as usize..].as_ptr() as *const usize;
        let a_ptr_original = a_ptr;
        match self.direction {
            Forward => {
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
                *fr += 2
                    * (unsafe { a_ptr.offset_from(a_ptr_original) } as Fr
                        + (if cfg!(target_endian = "little") {
                            cmp.trailing_zeros()
                        } else {
                            cmp.leading_zeros()
                        } / u8::BITS) as Fr);
            }
            Backward => {
                let cmp = loop {
                    unsafe {
                        a_ptr = a_ptr.offset(-1);
                        b_ptr = b_ptr.offset(-1);
                    }
                    let cmp = unsafe { *a_ptr ^ *b_ptr };
                    // TODO: Make the break the `likely` case?
                    if cmp != 0 {
                        break cmp;
                    }
                };
                *fr += 2
                    * (unsafe { a_ptr_original.offset_from(a_ptr) } as Fr - 1
                        + (if cfg!(target_endian = "little") {
                            // NOTE: this is reversed from the forward case.
                            cmp.leading_zeros()
                        } else {
                            cmp.trailing_zeros()
                        } / u8::BITS) as Fr);
            }
        }

        *fr
    }

    fn extend(&mut self, front: &mut Front<N>, a: Seq, b: Seq) -> bool {
        for d in front.range().clone() {
            let fr = &mut front.m_mut()[d];
            let fr_old = *fr;
            //println!("Diagonal {d} fr old {fr_old}");
            let fr_new = self.extend_diagonal(a, b, d as Fr, fr);
            let mut p = fr_to_pos(d, fr_old);
            for _ in fr_old..fr_new {
                p = p.add_diagonal(1);
                self.v.expand(p);
            }
        }

        if front.range().contains(&(a.len() as Fr - b.len() as Fr))
            && front.m_mut()[a.len() as Fr - b.len() as Fr] >= (a.len() + b.len()) as Fr
        {
            return true;
        }
        false
    }

    /// The range of diagonals to consider for the given cost `s`.
    /// Computes the minimum and maximum possible diagonal reachable for this `s`.
    /// TODO: For simplicity, this does not take into account gap-open costs currently.
    fn d_range(&self, s: Cost) -> RangeInclusive<Fr> {
        let mut start = -(self.cm.ins_or(0, |ins| s / ins) as Fr);
        for cm in &self.cm.affine {
            match cm.affine_type {
                InsertLayer => {
                    start = min(
                        start,
                        -(s.saturating_sub(cm.open).div_floor(cm.extend) as Fr),
                    )
                }
                DeleteLayer => {}
                _ => todo!(),
            };
        }
        let mut end = self.cm.del_or(0, |del| s / del) as Fr;
        for cm in &self.cm.affine {
            match cm.affine_type {
                InsertLayer => {}
                DeleteLayer => end = max(end, s.saturating_sub(cm.open).div_floor(cm.extend) as Fr),
                _ => todo!(),
            };
        }

        start..=end
    }

    /// Returns None when the distance is 0.
    fn init_fronts(&mut self, a: Seq, b: Seq) -> Option<Vec<Front<N>>> {
        // Find the first FR point, and return 0 if it already covers both sequences (ie when they are equal).
        let f = self.extend_diagonal(a, b, 0, &mut 0);
        if f >= (a.len() + b.len()) as Fr {
            return None;
        }

        // Expand points on the first run.
        let mut p = Pos::from(0, 0);
        for _ in 0..=f {
            self.v.expand(p);
            p = p.add_diagonal(1);
        }

        // Initialize the fronts.
        let mut fronts =
            vec![
                Front::new_with_buffer(Fr::MIN, 0..=0, self.left_buffer, self.right_buffer);
                self.top_buffer + 1
            ];
        fronts[self.top_buffer].m_mut()[0] = f;
        Some(fronts)
    }

    /// Detects if there is a diagonal such that the two fronts meet/overlap.
    /// The overlap can be in any of the affine layers.
    /// Returns: None is no overlap was found.
    /// Otherwise:
    /// - the layer where overlap was found (None for M, Some(i) for affine layer),
    /// - the diagonal and FR for the forward direction,
    /// - the diagonal and FR for the backward direction.
    /// NOTE: the two FR indices may not correspond to the same character, in the case of overlapping greedy matches.
    #[allow(dead_code)]
    fn fronts_overlap(
        &self,
        a: Seq,
        b: Seq,
        forward: &mut Front<N>,
        backward: &mut Front<N>,
    ) -> Option<(Option<usize>, (Fr, Fr), (Fr, Fr))> {
        // NOTE: This is the same for the forward and reverse direction.
        let d_target = a.len() as Fr - b.len() as Fr;
        let f_target = (a.len() + b.len()) as Fr;
        let mirror = |d| d_target - d;
        let d_range = max(*forward.range().start(), mirror(*backward.range().end()))
            ..=min(*forward.range().end(), mirror(*backward.range().start()));
        // TODO: Provide an (internal) iterator over Layers from Front that merges these two cases.
        // M
        for d in d_range.clone() {
            if forward.m()[d] + backward.m()[mirror(d)] >= f_target {
                return Some((
                    None,
                    (d, forward.m()[d]),
                    (mirror(d), forward.m()[mirror(d)]),
                ));
            }
        }
        // Affine layers
        for i in 0..N {
            for d in d_range.clone() {
                if forward.affine(i)[d] + backward.affine(i)[mirror(d)] >= f_target {
                    return Some((
                        Some(i),
                        (d, forward.affine(i)[d]),
                        (mirror(d), forward.affine(i)[mirror(d)]),
                    ));
                }
            }
        }
        None
    }

    /// Computes the next layer from the current one.
    /// `ca` is the `i`th character of sequence `a`.
    ///
    /// NOTE: `next` must already have the right range set.
    ///
    /// Returns `true` when the search completes.
    fn next_front(&mut self, a: Seq, b: Seq, prev: &[Front<N>], next: &mut Front<N>) -> bool {
        // Get the front `cost` before the last one.
        let get_front = |cost| &prev[prev.len() - cost as usize];

        match self.gap_variant {
            GapOpen => {
                // Loop over the entire dmin..=dmax range.
                // The boundaries are buffered so no boundary checks are needed.
                // TODO: Vectorize this loop, or at least verify the compiler does this.
                // TODO: Loop over a positive range that does not need additional shifting?
                println!("d range: {:?}", next.range());
                for d in next.range().clone() {
                    // The new value of next.m[d].
                    let mut f = Fr::MIN;
                    // Affine layers
                    for layer_idx in 0..N {
                        let cm = &self.cm.affine[layer_idx];
                        let affine_f = match cm.affine_type {
                            InsertLayer => max(
                                // Gap open
                                get_front(cm.open + cm.extend).m()[d + 1] + 1,
                                // Gap extend
                                get_front(cm.extend).affine(layer_idx)[d + 1] + 1,
                            ),
                            DeleteLayer => max(
                                // Gap open
                                get_front(cm.open + cm.extend).m()[d - 1] + 1,
                                // Gap extend
                                get_front(cm.extend).affine(layer_idx)[d - 1] + 1,
                            ),
                            _ => todo!(),
                        };
                        next.affine_mut(layer_idx)[d] = affine_f;
                        // Gap close
                        f = max(f, affine_f);
                    }
                    // Substitution
                    if let Some(cost) = self.cm.sub {
                        f = max(f, get_front(cost).m()[d] + 2);
                    }
                    // Insertion
                    if let Some(cost) = self.cm.ins {
                        f = max(f, get_front(cost).m()[d + 1] + 1);
                    }
                    // Deletion
                    if let Some(cost) = self.cm.del {
                        f = max(f, get_front(cost).m()[d - 1] + 1);
                    }
                    next.m_mut()[d] = f;

                    self.v.expand(fr_to_pos(d, f));
                }
                // Extend all points in the m layer and check if we're done.
                self.extend(next, a, b)
            }
            GapClose => {
                // See https://research.curiouscoding.nl/notes/affine-gap-close-cost/.
                for d in next.range().clone() {
                    // The new value of next.m[d].
                    let mut f = Fr::MIN;
                    // Substitution
                    if let Some(cost) = self.cm.sub {
                        f = max(f, get_front(cost).m()[d] + 2);
                    }
                    // Insertion
                    if let Some(cost) = self.cm.ins {
                        f = max(f, get_front(cost).m()[d + 1] + 1);
                    }
                    // Deletion
                    if let Some(cost) = self.cm.del {
                        f = max(f, get_front(cost).m()[d - 1] + 1);
                    }
                    // Affine layers: Gap close
                    for idx in 0..N {
                        let cm = &self.cm.affine[idx];
                        match cm.affine_type {
                            InsertLayer | DeleteLayer => {
                                // Gap close
                                f = max(f, get_front(cm.open + cm.extend).m()[d])
                            }
                            _ => todo!(),
                        };
                    }
                    next.m_mut()[d] = f;

                    self.v.expand(fr_to_pos(d, f));
                }
                // Extend all points in the m layer and check if we're done.
                if self.extend(next, a, b) {
                    return true;
                }

                for d in next.range().clone() {
                    // Affine layers: Gap open/extend
                    for idx in 0..N {
                        let cm = &self.cm.affine[idx];
                        next.affine_mut(idx)[d] = match cm.affine_type {
                            // max(Gap open, Gap extend)
                            InsertLayer => max(
                                next.m()[d + 1] + 1,
                                get_front(cm.extend).affine(idx)[d + 1] + 1,
                            ),
                            // max(Gap open, Gap extend)
                            DeleteLayer => max(
                                next.m()[d - 1] + 1,
                                get_front(cm.extend).affine(idx)[d - 1] + 1,
                            ),
                            _ => todo!(),
                        };
                    }
                    // FIXME
                    //v.expand(fr_to_pos(d, f));
                }
                false
            }
        }
    }
}

impl<const N: usize, V: VisualizerT> Aligner for DiagonalTransition<'_, AffineCost<N>, V> {
    type CostModel = AffineCost<N>;

    fn cost_model(&self) -> &Self::CostModel {
        &self.cm
    }

    /// The cost-only version uses linear memory.
    ///
    /// In particular, the number of fronts is max(sub, ins, del)+1.
    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        let Some(ref mut fronts) =
            self.init_fronts(a, b) else {return 0;};

        let mut s = 0;
        loop {
            s += 1;
            println!("S: {s}");
            // Rotate all fronts back by one, so that we can fill the new last layer.
            fronts.rotate_left(1);
            let (next, fronts) = fronts.split_last_mut().unwrap();

            next.reset_with_buffer(
                Fr::MIN,
                self.d_range(s),
                self.left_buffer,
                self.right_buffer,
            );
            if self.next_front(a, b, fronts, next) {
                return s;
            }
        }
    }

    /// NOTE: DT does not explore states; it only expands them.
    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Path, Cigar) {
        let Some(ref mut fronts) = self.init_fronts(a, b) else {
            return (0,vec![],Cigar::default());
        };

        self.v.expand(Pos(0, 0));

        let mut s = 0;
        loop {
            s += 1;

            let mut next = Front::new_with_buffer(
                Fr::MIN,
                self.d_range(s),
                self.left_buffer,
                self.right_buffer,
            );
            if self.next_front(a, b, fronts, &mut next) {
                // FIXME: Reconstruct path.
                return (s, vec![], Cigar::default());
            }

            fronts.push(next);
        }
    }

    fn cost_for_bounded_dist(&mut self, _a: Seq, _b: Seq, _s_bound: Cost) -> Option<Cost> {
        todo!()
    }

    fn align_for_bounded_dist(
        &mut self,
        _a: Seq,
        _b: Seq,
        _s_bound: Cost,
    ) -> Option<(Cost, Path, Cigar)> {
        todo!()
    }
}
