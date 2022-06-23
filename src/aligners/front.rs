use std::{
    fmt::Debug,
    ops::{Index, IndexMut, RangeInclusive},
};

use num_traits::{AsPrimitive, NumOps, NumRef, RefNum};

pub trait IndexType: NumOps + NumRef + Default + AsPrimitive<usize> + Copy + Debug {}
impl<I> IndexType for I where I: NumOps + NumRef + Default + AsPrimitive<usize> + Copy + Debug {}

/// A front contains the data for each affine layer, and a range to indicate which subset of diagonals/columns is computed for this front.
/// The offset indicates the position of the 0 column/diagonal.
///
/// T: the type of stored elements.
/// I: the index type.
#[derive(Clone, Debug)]
pub struct Front<const N: usize, T, I> {
    /// TODO: Merge the main and affine layers into a single allocation?
    /// TODO: Store layer-by-layer or position-by-position (ie index as
    /// [layer][position] or [position][layer])?
    /// The main layer.
    m: Vec<T>,
    /// The affine layer.
    affine: [Vec<T>; N],
    /// The inclusive range of values (diagonals/rows) this front corresponds to.
    range: RangeInclusive<I>,
    /// The left and right buffer we add before/after the range starts/ends.
    buffers: (I, I),
}

/// Indexing methods for `Front`.
impl<const N: usize, T, I> Front<N, T, I>
where
    I: IndexType,
    T: Copy,
{
    /// Create a new front for the given range.
    pub fn new(value: T, range: RangeInclusive<I>) -> Self
    where
        for<'l> &'l I: RefNum<I>,
    {
        Self::new_with_buffer(value, range, I::default(), I::default())
    }
    /// Resize the current front for the given range.
    /// Overwrites existing elements to the given value.
    pub fn reset(&mut self, value: T, range: RangeInclusive<I>)
    where
        for<'l> &'l I: RefNum<I>,
    {
        self.reset_with_buffer(value, range, I::default(), I::default())
    }

    /// Create a new front for the given range, using the given left/right buffer sizes.
    pub fn new_with_buffer(
        value: T,
        range: RangeInclusive<I>,
        left_buffer: I,
        right_buffer: I,
    ) -> Self
    where
        T: Copy,
        for<'l> &'l I: RefNum<I>,
    {
        let new_len: I = left_buffer + (range.end() - range.start() + I::one()) + right_buffer;
        Self {
            m: vec![value; new_len.as_()],
            // Vec is not Copy, so we use array::map instead.
            affine: [(); N].map(|_| vec![value; new_len.as_()]),
            range,
            buffers: (left_buffer, right_buffer),
        }
    }
    /// Resize the current front for the given range, using the given left/right buffer sizes.
    /// Overwrites existing elements to the given value.
    pub fn reset_with_buffer(
        &mut self,
        value: T,
        range: RangeInclusive<I>,
        left_buffer: I,
        right_buffer: I,
    ) where
        T: Clone,
        for<'l> &'l I: RefNum<I>,
    {
        let new_len: I = left_buffer + (range.end() - range.start() + I::one()) + right_buffer;
        self.m.clear();
        self.m.resize(new_len.as_(), value);
        for a in &mut self.affine {
            a.clear();
            a.resize(new_len.as_(), value);
        }
        self.range = range;
        self.buffers.0 = left_buffer;
        self.buffers.1 = right_buffer;
    }

    pub fn affine(&self, layer_idx: usize) -> Layer<'_, T, I> {
        Layer {
            l: &self.affine[layer_idx],
            range: self.range.clone(),
            buffers: self.buffers,
        }
    }

    pub fn affine_mut(&mut self, layer_idx: usize) -> MutLayer<'_, T, I> {
        MutLayer {
            l: &mut self.affine[layer_idx],
            range: self.range.clone(),
            buffers: self.buffers,
        }
    }

    pub fn m(&self) -> Layer<'_, T, I> {
        Layer {
            l: &self.m,
            range: self.range.clone(),
            buffers: self.buffers,
        }
    }
    pub fn m_affine(&self, layer_idx: usize) -> (Layer<'_, T, I>, Layer<'_, T, I>) {
        (
            Layer {
                l: &self.m,
                range: self.range.clone(),
                buffers: self.buffers,
            },
            Layer {
                l: &self.affine[layer_idx],
                range: self.range.clone(),
                buffers: self.buffers,
            },
        )
    }
    pub fn m_affine_mut(&mut self, layer_idx: usize) -> (MutLayer<'_, T, I>, MutLayer<'_, T, I>) {
        (
            MutLayer {
                l: &mut self.m,
                range: self.range.clone(),
                buffers: self.buffers,
            },
            MutLayer {
                l: &mut self.affine[layer_idx],
                range: self.range.clone(),
                buffers: self.buffers,
            },
        )
    }
    pub fn m_mut(&mut self) -> MutLayer<'_, T, I> {
        MutLayer {
            l: &mut self.m,
            range: self.range.clone(),
            buffers: self.buffers,
        }
    }

    /// Get a reference to the front's range.
    pub fn range(&self) -> &RangeInclusive<I> {
        &self.range
    }
}

/// A reference to a single layer of a single front.
/// Contains the offset needed to index it.
#[derive(Clone)]
pub struct Layer<'a, T, I> {
    /// The (affine) layer to use.
    /// TODO: Make this a slice instead of Vec.
    l: &'a [T],
    /// The index at which the range starts.
    range: RangeInclusive<I>,
    /// The left and right buffer we add before/after the range starts/ends.
    buffers: (I, I),
}

/// A mutable reference to a single layer of a single front.
/// Contains the offset needed to index it.
pub struct MutLayer<'a, T, I> {
    /// The (affine) layer to use.
    l: &'a mut [T],
    /// The index at which the range starts.
    range: RangeInclusive<I>,
    /// The left and right buffer we add before/after the range starts/ends.
    buffers: (I, I),
}

impl<'a, T, I> Layer<'a, T, I>
where
    I: IndexType,
{
    pub fn get(&self, index: I) -> Option<&T> {
        self.l
            .get((index + self.buffers.0 - self.range.start()).as_())
    }
}

/// Indexing for a Layer.
impl<'a, T, I> Index<I> for Layer<'a, T, I>
where
    I: IndexType,
{
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        &self.l[(index + self.buffers.0 - self.range.start()).as_()]
    }
}
/// Indexing for a mutable Layer.
impl<'a, T, I> Index<I> for MutLayer<'a, T, I>
where
    I: IndexType,
{
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        &self.l[(index + self.buffers.0 - self.range.start()).as_()]
    }
}
/// Indexing for a mutable Layer.
impl<'a, T, I> IndexMut<I> for MutLayer<'a, T, I>
where
    I: IndexType,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.l[(index + self.buffers.0 - self.range.start()).as_()]
    }
}
