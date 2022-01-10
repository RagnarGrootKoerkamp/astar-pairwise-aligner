use std::{collections::BTreeSet, ops::Bound::*};

use crate::prelude::*;

/// Sorts points by (x-y, x).
#[derive(PartialEq, Eq, Debug)]
struct AntiDiagonal(Pos);
impl PartialOrd for AntiDiagonal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}
impl Ord for AntiDiagonal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0 .0 as isize - self.0 .1 as isize, self.0 .0)
            .cmp(&(other.0 .0 as isize - other.0 .1 as isize, other.0 .0))
    }
}

/// A contour implementation that does push, query, and prune in logarithmic time.
/// FIXME: This still has some bugs.
#[derive(Default, Debug)]
pub struct SetContour {
    points: BTreeSet<AntiDiagonal>,
    /// The set of dominant points, sorted lexicographically.
    // TODO: This could be a BTreeSet instead, although that would likely be slower in practice.
    // Sorted by LexPos.
    dominant: Vec<Pos>,

    /// The maximal width of the front.
    width: usize,
}

impl Contour for SetContour {
    fn with_max_len(max_len: usize) -> Self {
        SetContour {
            width: 2 * max_len - 1,
            ..Self::default()
        }
    }

    fn push(&mut self, p: Pos) {
        self.points.insert(AntiDiagonal(p));
        if !self.contains(p) {
            // TODO: This binary search could reuse the result from contains.
            let idx = self
                .dominant
                // Search by AntiDiagonal order.
                .binary_search_by_key(&LexPos(p), |&p| LexPos(p))
                .unwrap_err();
            self.dominant.insert(idx, p);

            {
                // The lines below remove the points shadowed by the newly inserted dominant point.
                // Equivalent to but faster than:
                //self.dominant.drain_filter(|&mut AntiDiagonalPos(q)| q < p);
                let mut delete_from = idx;
                while delete_from > 0 && self.dominant[delete_from - 1] <= p {
                    delete_from -= 1;
                }
                self.dominant.drain(delete_from..idx);
            }
            //assert!(self.dominant.is_sorted());
        }
    }

    fn contains(&self, q: Pos) -> bool {
        // TODO: When self.dominant is small, do a loop instead.
        self.dominant
            .binary_search_by_key(&LexPos(q), |&p| LexPos(p))
            .map_or_else(
                // Check whether the first index lexicographically > q is also >= q in both coordinates.
                |idx| {
                    if let Some(p) = self.dominant.get(idx) {
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
        self.dominant
            .binary_search_by_key(&LexPos(q), |&p| LexPos(p))
            .is_ok()
    }

    fn prune(&mut self, p: Pos) -> bool {
        if self.is_dominant(p) {
            //println!("PRUNE SINGLE POINT {}", p);
            // TODO: This unnecessarily loops over all dominant points.
            let v = self.prune_filter(&mut |s| s == p);
            //println!("PRUNE SINGLE POINT {} DONE", p);
            v
        } else {
            self.points.remove(&AntiDiagonal(p))
        }
    }

    /// Loop over dominant points. If pruned, try new dominant points
    /// until one is found that is not pruned.
    /// TODO: prune_filter with hint, so we don't need to iterate the entire contour.
    fn prune_filter<F: FnMut(Pos) -> bool>(&mut self, mut f: &mut F) -> bool {
        // This vector will only be used when at least one point is pruned.
        let mut new_dominant = Vec::<Pos>::default();
        let mut change = false;

        for (i, d) in self.dominant.iter().enumerate() {
            if !f(*d) {
                if change {
                    new_dominant.push(*d);
                }
                //println!("Keep dominant {}", d);
            } else {
                //println!("Prune dominant {}", d);
                if !change {
                    // Copy over the prefix of elements that we skipped so far.
                    change = true;
                    new_dominant = self.dominant[0..i].to_vec();
                }

                let prev_d = if change {
                    new_dominant.last().copied()
                } else {
                    if i == 0 {
                        None
                    } else {
                        self.dominant.get(i - 1).copied()
                    }
                };
                let next_d = self.dominant.get(i + 1).copied();
                //println!("prev d {:?}", prev_d);
                //println!("next d {:?}", next_d);

                let map_d = &AntiDiagonal(*d);
                // Prune this point, and find new dominant neighbours.
                let mut to_remove = vec![*d];

                // 1. Find potential dominant points on the left.
                let before = self.points.range((Unbounded, Excluded(map_d)));
                let after = self.points.range((Excluded(map_d), Unbounded));

                // Iterate points before d till one of
                // - a point at least width below d: all future points will be below d.
                // - a point at least width left of prev_d: all future points will be left of prev_d.
                // - a dominant point at the same y as d: no need to search more.
                //println!("before");
                {
                    // While iterating, we keep the first next y that is not yet covered by a dominant point.
                    // We skip points not below this.
                    let mut next_y = next_d.map_or(0, |p| p.1 + 1);
                    // Keep track of which points were added. This is needed
                    // because we find these points in reverse order of how we want
                    // to add them.
                    let old_new_dominant_size = new_dominant.len();
                    for &AntiDiagonal(p) in before.rev() {
                        if p.1 >= d.1 + self.width {
                            break;
                        }
                        if let Some(prev_d) = prev_d {
                            // TODO: This is probably a void check.
                            if p.0 <= prev_d.0 - self.width {
                                break;
                            }
                            if p <= prev_d {
                                continue;
                            }
                        }
                        if p.1 < next_y {
                            continue;
                        }

                        // Skip points not less than the previous dominant point.
                        if !(p <= *d) {
                            continue;
                        }

                        if f(p) {
                            //println!("Prune non-dominant {}", p);
                            // Discard this.
                            to_remove.push(p);
                            continue;
                        }
                        //println!("New dominant: {}", p);

                        // Keep this, and add it as dominant.
                        // Also iterate over the other newly dominant point to make sure we don't shadow those.
                        for i in old_new_dominant_size.. {
                            if i == new_dominant.len() {
                                break;
                            }
                            if new_dominant[i] <= p {
                                new_dominant.remove(i);
                            }
                        }
                        new_dominant.push(p);
                        next_y = p.1 + 1;
                    }
                    new_dominant[old_new_dominant_size..].reverse();
                }

                // Iterate points after d till one of
                // - a point at least width right of d: all future points will be right of d.
                // - a point at least width above next_d: all future points will be above of next_d.
                // - a dominant point at the same x as d: no need to search more.
                //println!("after");
                {
                    // While iterating, we keep the first next x that is not yet covered by a dominant point.
                    // We skip points not right of this.
                    let mut next_x = prev_d.map_or(0, |p| p.0 + 1);
                    let old_new_dominant_size = new_dominant.len();
                    for &AntiDiagonal(p) in after {
                        if p.0 >= d.0 + self.width {
                            break;
                        }
                        if let Some(next_d) = next_d {
                            // TODO: This is probably a void check.
                            if p.1 <= next_d.1 - self.width {
                                break;
                            }
                            if p <= next_d {
                                continue;
                            }
                        }
                        if p.0 < next_x {
                            continue;
                        }

                        // Skip points not less than the previous dominant point.
                        if !(p <= *d) {
                            continue;
                        }

                        if f(p) {
                            // Discard this.
                            to_remove.push(p);
                            //println!("Prune non-dominant {}", p);
                            continue;
                        }
                        //println!("New dominant: {}", p);

                        // Keep this, and add it as dominant.
                        // Also iterate over the other newly dominant point to make sure we don't shadow those.
                        for i in old_new_dominant_size.. {
                            if i >= new_dominant.len() {
                                break;
                            }
                            if new_dominant[i] <= p {
                                new_dominant.remove(i);
                            }
                        }
                        new_dominant.push(p);
                        next_x = p.0 + 1;
                    }
                }

                // Only actually remove points at the end. BTrees do not allow
                // modification while iterating them.
                for p in to_remove {
                    //println!("Remove from points: {}", p);
                    self.points.remove(&AntiDiagonal(p));
                }
            }
        }
        if change {
            self.dominant = new_dominant;
        }
        change

        // todo!();
        // if self.points.drain_filter(|&mut LexPos(s)| f(s)).count() > 0 {
        //     // Rebuild the dominant front.
        //     self.points.sort();
        //     self.dominant.clear();
        //     let mut next_j = 0;
        //     for LexPos(p) in self.points.iter().rev() {
        //         if p.1 >= next_j {
        //             self.dominant.push(LexPos(*p));
        //             next_j = p.1 + 1;
        //         }
        //     }
        //     // We push points from right to left, but the vector should be sorted left to right.
        //     self.dominant.reverse();
        //     assert!(self.dominant.is_sorted());
        //     true
        // } else {
        //     false
        // }
    }

    fn len(&self) -> usize {
        self.points.len()
    }

    fn num_dominant(&self) -> usize {
        self.dominant.len()
    }
}
