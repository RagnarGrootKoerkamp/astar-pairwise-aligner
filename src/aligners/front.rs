use std::ops::{Add, Index, IndexMut, RangeInclusive};

use num_traits::AsPrimitive;

use super::layer::Layers;

pub trait IndexType: Add<Output = Self> + Sized + AsPrimitive<usize> + Copy {}
impl<I> IndexType for I where I: num_traits::AsPrimitive<usize> + std::ops::Add<Output = I> + Copy {}

/// A front contains the data for each affine layer, and a range to indicate which subset of diagonals/columns is computed for this front.
/// The offset indicates the position of the 0 column/diagonal.
///
/// T: the type of stored elements.
/// I: the index type.
#[derive(Clone)]
pub struct Front<const N: usize, T, I> {
    pub layers: Layers<N, Vec<T>>,
    /// The inclusive range of values (diagonals/columns) this front corresponds to.
    pub range: RangeInclusive<I>,
    /// The offset we need to index each layer.
    pub offset: I,
}

/// Indexing methods for `Front`.
impl<const N: usize, T, I> Front<N, T, I>
where
    I: IndexType,
{
    pub fn m(&self) -> Layer<'_, T, I> {
        Layer {
            l: &self.layers.m,
            offset: self.offset,
        }
    }
    pub fn affine(&self, index: usize) -> Layer<'_, T, I> {
        Layer {
            l: &self.layers.affine[index],
            offset: self.offset,
        }
    }
    pub fn m_mut(&mut self) -> MutLayer<'_, T, I> {
        MutLayer {
            l: &mut self.layers.m,
            offset: self.offset,
        }
    }
    pub fn affine_mut(&mut self, index: usize) -> MutLayer<'_, T, I> {
        MutLayer {
            l: &mut self.layers.affine[index],
            offset: self.offset,
        }
    }
}

/// A reference to a single layer of a single front.
/// Contains the offset needed to index it.
#[derive(Clone, Copy)]
pub struct Layer<'a, T, I> {
    /// The (affine) layer to use.
    l: &'a Vec<T>,
    /// The offset we need to index this layer.
    /// Equals `left_buffer - front.dmin`. Stored separately to suppport indexing
    /// without needing extra context.
    offset: I,
}
/// Indexing for a Layer.
impl<'a, T, I> Index<I> for Layer<'a, T, I>
where
    I: IndexType,
{
    type Output = T;

    fn index(&self, d: I) -> &Self::Output {
        &self.l[(self.offset + d).as_() as usize]
    }
}

/// A mutable reference to a single layer of a single front.
/// Contains the offset needed to index it.
pub struct MutLayer<'a, T, I> {
    /// The (affine) layer to use.
    l: &'a mut Vec<T>,
    /// The offset we need to index this layer.
    /// Equals `left_buffer - dmin`. Stored separately to suppport indexing
    /// without needing extra context.
    offset: I,
}
/// Indexing for a mutable Layer.
impl<'a, T, I> Index<I> for MutLayer<'a, T, I>
where
    I: IndexType,
{
    type Output = T;

    fn index(&self, d: I) -> &Self::Output {
        &self.l[(self.offset + d).as_() as usize]
    }
}
/// Indexing for a mutable Layer.
impl<'a, T, I> IndexMut<I> for MutLayer<'a, T, I>
where
    I: IndexType,
{
    fn index_mut(&mut self, d: I) -> &mut Self::Output {
        &mut self.l[(self.offset + d).as_() as usize]
    }
}
