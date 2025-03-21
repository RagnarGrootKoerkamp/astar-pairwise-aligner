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
