use std::{
    fmt::Debug,
    ops::{Index, IndexMut, Range, RangeInclusive},
};

use super::edit_graph::Layer as LayerIdx;
use num_traits::{AsPrimitive, NumOps, NumRef, RefNum};

pub trait IndexType:
    NumOps + NumRef + Default + AsPrimitive<usize> + Copy + Debug + std::iter::Step
{
}
impl<I> IndexType for I where
    I: NumOps + NumRef + Default + AsPrimitive<usize> + Copy + Debug + std::iter::Step
{
}

/// A front contains the data for each affine layer, and a range to indicate
/// which subset of diagonals/columns is computed for this front.
/// The offset indicates the position of the 0 column/diagonal.
///
/// T: the type of stored elements.
/// I: the index type.
#[derive(Clone)]
pub struct Front<const N: usize, T, I> {
    /// TODO: Merge the main and affine layers into a single array?
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

impl<const N: usize, T, I> Debug for Front<N, T, I>
where
    T: Debug + Clone,
    I: IndexType,
{
    fn fmt<'a>(&'a self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = &self.m[self.buffers.0.as_()..self.m.len() - self.buffers.1.as_()];
        let mut slices: [&[T]; N] = [&[]; N];
        for i in 0..N {
            slices[i] =
                &self.affine[i][self.buffers.0.as_()..self.affine[i].len() - self.buffers.1.as_()];
        }
        f.debug_struct("Front")
            .field("m", &m)
            .field("affine", &slices)
            .field("range", &self.range)
            .field("buffers", &self.buffers)
            .finish()
    }
}

/// `Fronts` is a vector of fronts, possibly with a buffer layer at the top.
/// TODO: Add `fronts.rotate()` and `fronts.grow()` functions to add a new front.
pub struct Fronts<const N: usize, T, I> {
    pub fronts: Vec<Front<N, T, I>>,
    /// The default value.
    value: T,
    /// The inclusive range of values this front corresponds to.
    range: RangeInclusive<I>,
    /// The top and bottom buffer we add before/after the range of fronts.
    buffers: (I, I),
    lr_buffers: (I, I),
}

impl<const N: usize, T, I> Debug for Fronts<N, T, I>
where
    T: Debug + Clone,
    I: IndexType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fronts")
            .field("fronts", &self.fronts)
            .field("value", &self.value)
            .field("range", &self.range)
            .field("buffers", &self.buffers)
            .field("lr_buffers", &self.lr_buffers)
            .finish()
    }
}

impl<const N: usize, T, I> Fronts<N, T, I>
where
    I: IndexType,
    T: Copy,
{
    /// Create a new front for the given range, using the given left/right buffer sizes.
    pub fn new(
        value: T,
        range: RangeInclusive<I>,
        range_fn: impl Fn(I) -> RangeInclusive<I>,
        top_buffer: I,
        bottom_buffer: I,
        left_buffer: I,
        right_buffer: I,
    ) -> Self
    where
        T: Copy,
        for<'l> &'l I: RefNum<I>,
    {
        Self {
            fronts: (range.start() - top_buffer..=range.end() + bottom_buffer)
                .map(|i| Front::new(value, range_fn(i), left_buffer, right_buffer))
                .collect(),
            value,
            range,
            buffers: (top_buffer, bottom_buffer),
            lr_buffers: (left_buffer, right_buffer),
        }
    }

    pub fn push(&mut self, range: RangeInclusive<I>)
    where
        T: Copy,
        for<'l> &'l I: RefNum<I>,
    {
        // We can not initialize all layers directly at the start, since we do not know the final distance s.
        self.fronts.push(Front::new(
            self.value,
            range,
            self.lr_buffers.0,
            self.lr_buffers.1,
        ));
        self.range = *self.range.start()..=*self.range.end() + I::one();
    }

    pub fn rotate(&mut self, range: RangeInclusive<I>)
    where
        for<'l> &'l I: RefNum<I>,
    {
        self.fronts.rotate_left(1);
        self.fronts.last_mut().unwrap().reset(self.value, range);
        self.range = *self.range.start() + I::one()..=*self.range.end() + I::one();
    }

    pub fn rotate_back(&mut self)
    where
        for<'l> &'l I: RefNum<I>,
    {
        self.fronts.rotate_right(1);
        self.range = *self.range.start() - I::one()..=*self.range.end() - I::one();
    }

    /// Get a reference to the fronts's range.
    #[must_use]
    pub fn range(&self) -> &RangeInclusive<I> {
        &self.range
    }

    /// Get a reference to the fronts's range.
    #[must_use]
    pub fn full_range(&self) -> RangeInclusive<I>
    where
        for<'l> &'l I: RefNum<I>,
    {
        *self.range.start() - self.buffers.0..=*self.range.end() + self.buffers.1
    }

    /// Get a reference to the fronts's range.
    #[must_use]
    pub fn last(&self) -> &Front<N, T, I> {
        self.fronts.last().unwrap()
    }

    #[must_use]
    pub fn split_at(&mut self, index: I) -> (&Front<N, T, I>, &mut Front<N, T, I>) {
        let index = (index + self.buffers.0 - self.range.start()).as_();
        let (begin, end) = self.fronts.split_at_mut(index);
        (&begin[index - 1], &mut end[0])
    }
}

impl<const N: usize, T, I: Default> Default for Front<N, T, I> {
    fn default() -> Self {
        Self {
            m: Vec::default(),
            affine: [(); N].map(|_| Vec::default()),
            range: I::default()..=I::default(),
            buffers: (I::default(), I::default()),
        }
    }
}

/// Indexing methods for `Front`.
impl<const N: usize, T, I> Front<N, T, I>
where
    I: IndexType,
    T: Copy,
{
    /// Create a new front for the given range, using the given left/right buffer sizes.
    pub fn new(value: T, range: RangeInclusive<I>, left_buffer: I, right_buffer: I) -> Self
    where
        T: Copy,
        for<'l> &'l I: RefNum<I>,
    {
        let new_len: I = if range.is_empty() {
            left_buffer + right_buffer
        } else {
            left_buffer + (range.end() - range.start() + I::one()) + right_buffer
        };
        Self {
            m: vec![value; new_len.as_()],
            // Vec is not Copy, so we use array::map instead.
            affine: [(); N].map(|_| vec![value; new_len.as_()]),
            range,
            buffers: (left_buffer, right_buffer),
        }
    }

    /// Resize the current front for the given range, reusing the buffer sizes.
    pub fn reset(&mut self, value: T, range: RangeInclusive<I>)
    where
        T: Clone,
        for<'l> &'l I: RefNum<I>,
    {
        self.range = range;
        let new_len: I = if self.range.is_empty() {
            self.buffers.0 + self.buffers.1
        } else {
            self.buffers.0 + (self.range.end() - self.range.start() + I::one()) + self.buffers.1
        };
        self.m.clear();
        self.m.resize(new_len.as_(), value);
        for a in &mut self.affine {
            a.clear();
            a.resize(new_len.as_(), value);
        }
    }

    /// Get a reference to the front's range.
    #[inline]
    pub fn range(&self) -> &RangeInclusive<I> {
        &self.range
    }

    // ========== FRONT INDEXING ==========

    #[inline]
    pub fn m(&self) -> Layer<'_, T, I> {
        Layer {
            l: &self.m,
            range: self.range.clone(),
            buffers: self.buffers,
        }
    }

    #[inline]
    pub fn m_mut(&mut self) -> MutLayer<'_, T, I> {
        MutLayer {
            l: &mut self.m,
            range: self.range.clone(),
            buffers: self.buffers,
        }
    }

    #[inline]
    pub fn affine(&self, layer_idx: usize) -> Layer<'_, T, I> {
        Layer {
            l: self.affine.index(layer_idx),
            range: self.range.clone(),
            buffers: self.buffers,
        }
    }

    #[inline]
    pub fn affine_mut(&mut self, layer_idx: usize) -> MutLayer<'_, T, I> {
        MutLayer {
            l: self.affine.index_mut(layer_idx),
            range: self.range.clone(),
            buffers: self.buffers,
        }
    }

    #[inline]
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

    // ========== FRONT INDEXING BY LAYER ==========

    #[inline]
    pub fn layer(&self, layer: LayerIdx) -> Layer<'_, T, I> {
        match layer {
            None => self.m(),
            Some(layer) => self.affine(layer),
        }
    }

    #[inline]
    pub fn layer_mut(&mut self, layer: LayerIdx) -> MutLayer<'_, T, I> {
        match layer {
            None => self.m_mut(),
            Some(layer) => self.affine_mut(layer),
        }
    }
}

// ========== LAYER STRUCTS ==========

/// A reference to a single layer of a single front.
/// Contains the offset needed to index it.
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

// ========== LAYER INDEXING ==========

impl<'a, T, I> Layer<'a, T, I>
where
    I: IndexType,
{
    #[inline]
    pub fn get(&self, index: I) -> Option<&'a T> {
        self.l
            .get((index + self.buffers.0 - self.range.start()).as_())
    }
}
impl<'a, T, I> Index<I> for Layer<'a, T, I>
where
    I: IndexType,
{
    type Output = T;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self
            .l
            .index((index + self.buffers.0 - self.range.start()).as_())
    }
}
impl<'a, T, I> Index<I> for MutLayer<'a, T, I>
where
    I: IndexType,
{
    type Output = T;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self
            .l
            .index((index + self.buffers.0 - self.range.start()).as_())
    }
}
impl<'a, T, I> IndexMut<I> for MutLayer<'a, T, I>
where
    I: IndexType,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.l
            .index_mut((index + self.buffers.0 - self.range.start()).as_())
    }
}

// ========== LAYER RANGE INDEXING ==========

impl<'a, T, I> Index<Range<I>> for Layer<'a, T, I>
where
    I: IndexType,
{
    type Output = [T];

    #[inline]
    fn index(&self, index: Range<I>) -> &Self::Output {
        &self.l[(index.start + self.buffers.0 - self.range.start()).as_()
            ..(index.end + self.buffers.0 - self.range.start()).as_()]
    }
}
impl<'a, T, I> Index<Range<I>> for MutLayer<'a, T, I>
where
    I: IndexType,
{
    type Output = [T];

    #[inline]
    fn index(&self, index: Range<I>) -> &Self::Output {
        &self.l[(index.start + self.buffers.0 - self.range.start()).as_()
            ..(index.end + self.buffers.0 - self.range.start()).as_()]
    }
}
impl<'a, T, I> IndexMut<Range<I>> for MutLayer<'a, T, I>
where
    I: IndexType,
{
    #[inline]
    fn index_mut(&mut self, index: Range<I>) -> &mut Self::Output {
        &mut self.l[(index.start + self.buffers.0 - self.range.start()).as_()
            ..(index.end + self.buffers.0 - self.range.start()).as_()]
    }
}

// ========== FRONTS INDEXING ==========

impl<const N: usize, T, I> Index<I> for Fronts<N, T, I>
where
    I: IndexType,
{
    type Output = Front<N, T, I>;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.fronts[(index + self.buffers.0 - self.range.start()).as_()]
    }
}
impl<const N: usize, T, I> IndexMut<I> for Fronts<N, T, I>
where
    I: IndexType,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.fronts[(index + self.buffers.0 - self.range.start()).as_()]
    }
}

// ========== FRONTS RANGE INDEXING ==========

impl<const N: usize, T, I> Index<RangeInclusive<I>> for Fronts<N, T, I>
where
    I: IndexType,
{
    type Output = [Front<N, T, I>];
    #[inline]
    fn index(&self, index: RangeInclusive<I>) -> &Self::Output {
        &self.fronts[(*index.start() + self.buffers.0 - self.range.start()).as_()
            ..=(*index.end() + self.buffers.0 - self.range.start()).as_()]
    }
}
impl<const N: usize, T, I> IndexMut<RangeInclusive<I>> for Fronts<N, T, I>
where
    I: IndexType,
{
    #[inline]
    fn index_mut(&mut self, index: RangeInclusive<I>) -> &mut Self::Output {
        &mut self.fronts[(*index.start() + self.buffers.0 - self.range.start()).as_()
            ..=(*index.end() + self.buffers.0 - self.range.start()).as_()]
    }
}
