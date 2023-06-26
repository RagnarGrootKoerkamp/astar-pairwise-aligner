use std::cell::RefCell;

use itertools::Itertools;

use super::*;
use crate::{prelude::*, split_vec::SplitVec, PRINT};

const D: bool = false;

/// A Contours implementation based on Contour layers with queries in O(log(r)^2).
#[derive(Default, Debug)]
pub struct HintContours<C: Contour> {
    contours: SplitVec<C>,
    // TODO: This should have units in the transformed domain instead.
    max_len: Layer,
    stats: RefCell<HintContourStats>,

    layers_removed: Layer,
}

#[derive(Default, Debug)]
struct HintContourStats {
    // Total number of prunes we do.
    prunes: usize,
    // Number of times f is evaluated.
    checked: usize,
    // Number of times f evaluates to true/false.
    checked_true: usize,
    checked_false: usize,

    // Average # layers a pruned point moves down.
    sum_prune_shifts: Layer,
    num_prune_shifts: usize,
    max_prune_shift: Layer,

    // Total number of layers processed.
    contours: usize,

    // Number of times a shift was possible.
    shifts: usize,

    // Number of times we stop pruning early.
    no_change: usize,
    shift_layers: usize,

    // Binary search stats
    score_with_hint_calls: usize,
    binary_search_fallback: usize,
    contains_calls: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Hint {
    original_layer: Layer,
}

impl Default for Hint {
    fn default() -> Self {
        Self {
            original_layer: Layer::MAX,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Shift {
    None,
    Layers(Layer),
    Inconsistent,
}

impl Shift {
    fn merge(&mut self, other: Shift) -> Shift {
        match *self {
            Shift::None => {
                *self = other;
            }
            Shift::Layers(s) => match other {
                Shift::None => {}
                Shift::Layers(o) if o == s => {}
                _ => {
                    *self = Shift::Inconsistent;
                }
            },
            Shift::Inconsistent => {}
        };
        *self
    }
}

impl<C: Contour> HintContours<C> {
    fn debug<R: Iterator<Item = Arrow>, F: Fn(&Pos) -> Option<R>>(
        &self,
        pos: Pos,
        v: Layer,
        arrows: &F,
    ) {
        eprintln!("BEFORE PRUNE of {pos} layer {v}");
        let radius = 4;
        // For each layer around the current one, print:
        for layer in max(v.saturating_sub(radius), 1)..min(v + radius, self.contours.len() as Layer)
        {
            // - the positions in the layer
            eprintln!("LAYER {layer}");
            self.contours[layer].iterate_points(|p: Pos| {
                let max_len = arrows(&p).map_or(0, |arrows| {
                    arrows.map(|a| a.score).max().expect("Empty arrows")
                });
                eprintln!("Max len: {max_len}");
                // - the arrows starting at each position.
                arrows(&p).map(|arrows| {
                    for a in arrows {
                        eprintln!("{a}");
                    }
                });
                // assert!(
                //     max_len > 0 || p == pos,
                //     "No arrows found for position {p} at layer {layer}"
                // );
            })
        }
    }

    /// Returns None when false, or the first layer >= v that contains the query point.
    fn is_score_at_least(&self, q: Pos, v: Layer) -> Option<Layer> {
        // Test if score >= mid by checking all points in contours mid..mid+r
        for w in v..min(v + self.max_len, self.contours.len() as Layer) {
            if self.contours[w as usize].contains(q) {
                return Some(w);
            }
        }
        None
    }

    /// Check that each arrow is in the correct layer.
    fn check_consistency<R: Iterator<Item = Arrow>, F: Fn(&Pos) -> Option<R>>(
        &mut self,
        arrows: &F,
    ) {
        if !D {
            return;
        }
        for layer in 1..self.contours.len() as Layer {
            let contour_layer = std::mem::take(&mut self.contours[layer]);
            contour_layer.iterate_points(|p: Pos| {
                let max_len = arrows(&p).map_or(0, |arrows| {
                    arrows.map(|a| a.score).max().expect("Empty arrows")
                });
                assert!(max_len > 0);
                let target_layer = chain_score(arrows, p, layer, &self.contours);
                assert!(
                    target_layer == Some(layer),
                    "BAD CONSISTENCY: {p} in layer {layer} should be in layer {target_layer:?}"
                );
            });
            self.contours[layer] = contour_layer;
        }
    }
}

/// Best score of the given `pos` by iterating over all arrows starting there, or `None` otherwise.
fn chain_score<C: Contour, R: Iterator<Item = Arrow>, F: Fn(&Pos) -> Option<R>>(
    arrows: &F,
    pos: Pos,
    v: Layer,
    contours: &SplitVec<C>,
) -> Option<Layer> {
    let pos_arrows = arrows(&pos)?;
    let mut max_score = 0;
    'fr: for arrow in pos_arrows {
        // Find the value at end_val via a linear search.
        let mut end_layer = v as Layer - 1;
        // Commented out because `contains` is not free.
        // FIXME: comment this again
        // assert!(
        //     !contours[v].contains(arrow.end),
        //     "Hint of {v} is no good! Contains {} for arrow {arrow}",
        //     arrow.end
        // );
        while !contours[end_layer].contains(arrow.end) {
            end_layer -= 1;

            // No need to continue when this value isn't going to be optimal anyway.
            if (end_layer + arrow.score as Layer) <= max_score {
                continue 'fr;
            }
        }

        let start_layer = end_layer + arrow.score as Layer;
        if D {
            let mut witness = None;
            assert!(contours[end_layer].contains(arrow.end));
            contours[end_layer].iterate_points(|p| {
                if arrow.end <= p {
                    witness = Some(p)
                }
            });
            let witness = witness.unwrap();
            println!("Arrow {arrow}: {end_layer}=>{start_layer} by {witness:?}");
        }
        max_score = max(max_score, start_layer);
    }
    if max_score == 0 {
        None
    } else {
        Some(max_score)
    }
}

impl<C: Contour> Contours for HintContours<C> {
    // NOTE: Arrows must satisfy the following 'consistency' properties:
    // - If there is an arrow A->B of cost c>1, there is also an arrow A'->B of cost c-1, where A' is an indel away from A.
    fn new_with_filter(
        arrows: impl IntoIterator<Item = Arrow>,
        max_len: Cost,
        mut filter: impl FnMut(&Arrow, Cost) -> bool,
    ) -> Self {
        let mut this = HintContours {
            contours: {
                let mut c = SplitVec::default();
                c.resize_with(1, || C::default());
                c
            },
            max_len: max_len as _,
            stats: Default::default(),
            layers_removed: 0,
        };
        this.contours[0usize].push(Pos(I::MAX, I::MAX));
        // Loop over all arrows from a given positions.
        for (start, pos_arrows) in &arrows.into_iter().group_by(|a| a.start) {
            let mut v = 0;
            let mut l = 0;
            // TODO: The this.score() could also be implemented using a fenwick tree, as done in LCSk++.
            for a in pos_arrows {
                let nv = this.score(a.end) + a.score as Cost;
                // Filter out arrows where filter returns false.
                if !filter(&a, nv) {
                    continue;
                }
                v = max(v, nv as Layer);
                l = max(l, a.score);
            }
            if v == 0 {
                // All arrows at pos filtered out.
                continue;
            }
            if (this.contours.len() as Layer) <= v {
                this.contours
                    .resize_with(v as usize + 1, || C::with_max_len(max_len));
            }
            this.contours[v].push(start);
        }

        this
    }

    /// The max sum of arrows starting at pos
    fn score(&self, q: Pos) -> Cost {
        // score >= low is known
        // score < high is known
        let mut low = 0;
        let mut high = self.contours.len() as Layer;
        while high - low > 1 {
            let mid = (low + high) / 2;
            if let Some(v) = self.is_score_at_least(q, mid) {
                low = v;
            } else {
                high = mid;
            }
        }
        low as _
    }

    fn parent(&self, q: Pos) -> (Cost, Pos) {
        let v = self.score(q);
        let parent = self.contours[v as Layer].parent(q);
        (v, parent)
    }

    // The layer for the parent node.
    type Hint = Hint;

    fn score_with_hint(&self, q: Pos, hint: Self::Hint) -> (Cost, Self::Hint)
    where
        Self::Hint: Default,
    {
        self.stats.borrow_mut().score_with_hint_calls += 1;
        // NOTE: v - #layers_removed is equivalent to v - (contours.len() -
        // hint_v) as written in the paper.
        let v = min(
            hint.original_layer.saturating_sub(self.layers_removed),
            self.contours.len() as Layer - 1,
        );

        const SEARCH_RANGE: Layer = 5;

        // Do a linear search for 5 steps, starting at contour v.
        self.stats.borrow_mut().contains_calls += 1;
        if let Some(v) = self.is_score_at_least(q, v) {
            // Go up.
            let mut best = v;
            let upper_bound = min(v + SEARCH_RANGE + 2, self.contours.len() as Layer);
            for w in v + 1..=upper_bound {
                self.stats.borrow_mut().contains_calls += 1;
                if w < self.contours.len() as Layer && self.contours[w].contains(q) {
                    best = w;
                }
                if w == self.contours.len() as Layer || w >= best + self.max_len {
                    return (
                        best as Cost,
                        Hint {
                            original_layer: best + self.layers_removed,
                        },
                    );
                }
            }
        } else {
            // Go down.
            self.stats.borrow_mut().contains_calls += 1;
            // NOTE: this iterates in reverse.
            for w in (v.saturating_sub(SEARCH_RANGE)..=v.saturating_sub(1)).rev() {
                if self.contours[w].contains(q) {
                    return (
                        w as Cost,
                        Hint {
                            original_layer: w + self.layers_removed,
                        },
                    );
                }
            }
        }
        self.stats.borrow_mut().binary_search_fallback += 1;

        // Fall back to binary search if not found close to the hint.
        let w = self.score(q);
        let new_hint = Hint {
            original_layer: w as Layer + self.layers_removed,
        };
        // eprintln!(
        //     "Binary search fallback for {q}: guess {v}, final {w}. Hint layer: {}, Layers removed {}. New hint: {}",
        //     hint.original_layer, self.layers_removed, new_hint.original_layer);
        // assert!(false);
        (w, new_hint)
    }

    // NOTE: The set of arrows must already been pruned by the caller.
    // This will update the internal contours structure corresponding to the arrows.
    fn prune_with_hint<R: Iterator<Item = Arrow>, F: Fn(&Pos) -> Option<R>>(
        &mut self,
        p: Pos,
        hint: Self::Hint,
        arrows: F,
    ) -> (bool, Cost) {
        // Work contour by contour.
        let v = self.score_with_hint(p, hint).0 as Layer;
        // NOTE: The chain score of the point can actually be anywhere in v-max_len+1..=v.
        let v = 'v: {
            // TODO: Figure out why not v - self.max_len + 1. The layer v - self.max_len is really needed sometimes.
            for w in (v.saturating_sub(self.max_len)..=v).rev() {
                if self.contours[w].contains_equal(p) {
                    break 'v w;
                }
            }
            // point is not present anymore anyway.
            return (false, 0);
            // self.debug(p, v, &arrows);
            // panic!("Did not find point {p} in contours around {v}!");
        };
        if D {
            eprintln!("Pruning {p} in layer {v}");
            self.debug(p, v, &arrows);
        }

        assert!(v > 0);

        self.stats.borrow_mut().prunes += 1;

        // Returns the max score of any arrow starting in the giving
        // position, and the maximum length of these arrows.

        let (new_p_score, mut first_to_check) =
            if let Some(s) = chain_score(&arrows, p, v, &self.contours) {
                assert!(s <= v);
                (Some(s), s + 1)
            } else {
                (None, v + 1)
            };

        // In case the longer arrows were not relevant, the value does not change.
        if new_p_score == Some(v) {
            return (false, 0);
        }

        // Remove the point from its layer.
        if !self.contours[v].prune(p) {
            self.debug(p, v, &arrows);
            panic!("Pruning {p} from layer {v} failed!");
        }
        // Add the point to its new layer.
        if let Some(new_p_score) = new_p_score {
            self.contours[new_p_score].push(p)
        }

        // If this was the last arrow in its layer and all arrows in the next
        // max_len layers depend on the pruned match, all of them will shift.
        let initial_shift = 'shift: {
            if self.contours[v].len() > 0 {
                break 'shift 0;
            }
            //eprintln!("Removed {p} last in layer {v}");
            let mut all_depend_on_pos = true;
            let rng = v + 1..min(v + self.max_len, self.contours.len() as Layer);
            for w in rng.clone() {
                self.contours[w].iterate_points(|pos| {
                    if let Some(arrows) = arrows(&pos) {
                        for a in arrows {
                            if !(a.end <= p) {
                                all_depend_on_pos = false;
                            }
                        }
                    }
                });
                if !all_depend_on_pos {
                    break 'shift 0;
                }
            }

            // eprintln!(
            //     "\n\n\n\nThe next layer only depended on this v! Removing empty layers from here on down!"
            // );
            // self.debug(p, v, arrows);

            // Delete all the empty layers from v downward -- those corresponding to the match that is now pruned.
            let mut removed = 0;
            for w in (0..=v).rev() {
                if self.contours[w].len() > 0 {
                    break;
                }
                self.layers_removed += 1;
                self.contours.remove(w as usize);
                first_to_check = min(first_to_check, w);
                removed += 1;
            }
            break 'shift removed;
        };
        if initial_shift > 0 {
            if D {
                eprintln!("THIS WAS LAST ARROW IN LAYER {v}. SHIFT DOWN BY {initial_shift}");
            }
            self.stats.borrow_mut().shifts += initial_shift;
        }

        // Loop over the matches in the next layer, and repeatedly prune while needed.
        self.update_layers(first_to_check, v, &arrows, None::<(_, fn(_) -> _)>);
        self.check_consistency(&arrows);
        (true, initial_shift as _)
    }

    /// Update layers starting at layer `v`, continuing at least to layer `last_change`.
    fn update_layers<R: Iterator<Item = Arrow>, F: Fn(&Pos) -> Option<R>>(
        &mut self,
        mut v: u32,
        mut last_change: u32,
        arrows: &F,
        right_of: Option<(I, impl Fn(Pos) -> Pos)>,
    ) {
        self.stats.borrow_mut().prunes += 1;

        // ALG: Ensure v is at least 1. Some matches needed for consistency are
        // in layer 0, but layer 0 should never be removed, since the fake
        // 'match at the end' is needed.
        v = max(v, 1);

        if PRINT {
            eprintln!("update_layers({v}, {last_change})");
        }
        last_change = max(last_change, v);
        let chain_score = |contours: &SplitVec<C>, pos: Pos, v: Layer| -> Option<Layer> {
            chain_score(arrows, pos, v, contours)
        };
        let mut fully_shifted_layers = 0;
        let mut rolling_shift = Shift::None;

        // Compensate for +1 below.
        v -= 1;
        loop {
            v += 1;
            if v >= self.contours.len() as Layer {
                // eprintln!("Ran out of contours");
                break;
            }
            // eprintln!("Udate layer {v}");
            self.stats.borrow_mut().contours += 1;
            // Extract the current layer so we can modify it while reading the other layers.
            // We need to make a reference here to help rust understand we borrow disjoint parts of self.
            let mut current = std::mem::take(&mut self.contours[v]);
            let mut current_shift = Shift::None;
            // This is the max layer that is a 'dependency' for the arrows in the current layer.
            let mut max_new_layer = 0;
            let changes = current.prune_filter(&mut |pos| -> bool {
                // This function decides whether the point pos from contour v
                // needs to be pruned from it.  For this, we (re)compute the
                // value at pos and if it's < v, we push is to the new contour
                // of its value.
                self.stats.borrow_mut().checked += 1;
                let Some(new_layer) = chain_score(&self.contours, pos, v) else {
                    // Points that are not present anymore in `arrows` are pruned.
                    return true;
                };
                assert!(new_layer <= v, "New layer {new_layer} should never be larger than current layer v={v}! Error at {pos}");
                // NOTE: This assertion only works when pruning 1 match at a time, not when pruning multiple.
                // assert!(
                //     v.saturating_sub(self.max_len) <= new_layer,
                //     "Point {pos} drops by more than max_len={} from current={v} to new_layer={new_layer}",
                //     self.max_len
                // );

                max_new_layer = max(max_new_layer, new_layer);
                // Value v is still up to date. No need to loop over the remaining arrows starting here.
                if new_layer == v {
                    if D {
                        eprintln!("f: Push {pos} from {v} to at least {new_layer}");
                    }
                    self.stats.borrow_mut().checked_false += 1;
                    current_shift = Shift::Inconsistent;
                    return false;
                }

                current_shift.merge(Shift::Layers(v - new_layer));

                // Either no arrows left (position already pruned), or none of its arrows yields value v.
                if D {
                    eprintln!("f: Push {pos} from {v} to {new_layer} shift {current_shift:?}");
                }
                self.contours[new_layer].push(pos);

                self.stats.borrow_mut().checked_true += 1;
                self.stats.borrow_mut().num_prune_shifts += 1;
                self.stats.borrow_mut().sum_prune_shifts += v - new_layer;
                self.stats.borrow_mut().max_prune_shift = max(
                    // copy to prevent re-borrow.
                    {let x = self.stats.borrow().max_prune_shift; x},
                    v - new_layer,
                );
                true
            });
            // Put current layer back in place
            self.contours[v] = current;

            if changes {
                last_change = max(last_change, v);
            } else {
                assert!(current_shift == Shift::None || current_shift == Shift::Inconsistent);
            }

            if v >= last_change.saturating_add(self.max_len as Layer) {
                // No further changes can happen.
                self.stats.borrow_mut().no_change += 1;
                break;
            }

            if self.contours[v].len() == 0 && current_shift != Shift::Inconsistent {
                if rolling_shift == Shift::None
                    || current_shift == Shift::None
                    || rolling_shift == current_shift
                {
                    if D {
                        eprintln!("EMPTIED LAYER {v}");
                    }
                    fully_shifted_layers += 1;
                    if rolling_shift == Shift::None {
                        rolling_shift = current_shift;
                    }
                }
            } else {
                fully_shifted_layers = 0;
                rolling_shift = Shift::None;
            }
            // NOTE: Points can drop more than `max_len` layers when multiple arrows are pruned at once.
            // assert!(
            //     // 0 happens when the layer was already empty.
            //     max_new_layer == 0 || max_new_layer + self.max_len >= v,
            //     "Pruning {} now layer {} new max {} drops more than {}.\nlast_change: {}, shift_to {:?}, layer size: {}",
            //     p,
            //     v,
            //     max_new_layer,
            //     self.max_len,
            //     last_change, current_shift, self.contours[v ].len()
            // );

            if let Shift::Layers(shift) = rolling_shift && v >= last_change {
                assert!(fully_shifted_layers > 0);
                // NOTE: this used to be `>= self.max_len`, but that does not work for arrows of length >= 2:
                // There are some tests that cover this.
                if fully_shifted_layers >= self.max_len + shift - 1 {
                    if D {
                        eprintln!("REMOVE {shift} CONTOURS, since {fully_shifted_layers} >= {}+{shift}-1 have shifted by {shift}", self.max_len);
                    }
                    // Shift all other contours one down.
                    self.stats.borrow_mut().shift_layers += 1;

                    for _ in 0..shift {
                        if D {
                            eprintln!("REMOVE CONTOUR {v}");
                        }
                        assert!(self.contours[v].len() == 0);
                        self.contours.remove(v as usize);
                        self.layers_removed += 1;
                        v -= 1;
                    }
                    break;
                }
            }
            if let Some((right_of, back_transform)) = &right_of {
                let mut stop = true;
                for v in v + 1..min(v + 1 + self.max_len, self.contours.len() as _) {
                    self.contours[v].iterate_points(|p| {
                        if back_transform(p).0 >= *right_of {
                            stop = false
                        }
                    });
                    if !stop {
                        break;
                    }
                }
                if stop {
                    break;
                }
            }
        }
        self.print_stats();
    }

    #[allow(unreachable_code, unused_variables)]
    fn print_stats(&mut self) {
        if !PRINT {
            return;
        }
        // TODO: MAKE A FLAG FOR THIS.
        // eprintln!("----------------------------");
        // if self.stats.borrow().prunes > 0 {
        //     return;
        // }
        let mut num = 0;
        let mut total_len = 0;
        let mut total_dom = 0;
        for c in &self.contours {
            if c.len() > 0 {
                num += 1;
            }
            total_len += c.len();
            total_dom += c.num_dominant();
        }
        eprintln!("#contours             {}", num);
        eprintln!("avg size              {}", total_len as f32 / num as f32);
        eprintln!("avg domn              {}", total_dom as f32 / num as f32);

        let HintContourStats {
            prunes,
            checked,
            checked_true,
            checked_false,
            sum_prune_shifts,
            num_prune_shifts,
            max_prune_shift,
            contours,
            shifts,
            no_change,
            shift_layers,
            binary_search_fallback,
            contains_calls,
            score_with_hint_calls,
        }: HintContourStats = *self.stats.borrow();

        return;
        if prunes == 0 {
            return;
        }

        eprintln!("#prunes               {}", prunes);
        eprintln!("contours per prune    {}", contours as f32 / prunes as f32);
        eprintln!("#checks               {}", checked);
        eprintln!("checked per prune     {}", checked as f32 / prunes as f32);
        eprintln!(
            "checked true per p    {}",
            checked_true as f32 / prunes as f32
        );
        eprintln!(
            "checked false per p   {}",
            checked_false as f32 / prunes as f32
        );
        eprintln!(
            "shift per check true  {}",
            sum_prune_shifts as f32 / num_prune_shifts as f32
        );
        eprintln!("max shift             {}", max_prune_shift);
        eprintln!("Stop count: no change    {}", no_change);
        eprintln!("Stop count: shift layers {}", shift_layers);
        eprintln!("layers removed total     {}", self.layers_removed);
        eprintln!(
            "layers removed change    {}",
            self.layers_removed as usize - shift_layers
        );

        eprintln!("#shifts                  {}", shifts);
        eprintln!("");
        eprintln!("score_hint calls         {}", score_with_hint_calls);
        eprintln!(
            "binary search flbck/call {}",
            binary_search_fallback as f32 / score_with_hint_calls as f32
        );
        eprintln!(
            "avg contains calls       {}",
            contains_calls as f32 / score_with_hint_calls as f32
        );

        eprintln!("----------------------------");
    }
}
