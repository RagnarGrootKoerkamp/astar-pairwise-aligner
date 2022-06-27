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
use super::edit_graph::EditGraph;
use super::nw::Path;
use super::{Aligner, Seq};
use crate::cost_model::*;
use crate::heuristic::{Heuristic, HeuristicInstance, ZeroCost};
use crate::prelude::Pos;
use crate::visualizer::{NoVisualizer, VisualizerT};
use std::cmp::{max, min};
use std::iter::zip;
use std::ops::RangeInclusive;

/// The type for storing furthest reaching points.
/// Sized, so that we can default them to -INF.
pub type Fr = i32;

type Front<const N: usize> = super::front::Front<N, Fr, Fr>;
type Fronts<const N: usize> = super::front::Fronts<N, Fr, Fr>;

/// The direction to run in.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}
use Direction::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GapCostHeuristic {
    Enable,
    Disable,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HistoryCompression {
    Enable,
    Disable,
}

/// Settings for the algorithm, and derived constants.
///
/// TODO: Split into two classes: A static user supplied config, and an instance
/// to use for a specific alignment. Similar to Heuristic vs HeuristicInstance.
/// The latter can contain the sequences, direction, and other specifics.
pub struct DiagonalTransition<CostModel, V: VisualizerT, H: Heuristic> {
    /// The CostModel to use, possibly affine.
    cm: CostModel,

    /// Whether to use the gap heuristic to the end to reduce the number of diagonals considered.
    use_gap_cost_heuristic: GapCostHeuristic,

    h: H,

    /// When true, calls to `align` store a compressed version of the full 'history' of visited states.
    #[allow(unused)]
    history_compression: HistoryCompression,

    v: V,

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
    top_buffer: Fr,
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
#[inline]
fn fr_to_coords(d: Fr, fr: Fr) -> (Fr, Fr) {
    ((fr + d) / 2, (fr - d) / 2)
}
#[inline]
fn fr_to_pos(d: Fr, fr: Fr) -> Pos {
    Pos(
        ((fr + d) / 2) as crate::prelude::I,
        ((fr - d) / 2) as crate::prelude::I,
    )
}

/// Given two sequences, a diagonal and point on it, expand it to a FR point.
fn extend_diagonal(direction: Direction, a: Seq, b: Seq, d: Fr, mut fr: Fr) -> Fr {
    let (i, j) = fr_to_coords(d, fr);
    if i as usize >= a.len() || j as usize >= b.len() {
        return fr;
    }

    // TODO: The end check can be avoided by appending `#` and `$` to `a` and `b`.
    match direction {
        Forward => {
            fr += 2 * zip(a[i as usize..].iter(), b[j as usize..].iter())
                .take_while(|(ca, cb)| ca == cb)
                .count() as Fr
        }
        Backward => {
            fr -= 2 * zip(a[..i as usize].iter().rev(), b[..j as usize].iter().rev())
                .take_while(|(ca, cb)| ca == cb)
                .count() as Fr
        }
    };
    fr
}

/// Given two sequences, a diagonal and point on it, expand it to a FR point.
///
/// This version compares one usize at a time.
/// FIXME: This needs sentinels at the starts/ends of the sequences to finish correctly.
#[allow(unused)]
fn extend_diagonal_packed(direction: Direction, a: Seq, b: Seq, d: Fr, mut fr: Fr) -> Fr {
    let i = (fr + d) / 2;
    let j = (fr - d) / 2;

    // cast [u8] to *const usize, to compare 8 bytes at a time.
    let mut a_ptr = a[i as usize..].as_ptr() as *const usize;
    let mut b_ptr = b[j as usize..].as_ptr() as *const usize;
    let a_ptr_original = a_ptr;
    match direction {
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
            fr += 2
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
            fr += 2
                * (unsafe { a_ptr_original.offset_from(a_ptr) } as Fr - 1
                    + (if cfg!(target_endian = "little") {
                        // NOTE: this is reversed from the forward case.
                        cmp.leading_zeros()
                    } else {
                        cmp.trailing_zeros()
                    } / u8::BITS) as Fr);
        }
    }

    fr
}

impl<const N: usize> DiagonalTransition<AffineCost<N>, NoVisualizer, ZeroCost> {
    pub fn new(
        cm: AffineCost<N>,
        use_gap_cost_heuristic: GapCostHeuristic,
        history_compression: HistoryCompression,
    ) -> Self {
        Self::new_variant(
            cm,
            use_gap_cost_heuristic,
            ZeroCost,
            history_compression,
            Forward,
            NoVisualizer,
        )
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic> DiagonalTransition<AffineCost<N>, V, H> {
    pub fn new_variant(
        cm: AffineCost<N>,
        use_gap_cost_heuristic: GapCostHeuristic,
        h: H,
        history_compression: HistoryCompression,
        direction: Direction,
        v: V,
    ) -> Self {
        // The maximum cost we look back:
        // max(substitution, indel, affine indel of size 1)
        let top_buffer = max(
            max(cm.sub.unwrap_or(0), 0),
            max(cm.max_ins_open_extend, cm.max_del_open_extend),
        ) as Fr;

        let left_buffer = max(
            // substitution, if allowed
            cm.sub.unwrap_or(0).div_ceil(cm.ins.unwrap_or(Cost::MAX)),
            // number of insertions (left moves) done in range of looking one deletion (right move) backwards
            1 + cm.max_del_open_extend.div_ceil(cm.min_ins_extend),
        ) as Fr;
        // Idem.
        let right_buffer = max(
            // substitution, if allowed
            cm.sub.unwrap_or(0).div_ceil(cm.del.unwrap_or(Cost::MAX)),
            // number of deletions (right moves) done in range of looking one insertion (left move) backwards
            1 + cm.max_ins_open_extend.div_ceil(cm.min_del_extend),
        ) as Fr;

        // Formulas need to move to EditGraph somehow. For Gap Close, here they are:
        if false {
            let _top_buffer = max(
                max(cm.sub.unwrap_or(0), max(cm.max_del_open, cm.max_ins_open)),
                max(cm.max_ins_extend, cm.max_del_extend),
            ) as Fr;

            let _left_buffer = max(
                // substitution, if allowed
                cm.sub
                    .unwrap_or(max(cm.max_del_open, cm.max_ins_open))
                    .div_ceil(cm.ins.unwrap_or(Cost::MAX)),
                // number of insertions (left moves) done in range of looking one deletion (right move) backwards
                1 + cm.max_del_extend.div_ceil(cm.min_ins_extend),
            ) as Fr;
            // Idem.
            let _right_buffer = max(
                // substitution, if allowed
                cm.sub
                    .unwrap_or(max(cm.max_del_open, cm.max_ins_open))
                    .div_ceil(cm.del.unwrap_or(Cost::MAX)),
                // number of deletions (right moves) done in range of looking one insertion (left move) backwards
                1 + cm.max_ins_extend.div_ceil(cm.min_del_extend),
            ) as Fr;
        }

        Self {
            cm,
            use_gap_cost_heuristic,
            h,
            v,
            top_buffer,
            left_buffer,
            right_buffer,
            direction,
            history_compression,
        }
    }

    fn extend(&mut self, front: &mut Front<N>, a: Seq, b: Seq) -> bool {
        for d in front.range().clone() {
            let fr = &mut front.m_mut()[d];
            if *fr < 0 {
                continue;
            }
            let fr_old = *fr;
            *fr = match self.direction {
                Forward => extend_diagonal(self.direction, a, b, d, *fr),
                Backward => extend_diagonal(
                    self.direction,
                    a,
                    b,
                    a.len() as Fr - b.len() as Fr - d,
                    a.len() as Fr + b.len() as Fr - *fr,
                ),
            };
            let mut p = fr_to_pos(d, fr_old);
            for _ in fr_old..*fr {
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
    fn d_range(
        &self,
        a: Seq,
        b: Seq,
        h: &H::Instance<'_>,
        s: Cost,
        s_bound: Option<Cost>,
        prev: &[Front<N>],
    ) -> RangeInclusive<Fr> {
        // The range that is reachable within cost s.
        let mut r = -(self.cm.max_ins_for_cost(s) as Fr)..=self.cm.max_del_for_cost(s) as Fr;

        let Some(s_bound) = s_bound else {
            return r;
        };

        // If needed and possible, reduce with gap_cost heuristic.
        if H::IS_DEFAULT {
            if self.use_gap_cost_heuristic == GapCostHeuristic::Enable {
                let d = a.len() as Fr - b.len() as Fr;
                let s_remaining = s_bound - s;
                // NOTE: Gap open cost was already paid, so we only restrict by extend cost.
                let gap_cost_r = d - (s_remaining / self.cm.min_del_extend) as Fr
                    ..=d + (s_remaining / self.cm.min_ins_extend) as Fr;
                r = max(*r.start(), *gap_cost_r.start())..=min(*r.end(), *gap_cost_r.end());
            }
            return r;
        } else {
            fn get_front<const N: usize>(fronts: &[Front<N>], cost: Cost) -> &Front<N> {
                &fronts[fronts.len() - cost as usize]
            }

            let mut d_min = Fr::MAX;
            let mut d_max = Fr::MIN;

            // Find an initial range.
            EditGraph::iterate_parents_dt(
                a,
                b,
                &self.cm,
                // TODO: Fix for affine layers.
                None,
                |di, dj, _layer, edge_cost| -> (Fr, Fr) {
                    let parent_front = get_front(prev, edge_cost);
                    d_min = min(d_min, *parent_front.range().start() + (di - dj));
                    d_max = max(d_max, *parent_front.range().end() + (di - dj));
                    (0, 0)
                },
                |_i, _j, _layer, _cigar_ops| {},
            );

            if d_max < d_min {
                return d_min..=d_max;
            }

            // Shrink the range as needed.

            let test = |d| {
                // Eval for given diagonal. Copied from `next_front`.
                // TODO: dedup.
                let mut fr = Fr::MIN;
                EditGraph::iterate_parents_dt(
                    a,
                    b,
                    &self.cm,
                    // TODO: Fix for affine layers.
                    None,
                    |di, dj, layer, edge_cost| -> (Fr, Fr) {
                        let fr = get_front(prev, edge_cost).layer(layer)[d + (di - dj) as Fr]
                            - (di + dj) as Fr;
                        fr_to_coords(d, fr)
                    },
                    |i, j, _layer, _cigar_ops| {
                        fr = max(fr, (i + j) as Fr);
                    },
                );
                let pos = fr_to_pos(d, fr);
                (pos.0 as usize) <= a.len()
                    && (pos.1 as usize) <= b.len()
                    && s + h.h(pos) <= s_bound
            };

            while d_min <= d_max && !test(d_min) {
                d_min += 1;
            }
            while d_min <= d_max && !test(d_max) {
                d_max -= 1;
            }

            d_min..=d_max
        }
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
    fn next_front(&mut self, a: Seq, b: Seq, fronts: &mut [Front<N>]) -> bool {
        // Get the front `cost` before the last one.
        fn get_front<const N: usize>(fronts: &mut [Front<N>], cost: Cost) -> &mut Front<N> {
            &mut fronts[fronts.len() - 1 - cost as usize]
        }

        // Loop over the entire dmin..=dmax range.
        // The boundaries are buffered so no boundary checks are needed.
        // TODO: Vectorize this loop, or at least verify the compiler does this.
        // TODO: Loop over a positive range that does not need additional shifting?
        for d in get_front(fronts, 0).range().clone() {
            EditGraph::iterate_layers(&self.cm, |layer| {
                let mut fr = Fr::MIN;
                EditGraph::iterate_parents_dt(
                    a,
                    b,
                    &self.cm,
                    layer,
                    |di, dj, layer, edge_cost| -> (Fr, Fr) {
                        let fr = get_front(fronts, edge_cost).layer(layer)[d + (di - dj) as Fr]
                            - (di + dj) as Fr;
                        fr_to_coords(d, fr)
                    },
                    |i, j, _layer, _cigar_ops| {
                        fr = max(fr, (i + j) as Fr);
                    },
                );
                get_front(fronts, 0).layer_mut(layer)[d] = fr;
                if fr >= 0 {
                    self.v.expand(fr_to_pos(d, fr));
                }
            });
        }
        // Extend all points in the m layer and check if we're done.
        self.extend(get_front(fronts, 0), a, b)
    }

    // Returns None when the sequences are equal.
    fn init_fronts(&mut self, a: Seq, b: Seq) -> Option<Fronts<N>> {
        let mut fronts = Fronts::new(
            Fr::MIN,
            // We only create a front for the s=0 layer.
            0..=0,
            // The range of the s=0 front is 0..=0.
            |_| 0..=0,
            // Additionally, we have `top_buffer` fronts before the current front.
            self.top_buffer,
            0,
            self.left_buffer,
            self.right_buffer,
        );

        let f = extend_diagonal(self.direction, a, b, 0, 0);
        fronts[0].m_mut()[0] = f;
        if f >= (a.len() + b.len()) as Fr {
            return None;
        }
        Some(fronts)
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic> Aligner
    for DiagonalTransition<AffineCost<N>, V, H>
{
    type CostModel = AffineCost<N>;

    fn cost_model(&self) -> &Self::CostModel {
        &self.cm
    }

    /// The cost-only version uses linear memory.
    ///
    /// In particular, the number of fronts is max(sub, ins, del)+1.
    fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, s_bound: Option<Cost>) -> Option<Cost> {
        let Some(mut fronts) = self.init_fronts(a, b) else {
            return Some(0);
        };

        let ref mut h = self.h.build(a, b, &bio::alphabets::dna::alphabet());
        let mut num_states = 0;

        for s in 1.. {
            if let Some(s_bound) = s_bound && s > s_bound {
                return None;
            }

            // Rotate all fronts back by one, so that we can fill the new last layer.
            fronts.fronts.rotate_left(1);
            let (next, rest) = fronts.fronts.split_last_mut().unwrap();
            // FIXME: Make sure the next range is not empty! Also in NW.
            let range = self.d_range(a, b, h, s, s_bound, rest);
            if range.is_empty() {
                return None;
            }
            num_states += range.end() - range.start();
            next.reset(Fr::MIN, range, self.left_buffer, self.right_buffer);
            if self.next_front(a, b, &mut fronts.fronts) {
                return Some(s);
            }
        }
        unreachable!()
    }

    fn align_for_bounded_dist(
        &mut self,
        a: Seq,
        b: Seq,
        s_bound: Option<Cost>,
    ) -> Option<(Cost, Path, Cigar)> {
        let Some(mut fronts) = self.init_fronts(a, b) else {
            return Some((0, vec![], Cigar::default()));
        };

        let ref mut h = self.h.build(a, b, &bio::alphabets::dna::alphabet());

        for s in 1.. {
            if let Some(s_bound) = s_bound && s > s_bound {
                return None;
            }

            // We can not initialize all layers directly at the start, since we do not know the final distance s.
            let range = self.d_range(a, b, h, s, s_bound, &fronts.fronts);
            if range.is_empty() {
                return None;
            }
            fronts.fronts.push(Front::new(
                Fr::MIN,
                range,
                self.left_buffer,
                self.right_buffer,
            ));
            if self.next_front(a, b, &mut fronts.fronts) {
                // FIXME: Reconstruct path.
                return Some((s, vec![], Cigar::default()));
            }
        }
        unreachable!()
    }
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use super::*;
    use crate::{
        generate::setup_sequences,
        heuristic::{ZeroCost, SH},
        matches::MatchConfig,
    };

    const N: usize = 1000;
    const E: f32 = 0.05;

    #[test]
    fn dt() {
        let (ref a, ref b) = setup_sequences(N, E);
        let mut aligner = DiagonalTransition::new_variant(
            LinearCost::new_unit(),
            GapCostHeuristic::Disable,
            ZeroCost,
            HistoryCompression::Disable,
            Direction::Forward,
            NoVisualizer,
        );
        let s = Instant::now();
        aligner.cost(a, b);
        println!("DT: {}", s.elapsed().as_secs_f32() * 1000.);
    }

    #[test]
    fn dt_gapcost() {
        let (ref a, ref b) = setup_sequences(N, E);
        let mut aligner = DiagonalTransition::new_variant(
            LinearCost::new_unit(),
            GapCostHeuristic::Enable,
            ZeroCost,
            HistoryCompression::Disable,
            Direction::Forward,
            NoVisualizer,
        );
        let s = Instant::now();
        aligner.cost_exponential_search(a, b);
        println!("DT: {}", s.elapsed().as_secs_f32() * 1000.);
    }

    #[test]
    fn dt_sh() {
        let (ref a, ref b) = setup_sequences(N, E);
        let mut aligner = DiagonalTransition::new_variant(
            LinearCost::new_unit(),
            GapCostHeuristic::Disable,
            SH {
                match_config: MatchConfig::exact(10),
                pruning: false,
            },
            HistoryCompression::Disable,
            Direction::Forward,
            NoVisualizer,
        );
        let s = Instant::now();
        aligner.cost_exponential_search(a, b);
        println!("DT: {}", s.elapsed().as_secs_f32() * 1000.);
    }

    #[test]
    fn dt_sh_oracle() {
        let (ref a, ref b) = setup_sequences(N, E);
        let mut aligner = DiagonalTransition::new_variant(
            LinearCost::new_unit(),
            GapCostHeuristic::Disable,
            SH {
                match_config: MatchConfig::exact(10),
                pruning: false,
            },
            HistoryCompression::Disable,
            Direction::Forward,
            NoVisualizer,
        );
        let cost = aligner.cost_exponential_search(a, b);
        let s = Instant::now();
        aligner.cost_for_bounded_dist(a, b, Some(cost));
        println!("DT: {}", s.elapsed().as_secs_f32() * 1000.);
    }
}
