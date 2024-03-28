use std::ops::{IndexMut, Range, RangeInclusive};

use pa_affine_types::{AffineCigar, AffineCost, State};
use pa_types::*;
use pa_vis::VisualizerInstance;

/// Left-exclusive range of columns to compute.
/// (-1, 0): the first column
/// (i, i+W): Compute column W given column i.
#[derive(Copy, Clone, Debug)]
pub struct IRange(pub I, pub I);

impl IRange {
    pub fn first_col() -> Self {
        Self(-1, 0)
    }
    pub fn len(&self) -> I {
        self.1 - self.0
    }
}

impl From<Range<I>> for IRange {
    fn from(r: Range<I>) -> Self {
        Self(r.start, r.end)
    }
}

impl Into<Range<I>> for IRange {
    fn into(self) -> Range<I> {
        self.0..self.1
    }
}

/// Inclusive range of rows to compute.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JRange(pub I, pub I);

impl JRange {
    pub fn is_empty(&self) -> bool {
        self.0 > self.1
    }
    pub fn len(&self) -> I {
        self.1 - self.0 + 1
    }
    pub fn exclusive_len(&self) -> I {
        self.1 - self.0
    }
    pub fn contains(&self, j: I) -> bool {
        self.0 <= j && j <= self.1
    }
}

impl From<RangeInclusive<I>> for JRange {
    fn from(r: RangeInclusive<I>) -> Self {
        Self(*r.start(), *r.end())
    }
}

impl Into<RangeInclusive<I>> for JRange {
    fn into(self) -> RangeInclusive<I> {
        self.0..=self.1
    }
}

/// Front type for the NW algorithm.
///
/// `Default` is only needed to pass an empty and usused front into `j_range`
/// for the initial column range.
pub trait NwFront: Default {
    /// The current rows in `0 ..= b.len()`.
    fn j_range(&self) -> JRange;
    /// The j_range, rounded to blocksize.
    /// This should only really be used for visualizer purposes.
    /// The NW algorithm itself should be agnostic to block size in the implementation.
    fn j_range_rounded(&self) -> JRange {
        self.j_range()
    }
    fn fixed_j_range(&self) -> Option<JRange>;
    /// Get the cost of row `j`.
    fn index(&self, j: I) -> Cost;
    /// Get the cost of row `j`.
    fn get(&self, j: I) -> Option<Cost>;
}

pub trait NwFrontsTag<const N: usize>: Copy + PartialEq {
    type Fronts<'a>: NwFronts<N>;
    const BLOCKSIZE: I;
    /// Constructs a new front.
    fn new<'a>(
        &self,
        trace: bool,
        a: Seq<'a>,
        b: Seq<'a>,
        cm: &'a AffineCost<N>,
    ) -> Self::Fronts<'a>;
}

pub trait NwFronts<const N: usize>: IndexMut<usize, Output = Self::Front> {
    type Front: NwFront;

    /// Initialize the front for i=0.
    /// This can be called multiple times to reuse an existing front.
    fn init(&mut self, initial_j_range: JRange);

    /// Compute the next `i_range` columns for `j_range`.
    /// `i_range` `start .. end` processes characters `start .. end` of `a`, and
    /// give the front at column `i`.
    // TODO: Pass in a visualizer.
    fn compute_next_block(
        &mut self,
        i_range: IRange,
        j_range: JRange,
        v: &mut impl VisualizerInstance,
    );

    fn reuse_next_block(&mut self, _i_range: IRange, _j_range: JRange) {
        unimplemented!();
    }

    /// Pop the last front.
    fn pop_last_front(&mut self) {
        todo!();
    }

    #[allow(unused)]
    fn cm(&self) -> &AffineCost<N>;
    /// The current column in `0 ..= a.len()`.
    #[allow(unused)]
    fn last_i(&self) -> I;
    /// Get the current front.
    fn last_front(&self) -> &Self::Front;

    /// Get the old range for the next front, if one exists.
    fn next_front_j_range(&self) -> Option<JRange> {
        None
        // unimplemented!();
    }

    /// Set the 'fixed' range of rows for the last front, that is, the interval
    /// `[start, end]` corresponding to the states with `f(u) <= f_max`.
    /// This isn't used by the front itself, but stored here for convenience.
    fn set_last_front_fixed_j_range(&mut self, fixed_j_range: Option<JRange>);

    fn trace(
        &mut self,
        _a: Seq,
        _b: Seq,
        _from: State,
        _to: State,
        _viz: &mut impl VisualizerInstance,
    ) -> AffineCigar;
}
