use crate::prelude::*;
use itertools::Itertools;
use smallvec::SmallVec;
use std::mem;

use super::*;

/// A contour implementation that does all operations in O(r).
#[derive(Default, Debug, Clone)]
pub struct BruteForceContour {
    pub points: SmallVec<[Pos; 2]>,
}

impl Contour for BruteForceContour {
    fn push(&mut self, p: Pos) {
        #[cfg(debug_assertions)]
        {
            let contains = self.points.contains(&p);
            assert!(!contains);
        }
        self.points.push(p);
    }
    fn contains_equal(&self, q: Pos) -> bool {
        self.points.contains(&q)
    }

    fn contains(&self, q: Pos) -> bool {
        self.points.iter().any(|s| q <= *s)
    }

    fn parent(&self, q: Pos) -> Pos {
        *self.points.iter().find(|s| q <= **s).unwrap()
    }

    fn is_dominant(&self, q: Pos) -> bool {
        !self.points.iter().any(|s| q < *s)
    }

    fn prune_filter<F: FnMut(Pos) -> bool>(&mut self, f: &mut F) -> bool {
        let mut change = false;
        self.points.retain(|&mut s| {
            let prune = f(s);
            if prune {
                change = true;
            }
            !prune
        });
        change
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

    fn iterate_points<F: FnMut(Pos)>(&self, mut f: F) {
        for p in &self.points {
            f(*p);
        }
    }

    fn print_points(&self) {
        for p in &self.points {
            println!("{p}");
        }
    }
}

/// A bruteforce Contours implementation answering queries in O(r), and pruning
/// in O(r^2) by rebuilding the entire datastructure.
#[derive(Default, Debug)]
pub struct BruteForceContours {
    valued_arrows: Vec<(Arrow, Cost)>,
}

impl Contours for BruteForceContours {
    fn new(arrows: impl IntoIterator<Item = Arrow>, _max_len: I) -> Self {
        let mut this = BruteForceContours {
            valued_arrows: Vec::default(),
        };
        for arrow in arrows {
            let val = this.score(arrow.end) + arrow.score as Cost;
            this.valued_arrows.push((arrow, val));
        }
        this
    }

    fn score(&self, q: Pos) -> Cost {
        self.valued_arrows
            .iter()
            .filter(|(arrow, _)| q <= arrow.start)
            .map(|(_arrow, value)| *value)
            .max()
            .unwrap_or(0)
    }

    fn parent(&self, q: Pos) -> (Cost, Pos) {
        self.valued_arrows
            .iter()
            .filter(|(arrow, _)| q <= arrow.start)
            .map(|(arrow, value)| (*value, arrow.start))
            .max_by_key(|&(a, p)| (a, LexPos(p)))
            .unwrap()
    }

    fn prune_with_hint(
        &mut self,
        pos: Pos,
        _hint: Self::Hint,
        arrows: &HashMap<Pos, Vec<Arrow>>,
    ) -> (bool, Cost) {
        let len_before = self.valued_arrows.len();
        let v = vec![];
        let pos_arrows = arrows.get(&pos).unwrap_or(&v);
        self.valued_arrows = Self::new(
            mem::take(&mut self.valued_arrows)
                .into_iter()
                .filter_map(|(a, _)| {
                    if a.start != pos {
                        Some(a)
                    } else {
                        // Check if a is contained in `arrows`.
                        if pos_arrows.contains(&a) {
                            Some(a)
                        } else {
                            None
                        }
                    }
                }),
            0,
        )
        .valued_arrows;
        let len_after = self.valued_arrows.len();
        (len_before != len_after, 0)
    }
}
