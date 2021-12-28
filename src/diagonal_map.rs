use crate::prelude::*;
use std::ops::Index;

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

pub struct OccupiedEntry<'a, V>(&'a mut V);
pub struct VacantEntry<'a, V>(&'a mut Option<V>);

impl<'a, V> OccupiedEntry<'a, V> {
    #[inline]
    pub fn get(&self) -> &V {
        self.0
    }
    #[inline]
    pub fn insert(&mut self, value: V) -> V {
        std::mem::replace(self.0, value)
    }
}
impl<'a, V> VacantEntry<'a, V> {
    #[inline]
    pub fn insert(self, value: V) -> &'a V {
        self.0.insert(value)
    }
}

pub enum Entry<'a, V> {
    Occupied(OccupiedEntry<'a, V>),
    Vacant(VacantEntry<'a, V>),
}

impl<V> DiagonalMap<V> {
    pub fn new(target: Pos) -> DiagonalMap<V> {
        DiagonalMap {
            above: Default::default(),
            below: Default::default(),
            target,
        }
    }

    #[inline]
    fn get_index(&self, &Pos(i, j): &Pos) -> DIndex {
        if i >= j {
            Above(i - j, j)
        } else {
            Below(j - i - 1, i)
        }
    }

    #[inline]
    fn insert_index(&mut self, idx: DIndex, v: V) -> Option<V> {
        self.grow(&idx);
        match idx {
            Above(i, j) => self.above[i][j].replace(v),
            Below(i, j) => self.below[i][j].replace(v),
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

    #[inline]
    pub fn get(&self, pos: &Pos) -> Option<&V> {
        match self.get_index(pos) {
            Above(i, j) => self.above.get(i)?.get(j)?.as_ref(),
            Below(i, j) => self.below.get(i)?.get(j)?.as_ref(),
        }
    }

    #[inline]
    pub fn get_mut<'a>(&'a mut self, pos: &Pos) -> Option<&'a mut V> {
        self.get_mut_index(&self.get_index(pos))
    }

    #[inline]
    fn get_mut_index<'a>(&'a mut self, idx: &DIndex) -> Option<&'a mut V> {
        match *idx {
            Above(i, j) => self.above.get_mut(i)?.get_mut(j)?.as_mut(),
            Below(i, j) => self.below.get_mut(i)?.get_mut(j)?.as_mut(),
        }
    }

    #[inline]
    pub fn insert(&mut self, pos: Pos, v: V) -> Option<V> {
        let idx = self.get_index(&pos);
        self.insert_index(idx, v)
    }

    /// We assume an insertion will happen, so grow the vector in advance.
    #[inline]
    pub fn entry(&mut self, pos: Pos) -> Entry<'_, V> {
        let idx = self.get_index(&pos);
        self.grow(&idx);
        let option = match idx {
            Above(i, j) => &mut self.above[i][j],
            Below(i, j) => &mut self.below[i][j],
        };

        match option {
            Some(ref mut v) => Entry::Occupied(OccupiedEntry(v)),
            None => Entry::Vacant(VacantEntry(option)),
        }
    }
}

impl<V: std::fmt::Debug> Index<&Pos> for DiagonalMap<V> {
    type Output = V;

    #[inline]
    fn index(&self, pos: &Pos) -> &Self::Output {
        match self.get_index(pos) {
            Above(i, j) => self.above[i][j].as_ref().unwrap(),
            Below(i, j) => self.below[i][j].as_ref().unwrap(),
        }
    }
}

pub trait ToPos {
    fn to_pos(&self) -> Pos;
}

impl ToPos for Pos {
    #[inline]
    fn to_pos(&self) -> Pos {
        *self
    }
}
