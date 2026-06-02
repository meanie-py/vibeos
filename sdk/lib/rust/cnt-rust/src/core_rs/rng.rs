//! `rand`/`srand` core, faithful to ReactOS `stdlib/rand_nt.c`. A
//! 48-bit-multiplier LCG over a `u64` seed (`next = next * 0x5deece66d
//! + 11`), returning bits 16..30 (`(next >> 16) & RAND_MAX`). The
//! hidden `static next` lives in the cabi shim (an `AtomicU64`, the
//! `strtok` precedent); the core is the pure step + the seed install,
//! so the differential test can drive identical `srand`/`rand`
//! sequences against the genuine C and assert parity.

/// `RAND_MAX` — ReactOS `<stdlib.h>` value (15-bit).
pub const RAND_MAX: i32 = 0x7fff;

/// One `rand()` step: advance the LCG state, return `(new_state,
/// result)` where `result = (new_state >> 16) & RAND_MAX`.
#[inline]
pub fn step(state: u64) -> (u64, i32) {
    let next = state.wrapping_mul(0x5dee_ce66d).wrapping_add(11);
    (next, ((next >> 16) & RAND_MAX as u64) as i32)
}

/// `srand(seed)`: install the new state. The C `unsigned seed` (32-bit)
/// is zero-extended into the 64-bit `next`.
#[inline]
pub fn seed(seed: u32) -> u64 {
    seed as u64
}
