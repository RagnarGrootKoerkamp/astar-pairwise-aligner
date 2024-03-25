use pa_heuristic::Prune;
use std::ffi::CString;

/// Align sequences `a` and `b` of length `a_len` and `b_len` using A*PA2-simple.
///
/// The returned cigar must be freed using `astarpa_free_cigar`.
#[no_mangle]
pub unsafe extern "C" fn astarpa2_simple(
    a: *const u8,
    a_len: usize,
    b: *const u8,
    b_len: usize,
    // output parameters
    cigar_ptr: *mut *mut u8,
    cigar_len: *mut usize,
) -> u64 {
    let a = std::slice::from_raw_parts(a, a_len);
    let b = std::slice::from_raw_parts(b, b_len);
    let (cost, cigar) = astarpa2::astarpa2_simple(a, b);
    let cigar_string = cigar.to_string();
    *cigar_len = cigar_string.len();
    *cigar_ptr = CString::new(cigar_string).unwrap().into_raw() as *mut u8;
    cost as _
}

/// Align sequences `a` and `b` of length `a_len` and `b_len` using A*PA2-full.
///
/// The returned cigar must be freed using `astarpa_free_cigar`.
#[no_mangle]
pub unsafe extern "C" fn astarpa2_full(
    a: *const u8,
    a_len: usize,
    b: *const u8,
    b_len: usize,
    // output parameters
    cigar_ptr: *mut *mut u8,
    cigar_len: *mut usize,
) -> u64 {
    let a = std::slice::from_raw_parts(a, a_len);
    let b = std::slice::from_raw_parts(b, b_len);
    let (cost, cigar) = astarpa2::astarpa2_full(a, b);
    let cigar_string = cigar.to_string();
    *cigar_len = cigar_string.len();
    *cigar_ptr = CString::new(cigar_string).unwrap().into_raw() as *mut u8;
    cost as _
}

/// Globally align sequences `a` and `b` of length `a_len` and `b_len`.
/// This uses A*PA with GCSH with DT, inexact matches (r=2), seed length k=15, and pruning by start of matches.
///
/// Returns the cost, and `cigar_ptr` and `cigar_len` are set to the location and length of the null-terminated cigar string.
/// This must be freed using `astarpa_free_cigar`.
#[no_mangle]
pub unsafe extern "C" fn astarpa(
    a: *const u8,
    a_len: usize,
    b: *const u8,
    b_len: usize,
    // output parameters
    cigar_ptr: *mut *mut u8,
    cigar_len: *mut usize,
) -> u64 {
    astarpa_gcsh(a, a_len, b, b_len, 2, 15, false, cigar_ptr, cigar_len)
}

/// Call A*PA with custom parameters `r` and `k`, and allow pruning by end of
/// matches in addition to the default pruning by start.
#[no_mangle]
pub unsafe extern "C" fn astarpa_gcsh(
    a: *const u8,
    a_len: usize,
    b: *const u8,
    b_len: usize,
    // Parameters
    r: usize,
    k: usize,
    prune_end: bool,
    // output parameters
    cigar_ptr: *mut *mut u8,
    cigar_len: *mut usize,
) -> u64 {
    let a = std::slice::from_raw_parts(a, a_len);
    let b = std::slice::from_raw_parts(b, b_len);
    let (cost, cigar) = astarpa::astarpa_gcsh(
        a,
        b,
        r as _,
        k as _,
        if prune_end { Prune::Both } else { Prune::Start },
    );
    let cigar_string = cigar.to_string();
    *cigar_len = cigar_string.len();
    *cigar_ptr = CString::new(cigar_string).unwrap().into_raw() as *mut u8;
    cost as _
}

/// Free a returned cigar string.
#[no_mangle]
pub unsafe extern "C" fn astarpa_free_cigar(cigar: *mut u8) {
    drop(CString::from_raw(cigar as *mut i8))
}
