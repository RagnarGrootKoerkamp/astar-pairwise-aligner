use pa_types::I;
use std::ops::{Deref, Range, RangeInclusive};

use crate::WI;

/// Left-exclusive range of columns to compute.
/// (-1, 0): the first column
/// (i, i+W): Compute column W given column i.
#[derive(Copy, Clone, Debug)]
pub struct IRange(pub I, pub I);

/// Inclusive range of rows to compute.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JRange(pub I, pub I);

/// JRange that is guaranteed to be rounded out.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RoundedOutJRange(JRange);
/// JRange that is guaranteed to be rounded in.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RoundedInJRange(JRange);

impl IRange {
    pub fn first_col() -> Self {
        Self(-1, 0)
    }
    pub fn len(&self) -> I {
        self.1 - self.0
    }
}

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

    pub fn round_out(&self) -> RoundedOutJRange {
        RoundedOutJRange(JRange(self.0 / WI * WI, self.1.next_multiple_of(WI)))
    }

    pub fn round_in(&self) -> RoundedInJRange {
        RoundedInJRange(JRange(self.0.next_multiple_of(WI), self.1 / WI * WI))
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

impl RoundedOutJRange {
    pub fn round_out(&self) -> Self {
        panic!("Already rounded out")
    }
    pub fn round_in(&self) -> RoundedInJRange {
        RoundedInJRange(self.0)
    }
}

impl RoundedInJRange {
    pub fn round_out(&self) -> RoundedOutJRange {
        RoundedOutJRange(self.0)
    }
    pub fn round_in(&self) -> Self {
        panic!("Already rounded in")
    }
}

impl Deref for RoundedOutJRange {
    type Target = JRange;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for RoundedInJRange {
    type Target = JRange;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
