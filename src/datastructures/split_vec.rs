//! A vector that is stored as two halves, such that erasing at (or near) the
//! current split point has constant time cost.
//!
//! Currently items are only ever moved from the front to the back, never in
//! reverse. This is good for removing items mostly from back to front.

use std::{
    cmp::Ordering,
    ops::{Index, IndexMut},
};

use crate::contour::Layer;

#[derive(Default, Debug)]
pub struct SplitVec<C> {
    /// The prefix of the vector.
    prefix: Vec<C>,
    /// The suffix stored of the vector, stored in reverse.
    suffix: Vec<C>,
}

impl<'a, C> IntoIterator for &'a SplitVec<C> {
    type Item = &'a C;

    type IntoIter =
        std::iter::Chain<std::slice::Iter<'a, C>, std::iter::Rev<std::slice::Iter<'a, C>>>;

    fn into_iter(
        self,
    ) -> std::iter::Chain<std::slice::Iter<'a, C>, std::iter::Rev<std::slice::Iter<'a, C>>> {
        self.prefix.iter().chain(self.suffix.iter().rev())
    }
}

impl<C> Index<usize> for SplitVec<C> {
    type Output = C;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.prefix.len() {
            &self.prefix[index]
        } else {
            &self.suffix[self.suffix.len() - 1 - (index - self.prefix.len())]
        }
    }
}

impl<C> IndexMut<usize> for SplitVec<C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.prefix.len() {
            &mut self.prefix[index]
        } else {
            let index = self.suffix.len() - 1 - (index - self.prefix.len());
            &mut self.suffix[index]
        }
    }
}

impl<C> Index<Layer> for SplitVec<C> {
    type Output = C;

    fn index(&self, index: Layer) -> &Self::Output {
        &self[index as usize]
    }
}

impl<C> IndexMut<Layer> for SplitVec<C> {
    fn index_mut(&mut self, index: Layer) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl<C> SplitVec<C> {
    pub fn len(&self) -> usize {
        self.prefix.len() + self.suffix.len()
    }

    pub fn get(&self, index: usize) -> Option<&C> {
        if index < self.len() {
            Some(&self[index])
        } else {
            None
        }
    }

    pub fn push(&mut self, value: C) {
        self.prefix.extend(self.suffix.drain(..).rev());
        self.prefix.push(value);
    }

    pub fn resize_with<F: FnMut() -> C>(&mut self, new_len: usize, f: F) {
        assert!(self.suffix.is_empty());
        self.prefix.resize_with(new_len, f);
    }

    pub fn remove(&mut self, index: usize) -> C {
        if index < self.prefix.len() {
            // Move items from front to back, and then pop the target item.
            self.suffix.extend(self.prefix.drain(index + 1..).rev());
            self.prefix.pop().unwrap()
        } else {
            let index = self.suffix.len() - 1 - (index - self.prefix.len());
            self.prefix.extend(self.suffix.drain(index + 1..).rev());
            self.suffix.pop().unwrap()
        }
    }

    pub fn binary_search_by<F>(&self, mut f: F) -> Result<usize, usize>
    where
        F: FnMut(&C) -> Ordering,
    {
        // If f(suffix.last())
        if let Some(split) = self.suffix.last() {
            match f(split) {
                Ordering::Less => {
                    // Binary search in the reverse of suffix.
                    match self.suffix.binary_search_by(|x| f(x).reverse()) {
                        Ok(pos) => Ok(self.prefix.len() + self.suffix.len() - 1 - pos),
                        Err(insert_pos) => Err(self.prefix.len() + self.suffix.len() - insert_pos),
                    }
                }
                Ordering::Equal => return Ok(self.prefix.len()),
                Ordering::Greater => self.prefix.binary_search_by(f),
            }
        } else {
            self.prefix.binary_search_by(f)
        }
    }
}
