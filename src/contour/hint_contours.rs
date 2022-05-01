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
        let radius = 6;
        for i in max(v.saturating_sub(radius), 1)..min(v + radius, self.contours.len() as Cost) {
            println!("{}: {:?}", i, self.contours[i]);
            self.contours[i].iterate_points(|p: Pos| {
                let l = arrows.get(&p).map_or(0, |arrows| {
                    arrows.iter().map(|a| a.len).max().expect("Empty arrows")
                });
                println!("{i} {p}: {l}");
                assert!(
                    l > 0 || p == pos,
                    "No arrows found for position {p} at layer {i}"
                );
            })
        }
        self.display_box(Pos(pos.0 - 100, pos.1 - 100), Pos(pos.0 + 100, pos.1 + 100));
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
        };
        this.contours[0usize].push(Pos(I::MAX, I::MAX));
        // Loop over all arrows from a given positions.
        for (start, pos_arrows) in &arrows.into_iter().group_by(|a| a.start) {
            let mut v = 0;
            let mut l = 0;
            // TODO: The this.value() could also be implemented using a fenwick tree, as done in LCSk++.
            for a in pos_arrows {
                let nv = this.value(a.end) + a.len as Cost;
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
            for w in v + 1 - l as Cost..=v {
                this.contours[w].push(start);
            }
        }

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
            if self.contours[v].contains(q) {
                // Go up.
                for v in v + 1..min(v + 1 + SEARCH_RANGE, self.contours.len() as Cost) {
                    self.stats.borrow_mut().contains_calls += 1;
                    if !self.contours[v].contains(q) {
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
                    if self.contours[v].contains(q) {
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
        const D: bool = false;
        // Work contour by contour.
        // 1. Remove p from it's first contour.
        let mut v = self.value_with_hint(p, hint).0;
        if D {
            self.debug(p, v, arrows);
        }


        assert!(v > 0);
        // Remove the point from all layers where it is present.
        {
            let mut pruned = false;
            for w in v.saturating_sub(self.max_len)..=v {
                pruned |= self.contours[w].prune(p);
            }
            if !pruned {
                self.debug(p, v, arrows);
                assert!(
                    pruned,
                    "Did not prune {p} from any of the layers {} to {v}",
                    v.saturating_sub(self.max_len)
                );
            }
        }

        self.stats.borrow_mut().prunes += 1;

        // If max_len consecutive layers are empty, shift everything down by this distance.
        {
            let mut all_empty = true;
            for w in (v + 1 - self.max_len) as usize..=v as usize {
                all_empty &= self.contours[w].len() == 0;
            }
            if all_empty {
                // Delete these max_len layers.
                for _ in 0..self.max_len {
                    if D {
                        println!("Delete layer {} of len {}", v, self.contours[v].len());
                    }
                    assert!(self.contours[v].len() == 0);
                    self.contours.remove(v as usize);
                    self.layers_removed += 1;
                    v -= 1;
                }
                return (true, self.max_len);
            }
        }

        // Loop over the matches in the next layer, and repeatedly prune while needed.
        let mut last_change = v;
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
                let pos_arrows = arrows.get(&pos).expect("No arrows found for position.");
                assert!(!pos_arrows.is_empty());
                let mut new_layer = 0;
                let mut l = 0;
                for arrow in pos_arrows {
                    // Find the value at end_val via a backwards search.
                    let mut end_layer = v - arrow.len as Cost;
                    while !self.contours[end_layer].contains(arrow.end) {
                        end_layer -= 1;

                        // No need to continue when this value isn't going to be optimal anyway.
                        if end_layer + arrow.len as Cost <= new_layer {
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
                    if start_layer > new_layer || (start_layer == new_layer && arrow.len < l) {
                        new_layer = start_layer;
                        l = arrow.len;
                    }
                }
                assert!(
                    v - self.max_len <= new_layer,
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

                    // Make sure this point is contained in its parent, and add shadow points if not.
                    // NOTE: This adds around 1% of total runtime for HintContours<CentralContour>.
                    let mut v = new_layer;
                    while v > 0 && !self.contours[v as usize - 1].contains(pos) {
                        v -= 1;
                        self.contours[v].push(pos);
                    }
                    for w in new_layer + 1 - l as Cost..new_layer {
                        if !self.contours[w].contains_equal(pos) {
                            self.contours[w].push(pos);
                        }
                    }

                    return false;
                }

                // Either no arrows left (position already pruned), or none of its arrows yields value v.
                if D{
                println!("f: Push {} to {} shift {:?}", pos, new_layer, current_shift);
                }
                for w in new_layer + 1 - l as Cost..=new_layer {
                    if !self.contours[w].contains_equal(pos) {
                        self.contours[w].push(pos);
                    }
                }

                current_shift.merge(Shift::Layers(v - new_layer));
                self.stats.borrow_mut().checked_true += 1;
                self.stats.borrow_mut().num_prune_shifts += 1;
                self.stats.borrow_mut().sum_prune_shifts += v - new_layer;
                self.stats.borrow_mut().max_prune_shift = max(
                    {
                        let x = self.stats.borrow().max_prune_shift;
                        x
                    },
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
}
