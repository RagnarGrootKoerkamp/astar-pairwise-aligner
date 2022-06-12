use super::nw::Layers;
use super::{Aligner, NoVisualizer, Visualizer};
use crate::cost_model::*;
use crate::prelude::{Pos, Sequence};
use std::cmp::{max, min};
use std::iter::zip;
use std::ops::{Index, IndexMut};

/// The type for storing FR points.
/// Sizes, so that we can default them to -INF.
type FR = i32;
/// One front consists of N+1 layers of vectors of FR points.
/// TODO: Should we instead make dmin..=dmax ranges per affine layer?
#[derive(Clone)]
struct Front<const N: usize> {
    layers: Layers<N, Vec<FR>>,
    /// The minimal `d` computed for this layer.
    /// Will be negative.
    dmin: isize,
    /// The maximal `d` computed for this layer.
    dmax: isize,
    /// The offset we need to index each layer.
    /// Equals `left_buffer - dmin`, but stored separately to suppport indexing
    /// without needing extra context.
    offset: isize,
}

/// Indexing methods for `Front`.
impl<const N: usize> Front<N> {
    fn m(&self) -> Layer<'_> {
        Layer {
            l: &self.layers.m,
            offset: self.offset,
        }
    }
    fn affine(&self, index: usize) -> Layer<'_> {
        Layer {
            l: &self.layers.affine[index],
            offset: self.offset,
        }
    }
    fn m_mut(&mut self) -> MutLayer<'_> {
        MutLayer {
            l: &mut self.layers.m,
            offset: self.offset,
        }
    }
    fn affine_mut(&mut self, index: usize) -> MutLayer<'_> {
        MutLayer {
            l: &mut self.layers.affine[index],
            offset: self.offset,
        }
    }
}

/// A reference to a single layer of a single front.
/// Contains the offset needed to index it.
#[derive(Clone, Copy)]
struct Layer<'a> {
    /// The (affine) layer to use.
    l: &'a Vec<FR>,
    /// The offset we need to index this layer.
    /// Equals `left_buffer - front.dmin`. Stored separately to suppport indexing
    /// without needing extra context.
    offset: isize,
}
/// Indexing for a Layer.
impl<'a> Index<isize> for Layer<'a> {
    type Output = FR;

    fn index(&self, d: isize) -> &Self::Output {
        &self.l[(self.offset + d) as usize]
    }
}

/// A mutable reference to a single layer of a single front.
/// Contains the offset needed to index it.
struct MutLayer<'a> {
    /// The (affine) layer to use.
    l: &'a mut Vec<FR>,
    /// The offset we need to index this layer.
    /// Equals `left_buffer - dmin`. Stored separately to suppport indexing
    /// without needing extra context.
    offset: isize,
}
/// Indexing for a mutable Layer.
impl<'a> Index<isize> for MutLayer<'a> {
    type Output = FR;

    fn index(&self, d: isize) -> &Self::Output {
        &self.l[(self.offset + d) as usize]
    }
}
/// Indexing for a mutable Layer.
impl<'a> IndexMut<isize> for MutLayer<'a> {
    fn index_mut(&mut self, d: isize) -> &mut Self::Output {
        &mut self.l[(self.offset + d) as usize]
    }
}

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

/// Diagonal transition algorithm, with support for affine costs.
///
/// Terminology and notation:
/// - Front: the furthest reaching points for a fixed distance s.
/// - Layer: the extra I/D matrices needed for each affine indel.
/// - Run: a sequence of states on the same diagonal with matching characters in
///   between, along which we greedy extend.
/// - Feather: a suboptimal branch of visited states growing off the main path.
/// - `s`: iterator over fronts; `s=0` is the first front at the top left.
/// - `idx`: iterator over the `N` affine layers.
/// - `d`: iterator over diagonals; `d=0` is the diagonal through the top left.
///      `d=1` is above `d=0`. From `d=0` to `d=1` is a deletion.
/// - `dmin`/`dmax`: the inclusive range of diagonals processed for a given front.
/// - `{top,left,right}_buffer`: additional allocated fronts/diagonals that remove
///   the need for boundary checks.
/// - `offset`: the index of diagonal `0` in a layer. `offset = left_buffer - dmin`.
///
/// TODO: Split into two classes: A static user supplied config, and an instance
/// to use for a specific alignment. Similar to Heuristic vs HeuristicInstance.
/// The latter can contain the sequences, direction, and other specifics.
pub struct DiagonalTransition<CM: CostModel> {
    /// The CostModel to use, possibly affine.
    cm: CM,

    /// Whether to use gap-open or gap-close costs.
    /// https://research.curiouscoding.nl/notes/affine-gap-close-cost/
    gap_variant: GapVariant,

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
    left_buffer: usize,
    right_buffer: usize,
}

impl<CM: CostModel> DiagonalTransition<CM> {
    pub fn new_variant(cm: CM, gap_variant: GapVariant, direction: Direction) -> Self {
        // The maximum cost we look back:
        // max(substitution, indel, affine indel of size 1)
        let top_buffer = max(
            max(
                cm.sub().unwrap_or(0),
                match gap_variant {
                    GapOpen => 0,
                    GapClose => max(cm.max_del_open_cost(), cm.max_ins_open_cost()),
                },
            ),
            match gap_variant {
                GapOpen => max(cm.max_ins_open_extend_cost(), cm.max_del_open_extend_cost()),
                GapClose => max(cm.max_ins_extend_cost(), cm.max_del_extend_cost()),
            },
        ) as usize;

        let left_buffer = max(
            // substitution, if allowed
            cm.sub()
                .unwrap_or(match gap_variant {
                    GapOpen => 0,
                    GapClose => max(cm.max_del_open_cost(), cm.max_ins_open_cost()),
                })
                .div_ceil(cm.ins().unwrap_or(Cost::MAX)),
            // number of insertions (left moves) done in range of looking one deletion (right move) backwards
            1 + match gap_variant {
                GapOpen => cm.max_del_open_extend_cost(),
                GapClose => cm.max_del_extend_cost(),
            }
            .div_ceil(cm.min_ins_extend_cost()),
        ) as usize;
        // Idem.
        let right_buffer = max(
            // substitution, if allowed
            cm.sub()
                .unwrap_or(match gap_variant {
                    GapOpen => 0,
                    GapClose => max(cm.max_del_open_cost(), cm.max_ins_open_cost()),
                })
                .div_ceil(cm.del().unwrap_or(Cost::MAX)),
            // number of deletions (right moves) done in range of looking one insertion (left move) backwards
            1 + match gap_variant {
                GapOpen => cm.max_ins_open_extend_cost(),
                GapClose => cm.max_ins_extend_cost(),
            }
            .div_ceil(cm.min_del_extend_cost()),
        ) as usize;
        Self {
            cm,
            gap_variant,
            top_buffer,
            left_buffer,
            right_buffer,
            direction,
        }
    }

    pub fn new(cm: CM) -> Self {
        Self::new_variant(cm, GapOpen, Forward)
    }

    /// Given two sequences, a diagonal and point on it, expand it to a FR point.
    #[inline]
    fn extend_diagonal(&self, a: &Sequence, b: &Sequence, d: FR, fr: &mut FR) -> FR {
        let j = *fr - d;

        // TODO: The end check can be avoided by appending `#` and `$` to `a` and `b`.
        match self.direction {
            Forward => {
                *fr += zip(a[*fr as usize..].iter(), b[j as usize..].iter())
                    .take_while(|(ca, cb)| ca == cb)
                    .count() as FR
            }
            Backward => {
                *fr += zip(
                    a[..a.len() - *fr as usize].iter().rev(),
                    b[..b.len() - j as usize].iter().rev(),
                )
                .take_while(|(ca, cb)| ca == cb)
                .count() as FR
            }
        }
        *fr
    }

    /// Given two sequences, a diagonal and point on it, expand it to a FR point.
    ///
    /// This version compares one usize at a time.
    /// FIXME: This needs sentinels at the starts/ends of the sequences to finish correctly.
    #[allow(unused)]
    #[inline]
    fn extend_diagonal_packed(&self, a: &Sequence, b: &Sequence, d: FR, fr: &mut FR) -> FR {
        let j = *fr - d;

        // cast [u8] to *const usize
        match self.direction {
            Forward => {
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
            }
            Backward => {
                // Get a pointer ending at the given location.
                let mut a_ptr = a[*fr as usize..].as_ptr() as *const usize;
                let mut b_ptr = b[j as usize..].as_ptr() as *const usize;
                let a_ptr_original = a_ptr;
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
                *fr += unsafe { a_ptr.offset_from(a_ptr_original) } as FR - 1
                    + (if cfg!(target_endian = "little") {
                        // NOTE: this is reversed from the forward case.
                        cmp.leading_zeros()
                    } else {
                        cmp.trailing_zeros()
                    } / u8::BITS) as FR;
            }
        }

        *fr
    }

    /// The first active diagonal for the given layer.
    #[inline]
    fn dmin(&self, s: Cost) -> isize {
        let mut x = -(self.cm.ins_or(0, |ins| s / ins) as isize);
        for cm in self.cm.affine() {
            match cm.affine_type {
                InsertLayer => {
                    x = min(
                        x,
                        -(s.saturating_sub(cm.open).div_floor(cm.extend) as isize),
                    )
                }
                DeleteLayer => {}
                _ => todo!(),
            };
        }
        x
    }
    /// The last active diagonal for the given layer.
    #[inline]
    fn dmax(&self, s: Cost) -> isize {
        let mut x = -(self.cm.del_or(0, |del| s / del) as isize);
        for cm in self.cm.affine() {
            match cm.affine_type {
                DeleteLayer => x = min(x, s.saturating_sub(cm.open).div_floor(cm.extend) as isize),
                InsertLayer => {}
                _ => todo!(),
            };
        }
        x
    }
}

impl<const N: usize> DiagonalTransition<AffineCost<N>> {
    /// Returns None when the distance is 0.
    fn init_fronts(
        &self,
        a: &Sequence,
        b: &Sequence,
        v: &mut impl Visualizer,
    ) -> Option<Vec<Front<N>>> {
        // Find the first FR point, and return 0 if it already covers both sequences (ie when they are equal).
        let f = self.extend_diagonal(a, b, 0, &mut 0);
        if f >= a.len() as FR && f >= b.len() as FR {
            return None;
        }

        // Expand points on the first run.
        let mut p = Pos::from(0, 0);
        for _ in 0..=f {
            v.expand(p);
            p = p.add_diagonal(1);
        }

        // Initialize the fronts.
        let mut fronts = vec![
            Front {
                layers: Layers::new(vec![FR::MIN; self.left_buffer + 1 + self.right_buffer]),
                dmin: 0,
                dmax: 0,
                offset: self.left_buffer as isize,
            };
            self.top_buffer + 1
        ];
        fronts[self.top_buffer].m_mut()[0] = f;
        Some(fronts)
    }

    fn extend(
        &self,
        front: &mut Front<N>,
        a: &Sequence,
        b: &Sequence,
        v: &mut impl Visualizer,
    ) -> bool {
        for d in front.dmin..=front.dmax {
            let f = &mut front.m_mut()[d];
            let f_old = *f;
            let f_new = self.extend_diagonal(a, b, d as FR, f);
            let mut p = Pos::from(f_old, f_old as isize + d);
            for _ in f_old..f_new {
                p = p.add_diagonal(1);
                v.expand(p);
            }
        }

        if front.m_mut()[a.len() as isize - b.len() as isize] >= a.len() as FR {
            return true;
        }
        false
    }

    /// Detects if there is a diagonal such that the two fronts meet/overlap.
    /// The overlap can be in any of the affine layers.
    /// Returns: None is no overlap was found.
    /// Otherwise:
    /// - the layer where overlap was found (None for M, Some(i) for affine layer),
    /// - the diagonal and FR for the forward direction,
    /// - the diagonal and FR for the backward direction.
    /// NOTE: the two FR indices may not correspond to the same character, in the case of overlapping greedy matches.
    fn fronts_overlap(
        &self,
        a: &Sequence,
        b: &Sequence,
        forward: &mut Front<N>,
        backward: &mut Front<N>,
    ) -> Option<(Option<usize>, (isize, FR), (isize, FR))> {
        // NOTE: This is the same for the forward and reverse direction.
        let d_target = a.len() as isize - b.len() as isize;
        let n = a.len() as FR;
        let mirror = |d| d_target - d;
        let d_range =
            max(forward.dmin, mirror(backward.dmax))..=min(forward.dmax, mirror(backward.dmin));
        // TODO: Provide an (internal) iterator over Layers from Front that merges these two cases.
        // M
        for d in d_range.clone() {
            if forward.m()[d] + backward.m()[mirror(d)] >= n {
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
                if forward.affine(i)[d] + backward.affine(i)[mirror(d)] >= n {
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
    /// Returns `true` when the search completes.
    fn next_front(
        &self,
        a: &Sequence,
        b: &Sequence,
        prev: &[Front<N>],
        next: &mut Front<N>,
        v: &mut impl Visualizer,
    ) -> bool {
        // Resize all affine layers.
        (&mut next.layers).into_iter().for_each(|l| {
            l.fill(FR::MIN);
            l.resize(
                self.left_buffer + (next.dmax - next.dmin) as usize + 1 + self.right_buffer,
                FR::MIN,
            );
        });

        // Get the front `cost` before the last one.
        let get_front = |cost| &prev[prev.len() - cost as usize];

        match self.gap_variant {
            GapOpen => {
                // Loop over the entire dmin..=dmax range.
                // The boundaries are buffered so no boundary checks are needed.
                // TODO: Vectorize this loop.
                // TODO: Loop over a positive range that does not need additional shifting?
                for d in next.dmin..=next.dmax {
                    // The new value of next.m[d].
                    let mut f = FR::MIN;
                    // Affine layers
                    for idx in 0..N {
                        let cm = &self.cm.affine[idx];
                        let affine_f = match cm.affine_type {
                            InsertLayer => max(
                                // Gap open
                                get_front(cm.open + cm.extend).m()[d + 1],
                                // Gap extend
                                get_front(cm.extend).affine(idx)[d + 1],
                            ),
                            DeleteLayer => max(
                                // Gap open
                                get_front(cm.open + cm.extend).m()[d - 1] + 1,
                                // Gap extend
                                get_front(cm.extend).affine(idx)[d - 1] + 1,
                            ),
                            _ => todo!(),
                        };
                        next.affine_mut(idx)[d] = affine_f;
                        // Gap close
                        f = max(f, affine_f);
                    }
                    // Substitution
                    if let Some(cost) = self.cm.sub() {
                        f = max(f, get_front(cost).m()[d] + 1);
                    }
                    // Insertion
                    if let Some(cost) = self.cm.ins() {
                        f = max(f, get_front(cost).m()[d + 1]);
                    }
                    // Deletion
                    if let Some(cost) = self.cm.del() {
                        f = max(f, get_front(cost).m()[d - 1] + 1);
                    }
                    next.m_mut()[d] = f;

                    v.expand(Pos::from(f, f as isize + d));
                }
                // Extend all points in the m layer and check if we're done.
                self.extend(next, a, b, v)
            }
            GapClose => {
                // See https://research.curiouscoding.nl/notes/affine-gap-close-cost/.
                for d in next.dmin..=next.dmax {
                    // The new value of next.m[d].
                    let mut f = FR::MIN;
                    // Substitution
                    if let Some(cost) = self.cm.sub() {
                        f = max(f, get_front(cost).m()[d] + 1);
                    }
                    // Insertion
                    if let Some(cost) = self.cm.ins() {
                        f = max(f, get_front(cost).m()[d + 1]);
                    }
                    // Deletion
                    if let Some(cost) = self.cm.del() {
                        f = max(f, get_front(cost).m()[d - 1] + 1);
                    }
                    // Affine layers
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

                    v.expand(Pos::from(f, f as isize + d));
                }
                // Extend all points in the m layer and check if we're done.
                if self.extend(next, a, b, v) {
                    return true;
                }

                for d in next.dmin..=next.dmax {
                    // Affine layers
                    for idx in 0..N {
                        let cm = &self.cm.affine[idx];
                        *&mut next.affine_mut(idx)[d] = match cm.affine_type {
                            // max(Gap open, Gap extend)
                            InsertLayer => max(next.m()[d + 1], get_front(cm.extend).m()[d + 1]),
                            // max(Gap open, Gap extend)
                            DeleteLayer => {
                                max(next.m()[d - 1] + 1, get_front(cm.extend).m()[d] + 1)
                            }
                            _ => todo!(),
                        };
                    }
                    // FIXME
                    //v.expand(Pos::from(f, f as isize + d));
                }
                false
            }
        }
    }
}

impl<const N: usize> Aligner for DiagonalTransition<AffineCost<N>> {
    /// The cost-only version uses linear memory.
    ///
    /// In particular, the number of fronts is max(sub, ins, del)+1.
    fn cost(&self, a: &Sequence, b: &Sequence) -> Cost {
        let Some(ref mut fronts) =
            self.init_fronts(a, b, &mut NoVisualizer) else {return 0;};

        let mut s = 0;
        loop {
            s += 1;
            // Rotate all fronts back by one, so that we can fill the new last layer.
            fronts.rotate_left(1);
            let (next, fronts) = fronts.split_last_mut().unwrap();
            // Update front parameters.
            next.dmin = self.dmin(s);
            next.dmax = self.dmax(s);
            next.offset = self.left_buffer as isize - next.dmin;
            if self.next_front(a, b, fronts, next, &mut NoVisualizer) {
                return s;
            }
        }
    }

    /// NOTE: DT does not explore states; it only expands them.
    fn visualize(&self, a: &Sequence, b: &Sequence, v: &mut impl Visualizer) -> Cost {
        let Some(ref mut fronts) = self.init_fronts(a, b, v) else {
            return 0;
        };

        v.expand(Pos(0, 0));

        let mut s = 0;
        loop {
            s += 1;

            // A temporary front without any content.
            let mut next = Front::<N> {
                layers: Layers::<N, Vec<FR>>::new(vec![]),
                dmin: self.dmin(s),
                dmax: self.dmax(s),
                offset: self.left_buffer as isize - self.dmin(s),
            };

            if self.next_front(a, b, fronts, &mut next, v) {
                // FIXME: Reconstruct path.
                return s;
            }

            fronts.push(next);
        }
    }
}
