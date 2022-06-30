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
use super::edit_graph::{CigarOps, EditGraph, Layer};
use super::{exponential_search, Aligner, Path, Seq, StateT};
use crate::aligners::cigar::CigarOp;
use crate::cost_model::*;
use crate::heuristic::{Heuristic, HeuristicInstance};
use crate::prelude::{to_string, Pos};
use crate::visualizer::VisualizerT;
use std::cmp::{max, min};
use std::iter::zip;
use std::ops::RangeInclusive;

/// The type for storing furthest reaching points.
/// Sized, so that we can default them to -INF.
pub type Fr = i32;

type Front<const N: usize> = super::front::Front<N, Fr, Fr>;
type Fronts<const N: usize> = super::front::Fronts<N, Fr, Fr>;

/// The direction to run in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Forward,
    Backward,
}
use Direction::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GapCostHeuristic {
    Enable,
    Disable,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

    /// When true, `align` uses divide & conquer to compute the alignment in linear memory.
    dc: bool,

    /// When true, calls to `align` store a compressed version of the full 'history' of visited states.
    #[allow(unused)]
    history_compression: HistoryCompression,

    pub v: V,

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DtState {
    d: Fr,
    fr: Fr,
    layer: Layer,
    s: Cost,
}

impl DtState {
    fn start() -> Self {
        DtState {
            d: 0,
            fr: 0,
            layer: None,
            s: 0,
        }
    }
    fn target(a: Seq, b: Seq, s: Cost) -> Self {
        DtState {
            d: a.len() as Fr - b.len() as Fr,
            fr: a.len() as Fr + b.len() as Fr,
            layer: None,
            s,
        }
    }
}

impl StateT for DtState {
    fn is_root(&self) -> bool {
        if self.d == 0 && self.fr == 0 && self.layer == None {
            assert!(self.s == 0);
            true
        } else {
            false
        }
    }

    fn pos(&self) -> Pos {
        assert!((self.d + self.fr) % 2 == 0);
        Pos(
            ((self.fr + self.d) / 2) as crate::prelude::I,
            ((self.fr - self.d) / 2) as crate::prelude::I,
        )
    }
}

/// Converts a pair of (diagonal index, furthest reaching) to a position.
/// TODO: Return Pos or usize instead?
#[inline]
fn fr_to_coords(d: Fr, fr: Fr) -> (Fr, Fr) {
    //assert!(fr < 0 || (d + fr) % 2 == 0);
    ((fr + d) / 2, (fr - d) / 2)
}
#[inline]
fn fr_to_pos(d: Fr, fr: Fr) -> Pos {
    //assert!((d + fr) % 2 == 0);
    Pos(
        ((fr + d) / 2) as crate::prelude::I,
        ((fr - d) / 2) as crate::prelude::I,
    )
}

/// Given two sequences, a diagonal and point on it, expand it to a FR point.
/// Returns the number of characters matched.
/// NOTE: `d` and `fr` must be in Forward domain here.
fn extend_diagonal(direction: Direction, a: Seq, b: Seq, d: Fr, fr: Fr) -> Fr {
    let (i, j) = fr_to_coords(d, fr);
    if i as usize > a.len() || j as usize > b.len() {
        return 0;
    }

    // TODO: The end check can be avoided by appending `#` and `$` to `a` and `b`.
    match direction {
        Forward => zip(a[i as usize..].iter(), b[j as usize..].iter())
            .take_while(|(ca, cb)| ca == cb)
            .count() as Fr,
        Backward => zip(a[..i as usize].iter().rev(), b[..j as usize].iter().rev())
            .take_while(|(ca, cb)| ca == cb)
            .count() as Fr,
    }
}

/// Given two sequences, a diagonal and point on it, expand it to a FR point.
///
/// This version compares one usize at a time.
/// NOTE: `d` and `fr` must be in Forward domain here.
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

impl<const N: usize, V: VisualizerT, H: Heuristic> DiagonalTransition<AffineCost<N>, V, H> {
    pub fn new(
        cm: AffineCost<N>,
        use_gap_cost_heuristic: GapCostHeuristic,
        h: H,
        dc: bool,
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
            dc,
            v,
            top_buffer,
            left_buffer,
            right_buffer,
            history_compression: HistoryCompression::Disable,
        }
    }

    fn extend(&mut self, front: &mut Front<N>, a: Seq, b: Seq, direction: Direction) -> bool {
        for d in front.range().clone() {
            let fr = &mut front.m_mut()[d];
            if *fr < 0 {
                continue;
            }
            let fr_old = *fr;
            match direction {
                Forward => {
                    *fr += 2 * extend_diagonal(direction, a, b, d, *fr);
                    for fr in (fr_old + 2..=*fr).step_by(2) {
                        self.v.expand(fr_to_pos(d, fr));
                    }
                }
                Backward => {
                    *fr += 2 * extend_diagonal(
                        direction,
                        a,
                        b,
                        a.len() as Fr - b.len() as Fr - d,
                        a.len() as Fr + b.len() as Fr - *fr,
                    );
                    for fr in (fr_old + 2..=*fr).step_by(2) {
                        self.v.expand(fr_to_pos(
                            a.len() as Fr - b.len() as Fr - d,
                            a.len() as Fr + b.len() as Fr - fr,
                        ));
                    }
                }
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
                |di, dj, _layer, edge_cost| -> Option<(Fr, Fr)> {
                    let parent_front = get_front(prev, edge_cost);
                    d_min = min(d_min, *parent_front.range().start() + (di - dj));
                    d_max = max(d_max, *parent_front.range().end() + (di - dj));
                    None
                },
                |_di, _dj, _i, _j, _layer, _edge_cost, _cigar_ops| {},
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
                    |di, dj, layer, edge_cost| -> Option<(Fr, Fr)> {
                        let fr = get_front(prev, edge_cost).layer(layer)[d + (di - dj) as Fr]
                            - (di + dj) as Fr;
                        if fr >= 0 {
                            Some(fr_to_coords(d, fr))
                        } else {
                            None
                        }
                    },
                    |_di, _dj, i, j, _layer, _edge_cost, _cigar_ops| {
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

    /// Computes the next layer from the current one.
    /// `ca` is the `i`th character of sequence `a`.
    ///
    /// NOTE: `next` must already have the right range set.
    ///
    /// Returns `true` when the search completes.
    fn next_front(
        &mut self,
        a: Seq,
        b: Seq,
        fronts: &mut [Front<N>],
        direction: Direction,
    ) -> bool {
        // Get the front `cost` before the last one.
        fn get_front<const N: usize>(fronts: &mut [Front<N>], cost: Cost) -> &mut Front<N> {
            &mut fronts[fronts.len() - 1 - cost as usize]
        }

        // Loop over the entire dmin..=dmax range.
        // The boundaries are buffered so no boundary checks are needed.
        // TODO: Vectorize this loop, or at least verify the compiler does this.
        // TODO: Loop over a positive range that does not need additional shifting?
        match direction {
            Direction::Forward => {
                for d in get_front(fronts, 0).range().clone() {
                    EditGraph::iterate_parent_layers(&self.cm, |layer| {
                        let mut fr = Fr::MIN;
                        EditGraph::iterate_parents_dt(
                            a,
                            b,
                            &self.cm,
                            layer,
                            |di, dj, layer, edge_cost| -> Option<(Fr, Fr)> {
                                let fr = get_front(fronts, edge_cost).layer(layer)
                                    [d + (di - dj) as Fr]
                                    - (di + dj) as Fr;
                                if fr >= 0 {
                                    Some(fr_to_coords(d, fr))
                                } else {
                                    None
                                }
                            },
                            |_di, _dj, i, j, _layer, _edge_cost, _cigar_ops| {
                                if i >= 0 && j >= 0 {
                                    fr = max(fr, (i + j) as Fr);
                                }
                            },
                        );
                        get_front(fronts, 0).layer_mut(layer)[d] = fr;
                        if fr >= 0 {
                            self.v.expand(fr_to_pos(d, fr));
                        }
                    });
                }
            }
            Direction::Backward => {
                let mirror = |(i, j)| (a.len() as Fr - i, b.len() as Fr - j);
                let mirror_pos = |Pos(i, j)| Pos(a.len() as u32 - i, b.len() as u32 - j);
                let max_fr = a.len() as Fr + b.len() as Fr;
                let mirror_fr = |fr| max_fr - fr;
                for d in get_front(fronts, 0).range().clone() {
                    println!("Next layer {direction:?} for d={d}");
                    EditGraph::iterate_child_layers(&self.cm, |layer| {
                        let mut fr = Fr::MIN;
                        EditGraph::iterate_children_dt(
                            a,
                            b,
                            &self.cm,
                            layer,
                            // NOTE: This returns a forward position.
                            // FIXME: Make sure this returns the correct position for homopolymer checking.
                            |di, dj, layer, edge_cost| -> Option<(Fr, Fr)> {
                                println!("Parent {di} {dj} {layer:?} {edge_cost}");
                                let fr = get_front(fronts, edge_cost).layer(layer)
                                    [d - (di - dj) as Fr]
                                    + (di + dj) as Fr;
                                println!("fr {fr}");
                                if fr >= 0 {
                                    Some(mirror(fr_to_coords(d, fr)))
                                } else {
                                    None
                                }
                            },
                            |_di, _dj, i, j, _layer, _edge_cost, _cigar_ops| {
                                if i <= a.len() as Fr && j <= b.len() as Fr {
                                    println!(
                                        "Set fr to max of {fr} and {} from coords {i},{j}",
                                        mirror_fr((i + j) as Fr)
                                    );
                                    fr = max(fr, mirror_fr((i + j) as Fr));
                                }
                            },
                        );
                        get_front(fronts, 0).layer_mut(layer)[d] = fr;
                        if fr <= max_fr {
                            self.v.expand(mirror_pos(fr_to_pos(d, fr)));
                        }
                    });
                }
            }
        }

        // We set the fr. point for the 0 diagonal to at least 0.
        // This ensures that fr[d][s] <= fr[d][s'] when s <= s'.
        // That in turn simplifies the overlap condition check, since we only need to check overlap for the two last fronts.
        let front = get_front(fronts, 0);
        if front.range().contains(&0) {
            front.m_mut()[0] = max(front.m()[0], 0);
        }

        // Extend all points in the m layer and check if we're done.
        self.extend(get_front(fronts, 0), a, b, direction)
    }

    // Returns None when the sequences are equal.
    fn init_fronts(&mut self, a: Seq, b: Seq, direction: Direction) -> Option<Fronts<N>> {
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

        fronts[0].m_mut()[0] = 0;
        if self.extend(&mut fronts[0], a, b, direction) {
            None
        } else {
            Some(fronts)
        }
    }

    /// Detects if there is a diagonal such that the two fronts meet/overlap.
    /// The overlap can be in any of the affine layers.
    /// Returns: None is no overlap was found.
    /// Otherwise: The middle state, as forward and as backward version.
    /// NOTE: the two FR indices may not correspond to the same character, in the case of overlapping greedy matches.
    #[allow(dead_code)]
    fn fronts_overlap(
        &self,
        a: Seq,
        b: Seq,
        forward: &Fronts<N>,
        backward: &Fronts<N>,
    ) -> Option<(DtState, DtState)> {
        // NOTE: This is the same for the forward and reverse direction.
        let fr_target = (a.len() + b.len()) as Fr;
        let mirror = |d| (a.len() as Fr - b.len() as Fr) - d;
        println!("Forward range {:?}", forward.last().range());
        println!("Backward range {:?}", backward.last().range());
        println!(
            "Mirror Backward range {:?}",
            mirror(*backward.last().range().end())..=mirror(*backward.last().range().start())
        );
        let d_range = max(
            *forward.last().range().start(),
            mirror(*backward.last().range().end()),
        )
            ..=min(
                *forward.last().range().end(),
                mirror(*backward.last().range().start()),
            );
        // TODO: Provide an (internal) iterator over Layers from Front that merges these two cases.
        // M
        let mut meet = None;
        let mut s_meet = None;
        EditGraph::iterate_parent_layers(&self.cm, |layer| {
            println!("Overlap layer {layer:?}");
            for d in d_range.clone() {
                println!(
                    "Overlap test {d}: {} + {} >= {fr_target}",
                    forward.last().layer(layer)[d],
                    backward.last().layer(layer)[mirror(d)]
                );
                // Cap values that are larger than the length of their diagonal.
                let f_fr = min(forward.last().layer(layer)[d], fr_target - mirror(d).abs());
                let b_fr = min(backward.last().layer(layer)[mirror(d)], fr_target - d.abs());
                if f_fr + b_fr >= fr_target {
                    let forward_fr = forward.last().layer(layer)[d];
                    let mut fw = DtState {
                        d,
                        fr: forward_fr,
                        layer,
                        s: *forward.range().end() as Cost,
                    };
                    let mut bw = DtState {
                        d: mirror(d),
                        fr: fr_target - forward_fr,
                        layer,
                        s: *backward.range().end() as Cost,
                    };

                    // It may be that we only detected overlap at the current forward_s+backward_s, but actually they already overlap earlier.
                    // For example, this may happen when insertions cost 10: forward_s and backward_s must both be 10 to detect a single insertion.
                    // Here, we decrease s as much as possible.
                    let mut fs = fw.s as Fr;
                    let mut bs = bw.s as Fr;

                    let test = |fs: Fr, bs: Fr| {
                        let f_fr = min(
                            forward[fs].layer(fw.layer)[fw.d],
                            fr_target - mirror(fw.d).abs(),
                        );
                        let b_fr = min(
                            backward[bs].layer(bw.layer)[bw.d],
                            fr_target - mirror(bw.d).abs(),
                        );
                        println!("mirror d: {}", bw.d);
                        println!("b range {:?}", backward[bs].range());
                        let ok = f_fr >= 0 && b_fr >= 0 && f_fr + b_fr >= fr_target;
                        println!("Shrink distances to {fs} {bs}: {f_fr} {b_fr} {ok}");
                        ok
                    };
                    while fs > *forward.full_range().start() && test(fs - 1, bs) {
                        fs -= 1;
                    }
                    while bs > *backward.full_range().start() && test(fs, bs - 1) {
                        bs -= 1;
                    }
                    fw.s = fs as Cost;
                    bw.s = bs as Cost;
                    fw.fr = forward[fs].layer(fw.layer)[fw.d];
                    bw.fr = fr_target - fw.fr;

                    if s_meet.is_none() || fw.s + bw.s < s_meet.unwrap() {
                        s_meet = Some(fw.s + bw.s);
                        meet = Some((fw, bw));
                    }
                }
            }
        });
        meet
    }

    /// Finds a path between two given states using divide & conquer.
    /// TODO: Improve this by skipping the overlap check when distances are already known.
    fn path_between_dc(
        &mut self,
        a: Seq,
        b: Seq,
        start_layer: Layer,
        end_layer: Layer,
    ) -> (Cost, Path, Cigar) {
        println!(
            "Path between {} {} {start_layer:?} {end_layer:?}",
            a.len(),
            b.len()
        );
        println!("Init forward");
        let Some(mut forward_fronts) = self.init_fronts(a, b, Direction::Forward) else {
            return (0, vec![], Cigar::default());
        };
        println!("Init backward");
        let Some(mut backward_fronts) = self.init_fronts(a, b, Direction::Backward) else {
            return (0, vec![], Cigar::default());
        };

        assert!(H::IS_DEFAULT);
        let ref mut h = self.h.build(a, b, &bio::alphabets::dna::alphabet());

        // The top level meet in the middle step is separate, since the distance is not known yet.
        // We check whether the fronts meet after each iteration.
        let (fw, bw) = 'meet: {
            for s in 1.. {
                // First, take a step in the forward front, then in the backward front.
                for dir in [Direction::Forward, Direction::Backward] {
                    let fronts = match dir {
                        Forward => &mut forward_fronts,
                        Backward => &mut backward_fronts,
                    };
                    let range = self.d_range(a, b, h, s, None, &fronts.fronts);
                    assert!(!range.is_empty());
                    fronts.rotate(range);
                    self.next_front(a, b, &mut fronts.fronts, dir);
                    println!("s: {s} {dir:?}");
                    println!("Forward:");
                    for front in &forward_fronts.fronts {
                        println!("{front:?}");
                    }
                    println!("Backward:");
                    for front in &backward_fronts.fronts {
                        println!("{front:?}");
                    }

                    if let Some(meet) = self.fronts_overlap(a, b, &forward_fronts, &backward_fronts)
                    {
                        println!(
                            "\n============= MATCH AT s={s} COST {} + {}\n",
                            meet.0.s, meet.1.s
                        );
                        break 'meet meet;
                    }
                }
            }
            unreachable!()
        };

        let Pos(i, j) = fw.pos();
        println!(
            "MIDDLE FOUND; RECURSING?\nA {}\nB {}\n",
            to_string(a),
            to_string(b)
        );
        println!("FW: {fw:?}");
        println!("BW: {bw:?}");

        println!("FORWARD\n");
        let mut left = if forward_fronts.full_range().contains(&0) {
            println!("Trace forward part!");
            // Rotate the front back as far as needed.
            while (fw.s as Fr) < *forward_fronts.range().end() {
                forward_fronts.rotate_back();
            }
            let (path, cigar) = self.trace(
                a,
                b,
                &forward_fronts,
                DtState {
                    d: 0,
                    fr: 0,
                    layer: start_layer,
                    s: 0,
                },
                fw,
                Direction::Forward,
            );
            (fw.s, path, cigar)
        } else {
            println!(
                "\n LEFT RECURSION\nCOST: {}\n{}\n{}\n",
                fw.s,
                to_string(&a[..i as usize]),
                to_string(&b[..j as usize])
            );
            let (cost, path, cigar) =
                self.path_between_dc(&a[..i as usize], &b[..j as usize], start_layer, fw.layer);
            assert_eq!(cost, fw.s);
            (cost, path, cigar)
        };
        println!("BACKWARD\n");
        let mut right = if backward_fronts.full_range().contains(&0) {
            println!("Trace backward part!");
            while (bw.s as Fr) < *backward_fronts.range().end() {
                backward_fronts.rotate_back();
            }
            let (mut path, mut cigar) = self.trace(
                a,
                b,
                &backward_fronts,
                DtState {
                    d: 0,
                    fr: 0,
                    layer: start_layer,
                    s: 0,
                },
                bw,
                Direction::Backward,
            );
            path.reverse();
            cigar.reverse();
            (bw.s, path, cigar)
        } else {
            println!(
                "\n RIGHT RECURSION\nCOST: {}\n{}\n{}\n",
                bw.s,
                to_string(&a[i as usize..]),
                to_string(&b[j as usize..])
            );
            let (cost, path, cigar) =
                self.path_between_dc(&a[i as usize..], &b[j as usize..], bw.layer, end_layer);
            assert_eq!(cost, bw.s);
            // Offset the path.

            (cost, path, cigar)
        };

        println!("LEFT:\n{left:?}");
        println!("RIGHT:\n{right:?}");

        // Join
        left.0 += right.0;
        left.1.append(&mut right.1);
        left.2.append(&mut right.2);

        println!("MERGED:\n{left:?}");
        left
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic> Aligner
    for DiagonalTransition<AffineCost<N>, V, H>
{
    type CostModel = AffineCost<N>;

    type Fronts = Fronts<N>;

    type State = DtState;

    fn cost_model(&self) -> &Self::CostModel {
        &self.cm
    }

    fn parent(
        &self,
        a: Seq,
        b: Seq,
        fronts: &Self::Fronts,
        st: Self::State,
        direction: Direction,
    ) -> Option<(Self::State, CigarOps)> {
        let mut max_fr = Fr::MIN;
        let mut parent = None;
        let mut cigar_ops = [None, None];

        println!("Parent of {st:?}");

        match direction {
            Forward => {
                if st.s > 0 {
                    EditGraph::iterate_parents_dt(
                        a,
                        b,
                        &self.cm,
                        st.layer,
                        |di, dj, layer, edge_cost| -> Option<(Fr, Fr)> {
                            let parent_cost = st.s as Fr - edge_cost as Fr;
                            if parent_cost < 0 || !fronts.full_range().contains(&parent_cost) {
                                println!("Parent {di} {dj} {edge_cost} with absolute cost {parent_cost} is out of range: {:?}", fronts.full_range());
                                return None;
                            }
                            let fr = fronts[parent_cost].layer(layer)[st.d + (di - dj) as Fr]
                                - (di + dj) as Fr;
                            if fr >= 0 {
                                Some(fr_to_coords(st.d, fr))
                            } else {
                                None
                            }
                        },
                        |di, dj, i, j, layer, edge_cost, ops| {
                            println!("Parent {di} {dj} {edge_cost} => ij {i} {j} fr {}", i + j);
                            let fr = (i + j) as Fr;
                            if fr > max_fr {
                                max_fr = fr;
                                parent = Some(DtState {
                                    d: st.d + (di - dj),
                                    fr: st.fr + (di + dj),
                                    layer,
                                    s: st.s - edge_cost as Cost,
                                });
                                cigar_ops = ops;
                            }
                        },
                    );
                }
                // Match
                // TODO: Add a setting to do greedy backtracking before checking other parents.
                if max_fr < st.fr {
                    let (i, j) = fr_to_coords(st.d, st.fr);
                    assert_eq!(a[i as usize - 1], b[j as usize - 1]);
                    parent = Some(st);
                    parent.as_mut().unwrap().fr -= 2;
                    cigar_ops = [Some(CigarOp::Match), None];
                }
            }

            Backward => {
                let mirror = |(i, j)| (a.len() as Fr - i, b.len() as Fr - j);
                //let mirror_pos = |Pos(i, j)| Pos(a.len() as u32 - i, b.len() as u32 - j);
                let mirror_fr = |fr| a.len() as Fr + b.len() as Fr - fr;
                // FIXME
                if st.s > 0 {
                    EditGraph::iterate_children_dt(
                        a,
                        b,
                        &self.cm,
                        st.layer,
                        |di, dj, layer, edge_cost| -> Option<(Fr, Fr)> {
                            let parent_cost = st.s as Fr - edge_cost as Fr;
                            if parent_cost < 0 || !fronts.full_range().contains(&parent_cost) {
                                return None;
                            }
                            println!("{di} {dj} {layer:?} {edge_cost}");
                            let fr = fronts[parent_cost].layer(layer)[st.d - (di - dj) as Fr];
                            //+ (di + dj) as Fr;
                            println!("fr: {fr}");
                            if fr >= 0 {
                                Some(mirror(fr_to_coords(st.d - (di - dj), fr)))
                            } else {
                                None
                            }
                        },
                        |di, dj, i, j, layer, edge_cost, ops| {
                            println!("i j {i} {j}");
                            let fr = mirror_fr((i + j) as Fr) + (di + dj) as Fr;
                            if fr > max_fr {
                                println!("New best fr {fr};  d {} => {}", st.d, st.d - (di - dj));
                                println!("St.fr: {} => {}", st.fr, st.fr - (di + dj));
                                max_fr = fr;
                                parent = Some(DtState {
                                    d: st.d - (di - dj),
                                    fr: st.fr - (di + dj),
                                    layer,
                                    s: st.s - edge_cost as Cost,
                                });
                                cigar_ops = ops;
                            }
                        },
                    );
                }
                // Match
                // TODO: Add a setting to do greedy backtracking before checking other parents.
                if max_fr < st.fr {
                    let (i, j) = mirror(fr_to_coords(st.d, st.fr));
                    println!("A {}\nB {}\ni j {i} {j}", to_string(a), to_string(b));
                    assert_eq!(a[i as usize], b[j as usize]);
                    parent = Some(st);
                    parent.as_mut().unwrap().fr -= 2;
                    cigar_ops = [Some(CigarOp::Match), None];
                }
            }
        }
        println!("Parent: {parent:?} ops: {cigar_ops:?}");
        Some((parent?, cigar_ops))
    }

    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        let cost = if self.use_gap_cost_heuristic == GapCostHeuristic::Enable || !H::IS_DEFAULT {
            exponential_search(
                self.cm.gap_cost(Pos(0, 0), Pos::from_lengths(a, b)),
                2.,
                |s| self.cost_for_bounded_dist(a, b, Some(s)).map(|c| (c, c)),
            )
            .1
        } else {
            self.cost_for_bounded_dist(a, b, None).unwrap()
        };
        self.v.last_frame(None);
        cost
    }

    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Path, Cigar) {
        if self.dc {
            return self.align_dc(a, b);
        }
        let (cost, path, cigar) =
            if self.use_gap_cost_heuristic == GapCostHeuristic::Enable || !H::IS_DEFAULT {
                exponential_search(
                    self.cm.gap_cost(Pos(0, 0), Pos::from_lengths(a, b)),
                    2.,
                    |s| {
                        self.align_for_bounded_dist(a, b, Some(s))
                            .map(|x @ (c, _, _)| (c, x))
                    },
                )
                .1
            } else {
                self.align_for_bounded_dist(a, b, None).unwrap()
            };
        self.v.last_frame(Some(&path));
        (cost, path, cigar)
    }

    /// The cost-only version uses linear memory.
    ///
    /// In particular, the number of fronts is max(sub, ins, del)+1.
    fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, s_bound: Option<Cost>) -> Option<Cost> {
        let Some(mut fronts) = self.init_fronts(a, b, Direction::Forward) else {
            return Some(0);
        };
        let ref mut h = self.h.build(a, b, &bio::alphabets::dna::alphabet());

        for s in 1.. {
            if let Some(s_bound) = s_bound && s > s_bound {
                return None;
            }
            let range = self.d_range(a, b, h, s, s_bound, &fronts.fronts);
            if range.is_empty() {
                return None;
            }
            fronts.rotate(range);
            if self.next_front(a, b, &mut fronts.fronts, Direction::Forward) {
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
        let Some(mut fronts) = self.init_fronts(a, b, Direction::Forward) else {
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
            fronts.push(range);
            if self.next_front(a, b, &mut fronts.fronts, Direction::Forward) {
                let (path, cigar) = self.trace(
                    a,
                    b,
                    &fronts,
                    DtState::start(),
                    DtState::target(a, b, s),
                    Direction::Forward,
                );
                return Some((s, path, cigar));
            }
        }
        unreachable!()
    }

    /// Finds an alignment in linear memory, by using divide & conquer.
    /// TODO: Add a bounded distance option here?
    fn align_dc(&mut self, a: Seq, b: Seq) -> (Cost, Path, Cigar) {
        // D&C does not work with a heuristic yet, since the target state (where
        // the fronts meet) is not know.
        assert!(H::IS_DEFAULT);
        assert!(self.use_gap_cost_heuristic == GapCostHeuristic::Disable);

        self.path_between_dc(a, b, None, None)
    }
}
