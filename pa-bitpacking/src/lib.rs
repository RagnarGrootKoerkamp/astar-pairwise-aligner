#![allow(incomplete_features)]
#![feature(
    let_chains,
    int_roundings,
    test,
    array_chunks,
    iter_array_chunks,
    array_windows,
    split_array,
    portable_simd,
    generic_const_exprs,
    concat_idents,
    bigint_helper_methods,
    core_intrinsics
)]

mod encoding;
pub mod simd;

use std::ops::{Deref, Range};

use bio::alphabets::{Alphabet, RankTransform};
pub use encoding::*;
use itertools::{izip, Itertools};
pub use pa_types::Seq;
use pa_types::{Sequence, I};

/// The type used for all bitvectors.
#[cfg(feature = "small_blocks")]
pub type B = u8;
/// The type used for all bitvectors.
#[cfg(not(feature = "small_blocks"))]
pub type B = u64;
/// The length of each bitvector.
pub const W: usize = B::BITS as usize;
/// The type used for differences.
pub type D = i64;
/// Default encoding used for horizontal differences.
pub type H = (u8, u8);

pub fn num_words(seq: Seq) -> usize {
    seq.len().div_ceil(W)
}

/// Newtype for compressed sequences that have characters 0,1,2,3.
pub struct CompressedSequence(Sequence);
#[derive(Copy, Clone)]
pub struct CompressedSeq<'a>(Seq<'a>);
impl Deref for CompressedSequence {
    type Target = Sequence;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a> Deref for CompressedSeq<'a> {
    type Target = Seq<'a>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl CompressedSequence {
    #[inline(always)]
    pub fn index(&self, index: Range<usize>) -> CompressedSeq {
        CompressedSeq(&self.0[index])
    }
}
impl<'a> Into<CompressedSeq<'a>> for &'a CompressedSequence {
    #[inline(always)]
    fn into(self) -> CompressedSeq<'a> {
        CompressedSeq(&self.0)
    }
}

pub type Profile = Vec<[B; 4]>;
pub type ProfileSlice<'a> = &'a [[B; 4]];

/// RankTransform `a` and `b`
#[inline(always)]
pub fn compress(a: Seq, b: Seq) -> (CompressedSequence, CompressedSequence) {
    let r = RankTransform::new(&Alphabet::new(b"ACGT"));
    let a = a.iter().map(|ca| r.get(*ca)).collect_vec();
    let b = b.iter().map(|ca| r.get(*ca)).collect_vec();
    (CompressedSequence(a), CompressedSequence(b))
}

/// RankTransform `a` and give the profile for `b`.
#[inline(always)]
pub fn profile(a: Seq, b: Seq) -> (CompressedSequence, Profile) {
    let r = RankTransform::new(&Alphabet::new(b"ACGT"));
    let a = a.iter().map(|ca| r.get(*ca)).collect_vec();
    let words = num_words(b);
    let mut pb: Profile = vec![[0; 4]; words];
    for (j, cb) in b.iter().enumerate() {
        pb[j / W][r.get(*cb) as usize] |= 1 << (j % W);
    }
    (CompressedSequence(a), pb)
}

/// Implements Myers '99 bitpacking based algorithm. Terminology is as in the
/// paper. The code is a translation from the implementation in Edlib.
///
/// Modifies `h0` (horizontal difference at top) and `v` (vertical differences
/// along the left) in place.
///
/// Given the scores below:
///
/// A0 - B0
/// |    |
/// A1 - B1
///   ...
/// AW - BW
///
/// h0 = B0 - A0
/// v[i] = A(i+1) - Ai
///
/// H and V are wrapper types to encode the horizontal and vertical differences
/// using a + and - indicator bit.
///
/// 20 operations.
#[inline(always)]
pub fn compute_block<H: HEncoding>(h0: &mut H, v: &mut V, eq: B) {
    let (pv, mv) = v.pm();
    let xv = eq | mv;
    let eq = eq | h0.m();
    // The add here contains the 'folding' magic that makes this algorithm
    // 'non-local' and prevents simple SIMDification. See Myers'99 for details.
    let xh = (((eq & pv).wrapping_add(pv)) ^ pv) | eq;
    let ph = mv | !(xh | pv);
    let mh = pv & xh;
    // Extract `hw` from `ph` and `mh`.
    // TODO: Use carry-bit from shit-left operation.
    // TODO: Use 63-bit vectors to save some operations.

    // Push `hw` out of `ph` and `mh` and shift in `h0`.
    // NOTE: overflowing_add uses the carry bit, but is slow because reading the
    // carry bit right after this instruction interrupts pipelining.
    // NOTE: overflowing_shl returns whether the shift is too large, not the shifted out bit.
    let phw = ph >> (W - 1);
    let mhw = mh >> (W - 1);
    let ph = (ph << 1) | h0.p();
    let mh = (mh << 1) | h0.m();

    // Update the inputs.
    *h0 = H::from(phw as B, mhw);
    *v = V::from(mh | !(xv | ph), ph & xv);
}

/// Wrapper that takes an unpacked h
#[inline(always)]
pub fn compute_block_split_h(ph0: &mut B, mh0: &mut B, v: &mut V, eq: B) {
    let h0 = &mut (*ph0, *mh0);
    compute_block(h0, v, eq);
    *ph0 = h0.0;
    *mh0 = h0.1;
}

/// Compute a range of columns, assuming horizontal input deltas of 1.
pub fn compute_rectangle(a: CompressedSeq, b: ProfileSlice, v: &mut [V]) -> D {
    assert_eq!(
        b.len(),
        v.len(),
        "Profile length {} must equal v length {}",
        b.len(),
        v.len()
    );
    let mut bot_delta = 0;
    for ca in a.iter() {
        let h = &mut H::one();
        for (cb, v) in izip!(b, v.iter_mut()) {
            compute_block(h, v, cb[*ca as usize]);
        }
        bot_delta += h.value();
    }
    bot_delta
}

/// Compute a rectangle, with given horizontal input deltas.
pub fn compute_rectangle_with_h(
    a: CompressedSeq,
    b: ProfileSlice,
    ph: &mut [B],
    mh: &mut [B],
    v: &mut [V],
) -> D {
    assert_eq!(a.len(), ph.len());
    assert_eq!(a.len(), mh.len());
    assert_eq!(b.len(), v.len());
    for (ca, ph, mh) in izip!(a.iter(), ph.iter_mut(), mh.iter_mut()) {
        let h = &mut (*ph, *mh);
        for (cb, v) in izip!(b, v.iter_mut()) {
            compute_block(h, v, cb[*ca as usize]);
        }
        *ph = h.0;
        *mh = h.1;
    }
    ph.iter().map(|x| *x as D).sum::<D>() - mh.iter().map(|x| *x as D).sum::<D>()
}

// Number of parellel simd rows.
const N: usize = 2;

/// Compute a block of columns using SIMD, assuming horizontal input deltas of 1.
/// Uses 2 SIMD rows in parallel for better instruction level parallelism.
pub fn compute_rectangle_simd(a: CompressedSeq, b: ProfileSlice, v: &mut [V]) -> D {
    if a.len() < 2 * 4 * N || b.len() < 4 * N {
        return compute_rectangle(a, b, v);
    }

    let ph = &mut vec![1; a.len()];
    let mh = &mut vec![0; a.len()];
    simd::compute_columns_simd_new::<N>(a, b, ph, mh, v)
}

/// Compute a block of columns using SIMD, assuming horizontal input deltas of 1.
/// Uses 2 SIMD rows in parallel for better instruction level parallelism.
pub fn compute_rectangle_simd_with_h(
    a: CompressedSeq,
    b: ProfileSlice,
    ph: &mut [B],
    mh: &mut [B],
    v: &mut [V],
) -> D {
    if a.len() < 2 * 4 * N || b.len() < 4 * N {
        return compute_rectangle_with_h(a, b, ph, mh, v);
    }
    assert_eq!(a.len(), ph.len());
    assert_eq!(a.len(), mh.len());
    assert_eq!(b.len(), v.len());

    simd::compute_columns_simd::<N>(a, b, ph, mh, v)
}

pub fn is_match(a: CompressedSeq, b: ProfileSlice, i: I, j: I) -> bool {
    let i = i as usize;
    let j = j as usize;
    ((b[j / W][a[i] as usize] >> (j % W)) & 1) == 1
}

pub mod new_profile {
    pub type Profile = Vec<(B, B)>;
    pub type ProfileSlice<'a> = &'a [(B, B)];
    use pa_types::I;

    use super::*;

    /// New profile experiment
    #[inline(always)]
    pub fn profile(a: Seq, b: Seq) -> (CompressedSequence, Profile) {
        let r = RankTransform::new(&Alphabet::new(b"ACGT"));
        let a = a.iter().map(|ca| r.get(*ca)).collect_vec();
        let words = num_words(b);
        let mut pb: Profile = vec![(0, 0); words];
        for (j, &cb) in b.iter().enumerate() {
            let cb = r.get(cb);
            // !cb[0]
            pb[j / W].0 |= ((cb as B & 1) ^ 1) << (j % W);
            // !cb[1]
            pb[j / W].1 |= (((cb as B >> 1) & 1) ^ 1) << (j % W);
        }
        (CompressedSequence(a), pb)
    }

    /// Compute a range of columns, assuming horizontal input deltas of 1.
    pub fn compute_rectangle(a: CompressedSeq, b: ProfileSlice, v: &mut [V]) -> D {
        assert_eq!(
            b.len(),
            v.len(),
            "Profile length {} must equal v length {}",
            b.len(),
            v.len()
        );
        let mut bot_delta = 0;
        for ca in a.iter() {
            let a0 = 0u64.wrapping_sub(*ca as B & 1);
            let a1 = 0u64.wrapping_sub((*ca as B >> 1) & 1);
            let h = &mut H::one();
            for (cb, v) in izip!(b, v.iter_mut()) {
                compute_block(h, v, (cb.0 ^ a0) & (cb.1 ^ a1));
            }
            bot_delta += h.value();
        }
        bot_delta
    }

    /// Compute a rectangle, with given horizontal input deltas.
    pub fn compute_rectangle_with_h(
        a: CompressedSeq,
        b: ProfileSlice,
        ph: &mut [B],
        mh: &mut [B],
        v: &mut [V],
    ) -> D {
        assert_eq!(a.len(), ph.len());
        assert_eq!(a.len(), mh.len());
        assert_eq!(b.len(), v.len());
        for (ca, ph, mh) in izip!(a.iter(), ph.iter_mut(), mh.iter_mut()) {
            let a0 = 0u64.wrapping_sub(*ca as B & 1);
            let a1 = 0u64.wrapping_sub((*ca as B >> 1) & 1);
            let h = &mut (*ph, *mh);
            for (cb, v) in izip!(b, v.iter_mut()) {
                compute_block(h, v, (cb.0 ^ a0) & (cb.1 ^ a1));
            }
            *ph = h.0;
            *mh = h.1;
        }
        ph.iter().map(|x| *x as D).sum::<D>() - mh.iter().map(|x| *x as D).sum::<D>()
    }

    /// Compute a block of columns using SIMD, assuming horizontal input deltas of 1.
    /// Uses 2 SIMD rows in parallel for better instruction level parallelism.
    pub fn compute_rectangle_simd(a: CompressedSeq, b: ProfileSlice, v: &mut [V]) -> D {
        if a.len() < 2 * 4 * N || b.len() < 4 * N {
            return compute_rectangle(a, b, v);
        }

        let ph = &mut vec![1; a.len()];
        let mh = &mut vec![0; a.len()];
        simd::new_profile::compute_columns_simd::<N>(a, b, ph, mh, v)
    }

    /// Compute a block of columns using SIMD, assuming horizontal input deltas of 1.
    /// Uses 2 SIMD rows in parallel for better instruction level parallelism.
    pub fn compute_rectangle_simd_with_h(
        a: CompressedSeq,
        b: ProfileSlice,
        ph: &mut [B],
        mh: &mut [B],
        v: &mut [V],
    ) -> D {
        if a.len() < 2 * 4 * N || b.len() < 4 * N {
            return compute_rectangle_with_h(a, b, ph, mh, v);
        }
        assert_eq!(a.len(), ph.len());
        assert_eq!(a.len(), mh.len());
        assert_eq!(b.len(), v.len());

        simd::new_profile::compute_columns_simd::<N>(a, b, ph, mh, v)
    }

    pub fn is_match(a: CompressedSeq, b: ProfileSlice, i: I, j: I) -> bool {
        let i = i as usize;
        let j = j as usize;
        let a0 = 0u64.wrapping_sub(a[i] as B & 1);
        let a1 = 0u64.wrapping_sub((a[i] as B >> 1) & 1);
        ((((b[j / W].0 ^ a0) & (b[j / W].1 ^ a1)) >> (j % W)) & 1) != 0
    }
}
