use crate::prelude::*;

/// A contour implementation that does push and query in logarithmic time, but prune in linear time.
#[derive(Default, Debug, Clone)]
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
