pub mod bruteforce;
pub mod equal_contour;
pub mod set_contour;
pub mod traits;

pub use bruteforce::*;
pub use set_contour::*;
pub use traits::*;

use std::{cmp::max, collections::HashMap, fmt::Debug};

use itertools::Itertools;

use crate::{graph::Pos, prelude::LexPos};

/// A contour implementation that does push and query in logarithmic time, but prune in linear time.
#[derive(Default, Debug)]
pub struct LogQueryContour {
    points: Vec<LexPos>,
    // The set of dominant points, sorted lexicographically.
    // TODO: This could be a BTreeSet instead, although that would likely be slower in practice.
    dominant: Vec<LexPos>,
}

impl Contour for LogQueryContour {
    fn push(&mut self, p: Pos) {
        self.points.push(LexPos(p));
        if !self.contains(p) {
            let idx = self.dominant.binary_search(&LexPos(p)).unwrap_err();
            self.dominant.insert(idx, LexPos(p));

            {
                // The lines below remove the points shadowed by the newly inserted dominant point.
                // Equivalent to but faster than:
                //self.dominant.drain_filter(|&mut LexPos(q)| q < p);
                let mut delete_from = idx;
                while delete_from > 0 && self.dominant[delete_from - 1].0 <= p {
                    delete_from -= 1;
                }
                self.dominant.drain(delete_from..idx);
            }
            //assert!(self.dominant.is_sorted());
        }
    }

    fn contains(&self, q: Pos) -> bool {
        // TODO: When self.dominant is small, do a loop instead.
        self.dominant.binary_search(&LexPos(q)).map_or_else(
            // Check whether the first index lexicographically > q is also >= q in both coordinates.
            |idx| {
                ////println!("{} {:?} {}", idx, self.dominant.get(idx), q);
                if let Some(LexPos(p)) = self.dominant.get(idx) {
                    q <= *p
                } else {
                    false
                }
            },
            // q is in dominant
            |_| true,
        )
    }

    fn is_dominant(&self, q: Pos) -> bool {
        self.dominant.binary_search(&LexPos(q)).is_ok()
    }

    fn prune_filter<F: FnMut(Pos) -> bool>(&mut self, f: &mut F) -> bool {
        // TODO: Make this more efficient.
        if self.points.drain_filter(|&mut LexPos(s)| f(s)).count() > 0 {
            // Rebuild the dominant front.
            self.points.sort();
            self.dominant.clear();
            let mut next_j = 0;
            for LexPos(p) in self.points.iter().rev() {
                if p.1 >= next_j {
                    self.dominant.push(LexPos(*p));
                    next_j = p.1 + 1;
                }
            }
            // We push points from right to left, but the vector should be sorted left to right.
            self.dominant.reverse();
            assert!(self.dominant.is_sorted());
            true
        } else {
            false
        }
    }

    fn len(&self) -> usize {
        self.points.len()
    }

    fn num_dominant(&self) -> usize {
        self.dominant.len()
    }
}

/// A Contours implementation based on Contour layers with value queries in O(log(r)^2).
///
/// A contour x may contain points p that are actually in contour x+1, but only have value x.
/// This happens e.g. when a length 1 arrow is shadowed by a length 2 arrow.
/// It would be wrong to store p in the x+1 contour, because pruning other
/// points in x+1 could make p dominant there, which is wrong.
/// Hence, we store p in the x contour. This implies that sometimes we add
/// points to a contour that are larger than other points it already contains.
#[derive(Default, Debug)]
pub struct NaiveContours<C: Contour> {
    contours: Vec<C>,
    // TODO: Do not use vectors inside a hashmap.
    arrows: HashMap<Pos, Vec<Arrow>>,
    max_len: usize,
    prune_stats: PruneStats,
}

#[derive(Default, Debug)]
struct PruneStats {
    // Total number of prunes we do.
    prunes: usize,
    // Number of times f is evaluated.
    checked: usize,
    // Number of times f evaluates to true/false.
    checked_true: usize,
    checked_false: usize,
    // Total number of layers processed.
    contours: usize,

    // Number of times we stop pruning early.
    no_change: usize,
    shift_layers: usize,
}

impl<C: Contour> NaiveContours<C> {
    /// Get the value of the given position.
    /// It can be that a contour is completely empty, and skipped by length>1 arrows.
    /// In that case, normal binary search would give a wrong answer.
    /// Thus, we always have to check multiple contours.
    fn value_in_slice(contours: &[C], q: Pos, max_len: usize) -> usize {
        // q is always contained in layer 0.
        let mut left = 1;
        let mut right = contours.len();
        let mut size = right;
        while left < right {
            let mid = left + size / 2;
            let mut found = false;
            for c in mid..mid + max_len {
                if c >= contours.len() {
                    break;
                }
                let contains = contours[c].contains(q);
                if contains {
                    found = true;
                    break;
                }
            }
            if found {
                left = mid + 1;
            } else {
                right = mid;
            }
            size = right - left;
        }
        left - 1
    }
}

impl<C: Contour> Contours for NaiveContours<C> {
    fn new(arrows: impl IntoIterator<Item = Arrow>, max_len: usize) -> Self {
        let mut this = NaiveContours {
            contours: vec![C::default()],
            arrows: HashMap::default(),
            max_len,
            prune_stats: Default::default(),
        };
        this.contours[0].push(Pos(usize::MAX, usize::MAX));
        // Loop over all arrows from a given positions.
        for (start, pos_arrows) in &arrows.into_iter().group_by(|a| a.start) {
            let mut v = 0;
            this.arrows.insert(start, pos_arrows.collect());
            for a in &this.arrows[&start] {
                assert_eq!((a.end.0 - a.start.0) + (a.end.1 - a.start.1), 2 * max_len);
                v = max(v, this.value(a.end) + a.len);
            }
            assert!(v > 0);
            if this.contours.len() <= v {
                this.contours
                    .resize_with(v + 1, || C::with_max_len(max_len));
            }
            ////println!("Push {} to layer {}", start, v);
            this.contours[v].push(start);
        }
        this
    }

    fn value(&self, q: Pos) -> usize {
        let v = Self::value_in_slice(&self.contours, q, self.max_len);
        ////println!("Value of {} : {}", q, v);
        v
    }

    fn prune(&mut self, p: Pos) {
        if self.arrows.remove(&p).is_none() {
            // This position was already pruned or never needed pruning.
            return;
        }

        // Work contour by contour.
        // 1. Remove p from it's first contour.
        let mut v = self.value(p);
        //for (i, c) in self.contours.iter().enumerate().rev() {
        //println!("{}: {:?}", i, c);
        //}

        // Prune the current point, and also any other lazily pruned points that become dominant.
        if !self.contours[v].prune_filter(&mut |pos| !self.arrows.contains_key(&pos)) {
            //println!("SKIP");
            return;
        }
        self.prune_stats.prunes += 1;
        //println!("PRUNE {} at LAYER {}", p, v);

        // Loop over the dominant matches in the next layer, and repeatedly prune while needed.
        let mut last_change = v;
        let mut num_emptied = 0;
        let mut previous_shift = None;
        loop {
            v += 1;
            if v >= self.contours.len() {
                break;
            }
            self.prune_stats.contours += 1;
            //println!("layer {}", v);
            //println!("{}: {:?}", v, self.contours[v]);
            //println!("{}: {:?}", v - 1, self.contours[v - 1]);
            let (up_to_v, current) = {
                let (up_to_v, from_v) = self.contours.as_mut_slice().split_at_mut(v);
                (up_to_v, &mut from_v[0])
            };
            // We need to make a reference here to help rust understand we borrow disjoint parts of self.
            let mut current_shift = None;
            let mut layer_best_start_val = 0;
            if current.prune_filter(&mut |pos| -> bool {
                // This function decides whether the point pos from contour v
                // needs to be pruned from it.  For this, we (re)compute the
                // value at pos and if it's < v, we push is to the new contour
                // of its value.
                self.prune_stats.checked += 1;
                //println!("f: {}", pos);
                let pos_arrows = match self.arrows.get(&pos) {
                    Some(arrows) => arrows,
                    None => {
                        //println!("f: Prune {} no arrows left", pos);
                        current_shift = Some(usize::MAX);
                        // If no arrows left for this position, prune it without propagating.
                        self.prune_stats.checked_true += 1;
                        return true;
                    }
                };
                assert!(!pos_arrows.is_empty());
                let mut best_start_val = 0;
                for arrow in pos_arrows {
                    // Find the value at end_val via a backwards search.
                    let mut end_val = v - arrow.len;
                    while !up_to_v[end_val].contains(arrow.end) {
                        end_val -= 1;

                        // No need to continue when this value isn't going to be optimal anyway.
                        if end_val + arrow.len <= best_start_val {
                            break;
                        }

                        #[cfg(feature = "faster")]
                        {
                            // We know that max_new_val will be within [v-max_len, v].
                            // Thus, value(arrow.end) will be in [v-max_len-arrow.len, v-arrow.len].
                            // For simplicity, we skip this check.
                            if end_val + self.max_len == v - arrow.len {
                                break;
                            }
                        }
                    }

                    let start_val = end_val + arrow.len;
                    best_start_val = max(best_start_val, start_val);
                    layer_best_start_val = max(layer_best_start_val, start_val);
                }
                // Value v is still up to date. No need to loop over the remaining arrows starting here.
                if best_start_val == v {
                    //println!("f: {} keeps value {}", pos, best_start_val);
                    self.prune_stats.checked_false += 1;
                    current_shift = Some(usize::MAX);
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
                up_to_v[best_start_val].push(pos);
                if current_shift.is_none() {
                    current_shift = Some(v - best_start_val);
                } else if current_shift.unwrap() != v - best_start_val {
                    current_shift = Some(usize::MAX);
                }
                self.prune_stats.checked_true += 1;
                return true;
            }) {
                last_change = v;
            }
            //println!("{}: {:?}", v, self.contours[v]);
            //println!("{}: {:?}", v - 1, self.contours[v - 1]);

            if v >= last_change + self.max_len {
                ////println!("Last change at {}, stopping at {}", last_change, v);
                // No further changes can happen.
                self.prune_stats.no_change += 1;
                break;
            }

            //println!(
            //"emptied {:?} shift {:?} last_change {:?}",
            //emptied_shift, shift_to, last_change
            //);
            if self.contours[v].len() == 0
                && (current_shift.is_none() || current_shift.unwrap() != usize::MAX)
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
                layer_best_start_val == 0 || layer_best_start_val >= v - self.max_len,
                "Pruning {} now layer {} new max {} drops more than {}.\nlast_change: {}, shift_to {:?}, layer size: {}",
                p,
                v,
                layer_best_start_val,
                self.max_len,
                last_change, current_shift, self.contours[v].len()
            );

            if num_emptied >= self.max_len {
                //println!("Emptied {}, stopping at {}", num_emptied, v);
                // Shift all other contours one down.
                if previous_shift.is_some() {
                    self.prune_stats.shift_layers += 1;

                    for _ in 0..previous_shift.unwrap() {
                        //println!("Delete layer {} of len {}", v, self.contours[v].len());
                        assert!(self.contours[v].len() == 0);
                        self.contours.remove(v);
                        v -= 1;
                    }
                    break;
                }
            }
        }
        while let Some(c) = self.contours.last() {
            if c.len() == 0 {
                self.contours.pop();
            } else {
                break;
            }
        }
        for l in (0..8).rev() {
            if self.contours.len() > l {
                ////println!("Contour {}: {:?}", l, self.contours[l]);
            }
        }
        // for (i, c) in self.contours.iter().enumerate().rev() {
        //     //println!("{}: {:?}", i, c);
        // }
    }

    fn print_stats(&self) {
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

        let PruneStats {
            prunes,
            checked,
            checked_true,
            checked_false,
            contours,
            no_change,
            shift_layers,
        }: PruneStats = self.prune_stats;

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
        println!("Stop count: no change    {}", no_change);
        println!("Stop count: shift layers {}", shift_layers);
        // println!(
        //     "Stop layer: no change    {}",
        //     sum_no_change_layers as f32 / no_change as f32
        // );
        // println!(
        //     "Stop layer: shift layers {}",
        //     sum_shift_stop_layers as f32 / shift_layers as f32
        // );
        // println!(
        //     "Rem. layer: no change    {}",
        //     sum_no_change_layers_remaining as f32 / no_change as f32
        // );
        // println!(
        //     "Rem. layer: shift layers {}",
        //     sum_shift_stop_layers_remaining as f32 / shift_layers as f32
        // );
        println!("----------------------------");
    }
}
