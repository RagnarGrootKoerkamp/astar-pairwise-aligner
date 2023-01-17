use super::cigar::CigarExt;
use super::dt::Direction;
use super::edit_graph::{CigarOps, EditGraph, State};
use super::{exponential_search, Aligner};
use super::{Seq, Sequence};
use crate::cost_model::*;
use crate::heuristic::{Heuristic, HeuristicInstance, NoCost};
use crate::prelude::{Pos, I};
use crate::visualizer::{NoVisualizer, VisualizerConfig, VisualizerT};
use itertools::chain;
use std::cmp::{max, min};
use std::ops::RangeInclusive;

/// Needleman-Wunsch aligner.
///
/// NOTE: Heuristics only support unit cost graph for now.
#[derive(Clone)]
pub struct NW<const N: usize, V: VisualizerConfig, H: Heuristic> {
    /// The cost model to use.
    pub cm: AffineCost<N>,

    /// When false, the band covers all states with distance <=s.
    /// When true, we only cover states with distance <=s/2.
    pub use_gap_cost_heuristic: bool,

    pub exponential_search: bool,

    pub local_doubling: bool,

    /// The heuristic to use.
    pub h: H,

    /// The visualizer to use.
    pub v: V,
}

impl<const N: usize> NW<N, NoVisualizer, NoCost> {
    pub fn new(cm: AffineCost<N>, use_gap_cost_heuristic: bool, exponential_search: bool) -> Self {
        Self {
            cm,
            use_gap_cost_heuristic,
            exponential_search,
            local_doubling: false,
            h: NoCost,
            v: NoVisualizer,
        }
    }
}

impl<const N: usize, V: VisualizerConfig, H: Heuristic> NW<N, V, H> {
    pub fn build<'a>(&self, a: Seq<'a>, b: Seq<'a>) -> NWInstance<'a, N, V, H> {
        NWInstance {
            a: pad(a),
            b: pad(b),
            params: self.clone(),
            h: self.h.build(a, b),
            v: self.v.build(a, b),
        }
    }
}

impl<const N: usize, V: VisualizerConfig, H: Heuristic> std::fmt::Debug for NW<N, V, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NW")
            .field("use_gap_cost_heuristic", &self.use_gap_cost_heuristic)
            .field("h", &self.h)
            .finish()
    }
}

pub struct NWInstance<'a, const N: usize, V: VisualizerConfig, H: Heuristic> {
    // NOTE: `a` and `b` are padded sequences and hence owned.
    pub a: Sequence,
    pub b: Sequence,

    pub params: NW<N, V, H>,

    /// The heuristic to use.
    pub h: H::Instance<'a>,

    /// The visualizer to use.
    pub v: V::Visualizer,
}

/// Type used for indexing sequences.
type Idx = isize;

// TODO: Instead use saturating add everywhere?
const INF: Cost = Cost::MAX / 2;

/// The base vector M, and one vector per affine layer.
/// TODO: Possibly switch to a Vec<Layer> instead.
type Front<const N: usize> = super::front::Front<N, Cost, Idx>;
type Fronts<const N: usize> = super::front::Fronts<N, Cost, Idx>;

/// NW DP only needs the cell just left and above of the current cell.
const LEFT_BUFFER: Idx = 2;
const RIGHT_BUFFER: Idx = 2;

impl<'a, const N: usize, V: VisualizerConfig, H: Heuristic> NWInstance<'a, N, V, H> {
    /// Computes the next front (front `i`) from the current one.
    ///
    /// `a` and `b` must be padded at the start by the same character.
    /// `i` and `j` will always be > 0.
    fn next_front(&mut self, i: Idx, f_max: Cost, prev: &Front<N>, next: &mut Front<N>) {
        for j in next.range().clone() {
            EditGraph::iterate_layers(&self.params.cm, |layer| {
                let mut best = INF;
                EditGraph::iterate_parents(
                    &self.a,
                    &self.b,
                    &self.params.cm,
                    /*greedy_matching=*/ false,
                    State::new(i, j, layer),
                    |di, dj, layer, edge_cost, _cigar_ops| {
                        let parent_cost = if di == 0 {
                            next.layer(layer).get(j + dj)
                        } else {
                            prev.layer(layer).get(j + dj)
                        };
                        if let Some(cost) = parent_cost {
                            best = min(best, cost + edge_cost);
                        }
                    },
                );
                next.layer_mut(layer)[j] = best;
            });
            let pos = Pos::from(i - 1, j - 1);
            self.v.expand_with_h(pos, next.m()[j], f_max, Some(&self.h));
        }
    }

    /// The range of rows `j` to consider in column `i`, when the cost is bounded by `f_bound`.
    fn j_range(&self, i: Idx, f_bound: Option<Cost>, prev: &Front<N>) -> RangeInclusive<Idx> {
        // Without a bound on the distance, we can notuse any heuristic.
        let Some(s) = f_bound else {
            return 1..=self.b.len() as Idx;
        };
        if H::IS_DEFAULT {
            // For the default heuristic, either use the full range of diagonals
            // covered by distance `f_max`, or do only the gap-cost to the end when
            // needed.
            let range = if self.params.use_gap_cost_heuristic {
                let d = self.b.len() as Idx - self.a.len() as Idx;
                // We subtract the cost needed to bridge the gap from the start to the end.
                let s = s - self
                    .params
                    .cm
                    .gap_cost(Pos(0, 0), Pos::target(&self.a, &self.b));
                // Each extra diagonal costs one insertion and one deletion.
                let extra_diagonals =
                    s / (self.params.cm.min_ins_extend + self.params.cm.min_del_extend);
                // NOTE: The range could be reduced slightly further by considering gap open costs.
                min(d, 0) - extra_diagonals as Idx..=max(d, 0) + extra_diagonals as Idx
            } else {
                -(self.params.cm.max_del_for_cost(s) as Idx)
                    ..=self.params.cm.max_ins_for_cost(s) as Idx
            };
            // crop
            max(i + *range.start(), 1)..=min(i + *range.end(), self.b.len() as Idx)
        } else {
            if i == 0 {
                0..=0
            } else {
                // Start with the range of the previous front.
                // Then:
                // Keep increasing the start while prev[start]+h() > f_max.
                // Keep decreasing the end while prev[end]+h() > f_max.
                // Keep increasing the end while prev[prev_end]+extend_cost*(end-prev_end)+h() > f_max.
                let mut start = *prev.range().start();

                // To fix the padded character, we do max(start, 1) and max(i,1).
                // TODO: include the cost needed to transition from column `prev`/`i-1` to the current column.
                // h.h has (-1, -1) to offset the padding.
                while start < self.b.len() as Idx
                    && start <= *prev.range().end() // FIXME: +1
                    // FIXME: the -1 at the end may not be needed with more precise analysis.
                    && prev.m()[start] + self.h.h(Pos::from(max(i, 1) - 1, max(start, 1) - 1))-1 > s
                {
                    start += 1;
                }

                start = max(start, 1);
                if start > prev.range().end() + 1 {
                    return start..=start - 1;
                }
                let prev_end = *prev.range().end();
                let prev_end_cost = prev.m()[prev_end];
                let mut end = prev_end;

                // Decrease end as needed.
                while end >= start
                    && min(prev.m()[end], *prev.m().get(end - 1).unwrap_or(&Cost::MAX))
                        + self.h.h(Pos::from(max(i, 1) - 1, end - 1))
                        > s
                {
                    end -= 1;
                }

                // Increase end as needed, when not already decreased.
                if end == prev_end {
                    // We use the cheapest possible way to extend vertically.
                    // h.h has (-1, -1) to offset the padding.
                    while end < self.b.len() as Idx
                        && prev_end_cost
                            + self
                                .params
                                .cm
                                .extend_cost(Pos::from(i - 1, prev_end), Pos::from(i, end + 1))
                            + self.h.h(Pos::from(i - 1, end + 1 - 1))
                            <= s
                    {
                        end += 1;
                    }
                }
                start..=end
            }
        }
    }

    pub fn align_local_band_doubling<'b>(&mut self) -> (Cost, CigarExt) {
        assert!(
            !H::IS_DEFAULT,
            "Local doubling needs a heuristic. Use -H zero to disable."
        );

        let mut fronts = Fronts::new(
            INF,
            // The fronts to create.
            0..=0 as Idx,
            // The range for each front.
            |i| self.j_range(i, Some(self.h.h(Pos(0, 0))), &Front::default()),
            0,
            0,
            LEFT_BUFFER,
            RIGHT_BUFFER,
        );
        fronts[0].m_mut()[0] = 0;

        // Front i has been computed up to this f.
        let mut f_max = vec![self.h.h(Pos(0, 0))];
        // Each time a front is grown, it grows to the least multiple of delta that is large enough.
        // Delta doubles after each grow.
        const DELTA_0: Cost = 2;
        let mut f_delta = vec![2];

        // The value of f at the tip. When going to the next front, this is
        // incremented until the range is non-empty.
        let mut f_tip = self.h.h(Pos(0, 0));

        let mut i = 0;
        loop {
            if i < self.a.len() as Idx {
                // Add a new front.
                i += 1;
                let mut range;
                loop {
                    // println!("{i} => {f_tip} try");
                    range = self.j_range(i, Some(f_tip), &fronts[i - 1]);
                    if !range.is_empty() {
                        break;
                    }
                    f_tip += 1;
                }
                f_max.push(f_tip);
                f_delta.push(DELTA_0);
                fronts.push(range);
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
                    let front = &fronts[start_i as Idx];
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
                    if front.m()[js as Idx] + self.h.h(Pos(start_i as I - 1, js as I - 1))
                        > f_max[start_i + 1]
                        && front.m()[je as Idx] + self.h.h(Pos(start_i as I - 1, je as I - 1))
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
                f_max[start_i] = (f_max[start_i + 1] + *delta - 1) / *delta * *delta;
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
            for i in start_i as Idx..=i {
                let range = self.j_range(i, Some(f_max[i as usize]), &fronts[i - 1]);
                let prev_range = fronts[i as Idx].range().clone();
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
                fronts[i as Idx].reset(INF, new_range.clone());
                let (prev, next) = fronts.split_at(i);
                self.next_front(i, f_max[i as usize], prev, next);

                // for j in new_range.clone() {
                //     println!(
                //         "row {j} \t g-prev {:10} \t h-new {}",
                //         fronts[i as Idx].m().get(j).unwrap_or(&Cost::MAX),
                //         h.h(Pos(i as I - 1, j as I - 1))
                //     )
                // }

                // Prune matches
                if self.h.is_seed_start_or_end(Pos(i as I - 1, 0)) {
                    // println!("Prune col {}\t {new_range:?}", i - 1);
                    let hint = self
                        .h
                        .h_with_hint(Pos(i as I, *new_range.start() as I), Default::default())
                        .1;
                    for j in new_range {
                        self.h.prune(Pos(i as I - 1, j as I), hint);
                    }
                }

                self.v.new_layer_with_h(Some(&self.h));
            }

            if i == self.a.len() as Idx
                && fronts[self.a.len() as Idx]
                    .range()
                    .contains(&(self.b.len() as Idx))
            {
                break;
            }
        }
        let dist = *fronts[self.a.len() as Idx]
            .m()
            .get(self.b.len() as Idx)
            .unwrap();
        let cigar = self.trace(
            &fronts,
            State {
                i: 1,
                j: 1,
                layer: None,
            },
            State {
                i: self.a.len() as Idx,
                j: self.b.len() as Idx,
                layer: None,
            },
            Direction::Forward,
        );
        self.v.last_frame_with_h(Some(&cigar), None, Some(&self.h));
        (dist, cigar)
    }

    fn parent(
        &self,
        fronts: &Fronts<N>,
        st: State,
        direction: Direction,
    ) -> Option<(State, CigarOps)> {
        assert!(direction == Direction::Forward);
        let cur_cost = fronts[st.i].layer(st.layer)[st.j];
        let mut parent = None;
        let mut cigar_ops: CigarOps = [None, None];
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
                            fronts[st.i + di].layer(new_layer).get(st.j + dj)
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
        fronts: &Fronts<N>,
        from: State,
        mut to: State,
        direction: Direction,
    ) -> CigarExt {
        let mut cigar = CigarExt::default();

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
        let ref mut prev = Front::default();
        let ref mut next = Front::new(INF, self.j_range(0, f_max, prev), LEFT_BUFFER, RIGHT_BUFFER);
        next.m_mut()[0] = 0;
        for i in 1..=self.a.len() as Idx {
            std::mem::swap(prev, next);
            // Update front size.
            let range = self.j_range(i, f_max, prev);
            if range.is_empty() {
                return None;
            }
            next.reset(INF, range);
            self.next_front(i, f_max.unwrap_or(0), prev, next);
            if !self.params.exponential_search {
                self.v.new_layer_with_h(Some(&self.h));
            }
        }
        if self.params.exponential_search {
            self.v.new_layer_with_h(Some(&self.h));
        }
        if let Some(&dist) = next.m().get(self.b.len() as Idx) {
            Some(dist)
        } else {
            None
        }
    }

    /// Tries to find a path with cost <= s.
    /// Returns None if cost > s, or the actual cost otherwise.
    // TODO: Pass `h` into this function, instead of re-initializing it repeatedly for exponential search.
    pub fn align_for_bounded_dist(&mut self, f_max: Option<Cost>) -> Option<(Cost, CigarExt)> {
        let mut fronts = Fronts::new(
            INF,
            // The fronts to create.
            0..=0 as Idx,
            // The range for each front.
            |i| self.j_range(i, f_max, &Front::default()),
            0,
            0,
            LEFT_BUFFER,
            RIGHT_BUFFER,
        );
        fronts[0].m_mut()[0] = 0;

        for i in 1..=self.a.len() as Idx {
            let prev = &fronts[i - 1];
            let range = self.j_range(i, f_max, prev);
            if range.is_empty() {
                return None;
            }
            let mut next = Front::new(INF, range, LEFT_BUFFER, RIGHT_BUFFER);
            self.next_front(i, f_max.unwrap_or(0), prev, &mut next);
            fronts.fronts.push(next);
            if !self.params.exponential_search {
                self.v.new_layer_with_h(Some(&self.h));
            }
        }

        if let Some(&dist) = fronts[self.a.len() as Idx].m().get(self.b.len() as Idx) {
            // We only track the actual path if `s` is small enough.
            if dist <= f_max.unwrap_or(INF) {
                let cigar = self.trace(
                    &fronts,
                    State {
                        i: 1,
                        j: 1,
                        layer: None,
                    },
                    State {
                        i: self.a.len() as Idx,
                        j: self.b.len() as Idx,
                        layer: None,
                    },
                    Direction::Forward,
                );
                return Some((dist, cigar));
            }
        }

        if self.params.exponential_search {
            self.v.new_layer_with_h(Some(&self.h));
        }
        None
    }
}

fn pad(a: Seq) -> Sequence {
    chain!(b"^", a).copied().collect()
}

impl<const N: usize, V: VisualizerConfig, H: Heuristic> Aligner for NW<N, V, H> {
    fn cost(&mut self, a: Seq, b: Seq) -> Cost {
        let mut nw = self.build(a, b);
        let cost = if self.exponential_search {
            exponential_search(self.cm.gap_cost(Pos(0, 0), Pos::target(a, b)), 2., |s| {
                nw.cost_for_bounded_dist(Some(s)).map(|c| (c, c))
            })
            .1
        } else {
            assert!(!self.use_gap_cost_heuristic && H::IS_DEFAULT);
            nw.cost_for_bounded_dist(None).unwrap()
        };
        nw.v.last_frame(None);
        cost
    }

    fn align(&mut self, a: Seq, b: Seq) -> (Cost, CigarExt) {
        let mut nw = self.build(a, b);
        let cc;
        if self.local_doubling {
            return nw.align_local_band_doubling();
        } else if self.exponential_search {
            cc = exponential_search(
                // TODO: Take a max with h(0,0) here.
                self.cm.gap_cost(Pos(0, 0), Pos::target(a, b)),
                2.,
                |s| nw.align_for_bounded_dist(Some(s)).map(|x @ (c, _)| (c, x)),
            )
            .1;
        } else {
            assert!(!self.use_gap_cost_heuristic && H::IS_DEFAULT);
            cc = nw.align_for_bounded_dist(None).unwrap();
        };
        nw.v.last_frame(Some(&cc.1));
        cc
    }

    fn cost_for_bounded_dist(&mut self, a: Seq, b: Seq, f_max: Cost) -> Option<Cost> {
        self.build(a, b).cost_for_bounded_dist(Some(f_max))
    }

    fn align_for_bounded_dist(&mut self, a: Seq, b: Seq, f_max: Cost) -> Option<(Cost, CigarExt)> {
        self.build(a, b).align_for_bounded_dist(Some(f_max))
    }
}
