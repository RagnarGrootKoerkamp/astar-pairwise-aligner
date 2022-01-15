use crate::prelude::*;
use std::ops::{Index, IndexMut};

#[derive(PartialEq, Eq)]
pub enum InsertIfSmallerResult {
    New,
    Smaller,
    Larger,
}

/// Trait that wraps DiagonalMap or Hashmap for entries along a diagonal.
pub trait DiagonalMapTrait<Pos, V>: Index<Pos, Output = V> + IndexMut<Pos> {
    fn new(target: Pos) -> Self;
    fn insert(&mut self, pos: Pos, v: V);
}

/// A HashMap drop-in replacement for 2D data that's dense around the diagonal.
pub struct DiagonalMap<V> {
    // TODO: Move from Option to a separate bit vector.
    above: Vec<Vec<V>>,
    below: Vec<Vec<V>>,
    target: Pos,
}

// TODO: Use some NonZero types to make this type smaller.
#[derive(Debug)]
enum DIndex {
    Above(usize, usize),
    Below(usize, usize),
}
use DIndex::*;

impl<V: Default> DiagonalMap<V> {
    #[inline]
    fn index_of(&self, &Pos(i, j): &Pos) -> DIndex {
        if i >= j {
            Above(i - j, j)
        } else {
            Below(j - i - 1, i)
        }
    }

    #[inline]
    fn get_mut_entry<'a>(&'a mut self, idx: &DIndex) -> &'a mut V {
        self.grow(idx);
        match *idx {
            Above(i, j) => &mut self.above[i][j],
            Below(i, j) => &mut self.below[i][j],
        }
    }

    #[inline]
    fn grow(&mut self, idx: &DIndex) {
        match *idx {
            // TODO: Reserving could be slightly more optimal.
            Above(i, _j) => {
                while self.above.len() <= i {
                    let len = max(self.target.0, self.target.1) + 1;
                    self.above.resize_with(i + 1, || {
                        let mut vec = Vec::new();
                        vec.resize_with(len, || V::default());
                        vec
                    });
                }
            }
            Below(i, _j) => {
                if self.below.len() <= i {
                    let len = max(self.target.0, self.target.1) + 1;
                    self.below.resize_with(i + 1, || {
                        let mut vec = Vec::new();
                        vec.resize_with(len, || V::default());
                        vec
                    });
                }
            }
        }
    }
}

impl<V: Default> DiagonalMapTrait<Pos, V> for DiagonalMap<V> {
    fn new(target: Pos) -> DiagonalMap<V> {
        DiagonalMap {
            above: Default::default(),
            below: Default::default(),
            target,
        }
    }

    #[inline]
    fn insert(&mut self, pos: Pos, v: V) {
        let idx = self.index_of(&pos);
        *self.get_mut_entry(&idx) = v;
    }
}

impl<V: Default> Index<Pos> for DiagonalMap<V> {
    type Output = V;

    #[inline]
    fn index(&self, pos: Pos) -> &Self::Output {
        match self.index_of(&pos) {
            Above(i, j) => &self.above[i][j],
            Below(i, j) => &self.below[i][j],
        }
    }
}

impl<V: Default> IndexMut<Pos> for DiagonalMap<V> {
    #[inline]
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        let idx = self.index_of(&pos);
        self.grow(&idx);
        match idx {
            Above(i, j) => &mut self.above[i][j],
            Below(i, j) => &mut self.below[i][j],
        }
    }
}

/// Implement DiagonalMapTrait for HashMap.
impl<V> Index<Pos> for HashMap<Pos, V> {
    type Output = V;

    #[inline]
    fn index(&self, pos: Pos) -> &Self::Output {
        &self[&pos]
    }
}
impl<V: Default> IndexMut<Pos> for HashMap<Pos, V> {
    #[inline]
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        self.entry(pos).or_default()
    }
}
impl<V: Default> DiagonalMapTrait<Pos, V> for HashMap<Pos, V> {
    fn new(_target: Pos) -> Self {
        Default::default()
    }

    fn insert(&mut self, pos: Pos, v: V) {
        self.insert(pos, v);
    }
}
