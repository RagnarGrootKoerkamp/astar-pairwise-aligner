#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Align sequences `a` and `b` of length `a_len` and `b_len` using A*PA2-simple.
 *
 * The returned cigar must be freed using `astarpa_free_cigar`.
 */
uint64_t astarpa2_simple(const uint8_t *a,
                         uintptr_t a_len,
                         const uint8_t *b,
                         uintptr_t b_len,
                         uint8_t **cigar_ptr,
                         uintptr_t *cigar_len);

/**
 * Align sequences `a` and `b` of length `a_len` and `b_len` using A*PA2-full.
 *
 * The returned cigar must be freed using `astarpa_free_cigar`.
 */
uint64_t astarpa2_full(const uint8_t *a,
                       uintptr_t a_len,
                       const uint8_t *b,
                       uintptr_t b_len,
                       uint8_t **cigar_ptr,
                       uintptr_t *cigar_len);

/**
 * Globally align sequences `a` and `b` of length `a_len` and `b_len`.
 * This uses A*PA with GCSH with DT, inexact matches (r=2), seed length k=15, and pruning by start of matches.
 *
 * Returns the cost, and `cigar_ptr` and `cigar_len` are set to the location and length of the null-terminated cigar string.
 * This must be freed using `astarpa_free_cigar`.
 */
uint64_t astarpa(const uint8_t *a,
                 uintptr_t a_len,
                 const uint8_t *b,
                 uintptr_t b_len,
                 uint8_t **cigar_ptr,
                 uintptr_t *cigar_len);

/**
 * Call A*PA with custom parameters `r` and `k`, and allow pruning by end of
 * matches in addition to the default pruning by start.
 */
uint64_t astarpa_gcsh(const uint8_t *a,
                      uintptr_t a_len,
                      const uint8_t *b,
                      uintptr_t b_len,
                      uintptr_t r,
                      uintptr_t k,
                      bool prune_end,
                      uint8_t **cigar_ptr,
                      uintptr_t *cigar_len);

/**
 * Free a returned cigar string.
 */
void astarpa_free_cigar(uint8_t *cigar);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
