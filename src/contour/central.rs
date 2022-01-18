use std::cell::Cell;

use crate::prelude::*;

/// A contour implementation that does push and query in logarithmic time, but prune in linear time.
/// This caches the last relevant dominant point to answer queries along the main diagonal faster.
#[derive(Default, Debug)]
pub struct CentralContour {
    points: Vec<LexPos>,
    /// The set of dominant points, sorted lexicographically.
    /// TODO: This could be a BTreeSet instead, although that would likely be slower in practice.
    dominant: Vec<LexPos>,
    /// The index of the last position in `dominant` that was relevant for a `contains` query.
    last_dominant: Cell<usize>,
}

impl Contour for CentralContour {
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
        // Try the points around last_dominant.
        let last_idx = self.last_dominant.get();
        if let Some(&LexPos(p)) = self.dominant.get(last_idx) {
            if q <= p {
                return true;
            }
            // Check some points to the left or right, depending on where q lies relative to x, before falling back to binary search.
            const CHECK_DIST: usize = 4;
            if q.0 > p.0 {
                for idx in last_idx + 1..min(last_idx + CHECK_DIST, self.dominant.len()) {
                    if q.1 > self.dominant[idx].0 .1 {
                        return false;
                    }
                    if q.0 <= self.dominant[idx].0 .0 {
                        self.last_dominant.set(idx);
                        return true;
                    }
                }
            } else {
                for idx in (last_idx.saturating_sub(CHECK_DIST)..last_idx).rev() {
                    if q.0 > self.dominant[idx].0 .0 {
                        return false;
                    }
                    if q.1 <= self.dominant[idx].0 .1 {
                        self.last_dominant.set(idx);
                        return true;
                    }
                }
            }
        }
        // Fallback for when the point wasn't found near the diagonal.
        self.dominant.binary_search(&LexPos(q)).map_or_else(
            // Check whether the first index lexicographically > q is also >= q in both coordinates.
            |idx| {
                ////println!("{} {:?} {}", idx, self.dominant.get(idx), q);
                if let Some(LexPos(p)) = self.dominant.get(idx) {
                    self.last_dominant.set(idx);
                    q <= *p
                } else {
                    false
                }
            },
            // q is in dominant
            |idx| {
                self.last_dominant.set(idx);
                true
            },
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
