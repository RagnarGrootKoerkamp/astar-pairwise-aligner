//! The basic bitpacked algorithm from Myers'99.
use crate::{HEncoding, Profile, B, S, V, W};
use std::simd::{LaneCount, SupportedLaneCount};

/// Implements the bitpacking algorithm explained in the Bitpal supplement.
/// Naming of h and v is swapped compared to their figure 4.
///
/// V encoding:
/// - vm (their D) for -1
/// - vmz (their S|D) for -1 or 0
///
/// H encoding:
/// - hz for 0
/// - hp for 1
///
/// h0: input horizontal delta that is shifted in.
/// hw: output horizontal delta that is shifted out.
///
/// 18 operations, excluding `eq`.
#[inline(always)]
pub fn compute_block<P: Profile, H: HEncoding>(h0: &mut H, v: &mut V, ca: &P::A, cb: &P::B) {
    let eq = P::eq(ca, cb); // this one is not counted as an operation
    let (vp, vm) = v.pm();
    let (vm, vmz) = (vm, !(vm | vp));
    let eq = eq | vm;
    //  // FIXME: IS THIS NEEDED
    //     let eq = eq | h0.m();
    let ris = !eq;
    let notmi = ris | vmz;
    let masksum = notmi.wrapping_add(vmz) & ris;
    let hz = masksum ^ notmi ^ vm;
    let hp = vm | (masksum & vmz);
    let hzw = hz >> (W - 1);
    let hpw = hp >> (W - 1);
    let hz = (hz << 1) | 0; //h0.z();
    let hp = (hp << 1) | h0.p();
    *h0 = H::from(hpw, (hpw | hzw) ^ 1);
    let vm = eq & hp;
    let vmz = hp | (eq & hz);
    *v = V::from(!(vm | vmz), vm);
}

#[inline(always)]
pub fn compute_block_wrapper<P: Profile, H: HEncoding>(
    h0: &mut H,
    v: &mut V,
    ca: &P::A,
    cb: &P::B,
) {
    let eq = P::eq(ca, cb); // this one is not counted as an operation
    let (vp, vm) = v.pm();

    let vmz = !vp;
    eprintln!("vm: {:064b}", vm);
    eprintln!("vmz {:064b}", vmz);
    let (hz0, hp0) = (1 ^ (h0.p() | h0.m()), h0.p());

    // start of BitPal code.
    let eq = eq | vm;
    let ris = !eq;
    eprintln!("ri: {:064b}", ris);
    let eq = eq | h0.m(); // Added by me.
    eprintln!("eq: {:064b}", eq);
    let notmi = ris | vmz;
    eprintln!("no: {:064b}", notmi);
    let risors = notmi ^ vm;
    eprintln!("rs: {:064b}", risors);
    let masksum = notmi.wrapping_add(vmz) & ris;
    eprintln!("su: {:064b}", masksum);
    let hz = masksum ^ risors;
    eprintln!("hz: {:064b}", hz);
    let hp = vm | (masksum & vmz);
    eprintln!("hp: {:064b}", hp);
    let hzw = hz >> (W - 1);
    let hpw = hp >> (W - 1);

    let hz = (hz << 1) | hz0;
    eprintln!("hz: {:064b}", hz);
    let hp = (hp << 1) | hp0;
    eprintln!("hp: {:064b}", hp);
    let vm = eq & hp;
    eprintln!("vm: {:064b}", vm);
    let vmz = hp | (eq & hz);
    eprintln!("vmz {:064b}", vmz);

    *h0 = H::from(hpw, (hpw | hzw) ^ 1);
    *v = V::from(!vmz, vm);
}

/// Simd version of `compute_block`.
///
/// This assumes HEncoding of `(u64,u64)`.
#[inline(always)]
pub fn compute_block_simd<const L: usize>(
    hz0: &mut S<L>,
    hp0: &mut S<L>,
    vm: &mut S<L>,
    vmz: &mut S<L>,
    eq: S<L>,
) where
    LaneCount<L>: SupportedLaneCount,
{
    let eq = eq | *vm;
    let ris = !eq;
    let notmi = ris | *vmz;
    let masksum = (notmi + *vmz) & ris;
    let hz = masksum ^ notmi ^ *vm;
    let hp = *vm | (masksum & *vmz);
    let right_shift = S::splat(W as B - 1);
    let hzw = hz >> right_shift;
    let hpw = hp >> right_shift;
    let left_shift = S::splat(1);
    let hz = (hz << left_shift) | *hz0;
    let hp = (hp << left_shift) | *hp0;
    *hz0 = hzw;
    *hp0 = hpw;
    *vm = eq & hp;
    *vmz = hp | (eq & hz);
}
