//! Small CRT helpers: pointer (de)obfuscation no-ops (mingw_helpers.c),
//! _matherr stub (_matherr.c), _get_output_format (misc/ofmt_stub.c),
//! __acrt_iob_func (stdio/acrt_iob_func.c), and the lconv-init `.CRT$XIC`
//! entry (charmax.c).

use crate::winapi::Pifv;
use core::ffi::c_void;

// startup/mingw_helpers.c — identity (pointer encoding disabled).
#[no_mangle]
pub unsafe extern "C" fn _decode_pointer(codedptr: *mut c_void) -> *mut c_void {
    codedptr
}
#[no_mangle]
pub unsafe extern "C" fn _encode_pointer(ptr: *mut c_void) -> *mut c_void {
    ptr
}

// startup/_matherr.c — dummy, like the MS CRT.
#[no_mangle]
pub unsafe extern "C" fn _matherr(_pexcept: *mut c_void) -> i32 {
    0
}

// misc/ofmt_stub.c
#[no_mangle]
pub unsafe extern "C" fn _get_output_format() -> i32 {
    0
}

// stdio/acrt_iob_func.c — classic msvcrt FILE (_iobuf) for the index
// arithmetic. Only pulled when a GCC libstdc++ object references it.
#[repr(C)]
pub struct FILE {
    _ptr: *mut u8,
    _cnt: i32,
    _base: *mut u8,
    _flag: i32,
    _file: i32,
    _charbuf: i32,
    _bufsiz: i32,
    _tmpfname: *mut u8,
}
extern "C" {
    fn __iob_func() -> *mut FILE;
}
#[no_mangle]
pub unsafe extern "C" fn __acrt_iob_func(index: i32) -> *mut FILE {
    __iob_func().add(index as usize)
}
// `const void* __imp___acrt_iob_func = __acrt_iob_func;` (i386 spelling
// carries the extra leading underscore → `__imp____acrt_iob_func`).
type AcrtIobFn = unsafe extern "C" fn(i32) -> *mut FILE;
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[used]
pub static __imp___acrt_iob_func: AcrtIobFn = __acrt_iob_func;
#[cfg(target_arch = "x86")]
#[no_mangle]
#[used]
pub static _imp____acrt_iob_func: AcrtIobFn = __acrt_iob_func;

// recalloc.c (upstream CORE-20596, added after this crate was translated): `_recalloc`
// for the `__declspec(dllimport) _recalloc` <malloc.h> gives every caller. XP's
// msvcrt.dll has no such export, so the body and the import-pointer variable live here,
// as in the C. Only pulled when a module references `_recalloc` (urlmon, d3dx9_*).
extern "C" {
    fn realloc(ptr: *mut core::ffi::c_void, size: usize) -> *mut core::ffi::c_void;
    fn _msize(ptr: *mut core::ffi::c_void) -> usize;
}
#[no_mangle]
pub unsafe extern "C" fn recalloc(mem: *mut core::ffi::c_void, num: usize, size: usize) -> *mut core::ffi::c_void {
    use crate::winapi::{_errno, calloc, memset};
    if mem.is_null() {
        return calloc(num, size);
    }
    let size = num.wrapping_mul(size); // the C multiplies unchecked, as here
    let old_size = _msize(mem);
    let ret = realloc(mem, size);
    if ret.is_null() {
        *_errno() = 12; // ENOMEM
        return core::ptr::null_mut();
    }
    if size > old_size {
        memset((ret as *mut u8).add(old_size) as *mut core::ffi::c_void, 0, size - old_size);
    }
    ret
}
type RecallocFn = unsafe extern "C" fn(*mut core::ffi::c_void, usize, usize) -> *mut core::ffi::c_void;
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[used]
pub static __imp__recalloc: RecallocFn = recalloc;
#[cfg(target_arch = "x86")]
#[no_mangle]
#[used]
pub static _imp___recalloc: RecallocFn = recalloc;

// startup/charmax.c — my_lconv_init runs from the `.CRT$XIC` init slot.
extern "C" {
    fn __lconv_init() -> i32;
}
unsafe extern "C" fn my_lconv_init() -> i32 {
    __lconv_init()
}
#[no_mangle]
#[link_section = ".CRT$XIC"]
#[used]
pub static __mingw_pinit: Pifv = Some(my_lconv_init);
