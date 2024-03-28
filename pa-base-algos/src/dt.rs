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
//! - `{left,top,bot}_buffer`: additional allocated fronts/diagonals that remove
//!   the need for boundary checks.
//! - `offset`: the index of diagonal `0` in a layer. `offset = top_buffer - dmin`.
//!
//!
use crate::edit_graph::{AffineCigarOps, EditGraph, StateT};
use crate::exponential_search;
use pa_affine_types::*;
use pa_heuristic::*;
use pa_types::*;
use pa_vis::*;
use std::cell::RefCell;
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
pub enum PathTracingMethod {
    ForwardGreedy,
    ReverseGreedy,
}

/// Settings for the algorithm, and derived constants.
///
/// TODO: Split into two classes: A static user supplied config, and an instance
/// to use for a specific alignment. Similar to Heuristic vs HeuristicInstance.
/// The latter can contain the sequences, direction, and other specifics.
#[derive(Clone)]
pub struct DiagonalTransition<const N: usize, V: VisualizerT, H: Heuristic> {
    cm: AffineCost<N>,

    /// Whether to use the gap heuristic to the end to reduce the number of diagonals considered.
    use_gap_cost_heuristic: GapCostHeuristic,

    h: H,

    /// The visualizer
    pub v: V,

    /// When true, `align` uses divide & conquer to compute the alignment in linear memory.
    pub dc: bool,

    pub local_doubling: bool,

    pub path_tracing_method: PathTracingMethod,
}

impl<const N: usize, V: VisualizerT, H: Heuristic> std::fmt::Debug for DiagonalTransition<N, V, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiagonalTransition")
            .field("use_gap_cost_heuristic", &self.use_gap_cost_heuristic)
            .field("h", &self.h)
            .field("dc", &self.dc)
            .field("local_doubling", &self.local_doubling)
            .field("path_tracing_method", &self.path_tracing_method)
            .finish()
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic> DiagonalTransition<N, V, H> {
    pub fn new(
        cm: AffineCost<N>,
        use_gap_cost_heuristic: GapCostHeuristic,
        h: H,
        dc: bool,
        v: V,
    ) -> Self {
        Self {
            cm,
            use_gap_cost_heuristic,
            h,
            dc,
            v,
            local_doubling: false,
            path_tracing_method: PathTracingMethod::ForwardGreedy,
        }
    }

    fn build<'a>(
        &self,
        a: Seq<'a>,
        b: Seq<'a>,
        v: &'a RefCell<V::Instance>,
    ) -> DTInstance<'a, N, V, H> {
        // The maximum cost we look back:
        let left_buf = EditGraph::max_edge_cost(&self.cm) as Fr;

        // FIXME: top_buf and bot_buf need updating for the new edit graph, and modification for the backward direction.
        let top_buf = max(
            // substitution, if allowed
            self.cm
                .sub
                .unwrap_or(0)
                .div_ceil(self.cm.ins.unwrap_or(Cost::MAX)),
            // number of insertions (left moves) done in range of looking one deletion (right move) backwards
            1 + self.cm.max_del_open_extend.div_ceil(self.cm.min_ins_extend),
        ) as Fr;
        // Idem.
        let bot_buf = max(
            // substitution, if allowed
            self.cm
                .sub
                .unwrap_or(0)
                .div_ceil(self.cm.del.unwrap_or(Cost::MAX)),
            // number of deletions (right moves) done in range of looking one insertion (left move) backwards
            1 + self.cm.max_ins_open_extend.div_ceil(self.cm.min_del_extend),
        ) as Fr;

        DTInstance {
            a,
            b,
            params: self.clone(),
            h: self.h.build(a, b),
            v,
            left_buf,
            top_buf,
            bot_buf,
        }
    }
}

pub struct DTInstance<'a, const N: usize, V: VisualizerT, H: Heuristic> {
    // NOTE: `a` and `b` are padded sequences and hence owned.
    pub a: Seq<'a>,
    pub b: Seq<'a>,

    pub params: DiagonalTransition<N, V, H>,

    /// The heuristic to use.
    pub h: H::Instance<'a>,

    /// The visualizer to use.
    pub v: &'a RefCell<V::Instance>,

    /// We add a few buffer layers to the left of the table, to avoid the need
    /// to check that e.g. `s` is at least the substitution cost before
    /// making a substitution.
    ///
    /// The value is the max of the substitution cost and all (affine) costs of a gap of size 1.
    left_buf: Fr,
    /// We also add a buffer to the top and bottom of each wavefront to reduce the need for if-statements.
    /// The size of the top buffer is the number of insertions that can be done for the cost of one deletion.
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
    /// v   *..x.*    <- top buffer: ceil(sub/ins) = ceil(5/2) = 3, bot buffer: ceil(sub/del) = ceil(5/3) = 2
    /// s     xx
    ///    *..xxx     <- 1 + ceil(del/ins) = 1 + ceil(3/2) = 3 buffer
    ///      xxxx.*   <- 1 + ceil(ins/del) = 1 + ceil(2/3) = 2 buffer
    ///      xxxx
    ///     XxxxxX    <- when computing these cells.
    ///
    /// For affine GapOpen costs, we replace the numerator by the maximum open+extend cost, and the numerator by the minimum extend cost.
    /// FIXME: For affine GapClose costs, we add the max open cost to the substitution cost.
    top_buf: Fr,
    bot_buf: Fr,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct DtState {
    pub d: Fr,
    pub fr: Fr,
    pub layer: Layer,
    pub s: Cost,
}

impl DtState {
    pub fn start() -> Self {
        DtState {
            d: 0,
            fr: 0,
            layer: None,
            s: 0,
        }
    }
    pub fn target(a: Seq, b: Seq, s: Cost) -> Self {
        DtState {
            d: a.len() as Fr - b.len() as Fr,
            fr: a.len() as Fr + b.len() as Fr,
            layer: None,
            s,
        }
    }
    pub fn from_pos(p: Pos, s: Cost) -> Self {
        DtState {
            d: p.0 as Fr - p.1 as Fr,
            fr: p.0 as Fr + p.1 as Fr,
            layer: None,
            s,
        }
    }
    pub fn to_pos(&self) -> Pos {
        Pos(((self.fr + self.d) / 2) as I, ((self.fr - self.d) / 2) as I)
    }
}

impl StateT for DtState {
    fn is_root(&self) -> bool {
        if self.d == 0 && self.fr == 0 && self.layer == None && self.s == 0 {
            true
        } else {
            false
        }
    }

    fn pos(&self) -> Pos {
        assert!((self.d + self.fr) % 2 == 0);
        Pos(((self.fr + self.d) / 2) as I, ((self.fr - self.d) / 2) as I)
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
    Pos(((fr + d) / 2) as I, ((fr - d) / 2) as I)
}
#[inline]
pub fn pos_to_fr(p: Pos) -> (Fr, Fr) {
    (p.0 as Fr - p.1 as Fr, p.0 as Fr + p.1 as Fr)
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
        Direction::Forward => zip(a[i as usize..].iter(), b[j as usize..].iter())
            .take_while(|(ca, cb)| ca == cb)
            .count() as Fr,
        Direction::Backward => zip(a[..i as usize].iter().rev(), b[..j as usize].iter().rev())
            .take_while(|(ca, cb)| ca == cb)
            .count() as Fr,
    }
}

/// Given two sequences, a diagonal and point on it, expand it to a FR point.
///
/// This version compares one usize at a time.
/// NOTE: `d` and `fr` must be in Forward domain here.
/// FIXME: This needs sentinels at the starts/ends of the sequences to finish correctly.
/// FIXME: Dereferencing unaligned pointers is not good. Use this instead:
/// https://doc.rust-lang.org/std/ptr/fn.read_unaligned.html
/// Thanks @Daniel Liu!
#[allow(unused)]
fn extend_diagonal_packed(direction: Direction, a: Seq, b: Seq, d: Fr, mut fr: Fr) -> Fr {
    let i = (fr + d) / 2;
    let j = (fr - d) / 2;

    // cast [u8] to *const usize, to compare 8 bytes at a time.
    let mut a_ptr = a[i as usize..].as_ptr() as *const usize;
    let mut b_ptr = b[j as usize..].as_ptr() as *const usize;
    let a_ptr_original = a_ptr;
    match direction {
        Direction::Forward => {
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
        Direction::Backward => {
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

impl<'a, const N: usize, V: VisualizerT, H: Heuristic> DTInstance<'a, N, V, H> {
    /// Returns true when the end is reached.
    fn extend(
        &mut self,
        g: Cost,
        // Only used for visualizing
        f_max: Cost,
        front: &mut Front<N>,
        offset: Pos,
        direction: Direction,
    ) -> bool {
        for d in front.range().clone() {
            let fr = &mut front.m_mut()[d];
            if *fr < 0 {
                continue;
            }
            let fr_old = *fr;
            match direction {
                Direction::Forward => {
                    *fr += 2 * extend_diagonal(direction, &self.a, &self.b, d, *fr);
                    for fr in (fr_old..*fr).step_by(2) {
                        self.v.borrow_mut().extend(
                            offset + fr_to_pos(d, fr),
                            g,
                            f_max,
                            Some(&self.h),
                        );
                    }
                    self.v
                        .borrow_mut()
                        .expand(offset + fr_to_pos(d, *fr), g, f_max, Some(&self.h));
                }
                Direction::Backward => {
                    *fr += 2 * extend_diagonal(
                        direction,
                        &self.a,
                        &self.b,
                        self.a.len() as Fr - self.b.len() as Fr - d,
                        self.a.len() as Fr + self.b.len() as Fr - *fr,
                    );
                    for fr in (fr_old..*fr).step_by(2) {
                        self.v.borrow_mut().extend(
                            offset
                                + fr_to_pos(
                                    self.a.len() as Fr - self.b.len() as Fr - d,
                                    self.a.len() as Fr + self.b.len() as Fr - fr,
                                ),
                            g,
                            f_max,
                            Some(&self.h),
                        );
                    }
                    self.v.borrow_mut().expand(
                        offset
                            + fr_to_pos(
                                self.a.len() as Fr - self.b.len() as Fr - d,
                                self.a.len() as Fr + self.b.len() as Fr - *fr,
                            ),
                        g,
                        f_max,
                        Some(&self.h),
                    );
                }
            }
        }

        let target_d = self.a.len() as Fr - self.b.len() as Fr;
        if front.range().contains(&target_d)
            && front.m()[target_d] >= (self.a.len() + self.b.len()) as Fr
        {
            return true;
        }
        false
    }

    /// The range of diagonals to consider for the given cost `g`.
    /// Computes the minimum and maximum possible diagonal reachable for this `g`.
    /// TODO: Some of the functions here should move to EditGraph.
    fn d_range(&self, g: Cost, f_max: Option<Cost>, fronts: &Fronts<N>) -> RangeInclusive<Fr> {
        let g = g as Fr;
        assert!(g > 0);
        let mut r = fronts[g - 1].range().clone();

        EditGraph::iterate_layers(&self.params.cm, |layer| {
            // Find an initial range.
            EditGraph::iterate_parents_dt(
                &self.params.cm,
                layer,
                |di, dj, layer, edge_cost| -> Option<(Fr, Fr)> {
                    // Get start and end of parent layer.
                    let pr = &fronts[g - edge_cost as Fr];
                    let mut start = *pr.range().start();
                    let mut end = *pr.range().end();
                    let d = di - dj;
                    // Shrink range while the parent layer has negative inf values.
                    while start + d < *r.start()
                        && start <= end
                        && pr.layer(layer)[start] == Fr::MIN
                    {
                        start += 1;
                    }
                    while end + d > *r.end() && start <= end && pr.layer(layer)[end] == Fr::MIN {
                        end -= 1;
                    }
                    if start <= end {
                        r = min(*r.start(), start - d)..=max(*r.end(), end - d);
                    }
                    None
                },
                |_di, _dj, _i, _j, _layer, _edge_cost, _cigar_ops| {},
            );
        });

        // If no bound on the cost was specified, return here.
        let Some(f_max) = f_max else {
            return r;
        };

        // Nothing to do.
        if H::IS_DEFAULT && self.params.use_gap_cost_heuristic == GapCostHeuristic::Disable {
            return r;
        }

        // If needed and possible, reduce with gap_cost heuristic.
        if H::IS_DEFAULT {
            assert!(self.params.use_gap_cost_heuristic == GapCostHeuristic::Enable);
            // Shrink the range by distance to end.
            let d = self.a.len() as Fr - self.b.len() as Fr;
            let h_max = f_max - g as Cost;
            // NOTE: Gap open cost was already paid, so we only restrict by extend cost.
            // TODO: Extract this from the EditGraph somehow.
            let gap_cost_r = d - (h_max / self.params.cm.min_del_extend) as Fr
                ..=d + (h_max / self.params.cm.min_ins_extend) as Fr;
            r = max(*r.start(), *gap_cost_r.start())..=min(*r.end(), *gap_cost_r.end());
            return r;
        } else {
            // Only one type of heuristic may be used.
            assert!(self.params.use_gap_cost_heuristic == GapCostHeuristic::Disable);
            let mut d_min = Fr::MAX;
            let mut d_max = Fr::MIN;

            // Find an initial range.
            EditGraph::iterate_parents_dt(
                &self.params.cm,
                // TODO: Fix for affine layers.
                None,
                |di, dj, _layer, edge_cost| -> Option<(Fr, Fr)> {
                    let parent_front = &fronts[g - edge_cost as Fr];
                    d_min = min(d_min, *parent_front.range().start() + (di - dj));
                    d_max = max(d_max, *parent_front.range().end() + (di - dj));
                    None
                },
                |_di, _dj, _i, _j, _layer, _edge_cost, _cigar_ops| {},
            );

            // println!("Initial range {d_min}..={d_max}");

            if d_max < d_min {
                return d_min..=d_max;
            }

            // Shrink the range as needed.

            let test = |d| {
                // Eval for given diagonal. Copied from `next_front`.
                // TODO: dedup.
                let mut fr = Fr::MIN;
                EditGraph::iterate_parents_dt(
                    &self.params.cm,
                    // TODO: Fix for affine layers.
                    None,
                    |di, dj, layer, edge_cost| -> Option<(Fr, Fr)> {
                        let fr = fronts[g - edge_cost as Fr].layer(layer)[d + (di - dj) as Fr]
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
                (pos.0 as usize) <= self.a.len()
                    && (pos.1 as usize) <= self.b.len()
                    && g as Cost + self.h.h(pos) <= f_max
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

    /// Computes layer g from the previous ones.
    /// `ca` is the `i`th character of sequence `a`.
    ///
    /// NOTE: `next` must already have the right range set.
    ///
    /// Returns `true` when the search completes.
    fn next_front(
        &mut self,
        g: Cost,
        f_max: Cost,
        fronts: &mut Fronts<N>,
        offset: Pos,
        start_layer: Layer,
        direction: Direction,
    ) -> bool {
        // Loop over the entire dmin..=dmax range.
        // The boundaries are buffered so no boundary checks are needed.
        // TODO: Vectorize this loop, or at least verify the compiler does this.
        // TODO: Loop over a positive range that does not need additional shifting?
        match direction {
            Direction::Forward => {
                for d in fronts[g as Fr].range().clone() {
                    EditGraph::iterate_layers(&self.params.cm, |layer| {
                        let mut fr = Fr::MIN;
                        EditGraph::iterate_parents_dt(
                            &self.params.cm,
                            layer,
                            |di, dj, layer, edge_cost| -> Option<(Fr, Fr)> {
                                let fr = fronts[g as Fr - edge_cost as Fr].layer(layer)
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
                        let val = &mut fronts[g as Fr].layer_mut(layer)[d];
                        *val = max(*val, fr);
                    });
                }
            }
            Direction::Backward => {
                let mirror = |(i, j)| (self.a.len() as Fr - i, self.b.len() as Fr - j);
                let max_fr = self.a.len() as Fr + self.b.len() as Fr;
                let mirror_fr = |fr| max_fr - fr;
                for d in fronts[g as Fr].range().clone() {
                    EditGraph::iterate_layers(&self.params.cm, |layer| {
                        let mut fr = Fr::MIN;
                        EditGraph::iterate_children_dt(
                            &self.params.cm,
                            layer,
                            // NOTE: This returns a forward position.
                            |di, dj, layer, edge_cost| -> Option<(Fr, Fr)> {
                                let fr = fronts[g as Fr - edge_cost as Fr].layer(layer)
                                    [d - (di - dj) as Fr]
                                    + (di + dj) as Fr;
                                if fr >= 0 {
                                    Some(mirror(fr_to_coords(d, fr)))
                                } else {
                                    None
                                }
                            },
                            |_di, _dj, i, j, _layer, _edge_cost, _cigar_ops| {
                                if i <= self.a.len() as Fr && j <= self.b.len() as Fr {
                                    fr = max(fr, mirror_fr((i + j) as Fr));
                                }
                            },
                        );
                        let val = &mut fronts[g as Fr].layer_mut(layer)[d];
                        *val = max(*val, fr);
                    });
                }
            }
        }

        // We set the fr. point for the 0 diagonal to at least 0.
        // This ensures that fr[d][s] <= fr[d][s'] when s <= s'.
        // That in turn simplifies the overlap condition check, since we only need to check overlap for the two last fronts.
        let front = &mut fronts[g as Fr];
        if front.range().contains(&0) {
            front.layer_mut(start_layer)[0] = max(front.layer(start_layer)[0], 0);
        }

        // Extend all points in the m layer and check if we're done.
        self.extend(g, f_max, &mut fronts[g as Fr], offset, direction)
    }

    // Returns None when the sequences are equal.
    fn init_fronts(
        &mut self,
        f_max: Cost,
        offset: Pos,
        start_layer: Layer,
        end_layer: Layer,
        direction: Direction,
    ) -> Result<Fronts<N>, (Cost, AffineCigar)> {
        let mut fronts = Fronts::new(
            Fr::MIN,
            // We only create a front for the s=0 layer.
            0..=0,
            // The range of the s=0 front is 0..=0.
            |i| if i == 0 { 0..=0 } else { 0..=-1 },
            // Additionally, we have `left_buffer` fronts before the current front.
            self.left_buf,
            0,
            self.top_buf,
            self.bot_buf,
        );

        fronts[0].layer_mut(start_layer)[0] = 0;

        // NOTE: The order of the && here matters!
        if start_layer == None
            && self.extend(0, f_max, &mut fronts[0], offset, direction)
            && end_layer == None
        {
            let mut cigar = AffineCigar::default();
            cigar.match_push(self.a.len() as I);
            Err((0, cigar))
        } else {
            Ok(fronts)
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
        forward: &Fronts<N>,
        backward: &Fronts<N>,
    ) -> Option<(DtState, DtState)> {
        // NOTE: This is the same for the forward and reverse direction.
        let fr_target = (self.a.len() + self.b.len()) as Fr;
        let mirror = |d| (self.a.len() as Fr - self.b.len() as Fr) - d;
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
        EditGraph::iterate_layers(&self.params.cm, |layer| {
            for d in d_range.clone() {
                if forward.last().layer(layer)[d] < 0 || backward.last().layer(layer)[mirror(d)] < 0
                {
                    continue;
                }
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
                        f_fr >= 0 && b_fr >= 0 && f_fr + b_fr >= fr_target
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
        offset: Pos,
        start_layer: Layer,
        end_layer: Layer,
    ) -> (Cost, AffineCigar) {
        let mut forward_fronts =
            match self.init_fronts(0, offset, start_layer, end_layer, Direction::Forward) {
                Ok(fronts) => fronts,
                Err(r) => return r,
            };
        let mut backward_fronts =
            match self.init_fronts(0, offset, end_layer, start_layer, Direction::Backward) {
                Ok(fronts) => fronts,
                Err(r) => return r,
            };

        assert!(H::IS_DEFAULT);

        // The top level meet in the middle step is separate, since the distance is not known yet.
        // We check whether the fronts meet after each iteration.
        let mut best_meet: Option<(DtState, DtState)> = None;
        'outer: {
            for s in 1.. {
                // First, take a step in the forward front, then in the backward front.
                for dir in [Direction::Forward, Direction::Backward] {
                    let fronts = match dir {
                        Direction::Forward => &mut forward_fronts,
                        Direction::Backward => &mut backward_fronts,
                    };
                    let range = self.d_range(s, None, fronts);
                    assert!(!range.is_empty());
                    fronts.rotate(range);
                    self.next_front(
                        s,
                        0,
                        fronts,
                        offset,
                        match dir {
                            Direction::Forward => start_layer,
                            Direction::Backward => end_layer,
                        },
                        dir,
                    );

                    if let Some(meet) = self.fronts_overlap(&forward_fronts, &backward_fronts) {
                        let better = if let Some(best_meet) = best_meet {
                            meet.0.s + meet.1.s < best_meet.0.s + best_meet.1.s
                        } else {
                            true
                        };
                        if better {
                            best_meet = Some(meet)
                        }
                    }
                    if let Some(best_meet) = best_meet
                        && (forward_fronts.range().end() + backward_fronts.range().end()) as Cost
                            >= best_meet.0.s
                                + best_meet.1.s
                                + EditGraph::max_edge_cost(&self.params.cm)
                    {
                        break 'outer;
                    }
                }
                self.v.borrow_mut().new_layer(Some(&self.h));
            }
        }

        let (fw, bw) = best_meet.unwrap();

        let pos @ Pos(i, j) = fw.pos();
        self.v
            .borrow_mut()
            .add_meeting_point::<H::Instance<'a>>(offset + pos);
        let mut left = if forward_fronts.full_range().contains(&0) {
            // Rotate the front back as far as needed.
            while (fw.s as Fr) < *forward_fronts.range().end() {
                forward_fronts.rotate_back();
            }
            let cigar = self.trace(
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
            (fw.s, cigar)
        } else {
            let (cost, cigar) = self
                .params
                .build(&self.a[..i as usize], &self.b[..j as usize], self.v)
                .path_between_dc(offset, start_layer, fw.layer);
            assert_eq!(cost, fw.s);
            (cost, cigar)
        };
        let mut right = if backward_fronts.full_range().contains(&0) {
            while (bw.s as Fr) < *backward_fronts.range().end() {
                backward_fronts.rotate_back();
            }
            let mut cigar = self.trace(
                &backward_fronts,
                DtState {
                    d: 0,
                    fr: 0,
                    layer: end_layer,
                    s: 0,
                },
                bw,
                Direction::Backward,
            );
            cigar.reverse();
            (bw.s, cigar)
        } else {
            let (cost, cigar) = self
                .params
                .build(&self.a[i as usize..], &self.b[j as usize..], self.v)
                .path_between_dc(offset + fw.pos(), bw.layer, end_layer);
            assert_eq!(cost, bw.s);

            (cost, cigar)
        };

        // Join
        left.0 += right.0;
        left.1.append(&mut right.1);

        left
    }

    pub fn align_for_bounded_dist<'b>(
        &mut self,
        f_max: Option<Cost>,
    ) -> Option<(Cost, AffineCigar)> {
        self.v
            .borrow_mut()
            .expand(Pos(0, 0), 0, f_max.unwrap_or(0), Some(&self.h));
        let mut fronts = match self.init_fronts(
            f_max.unwrap_or(0),
            Pos(0, 0),
            None,
            None,
            Direction::Forward,
        ) {
            Ok(fronts) => fronts,
            Err(r) => return Some(r),
        };

        let mut s = 0;
        loop {
            s += 1;
            if let Some(f_max) = f_max
                && s > f_max
            {
                return None;
            }

            // We can not initialize all layers directly at the start, since we do not know the final distance s.
            let range = self.d_range(s, f_max, &fronts);
            if range.is_empty() {
                return None;
            }
            fronts.push_default_front(range);
            if self.next_front(
                s,
                f_max.unwrap_or(0),
                &mut fronts,
                Pos(0, 0),
                None,
                Direction::Forward,
            ) {
                break;
            }
            self.v.borrow_mut().new_layer(Some(&self.h));
        }

        let cigar = self.trace(
            &fronts,
            DtState::start(),
            DtState::target(&self.a, &self.b, s),
            Direction::Forward,
        );
        self.visualize_last_frame(fronts, &cigar);
        Some((s, cigar))
    }

    pub fn align_local_band_doubling<'b>(&mut self) -> (Cost, AffineCigar) {
        const D: bool = false;

        // Front g has been computed up to this f.
        let mut f_max = vec![self.h.h(Pos(0, 0))];

        self.v
            .borrow_mut()
            .expand::<NoCostI>(Pos(0, 0), 0, f_max[0], None);
        let mut fronts = match self.init_fronts(f_max[0], Pos(0, 0), None, None, Direction::Forward)
        {
            Ok(fronts) => fronts,
            Err(r) => return r,
        };

        // Each time a front is grown, it grows to the least multiple of delta that is large enough.
        // Delta doubles after each grow.
        const GROWTH: Cost = 3;
        let mut f_delta = vec![GROWTH];

        // The value of f at the tip. When going to the next front, this is
        // incremented until the range is non-empty.
        let mut f_tip = self.h.h(Pos(0, 0));

        let mut g = 0;
        let distance = 'outer: loop {
            g += 1;
            // We can not initialize all layers directly at the start, since we do not know the final distance s.
            let mut range;
            loop {
                range = self.d_range(g, Some(f_tip), &fronts);
                if !range.is_empty() {
                    break;
                }
                f_tip += 1;
            }
            f_max.push(f_tip);
            f_delta.push(GROWTH);
            fronts.push_default_front(range);

            if D {
                println!("Compute {g} up to {f_tip}");
            }

            // Grow previous front sizes as long as their f_max is not large enough.
            let mut start_g = g as usize;
            let mut last_grow = 0;
            while start_g > 1 && f_max[start_g - 1] < f_max[start_g] {
                start_g -= 1;
                // Check if (after pruning) the range for start_g needs to grow at all.
                // TODO: Generalize to multiple layers.
                {
                    let front = &fronts[start_g as Fr];
                    // println!("Check existing front {start_g}: {front:?}");
                    let ks = *front.range().start();
                    let s = fr_to_pos(ks, front.m()[ks]);

                    let ke = *front.range().end();
                    let e = fr_to_pos(ke, front.m()[ke]);

                    if D && false {
                        println!("{start_g} {ks} {s}");
                        println!(
                            "Diagonal {ks}\t g {} + h {} > f_next {} (f_cur {})",
                            start_g,
                            self.h.h(s),
                            f_max[start_g + 1],
                            f_max[start_g]
                        );
                        println!(
                            "Diagonal {ke}\t g {} + h {} > f_next {} (f_cur {})",
                            start_g,
                            self.h.h(e),
                            f_max[start_g + 1],
                            f_max[start_g]
                        );
                    }
                    // FIXME: Generalize to more layers.
                    if start_g as Cost + self.h.h(s) > f_max[start_g + 1]
                        && start_g as Cost + self.h.h(e) > f_max[start_g + 1]
                    {
                        start_g += 1;
                        if D && false {
                            println!(
                                "Stop. Front {} is last to reuse. Col {start_g} is recomputed",
                                start_g - 1
                            );
                        }
                        break;
                    }
                }

                let before = f_max[start_g];
                let delta = &mut f_delta[start_g];
                f_max[start_g] = (f_max[start_g + 1] + *delta - 1) / *delta * *delta;
                if D && false {
                    println!(
                        "Grow {start_g} from {before} by {delta} to {}",
                        f_max[start_g]
                    );
                }
                if f_max[start_g] > last_grow {
                    last_grow = f_max[start_g];
                    if D {
                        println!(
                            "Grow {start_g} from {before} by {delta} to {}",
                            f_max[start_g]
                        );
                    }
                }
                assert!(
                    f_max[start_g] >= f_max[start_g + 1],
                    "Doubling not enough!? From {before} to {} by {delta} target {}",
                    f_max[start_g],
                    f_max[start_g + 1]
                );
                *delta *= GROWTH;
            }

            // Recompute all fronts from start_g upwards.
            for g in start_g as Cost..=g {
                let range = self.d_range(g, Some(f_max[g as usize]), &fronts);
                let prev_range = fronts[g as Fr].range().clone();
                let new_range =
                    min(*range.start(), *prev_range.start())..=max(*range.end(), *prev_range.end());
                fronts[g as Fr].reset(0, new_range);
                let done = self.next_front(
                    g,
                    f_max[g as usize],
                    &mut fronts,
                    Pos(0, 0),
                    None,
                    Direction::Forward,
                );
                if D && false {
                    println!(
                        "New front {g} at {}: {:?}",
                        f_max[g as usize], fronts[g as Fr]
                    );
                }
                // PRUNING
                // On expanding a state, we prune:
                // - the state itself if it is the start/end of a seed.
                // - the preceding seed start/end, if it is between the previous and current fronts.
                let front = &fronts[g as Fr];
                let prev_front = &fronts[g as Fr - 1];
                let h_before = self.h.h(Pos(0, 0));
                for k in front.range().clone() {
                    let p = fr_to_pos(k, front.m()[k]);
                    if p.0 >= self.a.len() as I || p.1 >= self.b.len() as I {
                        continue;
                    }
                    if self.h.is_seed_start_or_end(p) {
                        self.h.prune(p, Default::default());
                    }
                    // Try pruning the previous start-of-seed position on this diagonal.
                    if let Some(seeds) = &self.h.seeds()
                        && let Some(&prev_fr) = prev_front.m().get(k)
                        && let Some(prev_seed) = seeds.seed_ending_at(p)
                    {
                        let prev_p = p - Pos(p.0 - prev_seed.start, p.0 - prev_seed.start);
                        if pos_to_fr(prev_p).1 >= prev_fr {
                            self.h.prune(prev_p, Default::default());
                        }
                    }
                }
                let h_after = self.h.h(Pos(0, 0));
                if D && false {
                    println!("Pruning: {h_before} => {h_after}");
                }

                if done {
                    break 'outer g;
                }
            }

            self.v.borrow_mut().new_layer(Some(&self.h));
        };
        let cigar = self.trace(
            &fronts,
            DtState::start(),
            DtState::target(self.a, self.b, distance),
            Direction::Forward,
        );
        self.visualize_last_frame(fronts, &cigar);
        (distance, cigar)
    }

    fn visualize_last_frame(&mut self, fronts: Fronts<N>, cigar: &AffineCigar) {
        self.v.borrow_mut().last_frame::<H::Instance<'_>>(
            Some(&cigar),
            Some(
                &(|st| {
                    // Determine the cost for the position.
                    let mut dst = DtState::from_pos(st.pos(), 0);
                    dst.layer = st.layer;
                    loop {
                        let front = &fronts[dst.s as Fr];
                        if front.range().contains(&dst.d) && front.layer(dst.layer)[dst.d] >= dst.fr
                        {
                            break;
                        }
                        dst.s += 1;
                    }

                    self.parent(&fronts, dst, Direction::Forward).map(|x| {
                        let p = x.0.to_pos();
                        (
                            State {
                                i: p.0,
                                j: p.1,
                                layer: x.0.layer,
                            },
                            x.1,
                        )
                    })
                }),
            ),
            Some(&self.h),
        );
    }

    /// The cost-only version uses linear memory.
    ///
    /// In particular, the number of fronts is max(sub, ins, del)+1.
    fn cost_for_bounded_dist(&mut self, f_max: Option<Cost>) -> Option<Cost> {
        self.v
            .borrow_mut()
            .expand::<NoCostI>(Pos(0, 0), 0, f_max.unwrap_or(0), None);
        let mut fronts = match self.init_fronts(
            f_max.unwrap_or(0),
            Pos(0, 0),
            None,
            None,
            Direction::Forward,
        ) {
            Ok(fronts) => fronts,
            Err(r) => return Some(r.0),
        };

        for s in 1.. {
            if let Some(f_max) = f_max
                && s > f_max
            {
                return None;
            }
            let range = self.d_range(s, f_max, &fronts);
            if range.is_empty() {
                return None;
            }
            fronts.rotate(range);
            if self.next_front(
                s,
                f_max.unwrap_or(0),
                &mut fronts,
                Pos(0, 0),
                None,
                Direction::Forward,
            ) {
                self.v.borrow_mut().last_frame(None, None, Some(&self.h));
                return Some(s);
            }
            self.v.borrow_mut().new_layer(Some(&self.h));
        }

        unreachable!()
    }

    fn parent(
        &self,
        fronts: &Fronts<N>,
        st: DtState,
        direction: Direction,
    ) -> Option<(DtState, AffineCigarOps)> {
        if st.is_root() {
            return None;
        }
        let mut max_fr = Fr::MIN;
        let mut parent = None;
        let mut cigar_ops = [None, None];

        match direction {
            Forward => 'forward: {
                if self.params.path_tracing_method == PathTracingMethod::ReverseGreedy {
                    // If reverse greedy matching is asked for, walk backwards
                    // along matching edges if possible.
                    if st.layer == None {
                        let (i, j) = fr_to_coords(st.d, st.fr);
                        if i > 0
                            && j > 0
                            && let Some(ca) = self.a.get(i as usize - 1)
                            && let Some(cb) = self.b.get(j as usize - 1)
                            && ca == cb
                        {
                            parent = Some(st);
                            parent.as_mut().unwrap().fr -= 2;
                            cigar_ops = [Some(AffineCigarOp::Match), None];
                            break 'forward;
                        }
                    }
                }

                EditGraph::iterate_parents_dt(
                    &self.params.cm,
                    st.layer,
                    |di, dj, layer, edge_cost| -> Option<(Fr, Fr)> {
                        let parent_cost = st.s as Fr - edge_cost as Fr;
                        if parent_cost < 0 || !fronts.full_range().contains(&parent_cost) {
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
                        let fr = (i + j) as Fr;
                        // Prefer indel edges over substitution edges.
                        if fr > max_fr || (fr == max_fr && (di != dj || layer != None)) {
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

                // Match
                // TODO: Add a setting to do greedy backtracking before checking other parents.
                if max_fr < st.fr {
                    assert!(st.layer == None);
                    let (i, j) = fr_to_coords(st.d, st.fr);
                    assert!(i > 0 && j > 0, "bad coords {i} {j}");
                    assert_eq!(self.a[i as usize - 1], self.b[j as usize - 1]);
                    parent = Some(st);
                    parent.as_mut().unwrap().fr -= 2;
                    cigar_ops = [Some(AffineCigarOp::Match), None];
                }
            }

            Backward => {
                let mirror = |(i, j)| (self.a.len() as Fr - i, self.b.len() as Fr - j);
                //let mirror_pos = |Pos(i, j)| Pos(a.len() as u32 - i, b.len() as u32 - j);
                let mirror_fr = |fr| self.a.len() as Fr + self.b.len() as Fr - fr;

                EditGraph::iterate_children_dt(
                    &self.params.cm,
                    st.layer,
                    |di, dj, layer, edge_cost| -> Option<(Fr, Fr)> {
                        let parent_cost = st.s as Fr - edge_cost as Fr;
                        if parent_cost < 0 || !fronts.full_range().contains(&parent_cost) {
                            return None;
                        }
                        let fr = fronts[parent_cost].layer(layer)[st.d - (di - dj) as Fr];
                        //+ (di + dj) as Fr;
                        if fr >= 0 {
                            Some(mirror(fr_to_coords(st.d - (di - dj), fr)))
                        } else {
                            None
                        }
                    },
                    |di, dj, i, j, layer, edge_cost, ops| {
                        let fr = mirror_fr((i + j) as Fr) + (di + dj) as Fr;
                        if fr > max_fr {
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

                // Match
                // TODO: Add a setting to do greedy backtracking before checking other parents.
                if max_fr < st.fr {
                    let (i, j) = mirror(fr_to_coords(st.d, st.fr));
                    assert_eq!(self.a[i as usize], self.b[j as usize]);
                    parent = Some(st);
                    parent.as_mut().unwrap().fr -= 2;
                    cigar_ops = [Some(AffineCigarOp::Match), None];
                }
            }
        }
        Some((parent?, cigar_ops))
    }

    fn trace(
        &self,
        fronts: &Fronts<N>,
        from: DtState,
        mut to: DtState,
        direction: Direction,
    ) -> AffineCigar {
        let mut cigar = AffineCigar::default();

        while to != from {
            let (parent, cigar_ops) = self.parent(fronts, to, direction).unwrap();
            to = parent;
            for op in cigar_ops {
                if let Some(op) = op {
                    cigar.push_op(op);
                }
            }
        }
        cigar.reverse();
        cigar
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic> DiagonalTransition<N, V, H> {
    pub fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        let v = &RefCell::new(self.v.build(a, b));
        let mut dt = self.build(a, b, v);
        let cost = if self.use_gap_cost_heuristic == GapCostHeuristic::Enable || !H::IS_DEFAULT {
            exponential_search(0, self.cm.gap_cost(Pos(0, 0), Pos::target(a, b)), 2., |s| {
                dt.cost_for_bounded_dist(Some(s)).map(|c| (c, c))
            })
            .1
        } else {
            dt.cost_for_bounded_dist(None).unwrap()
        };
        dt.v.borrow_mut().last_frame::<NoCostI>(None, None, None);
        cost
    }

    pub fn align(&mut self, a: Seq, b: Seq) -> (Cost, AffineCigar) {
        let v = &RefCell::new(self.v.build(a, b));
        let mut dt = self.build(a, b, v);
        if self.dc {
            // D&C does not work with a heuristic yet, since the target state (where
            // the fronts meet) is not know.
            assert!(H::IS_DEFAULT);
            assert!(self.use_gap_cost_heuristic == GapCostHeuristic::Disable);
            assert!(!self.local_doubling);

            dt.v.borrow_mut().expand::<NoCostI>(Pos(0, 0), 0, 0, None);
            let (cost, cigar) = dt.path_between_dc(Pos(0, 0), None, None);
            dt.v.borrow_mut()
                .last_frame::<NoCostI>(Some(&cigar), None, None);
            (cost, cigar)
        } else {
            let cc;
            if self.local_doubling {
                assert!(
                    !H::IS_DEFAULT,
                    "Local doubling only works with a heuristic."
                );
                cc = dt.align_local_band_doubling();
            } else if self.use_gap_cost_heuristic == GapCostHeuristic::Enable || !H::IS_DEFAULT {
                cc = exponential_search(
                    0,
                    self.cm.gap_cost(Pos(0, 0), Pos::target(a, b)),
                    2.,
                    |s| dt.align_for_bounded_dist(Some(s)).map(|x @ (c, _)| (c, x)),
                )
                .1;
                //self.v.borrow_mut().last_frame(Some(&cc.1));
            } else {
                cc = dt.align_for_bounded_dist(None).unwrap();
            };
            cc
        }
    }

    pub fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, f_max: Cost) -> Option<Cost> {
        self.build(a, b, &RefCell::new(self.v.build(a, b)))
            .cost_for_bounded_dist(Some(f_max))
    }

    pub fn align_for_bounded_dist(
        &mut self,
        a: Seq,
        b: Seq,
        f_max: Cost,
    ) -> Option<(Cost, AffineCigar)> {
        self.build(a, b, &RefCell::new(self.v.build(a, b)))
            .align_for_bounded_dist(Some(f_max))
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic> AffineAligner for DiagonalTransition<N, V, H> {
    fn align_affine(&mut self, a: Seq, b: Seq) -> (Cost, Option<AffineCigar>) {
        let (cost, cigar) = self.align(a, b);
        (cost, Some(cigar))
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic> Aligner for DiagonalTransition<N, V, H> {
    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Option<Cigar>) {
        let (cost, cigar) = self.align(a, b);
        (cost, Some(cigar.into()))
    }
}
