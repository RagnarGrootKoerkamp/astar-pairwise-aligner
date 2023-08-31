use pa_types::I;
use std::ops::{Deref, Range};

use crate::WI;

/// Left-exclusive range of columns to compute.
/// (-1, 0): the first column
/// (i, i+W): Compute column W given column i.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct IRange(pub I, pub I);

/// Inclusive range of rows to compute.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct JRange(pub I, pub I);

/// JRange that is guaranteed to be rounded out.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RoundedOutJRange(JRange);
/// JRange that is guaranteed to be rounded in.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RoundedInJRange(JRange);

impl IRange {
    pub fn first_col() -> Self {
        Self(-1, 0)
    }
    pub fn len(&self) -> I {
        self.1 - self.0
    }

    pub fn push(&mut self, other: Self) {
        assert!(self.1 == other.0);
        self.1 = other.1;
    }

    pub fn pop(&mut self, other: Self) {
        assert!(
            self.1 == other.1,
            "Can not pop range {other:?} from {self:?}"
        );
        self.1 = other.0;
    }

    pub fn consecutive(self, other: Self) -> bool {
        self.1 == other.0
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
    pub fn contains_range(&self, other: Self) -> bool {
        self.0 <= other.0 && other.1 <= self.1
    }
    pub fn union(self, other: Self) -> Self {
        Self(self.0.min(other.0), self.1.max(other.1))
    }
    pub fn intersection(self, other: Self) -> Self {
        Self(self.0.max(other.0), self.1.min(other.1))
    }
    pub fn round_out(&self) -> RoundedOutJRange {
        RoundedOutJRange(JRange(self.0 / WI * WI, self.1.next_multiple_of(WI)))
    }
    pub fn round_in(&self) -> RoundedInJRange {
        RoundedInJRange(JRange(self.0.next_multiple_of(WI), self.1 / WI * WI))
    }
    pub fn assert_rounded(self) -> RoundedOutJRange {
        assert!(self.0 % WI == 0 && self.1 % WI == 0);
        RoundedOutJRange(self)
    }
}

impl RoundedOutJRange {
    pub fn round_out(&self) -> Self {
        panic!("Already rounded out")
    }
    pub fn round_in(&self) -> RoundedInJRange {
        RoundedInJRange(self.0)
    }
    pub fn intersection(self, other: Self) -> Self {
        Self(JRange::intersection(self.0, other.0))
    }
    /// v_range is the vertical exclusive range of height-W blocks to compute.
    pub fn v_range(&self) -> Range<usize> {
        (self.0 .0 / WI) as usize..(self.0 .1 / WI) as usize
    }
}

impl RoundedInJRange {
    pub fn round_out(&self) -> RoundedOutJRange {
        RoundedOutJRange(self.0)
    }
    pub fn round_in(&self) -> Self {
        panic!("Already rounded in")
    }
    /// v_range is the vertical exclusive range of height-W blocks to compute.
    pub fn v_range(&self) -> Range<usize> {
        (self.0 .0 / WI) as usize..(self.0 .1 / WI) as usize
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
