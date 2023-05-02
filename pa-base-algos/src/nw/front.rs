use std::ops::{Range, RangeInclusive};

use pa_affine_types::{AffineCigar, AffineCost, State};
use pa_types::*;

use crate::edit_graph::AffineCigarOps;

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
#[derive(Copy, Clone, Debug)]
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

pub trait NwFront: Default {
    /// The current rows in `0 ..= b.len()`.
    fn j_range(&self) -> JRange;
    /// The j_range, rounded to blocksize.
    fn j_range_rounded(&self) -> JRange {
        self.j_range()
    }
    /// Get the cost of row `j`.
    fn index(&self, j: I) -> Cost;
    /// Get the cost of row `j`.
    fn get(&self, j: I) -> Option<Cost>;
}

pub trait NwFrontsTag<const N: usize>: Copy {
    type Fronts<'a>: NwFronts<N>;
    const BLOCKSIZE: I;
    /// Constructs a new front and initializes it for `i=0`.
    fn new<'a>(
        trace: bool,
        a: Seq<'a>,
        b: Seq<'a>,
        cm: &'a AffineCost<N>,
        initial_j_range: JRange,
    ) -> Self::Fronts<'a>;
}

pub trait NwFronts<const N: usize> {
    type Front: NwFront;
    /// Compute the next `i_range` columns for `j_range`.
    /// `i_range` `start .. end` processes characters `start .. end` of `a`, and
    /// give the front at column `i`.
    // TODO: Pass in a visualizer.
    fn compute_next_block(&mut self, i_range: IRange, j_range: JRange);

    fn cm(&self) -> &AffineCost<N>;
    /// The current column in `0 ..= a.len()`.
    fn last_i(&self) -> I;
    /// Get the current front.
    fn last_front(&self) -> &Self::Front;

    fn parent(&self, st: State) -> Option<(State, AffineCigarOps)>;

    fn trace(&self, from: State, mut to: State) -> AffineCigar {
        let mut cigar = AffineCigar::default();

        while to != from {
            let (parent, cigar_ops) = self.parent(to).unwrap();
            to = parent;
            for op in cigar_ops {
                if let Some(op) = op {
                    cigar.push(op);
                }
            }
        }
        cigar.reverse();
        cigar
    }
}
