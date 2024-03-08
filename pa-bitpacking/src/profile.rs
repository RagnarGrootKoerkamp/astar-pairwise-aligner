use bio::alphabets::{Alphabet, RankTransform};
use itertools::Itertools;
use pa_types::Seq;

use crate::{B, W};

/// Builds a 'profile' of `b` in `64`-bit blocks, and compressed `a` into a `[0,1,2,3]` alphabet.
///
/// Returns a bitpacked `B` indicating which chars of `b` equal a given char of `a`.
pub trait Profile: Clone + Copy + std::fmt::Debug {
    type A;
    type B;
    fn build(a: Seq, b: Seq) -> (Vec<Self::A>, Vec<Self::B>);
    fn eq(ca: &Self::A, cb: &Self::B) -> B;
}

#[derive(Clone, Copy, Debug)]
pub struct ScatterProfile;

/// Compressed Character in [0,1,2,3] alphabet.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CC(u8);

impl Profile for ScatterProfile {
    type A = CC;
    type B = [B; 4];

    fn build(a: Seq, b: Seq) -> (Vec<CC>, Vec<Self::B>) {
        let r = RankTransform::new(&Alphabet::new(b"ACGT"));
        let pa = a.iter().map(|ca| CC(r.get(*ca))).collect_vec();
        let mut pb = vec![[0; 4]; b.len().div_ceil(W)];
        for (j, cb) in b.iter().enumerate() {
            pb[j / W][r.get(*cb) as usize] |= 1 << (j % W);
        }
        (pa, pb)
    }

    #[inline(always)]
    fn eq(ca: &Self::A, cb: &Self::B) -> B {
        cb[ca.0 as usize]
    }
}

pub use bit_profile::BitProfile;

// Many public types with private members here, to keep things clean.
pub mod bit_profile {
    use std::simd::{LaneCount, SupportedLaneCount};

    use pa_types::I;

    use crate::S;

    use super::*;

    /// Just a typename
    #[derive(Clone, Copy, Debug)]
    pub struct BitProfile;
    /// Indicate the 0-bit and 1-bit of a character.
    #[derive(Clone, Copy, Debug)]
    pub struct Bits(pub(crate) B, pub(crate) B);

    // TODO: Investigate the impact of storing `(u64,u64)` per character of `a`.
    // Might be bad for cache locality compared to a simple `u8`.
    impl Profile for BitProfile {
        /// Exploded bit-encoding of `a`.
        /// a = 0: (00..00, 00..00)
        /// a = 1: (11..11, 00..00)
        /// a = 2: (00..00, 11..11)
        /// a = 3: (11..11, 11..11)
        type A = Bits;
        /// 64-char packed *negated* bit-encoding of `b`.
        /// b = 0: (..1.., ..1..)
        /// b = 1: (..0.., ..1..)
        /// b = 2: (..1.., ..0..)
        /// b = 3: (..0.., ..0..)
        ///
        /// See `eq` for details.
        type B = Bits;

        fn build(a: Seq, b: Seq) -> (Vec<Self::A>, Vec<Self::B>) {
            let r = RankTransform::new(&Alphabet::new(b"ACGT"));
            let pa = a
                .iter()
                .map(|ca| {
                    let a = CC(r.get(*ca));
                    Bits(
                        (0 as B).wrapping_sub(a.0 as B & 1),
                        (0 as B).wrapping_sub((a.0 as B >> 1) & 1),
                    )
                })
                .collect_vec();
            let mut pb = vec![Bits(0, 0); b.len().div_ceil(W)];
            for (j, &cb) in b.iter().enumerate() {
                let cb = r.get(cb);
                // !cb[0]
                pb[j / W].0 |= ((cb as B & 1) ^ 1) << (j % W);
                // !cb[1]
                pb[j / W].1 |= (((cb as B >> 1) & 1) ^ 1) << (j % W);
            }
            (pa, pb)
        }

        /// `a` is equals to `b` if both bits are the same, so
        /// `(a.0 == b.0) & (a.1 == b.1)`
        /// where `.0` and `.1` are bit `0` and `1`, and `==` is bitwise.
        /// Since bitwise `==` does not exist, we can do
        /// `(a.0 == b.0) === !(a.0 ^ b.0) === a.0 ^ (!b.0)`.
        /// That's why we store `!b.0` and `!b.1` in the profile.
        #[inline(always)]
        fn eq(ca: &Self::A, cb: &Self::B) -> B {
            (ca.0 ^ cb.0) & (ca.1 ^ cb.1)
        }
    }
    impl BitProfile {
        #[inline(always)]
        pub fn eq_simd<const L: usize>(ca: (&S<L>, &S<L>), cb: (&S<L>, &S<L>)) -> S<L>
        where
            LaneCount<L>: SupportedLaneCount,
        {
            (ca.0 ^ cb.0) & (ca.1 ^ cb.1)
        }
        pub fn is_match(a: &[Bits], b: &[Bits], i: I, j: I) -> bool {
            (Self::eq(&a[i as usize], &b[j as usize / W]) & (1 << (j as usize % W))) != 0
        }
    }
}
