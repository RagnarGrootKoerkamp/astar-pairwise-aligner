/// TODO
/// - add bitpacking based implementation + block_height
/// - add reusing computed values when doing A*
use crate::dt::Direction;
use crate::edit_graph::{AffineCigarOps, EditGraph};
use crate::front::nw_front::{NwFront, NwFronts};
use crate::Domain;
use crate::{exponential_search, Strategy};
use pa_affine_types::*;
use pa_heuristic::*;
use pa_types::*;
use pa_vis_types::*;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::ops::{Range, RangeInclusive};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct AstarNwParams {
    /// The domain to compute.
    pub domain: Domain<()>,

    /// Heuristic to use for A* domain.
    pub heuristic: HeuristicParams,

    /// The strategy to use to compute the given domain.
    pub strategy: Strategy,

    /// Compute `block_width` columns at a time, to reduce overhead of metadata
    /// computations.
    pub block_width: I,
}

impl AstarNwParams {
    /// Build an `AstarStatsAligner` instance from
    pub fn make_aligner(&self) -> Box<dyn Aligner> {
        self.make_aligner_with_visualizer(NoVis)
    }

    /// Build a type-erased aligner object from parameters.
    pub fn make_aligner_with_visualizer<V: VisualizerT + 'static>(&self, v: V) -> Box<dyn Aligner> {
        struct Mapper<V: VisualizerT> {
            params: AstarNwParams,
            v: V,
        }
        impl<V: VisualizerT + 'static> HeuristicMapper for Mapper<V> {
            type R = Box<dyn Aligner>;
            fn call<H: Heuristic + 'static>(self, h: H) -> Box<dyn Aligner> {
                Box::new(NW {
                    cm: AffineCost::unit(),
                    domain: Domain::Astar(h),
                    strategy: self.params.strategy,
                    block_width: self.params.block_width,
                    v: self.v,
                })
            }
        }
        match self.domain {
            Domain::Astar(()) => self.heuristic.map(Mapper { params: *self, v }),
            d => Box::new(NW {
                cm: AffineCost::unit(),
                domain: d.into(),
                strategy: self.strategy,
                block_width: self.block_width,
                v,
            }),
        }
    }
}

/// Needleman-Wunsch aligner.
///
/// NOTE: Heuristics only support unit cost graph for now.
#[derive(Clone)]
pub struct NW<const N: usize, V: VisualizerT, H: Heuristic> {
    /// The cost model to use.
    pub cm: AffineCost<N>,

    /// The domain to compute.
    pub domain: Domain<H>,

    /// The strategy to use to compute the given domain.
    pub strategy: Strategy,

    /// Compute `block_width` columns at a time, to reduce overhead of metadata
    /// computations.
    pub block_width: I,

    /// The visualizer to use.
    pub v: V,
}

impl<const N: usize> NW<N, NoVis, NoCost> {
    pub fn new(cm: AffineCost<N>, use_gap_cost_heuristic: bool, exponential_search: bool) -> Self {
        Self {
            cm,
            domain: if use_gap_cost_heuristic {
                Domain::GapGap
            } else {
                Domain::Full
            },
            strategy: if exponential_search {
                // FIXME: Make this more general.
                Strategy::band_doubling()
            } else {
                Strategy::None
            },
            // FIXME: Make this more general.
            block_width: 32,
            v: NoVis,
        }
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic> NW<N, V, H> {
    pub fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> NWInstance<'a, N, V, H> {
        use Domain::*;
        NWInstance {
            a,
            b,
            params: self.clone(),
            domain: match self.domain {
                Full => Full,
                GapStart => GapStart,
                GapGap => GapGap,
                Astar(h) => Astar(h.build(a, b)),
            },
            v: self.v.build(a, b),
        }
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic> std::fmt::Debug for NW<N, V, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NW").field("domain", &self.domain).finish()
    }
}

pub struct NWInstance<'a, const N: usize, V: VisualizerT, H: Heuristic> {
    // NOTE: `a` and `b` are padded sequences and hence owned.
    pub a: Seq<'a>,
    pub b: Seq<'a>,

    pub params: NW<N, V, H>,

    /// The heuristic to use.
    pub domain: Domain<H::Instance<'a>>,

    /// The visualizer to use.
    pub v: V::Instance,
}

impl<'a, const N: usize, V: VisualizerT, H: Heuristic> NWInstance<'a, N, V, H> {
    /// The range of rows `j` to consider in column `i`, when the cost is bounded by `f_bound`.
    ///
    /// `i_range`: `[start, end)` range of columns, starting at `prev_i+1`,
    /// Can be e.g. `i..i+1` with `prev` the front for column `i-1`,
    /// or `i..i+W` to compute a block of `W` columns i .. i+W.
    /// Pass `0..1` for the range of the first column. `prev` is not not used.
    fn j_range(
        &self,
        i_range: Range<I>,
        f_max: Option<Cost>,
        prev: &NwFront<N>,
    ) -> RangeInclusive<I> {
        // Without a bound on the distance, we can only return the full range.
        let Some(f_max) = f_max else {
            return 0..=self.b.len() as I;
        };

        // Inclusive start column of the new block.
        let is = i_range.start;
        // Inclusive end column of the new block.
        let ie = i_range.end - 1;

        match &self.domain {
            Domain::Full => 0..=self.b.len() as I,
            Domain::GapStart => {
                // range: the max number of diagonals we can move up/down from the start with cost f.
                let range = -(self.params.cm.max_del_for_cost(f_max) as I)
                    ..=self.params.cm.max_ins_for_cost(f_max) as I;
                // crop
                max(is + *range.start(), 0)..=min(ie + *range.end(), self.b.len() as I)
            }
            Domain::GapGap => {
                let d = self.b.len() as I - self.a.len() as I;
                // We subtract the cost needed to bridge the gap from the start to the end.
                let s = f_max
                    - self
                        .params
                        .cm
                        .gap_cost(Pos(0, 0), Pos::target(&self.a, &self.b));
                // Each extra diagonal costs one insertion and one deletion.
                let extra_diagonals =
                    s / (self.params.cm.min_ins_extend + self.params.cm.min_del_extend);
                // NOTE: The range could be reduced slightly further by considering gap open costs.
                let range = min(d, 0) - extra_diagonals as I..=max(d, 0) + extra_diagonals as I;

                // crop
                max(is + *range.start(), 0)..=min(ie + *range.end(), self.b.len() as I)
            }
            Domain::Astar(h) => {
                // Instead of computing the start and end exactly, we bound them using the computed values of the previous range.

                let f = |i, j| prev.m()[j] + h.h(Pos(i, j));

                // Start: increment the start of the previous range until
                //        f<=f_max is satisfied in previous column.
                // End: decrement the end of the previous range until
                //      f<=f_max is satisfied in previous column.
                let mut start = *prev.range().start();
                let mut end = *prev.range().end();
                if is > 0 {
                    while start <= end + 1 && f(is - 1, start) > f_max {
                        start += 1;
                    }

                    while end >= start && f(is - 1, end) > f_max {
                        end -= 1;
                    }
                }

                // Early return for empty range.
                if start > end + 1 {
                    return start..=start - 1;
                }

                let u = Pos(is - 1, end);
                let du = if is == 0 { 0 } else { prev.m()[end] };
                let mut v = u;

                // Extend `v` diagonally one column at a time towards `ie`.
                // In each column, find the lowest `v` such that
                // `f(v) = g(v) + h(v) <= d(u) + extend_cost(u, v) + h(v) <= s`.
                //
                // NOTE: We can not directly go to the last column, since
                // the optimal path could then 'escape' through the bottom.
                // Without further reasoning, we must evaluate `h` at least
                // once per column.

                while v.0 < ie {
                    // Extend diagonally.
                    v += Pos(1, 1);

                    // TODO: Should we also attempt to decrease `v.1` here? I
                    // don't think it's needed for typical heuristics.

                    // Check if cell below is out-of-reach.
                    v.1 += 1;
                    while v.1 <= self.b.len() as I
                        && du + self.params.cm.extend_cost(u, v) + h.h(v) <= f_max
                    {
                        v.1 += 1;
                    }
                    v.1 -= 1;
                }
                start..=min(v.1, self.b.len() as I)
            }
        }
    }

    pub fn align_local_band_doubling<'b>(&mut self) -> (Cost, AffineCigar) {
        assert!(
            !H::IS_DEFAULT,
            "Local doubling needs a heuristic. Use -H zero to disable."
        );

        let h0 = self.domain.h().unwrap().h(Pos(0, 0));
        let mut fronts = NwFronts::new(
            &self.a,
            &self.b,
            &self.params.cm,
            self.j_range(0..1, Some(h0), &NwFront::default()),
        );

        // Front i has been computed up to this f.
        let mut f_max = vec![h0];
        // Each time a front is grown, it grows to the least multiple of delta that is large enough.
        // Delta doubles after each grow.
        const DELTA_0: Cost = 2;
        let mut f_delta = vec![2];

        // The value of f at the tip. When going to the next front, this is
        // incremented until the range is non-empty.
        let mut f_tip = self.domain.h().unwrap().h(Pos(0, 0));

        let mut i = 0;
        // This is a for loop over `i`, but once `i` reaches `a.len()`, the last
        // front is grown instead of increasing `i`.
        loop {
            if i < self.a.len() as I {
                // Add a new front.
                i += 1;
                let mut range;
                loop {
                    // println!("{i} => {f_tip} try");
                    range = self.j_range(i..i + 1, Some(f_tip), &fronts.fronts[i - 1]);
                    if !range.is_empty() {
                        break;
                    }
                    f_tip += 1;
                }
                f_max.push(f_tip);
                f_delta.push(DELTA_0);
                fronts.fronts.push_default_front(range);
            } else {
                // Only grow the last front.
                let delta = &mut f_delta[i as usize];
                // print!("Double last front from {} by {delta}", f_max[i as usize]);
                f_max[i as usize] = (f_max[i as usize] / *delta + 1) * *delta;
                // println!("to {}", f_max[i as usize]);
                *delta *= 2;
            }

            // Double previous front sizes as long as their f_max is not large enough.
            let mut start_i = i as usize;
            while start_i > 1 && f_max[start_i - 1] < f_max[start_i] {
                // Check if (after pruning) the range for start_i needs to grow at all.
                start_i -= 1;
                {
                    let front = &fronts.fronts[start_i as I];
                    let js = *front.range().start();
                    let je = *front.range().end();
                    // println!(
                    //     "Row {js}\t g {} + h {} > f_next {} (f_cur {})",
                    //     front.m()[js as Idx],
                    //     h.h(Pos(start_i as I - 1, js as I - 1)),
                    //     f_max[start_i + 1],
                    //     f_max[start_i]
                    // );
                    // println!(
                    //     "Row {je}\t g {} + h {} > f_next {} (f_cur {})",
                    //     front.m()[je as Idx],
                    //     h.h(Pos(start_i as I - 1, je as I - 1)),
                    //     f_max[start_i + 1],
                    //     f_max[start_i]
                    // );
                    // FIXME: Generalize to more layers.
                    // NOTE: -1's are to correct for sequence padding.
                    // NOTE: equality isn't good enough: in that case there
                    // could be adjacent states that also have equality.
                    if front.m()[js as I]
                        + self
                            .domain
                            .h()
                            .unwrap()
                            .h(Pos(start_i as I - 1, js as I - 1))
                        > f_max[start_i + 1]
                        && front.m()[je as I]
                            + self
                                .domain
                                .h()
                                .unwrap()
                                .h(Pos(start_i as I - 1, je as I - 1))
                            > f_max[start_i + 1]
                    {
                        start_i += 1;
                        // println!(
                        //     "Stop. Col {} is last to reuse. Col {start_i} is recomputed",
                        //     start_i - 1
                        // );
                        break;
                    }
                }

                let before = f_max[start_i];
                let delta = &mut f_delta[start_i];
                f_max[start_i] = f_max[start_i + 1].next_multiple_of(*delta);
                // println!("{start_i} => {before} -> {} \t ({delta})", f_max[start_i]);
                assert!(
                    f_max[start_i] >= f_max[start_i + 1],
                    "Doubling not enough!? From {before} to {} by {delta} target {}",
                    f_max[start_i],
                    f_max[start_i + 1]
                );
                *delta *= 2;
            }

            if start_i > 1 {
                // for j in fronts[start_i as Idx - 1].range().clone() {
                //     let i = start_i - 1;
                //     println!(
                //         "row {j} \t g-prev {:10} \t h-new {}",
                //         fronts[i as Idx].m().get(j).unwrap_or(&Cost::MAX),
                //         h.h(Pos(i as I - 1, j as I - 1))
                //     )
                // }
            }

            // Recompute all fronts from start_i upwards.
            for i in start_i as I..=i {
                let range = self.j_range(i..i + 1, Some(f_max[i as usize]), &fronts.fronts[i - 1]);
                let prev_range = fronts.fronts[i as I].range().clone();
                let new_range =
                    min(*range.start(), *prev_range.start())..=max(*range.end(), *prev_range.end());
                // println!(
                //     "Compute {i} for {} => {new_range:?} (prev {prev_range:?})",
                //     f_max[i as usize],
                // );
                // if range.is_empty() || true {
                //     for j in new_range.clone() {
                //         println!(
                //             "row {j} \t g-prev {:10} \t h-new {}",
                //             fronts[i as Idx].m().get(j).unwrap_or(&Cost::MAX),
                //             h.h(Pos(i as I - 1, j as I - 1))
                //         )
                //     }
                // }
                assert!(!new_range.is_empty());
                fronts.update_fronts(i..i + 1, new_range.clone(), |pos, g| {
                    self.v.expand(pos, g, f_max[i as usize], self.domain.h())
                });

                // for j in new_range.clone() {
                //     println!(
                //         "row {j} \t g-prev {:10} \t h-new {}",
                //         fronts[i as Idx].m().get(j).unwrap_or(&Cost::MAX),
                //         h.h(Pos(i as I - 1, j as I - 1))
                //     )
                // }

                // Prune matches
                if self
                    .domain
                    .h()
                    .unwrap()
                    .is_seed_start_or_end(Pos(i as I - 1, 0))
                {
                    let hint = self
                        .domain
                        .h()
                        .unwrap()
                        .h_with_hint(Pos(i as I - 1, *new_range.start() as I), Default::default())
                        .1;
                    for j in new_range {
                        self.domain
                            .h_mut()
                            .unwrap()
                            .prune(Pos(i as I - 1, j as I), hint);
                    }
                }

                self.v.new_layer(Some(self.domain.h().unwrap()));
            }

            if i == self.a.len() as I
                && fronts.fronts[self.a.len() as I]
                    .range()
                    .contains(&(self.b.len() as I))
            {
                break;
            }
        } // end loop

        let dist = *fronts.fronts[self.a.len() as I]
            .m()
            .get(self.b.len() as I)
            .unwrap();
        let cigar = self.trace(
            &fronts,
            State {
                i: 1,
                j: 1,
                layer: None,
            },
            State {
                i: self.a.len() as I,
                j: self.b.len() as I,
                layer: None,
            },
            Direction::Forward,
        );
        self.v
            .last_frame(Some(&cigar), None, Some(self.domain.h().unwrap()));
        (dist, cigar)
    }

    fn parent(
        &self,
        fronts: &NwFronts<N>,
        st: State,
        direction: Direction,
    ) -> Option<(State, AffineCigarOps)> {
        assert!(direction == Direction::Forward);
        let cur_cost = fronts.fronts[st.i].layer(st.layer)[st.j];
        let mut parent = None;
        let mut cigar_ops: AffineCigarOps = [None, None];
        EditGraph::iterate_parents(
            &self.a,
            &self.b,
            &self.params.cm,
            /*greedy_matching=*/ false,
            st,
            |di, dj, new_layer, cost, ops| {
                if parent.is_none()
                        // We use `get` to handle possible out-of-bound lookups.
                        && let Some(parent_cost) =
                            fronts.fronts[st.i + di].layer(new_layer).get(st.j + dj)
                        && cur_cost == parent_cost + cost
                    {
                        parent = Some(State::new(st.i + di, st.j + dj, new_layer));
                        cigar_ops = ops;
                    }
            },
        );
        Some((parent?, cigar_ops))
    }

    fn trace(
        &self,
        fronts: &NwFronts<N>,
        from: State,
        mut to: State,
        direction: Direction,
    ) -> AffineCigar {
        let mut cigar = AffineCigar::default();

        while to != from {
            let (parent, cigar_ops) = self.parent(fronts, to, direction).unwrap();
            to = parent;
            for op in cigar_ops {
                if let Some(op) = op {
                    cigar.push(op);
                }
            }
        }
        cigar.reverse();
        cigar
    }

    /// Test whether the cost is at most s.
    /// Returns None if cost > s, or the actual cost otherwise.
    fn cost_for_bounded_dist(&mut self, f_max: Option<Cost>) -> Option<Cost> {
        eprintln!("Bound: {f_max:?}");
        let initial_j_range = self.j_range(0..1, f_max, &NwFront::default());
        let mut fronts = NwFronts::new(&self.a, &self.b, &self.params.cm, initial_j_range);

        for i in (1..=self.a.len() as I).step_by(self.params.block_width as _) {
            let i_range = i..min(i + self.params.block_width, self.a.len() as I + 1);
            let j_range = self.j_range(i_range.clone(), f_max, fronts.last_front());
            if j_range.is_empty() {
                return None;
            }
            fronts.rotate_next_fronts(i_range, j_range, |pos, g| {
                self.v.expand(pos, g, f_max.unwrap_or(0), self.domain.h())
            });
            if self.params.strategy == Strategy::None {
                self.v.new_layer(self.domain.h());
            }
        }
        if self.params.strategy == Strategy::None {
            self.v.new_layer(self.domain.h());
        }
        if let Some(&dist) = fronts.last_front().m().get(self.b.len() as I) {
            Some(dist)
        } else {
            None
        }
    }

    /// Tries to find a path with cost <= s.
    /// Returns None if cost > s, or the actual cost otherwise.
    // TODO: Pass `h` into this function, instead of re-initializing it repeatedly for exponential search.
    pub fn align_for_bounded_dist(&mut self, f_max: Option<Cost>) -> Option<(Cost, AffineCigar)> {
        eprintln!("Bound: {f_max:?}");
        let initial_j_range = self.j_range(0..1, f_max, &NwFront::default());
        self.v.expand_block(
            Pos(0, 0),
            Pos(1, *initial_j_range.end() + 1),
            0,
            f_max.unwrap_or(0),
            self.domain.h(),
        );
        let mut fronts = NwFronts::new(&self.a, &self.b, &self.params.cm, initial_j_range);

        for i in (1..=self.a.len() as I).step_by(self.params.block_width as _) {
            let i_range = i..min(i + self.params.block_width, self.a.len() as I + 1);
            let j_range = self.j_range(i_range.clone(), f_max, fronts.last_front());
            if j_range.is_empty() {
                return None;
            }
            fronts.push_next_fronts(i_range, j_range, |pos, g| {
                self.v.expand(pos, g, f_max.unwrap_or(0), self.domain.h())
            });
            if self.params.strategy == Strategy::None {
                self.v.new_layer(self.domain.h());
            }
        }

        if let Some(&dist) = fronts.fronts[self.a.len() as I].m().get(self.b.len() as I) {
            // We only track the actual path if `s` is small enough.
            if dist <= f_max.unwrap_or(I::MAX) {
                let cigar = self.trace(
                    &fronts,
                    State {
                        i: 1,
                        j: 1,
                        layer: None,
                    },
                    State {
                        i: self.a.len() as I,
                        j: self.b.len() as I,
                        layer: None,
                    },
                    Direction::Forward,
                );
                return Some((dist, cigar));
            }
        }

        if matches!(self.params.strategy, Strategy::BandDoubling(_, _)) {
            self.v.new_layer(self.domain.h());
        }
        None
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic> NW<N, V, H> {
    fn band_doubling_params(
        &mut self,
        start: crate::DoublingStart,
        a: &[u8],
        b: &[u8],
        nw: &NWInstance<N, V, H>,
    ) -> (i32, i32) {
        let (start_f, start_increment) = match start {
            crate::DoublingStart::Zero => (0, 1),
            crate::DoublingStart::Gap => {
                let x = self.cm.gap_cost(Pos(0, 0), Pos::target(a, b));
                (x, x)
            }
            crate::DoublingStart::H0 => (
                nw.domain
                    .h()
                    .expect("DoublingStart::H0 requires an A* domain with heuristic.")
                    .h(Pos(0, 0)),
                1,
            ),
        };
        (start_f, start_increment)
    }

    pub fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        let mut nw = self.build(a, b);
        let cost = match self.strategy {
            Strategy::LocalDoubling => {
                unimplemented!();
            }
            Strategy::BandDoubling(start, factor) => {
                let (start_f, start_increment) = self.band_doubling_params(start, a, b, &nw);
                exponential_search(start_f, start_increment, factor, |s| {
                    nw.cost_for_bounded_dist(Some(s)).map(|c| (c, c))
                })
                .1
            }
            Strategy::None => {
                // FIXME: Allow single-shot alignment with specified domain and threshold.
                assert!(matches!(self.domain, Domain::Full));
                nw.cost_for_bounded_dist(None).unwrap()
            }
        };
        nw.v.last_frame::<NoCostI>(None, None, None);
        cost
    }

    pub fn align(&mut self, a: Seq, b: Seq) -> (Cost, AffineCigar) {
        let mut nw = self.build(a, b);
        let cc = match self.strategy {
            Strategy::LocalDoubling => {
                return nw.align_local_band_doubling();
            }
            Strategy::BandDoubling(start, factor) => {
                let (start_f, start_increment) = self.band_doubling_params(start, a, b, &nw);
                exponential_search(start_f, start_increment, factor, |s| {
                    nw.align_for_bounded_dist(Some(s)).map(|x @ (c, _)| (c, x))
                })
                .1
            }
            Strategy::None => {
                assert!(matches!(self.domain, Domain::Full));
                nw.align_for_bounded_dist(None).unwrap()
            }
        };
        nw.v.last_frame::<NoCostI>(Some(&cc.1), None, None);
        cc
    }

    pub fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, f_max: Cost) -> Option<Cost> {
        self.build(a, b).cost_for_bounded_dist(Some(f_max))
    }

    pub fn align_for_bounded_dist(
        &mut self,
        a: Seq,
        b: Seq,
        f_max: Cost,
    ) -> Option<(Cost, AffineCigar)> {
        self.build(a, b).align_for_bounded_dist(Some(f_max))
    }
}

impl<const N: usize, V: VisualizerT, H: Heuristic> AffineAligner for NW<N, V, H> {
    fn align_affine(&mut self, a: Seq, b: Seq) -> (Cost, Option<AffineCigar>) {
        let (cost, cigar) = self.align(a, b);
        (cost, Some(cigar))
    }
}

impl<V: VisualizerT, H: Heuristic> Aligner for NW<0, V, H> {
    fn align(&mut self, a: Seq, b: Seq) -> (Cost, Option<Cigar>) {
        let (cost, cigar) = self.align(a, b);
        (cost, Some(cigar.into()))
    }
}

// Worst case growth factor analysis
//
// 1, g, g^2, ...
//
// worst-case overshoot: g^k = g*s
// Assuming O(ng) work per guess (Gap, GapGap)
//   n(1+g+...+g^k) = n*(g*g^k-1)/(g-1) = n*(g^2 s-1)/(g-1) ~ ns g^2/(g-1)
//   minimize g^2/(g-1):
//   derivative 0: 0 = (2g (g-1) - g^2) / (g-1)^2 => 0 = g^2-2g = g(g-2)
// g=2
// 4ns
//
// Assuming O(g^2) work per guess (Dijkstra, Astar(GapCost), when errors are uniform)
//   1 + g^2 + g^4 + ... + g^2k ~ g^{2k+2} / (g^2-1) = ns g^4 / (g^2-1)
//   minimize g^4/(g^2-1)
//   derivative 0: 0 = 4g^3(g^2-1) - g^4 2g = 2g^5 - 4g^3 = 2 g^3 (g^2-2)
// g=sqrt(2)
// 2ns
// in case all errors are at the end and runtime is O(ng) per guess:
// 4.8 ns, only slightly worse than 4ns.
//
