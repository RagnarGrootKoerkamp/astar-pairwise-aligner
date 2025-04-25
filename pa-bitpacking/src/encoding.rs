use crate::{B, W};
use pa_types::{Cost, I};

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub struct V(B, B);
impl V {
    #[inline(always)]
    pub fn zero() -> Self {
        V(0, 0)
    }
    #[inline(always)]
    pub fn one() -> Self {
        V(B::MAX, 0)
    }
    #[inline(always)]
    pub fn from(p: B, m: B) -> Self {
        V(p, m)
    }
    #[inline(always)]
    pub fn value(&self) -> Cost {
        self.0.count_ones() as Cost - self.1.count_ones() as Cost
    }
    /// Value of the first `j` bits.
    /// NOTE: Requires `0 <= j < W`.
    #[inline(always)]
    pub fn value_of_prefix(&self, j: I) -> Cost {
        debug_assert!(0 <= j && j < W as I);
        let mask = (1 << j) - 1;
        (self.0 & mask).count_ones() as Cost - (self.1 & mask).count_ones() as Cost
    }
    /// Value of the last `j` bits.
    /// NOTE: Requires `j > 0`.
    #[inline(always)]
    pub fn value_of_suffix(&self, j: I) -> Cost {
        debug_assert!(0 < j && j <= W as I);
        let mask = !(((1 as B) << (W as I - j)).wrapping_sub(1));
        (self.0 & mask).count_ones() as Cost - (self.1 & mask).count_ones() as Cost
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
    pub(crate) fn one_mut(&mut self) -> &mut B {
        &mut self.0
    }
    pub fn value_to(v: &Vec<V>, j: I) -> Cost {
        let mut s = 0;
        for vj in &v[0..j as usize / 64] {
            s += vj.value();
        }
        if j % 64 != 0 {
            s += v[j as usize / 64].value_of_prefix(j % 64);
        }
        s
    }
    pub fn value_from(v: &Vec<V>, j: I) -> Cost {
        let mut s = 0;
        if j % 64 != 0 {
            s += v[j as usize / 64].value_of_suffix(64 - j % 64);
        }
        for vj in &v[j.div_ceil(64) as usize..] {
            s += vj.value();
        }
        s
    }
}

pub trait HEncoding: Copy {
    fn zero() -> Self;
    fn one() -> Self;
    fn from(p: B, m: B) -> Self;
    fn value(&self) -> Cost;
    fn p(&self) -> B;
    fn m(&self) -> B;
    #[inline(always)]
    fn pm(&self) -> (B, B) {
        (self.p(), self.m())
    }
}

impl HEncoding for i8 {
    #[inline(always)]
    fn zero() -> Self {
        0
    }
    #[inline(always)]
    fn one() -> Self {
        1
    }
    #[inline(always)]
    fn from(p: B, m: B) -> Self {
        p as i8 - m as i8
    }
    #[inline(always)]
    fn value(&self) -> Cost {
        *self as Cost
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
    fn zero() -> Self {
        (0, 0)
    }
    #[inline(always)]
    fn one() -> Self {
        (1, 0)
    }
    #[inline(always)]
    fn from(p: B, m: B) -> Self {
        (p as u8, m as u8)
    }
    #[inline(always)]
    fn value(&self) -> Cost {
        self.0 as Cost - self.1 as Cost
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
    fn zero() -> Self {
        (0, 0)
    }
    #[inline(always)]
    fn one() -> Self {
        (1, 0)
    }
    #[inline(always)]
    fn from(p: B, m: B) -> Self {
        (p, m)
    }
    #[inline(always)]
    fn value(&self) -> Cost {
        self.0 as Cost - self.1 as Cost
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
