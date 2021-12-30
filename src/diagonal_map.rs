use crate::prelude::*;
use std::{collections::hash_map::Entry, ops::Index};

#[derive(PartialEq, Eq)]
pub enum InsertIfSmallerResult {
    New,
    Smaller,
    Larger,
}

/// Trait that wraps DiagonalMap or Hashmap for entries along a diagonal.
pub trait DiagonalMapTrait<Pos, V>: Index<Pos, Output = V> {
    fn new(target: Pos) -> Self;
    fn get(&self, pos: &Pos) -> Option<&V>;
    fn insert(&mut self, pos: Pos, v: V) -> Option<V>;
    fn insert_if_smaller(&mut self, pos: Pos, v: V) -> InsertIfSmallerResult
    where
        V: Ord;
}

/// A HashMap drop-in replacement for 2D data that's dense around the diagonal.
pub struct DiagonalMap<V> {
    // TODO: Move from Option to a separate bit vector.
    above: Vec<Vec<Option<V>>>,
    below: Vec<Vec<Option<V>>>,
    target: Pos,
}

// TODO: Use some NonZero types to make this type smaller.
#[derive(Debug)]
enum DIndex {
    Above(usize, usize),
    Below(usize, usize),
}
use DIndex::*;

impl<V> DiagonalMap<V> {
    #[inline]
    fn index_of(&self, &Pos(i, j): &Pos) -> DIndex {
        if i >= j {
            Above(i - j, j)
        } else {
            Below(j - i - 1, i)
        }
    }

    #[inline]
    fn get_mut_entry<'a>(&'a mut self, idx: &DIndex) -> &'a mut Option<V> {
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
                        vec.resize_with(len, || None);
                        vec
                    });
                }
            }
            Below(i, _j) => {
                if self.below.len() <= i {
                    let len = max(self.target.0, self.target.1) + 1;
                    self.below.resize_with(i + 1, || {
                        let mut vec = Vec::new();
                        vec.resize_with(len, || None);
                        vec
                    });
                }
            }
        }
    }
}

impl<V> DiagonalMapTrait<Pos, V> for DiagonalMap<V> {
    fn new(target: Pos) -> DiagonalMap<V> {
        DiagonalMap {
            above: Default::default(),
            below: Default::default(),
            target,
        }
    }

    #[inline]
    fn get(&self, pos: &Pos) -> Option<&V> {
        match self.index_of(pos) {
            Above(i, j) => self.above.get(i)?.get(j)?.as_ref(),
            Below(i, j) => self.below.get(i)?.get(j)?.as_ref(),
        }
    }

    #[inline]
    fn insert(&mut self, pos: Pos, v: V) -> Option<V> {
        let idx = self.index_of(&pos);
        self.get_mut_entry(&idx).replace(v)
    }
    /// Insert the given value if it is smaller than the current value.
    /// Returns true when inserted successfully.
    #[inline]
    fn insert_if_smaller(&mut self, pos: Pos, v: V) -> InsertIfSmallerResult
    where
        V: Ord,
    {
        let idx = self.index_of(&pos);
        match self.get_mut_entry(&idx) {
            x @ None => {
                *x = Some(v);
                InsertIfSmallerResult::New
            }
            Some(cur_v) if v < *cur_v => {
                *cur_v = v;
                InsertIfSmallerResult::Smaller
            }
            Some(_) => InsertIfSmallerResult::Larger,
        }
    }
}

impl<V> Index<Pos> for DiagonalMap<V> {
    type Output = V;

    #[inline]
    fn index(&self, pos: Pos) -> &Self::Output {
        match self.index_of(&pos) {
            Above(i, j) => self.above[i][j].as_ref().unwrap(),
            Below(i, j) => self.below[i][j].as_ref().unwrap(),
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
impl<V> DiagonalMapTrait<Pos, V> for HashMap<Pos, V> {
    fn new(_target: Pos) -> Self {
        Default::default()
    }

    fn get(&self, pos: &Pos) -> Option<&V> {
        self.get(pos)
    }

    fn insert(&mut self, pos: Pos, v: V) -> Option<V> {
        self.insert(pos, v)
    }
    fn insert_if_smaller(&mut self, pos: Pos, v: V) -> InsertIfSmallerResult
    where
        V: Ord,
    {
        match self.entry(pos) {
            Entry::Vacant(entry) => {
                entry.insert(v);
                InsertIfSmallerResult::New
            }
            Entry::Occupied(mut entry) => {
                if v < *entry.get() {
                    entry.insert(v);
                    InsertIfSmallerResult::Smaller
                } else {
                    InsertIfSmallerResult::Larger
                }
            }
        }
    }
}
