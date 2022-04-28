use std::{cell::RefCell, cmp::Ordering};

use itertools::Itertools;

use crate::prelude::*;

/// A Contours implementation based on Contour layers with value queries in O(log(r)^2).
///
/// A contour x may contain points p that are actually in contour x+1, but only have value x.
/// This happens e.g. when a length 1 arrow is shadowed by a length 2 arrow.
/// It would be wrong to store p in the x+1 contour, because pruning other
/// points in x+1 could make p dominant there, which is wrong.
/// Hence, we store p in the x contour. This implies that sometimes we add
/// points to a contour that are larger than other points it already contains.
#[derive(Default, Debug)]
pub struct HintContours<C: Contour> {
    contours: SplitVec<C>,
    // TODO: This should have units in the transformed domain instead.
    max_len: I,
    stats: RefCell<HintContourStats>,

    layers_removed: Cost,

    // ops
    ops: usize,
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

    // Number of times we stop pruning early.
    no_change: usize,
    shift_layers: usize,

    // Binary search stats
    value_with_hint_calls: usize,
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

impl<C: Contour> Contours for HintContours<C> {
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
            ops: 0,
        };
        this.contours[0].push(Pos(I::MAX, I::MAX));
        // Loop over all arrows from a given positions.
        for (start, pos_arrows) in &arrows.into_iter().group_by(|a| a.start) {
            //println!("ARROWS: {start:?}");
            let mut v = 0;
            // TODO: The this.value() could also be implemented using a fenwick tree, as done in LCSk++.
            for a in pos_arrows {
                // TODO: This is only true for gap-cost, not in more general contour settings.
                //assert_eq!((a.end.0 - a.start.0) + (a.end.1 - a.start.1), 2 * max_len);
                v = max(v, this.value(a.end) + a.len as Cost);
                //println!("ARROWS to: {a:?}: {v}");
            }
            assert!(v > 0);
            if this.contours.len() as Cost <= v {
                this.contours
                    .resize_with(v as usize + 1, || C::with_max_len(max_len));
            }
            //println!("Push {} to layer {}", start, v);
            this.contours[v as usize].push(start);
            while v > 0 && !this.contours[v as usize - 1].contains(start) {
                v -= 1;
                this.contours[v as usize].push(start);
            }
        }
        // for k in 0..this.contours.len() {
        //     //this.contours[k].print_points();
        //     //println!("{}: {:?}", k, this.contours[k]);
        // }

        this
    }

    /// The max sum of arrows starting at pos
    fn value(&self, q: Pos) -> Cost {
        self.contours
            .binary_search_by(|contour| {
                if contour.contains(q) {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .unwrap_err() as Cost
            - 1
    }

    // The layer for the parent node.
    type Hint = Hint;

    fn value_with_hint(&self, q: Pos, hint: Self::Hint) -> (Cost, Self::Hint)
    where
        Self::Hint: Default,
    {
        self.stats.borrow_mut().value_with_hint_calls += 1;
        // return (self.value(q), Hint::default());
        let v = hint.original_layer.saturating_sub(self.layers_removed);

        const SEARCH_RANGE: Cost = 5;

        // Do a linear search for 5 steps, starting at contour v.
        if v < self.contours.len() as Cost {
            self.stats.borrow_mut().contains_calls += 1;
            if self.contours[v as usize].contains(q) {
                // Go up.
                for v in v + 1..min(v + 1 + SEARCH_RANGE, self.contours.len() as Cost) {
                    self.stats.borrow_mut().contains_calls += 1;
                    if !self.contours[v as usize].contains(q) {
                        return (
                            v - 1,
                            Hint {
                                original_layer: v - 1 + self.layers_removed,
                            },
                        );
                    }
                }
            } else {
                // Go down.
                self.stats.borrow_mut().contains_calls += 1;
                for v in (v.saturating_sub(SEARCH_RANGE)..v).rev() {
                    if self.contours[v as usize].contains(q) {
                        return (
                            v,
                            Hint {
                                original_layer: v + self.layers_removed,
                            },
                        );
                    }
                }
            }
        }
        self.stats.borrow_mut().binary_search_fallback += 1;

        // Fall back to binary search if not found close to the hint.
        let v = self.value(q);
        (
            v,
            Hint {
                original_layer: v + self.layers_removed,
            },
        )
    }

    fn prune_with_hint(
        &mut self,
        p: Pos,
        hint: Hint,
        arrows: &HashMap<Pos, Vec<Arrow>>,
    ) -> (bool, Cost) {
        // Work contour by contour.
        // 1. Remove p from it's first contour.
        let mut v = self.value_with_hint(p, hint).0;
        // println!("BEFORE PRUNE of {p} layer {v}");
        // for i in v.saturating_sub(4)..min(v + 4, self.contours.len() as Cost) {
        //     println!("{}: {:?}", i, self.contours[i as usize]);
        // }

        // Prune the current point, and also any other lazily pruned points that become dominant.
        // If nothing changed, return false.
        if v == 0 || !self.contours[v as usize].prune_filter(&mut |pos| !arrows.contains_key(&pos))
        {
            //println!("SKIP");
            return (false, 0);
        }

        {
            // Also remove the point from other contours where it is dominant.
            let mut shadow_v = v - 1;
            while self.contours[shadow_v as usize].is_dominant(p) {
                self.contours[shadow_v as usize].prune(p);
                shadow_v -= 1;
            }
        }

        self.stats.borrow_mut().prunes += 1;
        //println!("PRUNE {} at LAYER {}", p, v);

        // If this layer becomes empty, and the max_len-1 layers below it are
        // also empty, shift everything down by max_dist, and report this back
        // for efficient updating of heuristic values in the queue.
        if self.contours[v as usize].len() == 0 {
            let mut all_empty = true;
            for w in (v + 1 - self.max_len) as usize..v as usize {
                all_empty &= self.contours[w].len() == 0;
            }
            if all_empty {
                // Delete these max_len layers.
                for _ in 0..self.max_len {
                    //println!("Delete layer {} of len {}", v, self.contours[v].len());
                    assert!(self.contours[v as usize].len() == 0);
                    self.ops += self.contours[v as usize].ops();
                    self.contours.remove(v as usize);
                    self.layers_removed += 1;
                    v -= 1;
                }
                return (true, self.max_len);
            }
        }

        // Loop over the dominant matches in the next layer, and repeatedly prune while needed.
        let mut last_change = v;
        let mut num_emptied = 0;
        let mut previous_shift = None;
        loop {
            v += 1;
            if v >= self.contours.len() as Cost {
                break;
            }
            self.stats.borrow_mut().contours += 1;
            //println!("layer {}", v);
            //println!("{}: {:?}", v, self.contours[v]);
            //println!("{}: {:?}", v - 1, self.contours[v - 1]);
            // Extract the current layer so we can modify it while reading the other layers.
            let mut current = std::mem::take(&mut self.contours[v as usize]);
            // We need to make a reference here to help rust understand we borrow disjoint parts of self.
            let mut current_shift = None;
            let mut layer_best_start_val = 0;
            let changes = current.prune_filter(&mut |pos| -> bool {
                // This function decides whether the point pos from contour v
                // needs to be pruned from it.  For this, we (re)compute the
                // value at pos and if it's < v, we push is to the new contour
                // of its value.
                self.stats.borrow_mut().checked += 1;
                //println!("f: {}", pos);
                let pos_arrows = match arrows.get(&pos) {
                    None => {
                        //println!("f: Prune {} no arrows left", pos);
                        current_shift = Some(Cost::MAX);
                        // If no arrows left for this position, prune it without propagating.
                        self.stats.borrow_mut().checked_true += 1;
                        return true;
                    }
                    Some(arrows) => arrows,
                };
                assert!(!pos_arrows.is_empty());
                let mut best_start_val = 0;
                for arrow in pos_arrows {
                    // Find the value at end_val via a backwards search.
                    let mut end_val = v - arrow.len as Cost;
                    while !self.contours[end_val as usize].contains(arrow.end) {
                        end_val -= 1;

                        // No need to continue when this value isn't going to be optimal anyway.
                        if end_val + arrow.len as Cost <= best_start_val {
                            break;
                        }

                        if FAST_ASSUMPTIONS {
                            // We know that max_new_val will be within [v-max_len, v].
                            // Thus, value(arrow.end) will be in [v-max_len-arrow.len, v-arrow.len].
                            // For simplicity, we skip this check.
                            if end_val + self.max_len == v - arrow.len as Cost {
                                break;
                            }
                        }
                    }

                    let start_val = end_val + arrow.len as Cost;
                    best_start_val = max(best_start_val, start_val);
                    layer_best_start_val = max(layer_best_start_val, start_val);
                }
                // Value v is still up to date. No need to loop over the remaining arrows starting here.
                if best_start_val >= v {
                    assert!(best_start_val == v);
                    //println!("f: {} keeps value {}", pos, best_start_val);
                    self.stats.borrow_mut().checked_false += 1;
                    current_shift = Some(Cost::MAX);

                    // Make sure this point is contained in its parent, and add shadow points if not.
                    // NOTE: This adds around 1% of total runtime for HintContours<CentralContour>.
                    let mut v = best_start_val;
                    while v > 0 && !self.contours[v as usize - 1].contains(pos) {
                        v -= 1;
                        self.contours[v as usize].push(pos);
                    }

                    return false;
                }

                //println!("f: {} new value {}", pos, max_new_val);
                // NOTE: This assertion does not always hold. In particular,
                // when the Contour implementation is lazy about pruning
                // non-dominant points, it may happen that e.g. a value 8 contour contains points with value 7.
                // After removing a match of length max_len=2, this would drop to 5, which is less than 8 - 2.
                // assert!(v - max_len <= max_new_val && max_new_val <= v,);

                // Either no arrows left (position already pruned), or none of its arrows yields value v.
                // println!(
                //     "f: Push {} to {} shift {:?}",
                //     pos, best_start_val, current_shift
                // );
                {
                    let mut v = best_start_val;
                    if !self.contours[v as usize].contains_equal(pos) {
                        self.contours[v as usize].push(pos);
                    }
                    v -= 1;
                    // Layer 0 is guaranteed to contain everything.
                    while !self.contours[v as usize].contains(pos) {
                        self.contours[v as usize].push(pos);
                        v -= 1;
                    }
                }

                if current_shift.is_none() {
                    current_shift = Some(v - best_start_val);
                } else if current_shift.unwrap() != v - best_start_val {
                    current_shift = Some(Cost::MAX);
                }
                self.stats.borrow_mut().checked_true += 1;
                self.stats.borrow_mut().num_prune_shifts += 1;
                self.stats.borrow_mut().sum_prune_shifts += v - best_start_val;
                self.stats.borrow_mut().max_prune_shift = max(
                    {
                        let x = self.stats.borrow().max_prune_shift;
                        x
                    },
                    v - best_start_val,
                );
                true
            });
            if changes {
                last_change = v;
            }

            // Put current layer back in place
            self.contours[v as usize] = current;

            //println!("{}: {:?}", v, self.contours[v]);
            //println!("{}: {:?}", v - 1, self.contours[v - 1]);
            // println!("after");

            if v >= last_change + self.max_len as Cost {
                ////println!("Last change at {}, stopping at {}", last_change, v);
                // No further changes can happen.
                self.stats.borrow_mut().no_change += 1;
                break;
            }

            //println!(
            //"emptied {:?} shift {:?} last_change {:?}",
            //emptied_shift, shift_to, last_change
            //);
            if self.contours[v as usize].len() == 0
                && (current_shift.is_none() || current_shift.unwrap() != Cost::MAX)
            {
                if previous_shift.is_none()
                    || current_shift.is_none()
                    || previous_shift == current_shift
                {
                    num_emptied += 1;
                    if previous_shift.is_none() {
                        previous_shift = current_shift;
                    }
                }
                //println!("Num emptied to {} shift {:?}", num_emptied, emptied_shift);
            } else {
                num_emptied = 0;
                previous_shift = None;
                //println!("Num emptied reset");
            }
            assert!(
                // 0 happens when the layer was already empty.
                layer_best_start_val == 0 || layer_best_start_val + self.max_len >= v,
                "Pruning {} now layer {} new max {} drops more than {}.\nlast_change: {}, shift_to {:?}, layer size: {}",
                p,
                v,
                layer_best_start_val,
                self.max_len,
                last_change, current_shift, self.contours[v as usize].len()
            );

            if num_emptied >= self.max_len {
                //println!("Emptied {}, stopping at {}", num_emptied, v);
                // Shift all other contours one down.
                if let Some(previous_shift) = previous_shift {
                    self.stats.borrow_mut().shift_layers += 1;

                    for _ in 0..previous_shift {
                        //println!("Delete layer {} of len {}", v, self.contours[v].len());
                        assert!(self.contours[v as usize].len() == 0);
                        self.contours.remove(v as usize);
                        self.layers_removed += 1;
                        v -= 1;
                    }
                    break;
                }
            }
        }
        // while let Some(c) = self.contours.last() {
        //     if c.len() == 0 {
        //         self.contours.pop();
        //     } else {
        //         break;
        //     }
        // }
        for k in (0..8).rev() {
            if v <= 4 {
                continue;
            }
            let k = v as usize - 4 + k;
            if self.contours.len() > k {
                //println!("Contour {}: {:?}", k, self.contours[k]);
            }
        }
        // for (i, c) in self.contours.iter().enumerate().rev() {
        //     //println!("{}: {:?}", i, c);
        // }
        (true, 0)
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
            contours,
            no_change,
            shift_layers,
            value_with_hint_calls,
            binary_search_fallback,
            contains_calls,
            max_prune_shift,
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
        println!("value_hint calls         {}", value_with_hint_calls);
        println!(
            "%binary search fallback  {}",
            binary_search_fallback as f32 / value_with_hint_calls as f32
        );
        println!(
            "avg contains calls       {}",
            contains_calls as f32 / value_with_hint_calls as f32
        );

        println!("----------------------------");
    }

    fn ops(&self) -> usize {
        //self.stats.borrow().value_with_hint_calls
        self.ops + self.contours.into_iter().map(|c| c.ops()).sum::<usize>()
    }
}
