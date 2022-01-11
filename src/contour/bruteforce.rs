use std::mem;

use itertools::Itertools;

use crate::prelude::*;

/// A contour implementation that does all operations in O(r).
#[derive(Default, Debug)]
pub struct BruteForceContour {
    points: Vec<Pos>,
}

impl Contour for BruteForceContour {
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
            if q < s {
                return false;
            }
        }
        return true;
    }

    fn prune_filter<F: FnMut(Pos) -> bool>(&mut self, f: &mut F) -> bool {
        self.points.drain_filter(|&mut s| f(s)).count() > 0
    }

    fn len(&self) -> usize {
        self.points.len()
    }

    fn num_dominant(&self) -> usize {
        let mut x = self
            .points
            .iter()
            .filter(|p| self.is_dominant(**p))
            .collect_vec();
        x.sort_by_key(|p| LexPos(**p));
        x.dedup();
        x.len()
    }
}

/// A bruteforce Contours implementation answering queries in O(r), and pruning
/// in O(r^2) by rebuilding the entire datastructure.
#[derive(Default, Debug)]
pub struct BruteforceContours {
    valued_arrows: Vec<(Arrow, usize)>,
}

impl Contours for BruteforceContours {
    fn new(arrows: impl IntoIterator<Item = Arrow>, _max_len: usize) -> Self {
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
            0,
        )
        .valued_arrows;
        //println!("Size after pruning {}: {}", pos, self.valued_arrows.len());
    }
}
