use crate::prelude::*;
use itertools::Itertools;
use smallvec::SmallVec;
use std::{cell::UnsafeCell, mem};

use super::*;

/// A contour implementation that does all operations in O(r).
#[derive(Default, Debug)]
pub struct RotateToFrontContour {
    /// In typical cases the average size of contours is 1.5 at the start (before pruning).
    pub points: UnsafeCell<SmallVec<[Pos; 2]>>,
}
impl Clone for RotateToFrontContour {
    fn clone(&self) -> Self {
        unsafe {
            Self {
                points: UnsafeCell::new((*self.points.get()).clone()),
            }
        }
    }
}

impl Contour for RotateToFrontContour {
    fn push(&mut self, p: Pos) {
        self.points.get_mut().push(p);
    }
    fn contains_equal(&self, q: Pos) -> bool {
        unsafe { (*self.points.get()).contains(&q) }
    }

    fn contains(&self, q: Pos) -> bool {
        unsafe {
            let points = &mut *self.points.get();
            if let Some(idx) = points.iter().position(|s| q <= *s) {
                if idx > 0 {
                    points[0..=idx].rotate_right(1);
                }
                true
            } else {
                false
            }
        }
    }

    fn parent(&self, q: Pos) -> Pos {
        unsafe { *(*self.points.get()).iter().find(|s| q <= **s).unwrap() }
    }

    fn is_dominant(&self, q: Pos) -> bool {
        unsafe { !(*self.points.get()).iter().any(|s| q < *s) }
    }

    fn prune_filter<F: FnMut(Pos) -> bool>(&mut self, f: &mut F) -> bool {
        let mut change = false;
        self.points.get_mut().retain(|&mut s| {
            let prune = f(s);
            if prune {
                change = true;
            }
            !prune
        });
        change
    }

    fn len(&self) -> usize {
        unsafe { (*self.points.get()).len() }
    }

    fn num_dominant(&self) -> usize {
        unsafe {
            let mut x = (*self.points.get())
                .iter()
                .filter(|p| self.is_dominant(**p))
                .collect_vec();
            x.sort_by_key(|p| LexPos(**p));
            x.dedup();
            x.len()
        }
    }

    fn iterate_points<F: FnMut(Pos)>(&self, mut f: F) {
        unsafe {
            for p in &*self.points.get() {
                f(*p);
            }
        }
    }

    fn print_points(&self) {
        unsafe {
            for p in &*self.points.get() {
                println!("{p}");
            }
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

    fn prune_with_hint<R: Iterator<Item = Arrow>, F: Fn(&Pos) -> Option<R>>(
        &mut self,
        pos: Pos,
        _hint: Self::Hint,
        arrows: F,
    ) -> (bool, Cost) {
        let len_before = self.valued_arrows.len();
        let pos_arrows = arrows(&pos).map(|pa| pa.collect_vec()).unwrap_or_default();
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
