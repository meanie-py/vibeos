//! stdlib/rand_s.c — rand_s over advapi32's SystemFunction036 (RtlGenRandom).

use crate::invalid_param::_invalid_parameter;
use crate::winapi::*;

const EINVAL: i32 = 22;

// BOOLEAN (WINAPI*)(PVOID, ULONG)
type PfnSystemFunction036 = unsafe extern "system" fn(PVOID, ULONG) -> u8;

#[no_mangle]
pub static mut g_pfnSystemFunction036: Option<PfnSystemFunction036> = None;

extern "C" {
    fn _errno() -> *mut i32;
}

#[no_mangle]
pub unsafe extern "C" fn rand_s(pval: *mut u32) -> i32 {
    if pval.is_null() {
        _invalid_parameter(
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null(),
            0,
            0,
        );
        *_errno() = EINVAL;
        return EINVAL;
    }

    *pval = 0;

    if (*(&raw const g_pfnSystemFunction036)).is_none() {
        let hadvapi32 = LoadLibraryA(b"advapi32.dll\0".as_ptr());
        if hadvapi32.is_null() {
            _invalid_parameter(
                core::ptr::null(),
                core::ptr::null(),
                core::ptr::null(),
                0,
                0,
            );
            *_errno() = EINVAL;
            return EINVAL;
        }
        let p = GetProcAddress(hadvapi32, b"SystemFunction036\0".as_ptr());
        if p.is_none() {
            _invalid_parameter(
                core::ptr::null(),
                core::ptr::null(),
                core::ptr::null(),
                0,
                0,
            );
            *_errno() = EINVAL;
            FreeLibrary(hadvapi32);
            return EINVAL;
        }
        g_pfnSystemFunction036 = Some(core::mem::transmute::<_, PfnSystemFunction036>(p));
    }

    let f = g_pfnSystemFunction036.unwrap();
    if f(pval as PVOID, core::mem::size_of::<u32>() as ULONG) == 0 {
        _invalid_parameter(
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null(),
            0,
            0,
        );
        *_errno() = EINVAL;
        return EINVAL;
    }

    0
}

// import alias so a GCC libstdc++ object can link against rand_s. Final
// symbol is `__imp_rand_s` on every arch.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[used]
pub static __imp_rand_s: unsafe extern "C" fn(*mut u32) -> i32 = rand_s;
#[cfg(target_arch = "x86")]
#[no_mangle]
#[used]
pub static _imp_rand_s: unsafe extern "C" fn(*mut u32) -> i32 = rand_s;
