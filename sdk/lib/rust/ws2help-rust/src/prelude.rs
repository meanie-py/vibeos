//! ws2help-rust prelude — the single import surface the translated modules use.
//!
//! Same role as ws2_32-rust/wshtcpip-rust's prelude: bridges the gaps between the bindgen
//! `types` island and the translated C:
//!   1. bindgen RUSTIFIED the C enums — alias the bare names the C source actually uses.
//!   2. bindgen SKIPS cast-#defines / the precomp workaround defines (SO_OPENTYPE,
//!      IOCTL_WS2IFSL_SET_HANDLE) — transcribe them with a header/source citation.
//!   3. C MACROS/inlines (NT_SUCCESS, RtlZeroMemory, WS2HELP_PROLOG) have NO linkable symbol —
//!      implement them here (never as an out-of-line no-op `extern fn`, field-manual §9.65).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
use core::ffi::c_void;
use core::sync::atomic::{AtomicI32, AtomicPtr, Ordering};

// ---- 1. Constants bindgen skips (cast-#defines / precomp workaround defines) ----

// precomp.h "Missing definition" block: `#define SO_OPENTYPE 0x20`
// (dll/win32/ws2help/precomp.h). getsockopt optname; cast to c_int at the call site.
pub const SO_OPENTYPE: DWORD = 0x20;

// winsock2.h:236 `#define INVALID_SOCKET (SOCKET)(~0)`; SOCKET = UINT_PTR, so all-bits-set.
pub const INVALID_SOCKET: SOCKET = !0;

// winnt_old.h:287 `FILE_ALL_ACCESS = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x1FF`, i.e.
// 0x000F0000 (STANDARD_RIGHTS_REQUIRED, setypes.h:19) | 0x00100000 (SYNCHRONIZE, setypes.h:18)
// | 0x1FF = 0x001F01FF. ACCESS_MASK = DWORD.
pub const FILE_ALL_ACCESS: ACCESS_MASK = 0x001F_01FF;

// winreg.h:12 `#define HKEY_LOCAL_MACHINE ((HKEY)(LONG_PTR)(LONG)0x80000002)`. The (LONG)
// truncates to i32 (0x80000002 -> negative), then (LONG_PTR) sign-extends before the pointer
// cast — hence u32 -> i32 -> isize -> ptr (matters on 64-bit: 0xFFFFFFFF80000002).
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;

// ---- 2. C macros / DDK inlines with no linkable symbol — reimplemented inline, never as an
// out-of-line no-op `extern fn` (field-manual §9.65). ----

// precomp.h: `#define WS2HELP_PROLOG() (Ws2helpInitialized ? ERROR_SUCCESS : Ws2helpInitialize())`.
// The globals/fn it needs live in dllmain (the module that owns the init state).
#[inline(always)]
pub unsafe fn WS2HELP_PROLOG() -> DWORD {
    if crate::ws2help::dllmain::Ws2helpInitialized != 0 {
        ERROR_SUCCESS
    } else {
        crate::ws2help::dllmain::Ws2helpInitialize()
    }
}

// NT_SUCCESS: DDK macro `((NTSTATUS)(x) >= 0)`.
#[inline(always)]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// RtlZeroMemory / RtlCopyMemory: DDK write_bytes / memmove macros; `len` is a byte count.
#[inline(always)]
pub unsafe fn RtlZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}
#[inline(always)]
pub unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy(src as *const u8, dst as *mut u8, len);
}

// ---- 3. Interlocked* — compiler intrinsics (`_Interlocked*`), no link symbol. Same impl as
// ws2_32-rust's prelude: the +1/-1 restores the C "value AFTER the op" contract, since
// fetch_add/fetch_sub return the value BEFORE. Getting that wrong off-by-ones every refcount. ----
#[inline(always)]
pub unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (*(addend as *const AtomicI32)).fetch_add(1, Ordering::SeqCst) + 1
}
#[inline(always)]
pub unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (*(addend as *const AtomicI32)).fetch_sub(1, Ordering::SeqCst) - 1
}
#[inline(always)]
pub unsafe fn InterlockedCompareExchange(dest: *mut LONG, exchange: LONG, comparand: LONG) -> LONG {
    match (*(dest as *const AtomicI32)).compare_exchange(
        comparand,
        exchange,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(prev) | Err(prev) => prev,
    }
}
#[inline(always)]
pub unsafe fn InterlockedCompareExchangePointer(
    dest: *mut *mut c_void,
    exchange: *mut c_void,
    comparand: *mut c_void,
) -> *mut c_void {
    let a = &*(dest as *const AtomicPtr<c_void>);
    match a.compare_exchange(comparand, exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(prev) | Err(prev) => prev,
    }
}
