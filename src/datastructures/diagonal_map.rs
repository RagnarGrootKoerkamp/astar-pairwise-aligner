use crate::prelude::*;
use std::{
    cell::RefCell,
    ops::{Index, IndexMut},
};

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
    fn get(&self, pos: Pos) -> Option<&V>;
    fn get_mut(&mut self, pos: Pos) -> &mut V;
    fn dm_capacity(&self) -> usize;
}

/// A HashMap drop-in replacement for 2D data that's dense around the diagonal.
pub struct DiagonalMap<V> {
    offset: RefCell<Vec<Cost>>,
    above: Vec<Vec<V>>,
    below: Vec<Vec<V>>,
    // For each diagonal, allocate a number of blocks of length ~sqrt(n).
    blocks_per_diagonal: I,
    lg_block_size: usize,
    allocated_blocks: usize,
}

// TODO: Use some NonZero types to make this type smaller.
#[derive(Debug)]
enum DIndex {
    Above(I, I),
    Below(I, I),
}
use DIndex::*;

impl<V: Default + std::clone::Clone + Copy> DiagonalMap<V> {
    #[inline]
    fn index_of(&self, &Pos(i, j): &Pos) -> DIndex {
        let o = if DIAGONAL_MAP_OFFSET {
            let ref mut o = self.offset.borrow_mut()[i as usize];
            if *o == Cost::MAX {
                *o = j;
            }
            *o
        } else {
            i
        };
        if j <= o {
            Above(
                self.blocks_per_diagonal * (o - j) + (i >> self.lg_block_size),
                i & ((1 << self.lg_block_size) - 1),
            )
        } else {
            Below(
                self.blocks_per_diagonal * (j - o - 1) + (i >> self.lg_block_size),
                i & ((1 << self.lg_block_size) - 1),
            )
        }
    }

    #[inline]
    fn get_mut_entry(&mut self, idx: &DIndex) -> &mut V {
        self.grow(idx);
        // Unsafe is ok because `grow` makes sure these are within bounds.
        match *idx {
            Above(i, j) => unsafe {
                self.above
                    .get_unchecked_mut(i as usize)
                    .get_unchecked_mut(j as usize)
            },
            Below(i, j) => unsafe {
                self.below
                    .get_unchecked_mut(i as usize)
                    .get_unchecked_mut(j as usize)
            },
        }
    }

    #[inline]
    fn contains(&self, idx: &DIndex) -> bool {
        match *idx {
            // TODO: Reserving could be slightly more optimal.
            Above(i, j) => {
                if i >= self.above.len() as I {
                    return false;
                }
                if j >= self.above[i as usize].len() as I {
                    return false;
                }
            }
            Below(i, j) => {
                if i >= self.below.len() as I {
                    return false;
                }
                if j >= self.below[i as usize].len() as I {
                    return false;
                }
            }
        }
        true
    }

    #[inline]
    fn grow(&mut self, idx: &DIndex) {
        match *idx {
            // TODO: Reserving could be slightly more optimal.
            Above(i, j) => {
                if i >= self.above.len() as I {
                    self.above.resize_with(i as usize + 1, Vec::default);
                }
                if j >= self.above[i as usize].len() as I {
                    self.allocated_blocks += 1;
                    self.above[i as usize] = vec![V::default(); 1 << self.lg_block_size];
                }
            }
            Below(i, j) => {
                if i >= self.below.len() as I {
                    self.below.resize_with(i as usize + 1, Vec::default);
                }
                if j >= self.below[i as usize].len() as I {
                    self.allocated_blocks += 1;
                    self.below[i as usize] = vec![V::default(); 1 << self.lg_block_size];
                }
            }
        }
    }
}

impl<V: Default + Clone + Copy> DiagonalMapTrait<Pos, V> for DiagonalMap<V> {
    fn new(target: Pos) -> DiagonalMap<V> {
        // Block size should be a minimum size to prevent too small allocations.
        let mut lg_block_size = 8;
        let mut block_size = 1 << lg_block_size;
        let n = max(target.0, target.1);
        while block_size * block_size < n {
            block_size *= 2;
            lg_block_size += 1;
        }
        let num_blocks = (n >> lg_block_size) + 1;

        // Reserve length n arrays, roughly corresponding to a sqrt(n) band.
        let m = min(target.0, target.1);
        DiagonalMap {
            offset: RefCell::new(if DIAGONAL_MAP_OFFSET {
                vec![Cost::MAX; target.0 as usize + 1]
            } else {
                (0..target.0 + 1).collect()
            }),
            above: vec![Vec::default(); (max(n - m, 3) * num_blocks) as usize],
            below: vec![Vec::default(); (max(n - m, 3) * num_blocks) as usize],
            blocks_per_diagonal: num_blocks,
            lg_block_size,
            allocated_blocks: 0,
        }
    }

    fn get(&self, pos: Pos) -> Option<&V> {
        let idx = self.index_of(&pos);
        if self.contains(&idx) {
            Some(&self[pos])
        } else {
            None
        }
    }

    #[inline]
    fn get_mut(&mut self, pos: Pos) -> &mut V {
        self.get_mut_entry(&self.index_of(&pos))
    }

    #[inline]
    fn insert(&mut self, pos: Pos, v: V) {
        let idx = self.index_of(&pos);
        *self.get_mut_entry(&idx) = v;
    }

    fn dm_capacity(&self) -> usize {
        self.allocated_blocks << self.lg_block_size
    }
}

impl<V: Default + Clone + Copy> Index<Pos> for DiagonalMap<V> {
    type Output = V;

    #[inline]
    fn index(&self, pos: Pos) -> &Self::Output {
        match self.index_of(&pos) {
            Above(i, j) => &self.above[i as usize][j as usize],
            Below(i, j) => &self.below[i as usize][j as usize],
        }
    }
}

impl<V: Default + Clone + Copy> IndexMut<Pos> for DiagonalMap<V> {
    #[inline]
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        let idx = self.index_of(&pos);
        match idx {
            Above(i, j) => &mut self.above[i as usize][j as usize],
            Below(i, j) => &mut self.below[i as usize][j as usize],
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
        self.get_mut(&pos).unwrap()
    }
}
impl<V: Default> DiagonalMapTrait<Pos, V> for HashMap<Pos, V> {
    fn new(_target: Pos) -> Self {
        Default::default()
    }

    fn get(&self, pos: Pos) -> Option<&V> {
        self.get(&pos)
    }

    fn get_mut(&mut self, pos: Pos) -> &mut V {
        self.entry(pos).or_default()
    }

    fn insert(&mut self, pos: Pos, v: V) {
        self.insert(pos, v);
    }

    fn dm_capacity(&self) -> usize {
        self.capacity()
    }
}
