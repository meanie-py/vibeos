//! `sdk/lib/ucrt_reactos/` (ReactOS's own glue beside the Microsoft UCRT), ported:
//! `math/matherr.cpp` (Timo Kreuzer, MIT) -- the user `_matherr` hook -- and the
//! three `locale/*.c` forwarders to kernel32. matherr.cpp was the last file in
//! the tree reaching into Microsoft's internal headers (`corecrt_internal.h`),
//! for nothing more than a `dual_state_global` and the pointer-encoding helpers;
//! with the forwarders here too, the `ucrt` CMake target has nothing left to build.
//!
//! Details that are silent if wrong:
//! * The hook is stored ENCODED (`__crt_fast_encode_pointer`, XOR with
//!   `__security_cookie`, as in `invalid_parameter.rs`), and
//!   `__acrt_initialize_user_matherr` is handed the encoded null by
//!   `internal/init.rs` -- a plain 0 would decode to the cookie and be called.
//! * No global state isolation in this build, so the dual-state global is one
//!   value.

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::sync::atomic::{AtomicUsize, Ordering};

/// `struct _exception` (math.h).
#[repr(C)]
pub struct Exception {
    pub type_: c_int,
    pub name: *mut c_char,
    pub arg1: f64,
    pub arg2: f64,
    pub retval: f64,
}

pub type UserMathErrorFunctionPointer = Option<unsafe extern "C" fn(*mut Exception) -> c_int>;

extern "C" {
    static mut __security_cookie: usize;
}

static ACRT_GLOBAL_USER_MATHERR: AtomicUsize = AtomicUsize::new(0);

unsafe fn decoded() -> UserMathErrorFunctionPointer {
    let raw = ACRT_GLOBAL_USER_MATHERR.load(Ordering::Relaxed) ^ __security_cookie;
    core::mem::transmute::<usize, UserMathErrorFunctionPointer>(raw)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_initialize_user_matherr(encoded_null: *mut c_void) {
    ACRT_GLOBAL_USER_MATHERR.store(encoded_null as usize, Ordering::Relaxed);
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_has_user_matherr() -> bool {
    decoded().is_some()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_invoke_user_matherr(exp: *mut Exception) -> c_int {
    match decoded() {
        Some(f) => f(exp),
        None => 0,
    }
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __setusermatherr(f: UserMathErrorFunctionPointer) {
    let raw = f.map_or(0, |f| f as usize);
    ACRT_GLOBAL_USER_MATHERR.store(raw ^ __security_cookie, Ordering::Relaxed);
}

/// Called by libm on a domain/range error.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _invoke_matherr(type_: c_int, name: *mut c_char, arg1: f64, arg2: f64, retval: f64) -> c_int {
    let mut excpt = Exception { type_, name, arg1, arg2, retval };
    __acrt_invoke_user_matherr(&mut excpt)
}

// ── locale/*.c: cdecl forwarders to the kernel32 APIs ───────────────────────

extern "system" {
    fn MultiByteToWideChar(
        cp: c_uint,
        flags: u32,
        src: *const c_char,
        src_len: c_int,
        dst: *mut u16,
        dst_len: c_int,
    ) -> c_int;
    fn WideCharToMultiByte(
        cp: c_uint,
        flags: u32,
        src: *const u16,
        src_len: c_int,
        dst: *mut c_char,
        dst_len: c_int,
        default_char: *const c_char,
        used_default: *mut c_int,
    ) -> c_int;
    fn GetStringTypeW(info_type: u32, src: *const u16, cch_src: c_int, char_type: *mut u16) -> c_int;
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_MultiByteToWideChar(
    cp: c_uint,
    flags: u32,
    src: *const c_char,
    src_len: c_int,
    dst: *mut u16,
    dst_len: c_int,
) -> c_int {
    MultiByteToWideChar(cp, flags, src, src_len, dst, dst_len)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_WideCharToMultiByte(
    cp: c_uint,
    flags: u32,
    src: *const u16,
    src_len: c_int,
    dst: *mut c_char,
    dst_len: c_int,
    default_char: *const c_char,
    used_default: *mut c_int,
) -> c_int {
    WideCharToMultiByte(cp, flags, src, src_len, dst, dst_len, default_char, used_default)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_GetStringTypeW(info_type: u32, src: *const u16, cch_src: c_int, char_type: *mut u16) -> c_int {
    GetStringTypeW(info_type, src, cch_src, char_type)
}

// sdk/lib/vcruntime/i386/_chkesp.s: MSVC debug code calls `_chkesp` with ZF from comparing
// esp before and after a call; a mismatch tail-jumps to `_chkesp_failed`.
#[cfg(all(target_arch = "x86", target_os = "windows"))]
core::arch::global_asm!(
    ".text",
    ".globl __chkesp",
    "__chkesp:",
    "    jnz .Lchkesp_failed",
    "    ret",
    ".Lchkesp_failed:",
    "    jmp __chkesp_failed",
);

// dll/win32/ucrtbase/stubs.c, the rest of it (`__acrt_initialize_sse2` is internal/init.rs).

/// `atexit`: libsupc++ (linked into ucrtbase for `__cxa_guard_*`) calls it; UCRT's own
/// headers inline it over `_crt_atexit`, so ucrtbase defines no export by this name.
#[cfg(feature = "startup")]
#[no_mangle]
pub unsafe extern "C" fn atexit(function: Option<unsafe extern "C" fn()>) -> core::ffi::c_int {
    crate::startup::onexit::_crt_atexit(function)
}

extern "C" {
    fn malloc(size: usize) -> *mut core::ffi::c_void;
    fn free(mem: *mut core::ffi::c_void);
}

/// `operator_new` / `operator_delete`: the C names Wine's cpp.c (msvcrt_shared) allocates
/// exception objects through.
#[no_mangle]
pub unsafe extern "C" fn operator_new(size: usize) -> *mut core::ffi::c_void {
    malloc(size)
}
#[no_mangle]
pub unsafe extern "C" fn operator_delete(mem: *mut core::ffi::c_void) {
    free(mem)
}

/// `_chkesp_failed` (i386): the `_chkesp` mismatch path above jumps here; break into the
/// debugger, as stubs.c's `__debugbreak()` did.
#[cfg(all(target_arch = "x86", target_os = "windows"))]
#[no_mangle]
pub unsafe extern "C" fn _chkesp_failed() {
    core::arch::asm!("int3", options(nomem, nostack));
}

/// `_local_unwind` on arm64. ucrtbase.spec forwarded it to `ntdll._local_unwind` for all of
/// win64, but ntdll exports that name on x86_64 only, so the arm64 export named nothing
/// (arm64 ntdll's export table has no `_local_unwind`). The body is Wine's x86_64 one, as msvcrt.dll's arm64
/// `_local_unwind` (crt-shared-rust except_arm64.rs); amd64 keeps the forward.
#[cfg(all(target_arch = "aarch64", target_os = "windows"))]
#[no_mangle]
pub unsafe extern "C" fn _local_unwind(frame: *mut c_void, target: *mut c_void) {
    extern "system" {
        fn RtlUnwind(frame: *mut c_void, target: *mut c_void, rec: *mut c_void, retval: *mut c_void);
    }
    RtlUnwind(frame, target, core::ptr::null_mut(), core::ptr::null_mut());
}
