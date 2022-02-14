use crate::prelude::*;
use itertools::Itertools;
use smallvec::SmallVec;
use std::mem;

/// A contour implementation that does all operations in O(r).
#[derive(Default, Debug, Clone)]
pub struct BruteForceContour {
    points: SmallVec<[Pos; 2]>,
}

impl Contour for BruteForceContour {
    fn push(&mut self, p: Pos) {
        self.points.push(p);
    }

    fn contains(&self, q: Pos) -> bool {
        self.points.iter().any(|s| q <= *s)
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
            let val = this.value(arrow.end) + arrow.len;
            this.valued_arrows.push((arrow, val));
        }
        this
    }

    fn value(&self, q: Pos) -> Cost {
        self.valued_arrows
            .iter()
            .filter(|(arrow, _)| q <= arrow.start)
            .map(|(_arrow, value)| *value)
            .max()
            .unwrap_or(0)
    }

    fn prune_with_hint(
        &mut self,
        pos: Pos,
        _hint: Self::Hint,
        _arrows: &HashMap<Pos, Vec<Arrow>>,
    ) -> (bool, Cost) {
        let len_before = self.valued_arrows.len();
        self.valued_arrows = Self::new(
            mem::take(&mut self.valued_arrows)
                .into_iter()
                .filter_map(|(a, _)| if a.start != pos { Some(a) } else { None }),
            0,
        )
        .valued_arrows;
        let len_after = self.valued_arrows.len();
        (len_before != len_after, 0)
    }
}
