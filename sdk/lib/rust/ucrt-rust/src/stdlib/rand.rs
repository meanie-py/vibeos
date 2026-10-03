//! `rand.cpp` (`srand`, `rand`) and `rand_s.cpp`.

use super::{invalid_einval, _errno, EINVAL, ENOMEM};
use crate::ptd::AcrtPtd;
use core::ffi::{c_int, c_uint, c_void};

/// `stdlib.h` `RAND_MAX`.
const CRT_RAND_MAX: c_uint = 0x7fff;

extern "C" {
    fn __acrt_getptd() -> *mut AcrtPtd;
}

extern "system" {
    /// `corecrt_internal.h`: `BOOLEAN WINAPI __acrt_RtlGenRandom(PVOID, ULONG)`
    /// (the `SystemFunction036` thunk in `internal/winapi_thunks.cpp`).
    fn __acrt_RtlGenRandom(buffer: *mut c_void, buffer_count: u32) -> u8;
}

/// `void srand(unsigned int)`: seeds THIS thread's state.
#[no_mangle]
pub unsafe extern "C" fn srand(seed: c_uint) {
    (*__acrt_getptd()).rand_state = seed;
}

/// `int rand()`: the per-thread LCG.
#[no_mangle]
pub unsafe extern "C" fn rand() -> c_int {
    let ptd = __acrt_getptd();
    (*ptd).rand_state = (*ptd).rand_state.wrapping_mul(214013).wrapping_add(2531011);
    (((*ptd).rand_state >> 16) & CRT_RAND_MAX) as c_int
}

/// `errno_t rand_s(unsigned int*)`: NULL is `_VALIDATE_RETURN_ERRCODE`
/// (EINVAL + handler, returns EINVAL); `*result` is zeroed BEFORE the RNG
/// call, and an RNG failure is ENOMEM (set and returned).
#[no_mangle]
pub unsafe extern "C" fn rand_s(result: *mut c_uint) -> c_int {
    if result.is_null() {
        invalid_einval();
        return EINVAL;
    }
    *result = 0;

    if __acrt_RtlGenRandom(result.cast::<c_void>(), core::mem::size_of::<c_uint>() as u32) == 0 {
        *_errno() = ENOMEM;
        return *_errno();
    }

    0
}
