use pa_types::I;

use crate::{B, D, W};

#[derive(Clone, Default, Copy)]
pub struct V(B, B);
impl V {
    #[inline(always)]
    pub fn one() -> Self {
        V(B::MAX, 0)
    }
    #[inline(always)]
    pub fn from(p: B, m: B) -> Self {
        V(p, m)
    }
    #[inline(always)]
    pub fn value(&self) -> D {
        self.0.count_ones() as D - self.1.count_ones() as D
    }
    /// Value of the first `j` bits.
    #[inline(always)]
    pub fn value_of_prefix(&self, j: I) -> D {
        assert!(0 <= j && j < W as I);
        let mask = (1 << j) - 1;
        (self.0 & mask).count_ones() as D - (self.1 & mask).count_ones() as D
    }
    /// Value of the last `j` bits.
    #[inline(always)]
    pub fn value_of_suffix(&self, j: I) -> D {
        if j == 0 {
            return 0;
        }
        assert!(0 < j && j <= W as I);
        let mask = !(((1 as B) << (W as I - j)).wrapping_sub(1));
        (self.0 & mask).count_ones() as D - (self.1 & mask).count_ones() as D
    }
    #[inline(always)]
    pub fn pm(&self) -> (B, B) {
        (self.0, self.1)
    }
    #[inline(always)]
    pub fn p(&self) -> B {
        self.0
    }
    #[inline(always)]
    pub fn m(&self) -> B {
        self.1
    }
}

pub trait HEncoding: Copy {
    fn one() -> Self;
    fn from(p: B, m: B) -> Self;
    fn value(&self) -> D;
    fn p(&self) -> B;
    fn m(&self) -> B;
}

impl HEncoding for i8 {
    #[inline(always)]
    fn one() -> Self {
        1
    }
    #[inline(always)]
    fn from(p: B, m: B) -> Self {
        p as i8 - m as i8
    }
    #[inline(always)]
    fn value(&self) -> D {
        *self as D
    }
    #[inline(always)]
    fn p(&self) -> B {
        (*self > 0) as B
    }
    #[inline(always)]
    fn m(&self) -> B {
        (*self < 0) as B
    }
}

impl HEncoding for (u8, u8) {
    #[inline(always)]
    fn one() -> Self {
        (1, 0)
    }
    #[inline(always)]
    fn from(p: B, m: B) -> Self {
        (p as u8, m as u8)
    }
    #[inline(always)]
    fn value(&self) -> D {
        self.0 as D - self.1 as D
    }
    #[inline(always)]
    fn p(&self) -> B {
        self.0 as B
    }
    #[inline(always)]
    fn m(&self) -> B {
        self.1 as B
    }
}

#[cfg(not(feature = "small_blocks"))]
impl HEncoding for (B, B) {
    #[inline(always)]
    fn one() -> Self {
        (1, 0)
    }
    #[inline(always)]
    fn from(p: B, m: B) -> Self {
        (p, m)
    }
    #[inline(always)]
    fn value(&self) -> D {
        self.0 as D - self.1 as D
    }
    #[inline(always)]
    fn p(&self) -> B {
        self.0
    }
    #[inline(always)]
    fn m(&self) -> B {
        self.1
    }
}
