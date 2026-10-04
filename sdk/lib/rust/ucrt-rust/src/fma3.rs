//! `_set_FMA3_enable` and the FMA3 detection ucrtbase's `.CRT$XIC` initializer runs
//! (`__acrt_initialize_fma3`, internal/init.rs): sdk/lib/crt's math/libm_sse2/
//! fma3_available.c (AMD, MIT) on amd64, and math/arm64/fma3_available.c's stubs on arm64,
//! which has no FMA3 to detect. i386 has neither (the export is `-arch=win64`).

use core::ffi::c_int;

#[no_mangle]
pub static mut __fma3_is_available: c_int = 0;
#[no_mangle]
pub static mut __use_fma3_lib: c_int = 0;

/// `_set_FMA3_enable`: honoured only when FMA3 was detected; returns the setting in force.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn _set_FMA3_enable(flag: c_int) -> c_int {
    if __fma3_is_available != 0 {
        __use_fma3_lib = flag;
    }
    __use_fma3_lib
}

/// CPUID leaf 1, ECX bit 12 (FMA).
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_fma3() -> c_int {
    let ecx = core::arch::x86_64::__cpuid(1).ecx;
    __fma3_is_available = (ecx & (1 << 12) != 0) as c_int;
    __use_fma3_lib = __fma3_is_available;
    0
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub extern "C" fn _set_FMA3_enable(_flag: c_int) -> c_int {
    0
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub extern "C" fn __acrt_initialize_fma3() -> c_int {
    0
}
