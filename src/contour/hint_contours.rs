use std::cell::RefCell;

use itertools::Itertools;

use crate::prelude::*;

/// A Contours implementation based on Contour layers with queries in O(log(r)^2).
#[derive(Default, Debug)]
pub struct HintContours<C: Contour> {
    contours: SplitVec<C>,
    // TODO: This should have units in the transformed domain instead.
    max_len: I,
    stats: RefCell<HintContourStats>,

    layers_removed: Cost,
    start: Pos,
    target: Pos,
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
    sum_prune_shifts: Cost,
    num_prune_shifts: usize,
    max_prune_shift: Cost,

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
    original_layer: Cost,
}

impl Default for Hint {
    fn default() -> Self {
        Self {
            original_layer: Cost::MAX,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Shift {
    None,
    Layers(Cost),
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
    fn debug(&self, pos: Pos, v: Cost, arrows: &HashMap<Pos, Vec<Arrow>>) {
        println!("BEFORE PRUNE of {pos} layer {v}");
        let radius = 4;
        for i in max(v.saturating_sub(radius), 1)..min(v + radius, self.contours.len() as Cost) {
            println!("{}: {:?}", i, self.contours[i]);
            self.contours[i].iterate_points(|p: Pos| {
                let l = arrows.get(&p).map_or(0, |arrows| {
                    arrows.iter().map(|a| a.len).max().expect("Empty arrows")
                });
                println!("{i} {p}: {l}");
                arrows.get(&p).map(|arrows| {
                    for a in arrows {
                        println!("{a}");
                    }
                });
                assert!(
                    l > 0 || p == pos,
                    "No arrows found for position {p} at layer {i}"
                );
            })
        }
    }

    /// Returns None when false, or the first layer >= v that contains the query point.
    fn is_score_at_least(&self, q: Pos, v: Cost) -> Option<Cost> {
        // Test if score >= mid by checking all points in contours mid..mid+r
        for w in v..min(v + self.max_len, self.contours.len() as Cost) {
            if self.contours[w as usize].contains(q) {
                return Some(w);
            }
        }
        None
    }
}

impl<C: Contour> Contours for HintContours<C> {
    // NOTE: Arrows must satisfy the following 'consistency' properties:
    // - If there is an arrow A->B of cost c>1, there is also an arrow A'->B of cost c-1, where A' is an indel away from A.
    fn new(arrows: impl IntoIterator<Item = Arrow>, max_len: I) -> Self {
        let mut this = HintContours {
            contours: {
                let mut c = SplitVec::default();
                c.resize_with(1, || C::default());
                c
            },
            max_len,
            stats: Default::default(),
            layers_removed: 0,
            start: Pos(I::MAX, I::MAX),
            target: Pos(0, 0),
        };
        this.contours[0usize].push(Pos(I::MAX, I::MAX));
        // Loop over all arrows from a given positions.
        for (start, pos_arrows) in &arrows.into_iter().group_by(|a| a.start) {
            let mut v = 0;
            let mut l = 0;
            // TODO: The this.score() could also be implemented using a fenwick tree, as done in LCSk++.
            for a in pos_arrows {
                this.start.0 = min(this.start.0, a.end.0);
                this.start.1 = min(this.start.1, a.end.1);
                this.target.0 = max(this.target.0, a.end.0);
                this.target.1 = max(this.target.1, a.end.1);
                let nv = this.score(a.end) + a.len as Cost;
                if nv > v || (nv == v && a.len < l) {
                    v = nv;
                }
                l = max(l, a.len);
            }
            assert!(v > 0);
            if this.contours.len() as Cost <= v {
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
        let mut high = self.contours.len() as Cost;
        while high - low > 1 {
            let mid = (low + high) / 2;
            if let Some(v) = self.is_score_at_least(q, mid) {
                low = v;
            } else {
                high = mid;
            }
        }
        low
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
            self.contours.len() as Cost - 1,
        );

        const SEARCH_RANGE: Cost = 5;

        // Do a linear search for 5 steps, starting at contour v.
        self.stats.borrow_mut().contains_calls += 1;
        if let Some(v) = self.is_score_at_least(q, v) {
            // Go up.
            let mut best = v;
            let upper_bound = min(v + SEARCH_RANGE + 2, self.contours.len() as Cost);
            for w in v + 1..=upper_bound {
                self.stats.borrow_mut().contains_calls += 1;
                if w < self.contours.len() as Cost && self.contours[w].contains(q) {
                    best = w;
                }
                if w == self.contours.len() as Cost || w >= best + self.max_len {
                    return (
                        best,
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
            for w in (v.saturating_sub(SEARCH_RANGE)..=v - 1).rev() {
                if self.contours[w].contains(q) {
                    return (
                        w,
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
        (
            w,
            Hint {
                original_layer: w + self.layers_removed,
            },
        )
    }

    // NOTE: The set of arrows must already been pruned by the caller.
    // This will update the internal contours structure corresponding to the arrows.
    fn prune_with_hint(
        &mut self,
        p: Pos,
        hint: Hint,
        arrows: &HashMap<Pos, Vec<Arrow>>,
    ) -> (bool, Cost) {
        const D: bool = false;
        // Work contour by contour.
        let v = self.score_with_hint(p, hint).0;
        // NOTE: The chain score of the point can actually be anywhere in v-max_len+1..=v.
        let mut v = 'v: {
            // TODO: Figure out why not v - self.max_len + 1. The layer v - self.max_len is really needed sometimes.
            for w in (v.saturating_sub(self.max_len)..=v).rev() {
                if self.contours[w].contains_equal(p) {
                    break 'v w;
                }
            }
            self.debug(p, v, arrows);
            panic!("Did not find point {p} in contours around {v}!");
        };
        if D {
            println!("Pruning {p} in layer {v}");
            self.debug(p, v, arrows);
        }

        assert!(v > 0);

        self.stats.borrow_mut().prunes += 1;

        // Returns the max score of any arrow starting in the giving
        // position, and the maximum length of these arrows.
        let chain_score = |contours: &SplitVec<C>, pos: Pos, v: Cost| -> Cost {
            let Some(pos_arrows) = arrows.get(&pos) else {
                panic!("No arrows found for position {pos} around layer {v}.");
            };
            assert!(!pos_arrows.is_empty());
            let mut max_score = 0;
            for arrow in pos_arrows {
                // Find the value at end_val via a linear search.
                let mut end_layer = v as Cost - 1;
                assert!(
                    !contours[v].contains(arrow.end),
                    "Hint of {v} is no good! Contains {} for arrow {arrow}",
                    arrow.end
                );
                while !contours[end_layer].contains(arrow.end) {
                    end_layer -= 1;

                    // No need to continue when this value isn't going to be optimal anyway.
                    if end_layer + arrow.len as Cost <= max_score {
                        break;
                    }

                    if FAST_ASSUMPTIONS {
                        // We know that max_new_val will be within [v-max_len, v].
                        // Thus, value(arrow.end) will be in [v-max_len-arrow.len, v-arrow.len].
                        // For simplicity, we skip this check.
                        if end_layer + self.max_len == v - arrow.len as Cost {
                            break;
                        }
                    }
                }

                let start_layer = end_layer + arrow.len as Cost;
                max_score = max(max_score, start_layer);
            }
            max_score
        };

        let (new_p_score, mut first_to_check) = if arrows.contains_key(&p) {
            let s = chain_score(&self.contours, p, v);
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
            self.debug(p, v, arrows);
            panic!("Pruning {p} from layer {v} failed!");
        }
        // Add the point to its new layer.
        if let Some(new_p_score) = new_p_score {
            self.contours[new_p_score].push(p)
        }

        // If this was the last arrow in its layer and all arrows in the next
        // max_len layers depend on the pruned match, all of them will shift.
        let shift = 'shift: {
            if self.contours[v].len() > 0 {
                break 'shift 0;
            }
            //println!("Removed {p} last in layer {v}");
            let mut all_depend_on_pos = true;
            let rng = v + 1..min(v + self.max_len, self.contours.len() as Cost);
            for w in rng.clone() {
                self.contours[w].iterate_points(|pos| {
                    for a in &arrows[pos] {
                        if !(a.end <= p) {
                            all_depend_on_pos = false;
                        }
                    }
                });
                if !all_depend_on_pos {
                    break 'shift 0;
                }
            }

            // println!(
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
        if shift > 0 {
            self.stats.borrow_mut().shifts += 1;
        }

        // Loop over the matches in the next layer, and repeatedly prune while needed.
        let mut last_change = v;
        v = first_to_check - 1;
        let mut num_emptied = 0;
        let mut previous_shift = Shift::None;
        loop {
            v += 1;
            if v >= self.contours.len() as Cost {
                break;
            }
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
                let new_layer = chain_score(&self.contours, pos, v);
                assert!(new_layer <= v);
                assert!(
                    v.saturating_sub(self.max_len) <= new_layer,
                    "Point {pos} drops by more than max_len={} from current={v} to new_layer={new_layer}",
                    self.max_len
                );

                max_new_layer = max(max_new_layer, new_layer);
                // Value v is still up to date. No need to loop over the remaining arrows starting here.
                if new_layer == v {
                    if D{
                        println!("f: {pos} from {v} to at least {new_layer}");
                    }
                    self.stats.borrow_mut().checked_false += 1;
                    current_shift = Shift::Inconsistent;
                    return false;
                }

                current_shift.merge(Shift::Layers(v - new_layer));

                // Either no arrows left (position already pruned), or none of its arrows yields value v.
                if D{
                    println!("f: Push {} to {} shift {:?}", pos, new_layer, current_shift);
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
            if changes {
                last_change = v;
            }

            // Put current layer back in place
            self.contours[v] = current;

            if v >= last_change + self.max_len as Cost {
                // No further changes can happen.
                self.stats.borrow_mut().no_change += 1;
                break;
            }

            if self.contours[v].len() == 0 && current_shift != Shift::Inconsistent {
                if previous_shift == Shift::None
                    || current_shift == Shift::None
                    || previous_shift == current_shift
                {
                    num_emptied += 1;
                    if previous_shift == Shift::None {
                        previous_shift = current_shift;
                    }
                }
            } else {
                num_emptied = 0;
                previous_shift = Shift::None;
            }
            assert!(
                // 0 happens when the layer was already empty.
                max_new_layer == 0 || max_new_layer + self.max_len >= v,
                "Pruning {} now layer {} new max {} drops more than {}.\nlast_change: {}, shift_to {:?}, layer size: {}",
                p,
                v,
                max_new_layer,
                self.max_len,
                last_change, current_shift, self.contours[v ].len()
            );

            if num_emptied >= self.max_len {
                // Shift all other contours one down.
                if let Shift::Layers(previous_shift) = previous_shift {
                    self.stats.borrow_mut().shift_layers += 1;

                    for _ in 0..previous_shift {
                        assert!(self.contours[v].len() == 0);
                        self.contours.remove(v as usize);
                        self.layers_removed += 1;
                        v -= 1;
                    }
                    break;
                }
            }
        }
        (true, shift)
    }

    #[allow(unreachable_code)]
    fn print_stats(&self) {
        // TODO: MAKE A FLAG FOR THIS.
        return;
        println!("----------------------------");
        let num = self.contours.len();
        let mut total_len = 0;
        let mut total_dom = 0;
        for c in &self.contours {
            total_len += c.len();
            total_dom += c.num_dominant();
        }
        println!("#contours             {}", num);
        println!("avg size              {}", total_len as f32 / num as f32);
        println!("avg domn              {}", total_dom as f32 / num as f32);

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

        if prunes == 0 {
            return;
        }

        println!("#prunes               {}", prunes);
        println!("contours per prune    {}", contours as f32 / prunes as f32);
        println!("#checks               {}", checked);
        println!("checked per prune     {}", checked as f32 / prunes as f32);
        println!(
            "checked true per p    {}",
            checked_true as f32 / prunes as f32
        );
        println!(
            "checked false per p   {}",
            checked_false as f32 / prunes as f32
        );
        println!(
            "shift per check true  {}",
            sum_prune_shifts as f32 / num_prune_shifts as f32
        );
        println!("max shift             {}", max_prune_shift);
        println!("Stop count: no change    {}", no_change);
        println!("Stop count: shift layers {}", shift_layers);
        println!("layers removed total     {}", self.layers_removed);
        println!(
            "layers removed change    {}",
            self.layers_removed as usize - shift_layers
        );

        println!("#shifts                  {}", shifts);
        println!("");
        println!("score_hint calls         {}", score_with_hint_calls);
        println!(
            "%binary search fallback  {}",
            binary_search_fallback as f32 / score_with_hint_calls as f32
        );
        println!(
            "avg contains calls       {}",
            contains_calls as f32 / score_with_hint_calls as f32
        );

        println!("----------------------------");
    }
}
