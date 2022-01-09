use std::{
    cmp::max,
    collections::HashMap,
    fmt::{Debug, Display},
    mem,
};

use itertools::Itertools;

use crate::{graph::Pos, prelude::LexPos};

/// A datastructure that contains the contours of non-dominant points.
/// The 'main' contour is the set of dominant points: {P: P >= S for all S}.
/// It returns whether a query point Q is inside the contour: {is there an S s.t. Q <= S}.
/// This is an online/dynamic datastructure that allows addition and removal of points:
/// - Addition of P. Usually on the top left: {not (S < P) for all S}, but not always (see NaiveContours).
/// - Removal of P.
// TODO: An implementation that does lookup in O(lg(n))
// TODO: An implementation that does lookup, and push (and pop) in O(lg(n))
// TODO: An implementation that does lookup in O(1), using a hint.
pub trait Contour: Default + Debug {
    /// Add a new point to the graph.
    /// This point must be 'smaller' (actually: not larger) than every existing point.
    fn push(&mut self, _p: Pos);
    /// Is point `q` above/top-left of the contour.
    fn contains(&self, _q: Pos) -> bool;
    /// Is this point dominant?
    fn is_dominant(&self, _q: Pos) -> bool;
    /// Remove the point at the given position, and shift all contours.
    /// Returns whether p was dominant.
    fn prune(&mut self, p: Pos) -> bool {
        self.prune_filter(|s| s == p)
    }
    /// Prune all points for which f returns true.
    /// This should keep feeding points until all dominant points should remain.
    fn prune_filter<F: FnMut(Pos) -> bool>(&mut self, f: F) -> bool;

    fn len(&self) -> usize;
    fn dominant(&self) -> usize;
}

/// An arrow implies f(start) >= f(end) + len.
/// This is only needed for Contours, since Contour already assumes the
pub struct Arrow {
    pub start: Pos,
    pub end: Pos,
    pub len: usize,
}

impl std::fmt::Display for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{:?} => {:?} [{}]",
            self.start, self.end, self.len
        ))
    }
}

impl Debug for Arrow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Arrow as Display>::fmt(self, f)
    }
}

/// A datastructure that contains multiple contours.
/// Supports incremental building from matches, querying, and pruning.
/// The structure is built by pushing matches in decreasing order.
pub trait Contours: Default + Debug {
    /// Build the contours from a set of arrows.
    /// NOTE: Arrows must be reverse sorted by start.
    fn new(_arrows: impl IntoIterator<Item = Arrow>) -> Self;
    /// The value of the contour this point is on.
    /// Hint is guaranteed to be for the current position.
    fn value(&self, _q: Pos) -> usize;
    /// Remove the point at the given position, and shift all contours.
    /// TODO: also prune all arrows ending in the given position.
    ///       or at least when this is the only outgoing arrow.
    fn prune(&mut self, _p: Pos);
}

/// A contour implementation that does all operations in O(r).
#[derive(Default, Debug)]
pub struct NaiveContour {
    points: Vec<Pos>,
}

impl Contour for NaiveContour {
    fn push(&mut self, p: Pos) {
        self.points.push(p);
    }

    fn contains(&self, q: Pos) -> bool {
        for &s in &self.points {
            if q <= s {
                return true;
            }
        }
        return false;
    }

    fn is_dominant(&self, q: Pos) -> bool {
        for &s in &self.points {
            if !(q >= s) {
                return false;
            }
        }
        return true;
    }

    fn prune_filter<F: FnMut(Pos) -> bool>(&mut self, mut f: F) -> bool {
        self.points.drain_filter(|&mut s| f(s)).count() > 0
    }

    fn len(&self) -> usize {
        self.points.len()
    }

    fn dominant(&self) -> usize {
        self.points.iter().filter(|p| self.is_dominant(**p)).count()
    }
}

/// A contour implementation that keeps the optimal front, which is rebuilt on every prune.
#[derive(Default, Debug)]
pub struct LogContour {
    points: Vec<LexPos>,
    // The set of dominant points, sorted lexicographically.
    dominant: Vec<LexPos>,
}

impl Contour for LogContour {
    fn push(&mut self, p: Pos) {
        self.points.push(LexPos(p));
        if !self.contains(p) {
            let idx = self.dominant.binary_search(&LexPos(p)).unwrap_err();
            self.dominant.insert(idx, LexPos(p));
            let mut delete_from = idx;
            while delete_from > 0 && self.dominant[delete_from - 1].0 <= p {
                delete_from -= 1;
            }
            self.dominant.drain(delete_from..idx);
            //self.dominant.drain_filter(|&mut LexPos(q)| q < p);
            //assert!(self.dominant.is_sorted());
        }
    }

    fn contains(&self, q: Pos) -> bool {
        // TODO: When self.dominant is small, do a loop instead.
        self.dominant.binary_search(&LexPos(q)).map_or_else(
            // Check whether the first index lexicographically > q is also >= q in both coordinates.
            |idx| {
                //println!("{} {:?} {}", idx, self.dominant.get(idx), q);
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

    fn prune_filter<F: FnMut(Pos) -> bool>(&mut self, mut f: F) -> bool {
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

    fn dominant(&self) -> usize {
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
    max_arrow: usize,
}

impl<C: Contour> NaiveContours<C> {
    /// Get the value of the given position.
    /// It can be that a contour is completely empty, and skipped by length>1 arrows.
    /// In that case, normal binary search would give a wrong answer.
    /// Thus, we always have to check multiple contours.
    fn value_in_slice(contours: &[C], q: Pos, max_arrow: usize) -> usize {
        // q is always contained in layer 0.
        let mut left = 1;
        let mut right = contours.len();
        let mut size = right;
        while left < right {
            let mid = left + size / 2;
            let mut found = false;
            for c in mid..mid + max_arrow {
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
    fn new(arrows: impl IntoIterator<Item = Arrow>) -> Self {
        let mut this = NaiveContours {
            contours: vec![C::default()],
            arrows: HashMap::default(),
            max_arrow: 0,
        };
        this.contours[0].push(Pos(usize::MAX, usize::MAX));
        // Loop over all arrows from a given positions.
        for (start, pos_arrows) in &arrows.into_iter().group_by(|a| a.start) {
            let mut v = 0;
            this.arrows.insert(start, pos_arrows.collect());
            for a in &this.arrows[&start] {
                this.max_arrow = max(this.max_arrow, a.len);
                v = max(v, this.value(a.end) + a.len);
                //this.arrows.entry(start).or_default().push(a);
            }
            assert!(v > 0);
            if this.contours.len() <= v {
                this.contours.resize_with(v + 1, || C::default());
            }
            //println!("Push {} to layer {}", start, v);
            this.contours[v].push(start);
        }
        this
    }

    fn value(&self, q: Pos) -> usize {
        let v = Self::value_in_slice(&self.contours, q, self.max_arrow);
        //println!("Value of {} : {}", q, v);
        v
    }

    fn prune(&mut self, p: Pos) {
        if self.arrows.remove(&p).is_none() {
            // This position was already pruned.
            return;
        }

        // Work contour by contour.
        // 1. Remove p from it's first contour.
        let mut v = self.value(p);

        if !self.contours[v].prune(p) {
            return;
        }
        //println!("Value of {} = {}", p, v);

        // Loop over the dominant matches in the next layer, and repeatedly prune while needed.
        let mut last_change = v;
        let mut num_emptied = 0;
        let mut emptied_shift = None;
        loop {
            v += 1;
            if v >= self.contours.len() {
                break;
            }
            //println!("layer {} max_arrow {}", v, self.max_arrow);
            let (up_to_v, current) = {
                let (up_to_v, from_v) = self.contours.as_mut_slice().split_at_mut(v);
                (up_to_v, &mut from_v[0])
            };
            // We need to make a reference here to help rust understand we borrow disjoint parts of self.
            let arrows = &self.arrows;
            let max_arrow = self.max_arrow;
            let mut shift_to = None;
            if current.prune_filter(|pos| {
                if let Some(pos_arrows) = arrows.get(&pos) {
                    let mut max_new_val = 0;
                    assert!(!pos_arrows.is_empty());
                    //println!("Arrows {:?}", pos_arrows);
                    for arrow in pos_arrows {
                        // TODO: Start looking at contour v instead of a binary search.
                        let new_val =
                            Self::value_in_slice(up_to_v, arrow.end, max_arrow) + arrow.len;
                        //println!("Pos {} from {} to {}", arrow.start, v, new_val);
                        // value v is still up to date.
                        if new_val == v {
                            return false;
                        }
                        max_new_val = max(max_new_val, new_val);
                    }
                    assert!(v - max_arrow <= max_new_val && max_new_val <= v);
                    // Either no arrows left (position already pruned), or none of its arrows yields value v.
                    //println!("Push {} to {}", pos, max_new_val);
                    up_to_v[max_new_val].push(pos);
                    if shift_to.is_none() {
                        shift_to = Some(v - max_new_val);
                    } else if shift_to.unwrap() != max_new_val {
                        shift_to = Some(usize::MAX);
                    }
                    return true;
                } else {
                    shift_to = Some(usize::MAX);
                    // If no arrows left for this position, prune it without propagating.
                    return true;
                }
            }) {
                last_change = v;
            }

            if v >= last_change + self.max_arrow {
                //println!("Last change at {}, stopping at {}", last_change, v);
                // No further changes can happen.
                break;
            }

            if self.contours[v].len() == 0
                && (shift_to.is_none() || shift_to.unwrap() != usize::MAX)
            {
                if emptied_shift.is_none() || shift_to.is_none() || emptied_shift == shift_to {
                    num_emptied += 1;
                    if emptied_shift.is_none() {
                        emptied_shift = shift_to;
                    }
                }
                //println!("Num emptied to {} shift {:?}", num_emptied, emptied_shift);
            } else {
                num_emptied = 0;
                emptied_shift = None;
                //println!("Num emptied reset");
            }
            if num_emptied >= self.max_arrow {
                //println!("Emptied {}, stopping at {}", num_emptied, v);
                // Shift all other contours one down.
                if emptied_shift.is_some() {
                    for _ in 0..emptied_shift.unwrap() {
                        assert!(self.contours[v].len() == 0);
                        self.contours.remove(v);
                        v -= 1;
                    }
                    break;
                }
            }
        }
        for l in (0..8).rev() {
            if self.contours.len() > l {
                //println!("Contour {}: {:?}", l, self.contours[l]);
            }
        }
    }
}

/// A bruteforce Contours implementation answering queries in O(r), and pruning
/// in O(r^2) by rebuilding the entire datastructure.
#[derive(Default, Debug)]
pub struct BruteforceContours {
    valued_arrows: Vec<(Arrow, usize)>,
}

impl Contours for BruteforceContours {
    fn new(arrows: impl IntoIterator<Item = Arrow>) -> Self {
        let mut this = BruteforceContours {
            valued_arrows: Vec::default(),
        };
        for arrow in arrows {
            let val = this.value(arrow.end) + arrow.len;
            this.valued_arrows.push((arrow, val));
        }
        this
    }

    fn value(&self, q: Pos) -> usize {
        self.valued_arrows
            .iter()
            .filter(|(arrow, _)| q <= arrow.start)
            .map(|(_arrow, value)| *value)
            .max()
            .unwrap_or(0)
    }

    fn prune(&mut self, pos: Pos) {
        //println!("Size before pruning {}: {}", pos, self.valued_arrows.len());
        //println!("Arrows {:?}", self.valued_arrows);
        self.valued_arrows = Self::new(
            mem::take(&mut self.valued_arrows)
                .into_iter()
                .filter_map(|(a, _)| if a.start != pos { Some(a) } else { None }),
        )
        .valued_arrows;
        //println!("Size after pruning {}: {}", pos, self.valued_arrows.len());
    }
}
