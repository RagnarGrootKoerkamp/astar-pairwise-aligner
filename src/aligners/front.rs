use std::ops::{Index, IndexMut, RangeInclusive, Sub};

use num_traits::AsPrimitive;

/// For a given AffineCost<N>, NW and DT use a main M layer, and N affine layers.
/// This struct wraps this and provides an iterator over all layers.
#[derive(Clone)]
pub struct Layers<const N: usize, T> {
    m: T,
    affine: [T; N],
}

impl<const N: usize, T> Layers<N, T> {
    pub fn new(m: T) -> Self
    where
        T: Clone,
    {
        let affine = [(); N].map(|_| m.clone());
        Self { m, affine }
    }
}

pub trait IndexType: Sub<Output = Self> + Sized + AsPrimitive<usize> + Copy {}
impl<I> IndexType for I where I: num_traits::AsPrimitive<usize> + std::ops::Sub<Output = I> + Copy {}

/// A front contains the data for each affine layer, and a range to indicate which subset of diagonals/columns is computed for this front.
/// The offset indicates the position of the 0 column/diagonal.
///
/// T: the type of stored elements.
/// I: the index type.
#[derive(Clone)]
pub struct Front<const N: usize, T, I> {
    /// TODO: Inline Layers struct here.
    /// TODO: Do not make this public.
    pub layers: Layers<N, Vec<T>>,
    /// The inclusive range of values (diagonals/rows) this front corresponds to.
    pub range: RangeInclusive<I>,
    /// The offset we need to index each layer.
    /// `offset` is the index corresponding to m[0].
    /// To get index `i`, find it at position `i - offset`.
    pub offset: I,
}

/// Indexing methods for `Front`.
impl<const N: usize, T, I> Front<N, T, I>
where
    I: IndexType,
{
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        self.layers.m.resize(new_len, value.clone());
        for affine_layer in &mut self.layers.affine {
            affine_layer.resize(new_len, value.clone());
        }
    }

    pub fn m(&self) -> Layer<'_, T, I> {
        Layer {
            l: &self.layers.m,
            offset: self.offset,
        }
    }
    pub fn affine(&self, layer_idx: usize) -> Layer<'_, T, I> {
        Layer {
            l: &self.layers.affine[layer_idx],
            offset: self.offset,
        }
    }
    pub fn m_mut(&mut self) -> MutLayer<'_, T, I> {
        MutLayer {
            l: &mut self.layers.m,
            offset: self.offset,
        }
    }
    pub fn affine_mut(&mut self, layer_idx: usize) -> MutLayer<'_, T, I> {
        MutLayer {
            l: &mut self.layers.affine[layer_idx],
            offset: self.offset,
        }
    }
    pub fn m_affine(&self, layer_idx: usize) -> (Layer<'_, T, I>, Layer<'_, T, I>) {
        (
            Layer {
                l: &self.layers.m,
                offset: self.offset,
            },
            Layer {
                l: &self.layers.affine[layer_idx],
                offset: self.offset,
            },
        )
    }
    pub fn m_affine_mut(&mut self, layer_idx: usize) -> (MutLayer<'_, T, I>, MutLayer<'_, T, I>) {
        (
            MutLayer {
                l: &mut self.layers.m,
                offset: self.offset,
            },
            MutLayer {
                l: &mut self.layers.affine[layer_idx],
                offset: self.offset,
            },
        )
    }
}

/// A reference to a single layer of a single front.
/// Contains the offset needed to index it.
#[derive(Clone, Copy)]
pub struct Layer<'a, T, I> {
    /// The (affine) layer to use.
    /// TODO: Make this a slice instead of Vec.
    l: &'a Vec<T>,
    /// The offset we need to index this layer.
    /// Equals `left_buffer - front.dmin`. Stored separately to suppport indexing
    /// without needing extra context.
    offset: I,
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

impl<'a, T, I> Layer<'a, T, I>
where
    I: IndexType,
{
    pub fn get(&self, d: I) -> Option<&T> {
        self.l.get((d - self.offset).as_())
    }
}

/// Indexing for a Layer.
impl<'a, T, I> Index<I> for Layer<'a, T, I>
where
    I: IndexType + std::fmt::Debug,
{
    type Output = T;

    fn index(&self, d: I) -> &Self::Output {
        &self.l[(d - self.offset).as_()]
    }
}
/// Indexing for a mutable Layer.
impl<'a, T, I> Index<I> for MutLayer<'a, T, I>
where
    I: IndexType,
{
    type Output = T;

    fn index(&self, d: I) -> &Self::Output {
        &self.l[(d - self.offset).as_()]
    }
}
/// Indexing for a mutable Layer.
impl<'a, T, I> IndexMut<I> for MutLayer<'a, T, I>
where
    I: IndexType,
{
    fn index_mut(&mut self, d: I) -> &mut Self::Output {
        &mut self.l[(d - self.offset).as_()]
    }
}
