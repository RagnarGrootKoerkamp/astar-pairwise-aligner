use crate::prelude::*;
use itertools::Itertools;
use std::collections::BTreeSet;

use super::*;

/// A contour implementation that does all operations in O(r).
#[derive(Default, Debug, Clone)]
pub struct SortedContour {
    /// In typical cases the average size of contours is 1.5 at the start (before pruning).
    points: BTreeSet<LexPos>,
}

impl Contour for SortedContour {
    fn push(&mut self, p: Pos) {
        #[cfg(debug_assertions)]
        {
            let contains = self.points.contains(&LexPos(p));
            assert!(!contains);
        }
        self.points.insert(LexPos(p));
    }
    fn contains_equal(&self, q: Pos) -> bool {
        self.points.contains(&LexPos(q))
    }

    fn contains(&self, q: Pos) -> bool {
        for parent in self.points.range(LexPos(q)..).take(2) {
            if q <= parent.0 {
                return true;
            }
        }

        false
    }

    fn parent(&self, q: Pos) -> Pos {
        if let Some(parent) = self.points.range(LexPos(q)..).next() {
            assert!(q <= parent.0, "Parent not found");
            parent.0
        } else {
            panic!("Parent not found");
        }
    }

    fn is_dominant(&self, q: Pos) -> bool {
        todo!();
        // !self.points.iter().any(|s| q < *s)
    }

    fn prune_filter<F: FnMut(Pos) -> bool>(&mut self, f: &mut F) -> bool {
        let mut change = false;
        self.points.retain(|s| {
            let prune = f(s.0);
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
            .filter(|p| self.is_dominant(p.0))
            .collect_vec();
        x.sort();
        x.dedup();
        x.len()
    }

    fn iterate_points<F: FnMut(Pos)>(&self, mut f: F) {
        for p in &self.points {
            f(p.0);
        }
    }

    fn print_points(&self) {
        for p in &self.points {
            println!("{}", p.0);
        }
    }
}
