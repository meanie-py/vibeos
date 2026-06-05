//! Loader-local symbol providers for the shared `cmboot-rust` crate.
//!
//! `ntoskrnl/config/cmboot.c` is "shared by both the kernel and the bootloader"
//! and reaches three symbols that the *consumer* must supply (cmboot-rust only
//! imports them):
//!
//!   * `InitSafeBootMode` — in the kernel this is a real global owned by
//!     ex/init; in the bootloader, `cmboot.c` itself defined the local
//!     `ULONG InitSafeBootMode = 0;` (its `#ifdef _BLDR_` block). Now that the
//!     loaders use cmboot-rust instead of the C, that loader-local global lives
//!     here.
//!   * `CMTRACE` — `cmboot.c` `#define`d it to `TRACE(...)` (DPRINT). The kernel
//!     resolves the symbol via config/inlines.rs; the loader had no symbol (it
//!     was a macro). We provide a no-op (FreeLDR drops the registry trace).
//!   * `IS_NULL_TERMINATED` — also a `#define` in `cmboot.c`. The kernel's
//!     inlines.rs stubs it to `1`; the loader used the real macro, so we
//!     implement the real check here:
//!     `(Size >= sizeof(WCHAR)) && (Buffer[Size/sizeof(WCHAR) - 1] == UNICODE_NULL)`.
//!
//! Only built for the loader archives that actually link cmboot-rust
//! (uefildr.efi via `uefi`, rosload.exe via `rosload`); the kernel never sees
//! this module.

use core::ffi::c_void;

// HACK (verbatim from cmboot.c): non-NT-compatible SafeBoot support. The C
// `ULONG InitSafeBootMode = 0;`. cmboot-rust reads it via `extern static`.
#[no_mangle]
pub static mut InitSafeBootMode: u32 = 0;

// `#define CMTRACE(x, fmt, ...) TRACE(fmt, ...)` — no-op in the loader. Signature
// matches the kernel's config/inlines.rs provider (4 fixed cdecl args); cmboot's
// single call site passes exactly four, so this is call-compatible with cmboot's
// variadic `extern` declaration on every arch.
#[no_mangle]
pub unsafe extern "C" fn CMTRACE(
    _filter: u32,
    _format: *const i8,
    _arg1: *const i8,
    _arg2: *mut c_void,
) {
}

// `#define IS_NULL_TERMINATED(Buffer, Size) \
//     (((Size) >= sizeof(WCHAR)) && ((Buffer)[(Size)/sizeof(WCHAR) - 1] == UNICODE_NULL))`
#[no_mangle]
pub unsafe extern "C" fn IS_NULL_TERMINATED(Buffer: *const u16, Size: u32) -> u8 {
    let wchar = core::mem::size_of::<u16>() as u32; // sizeof(WCHAR) == 2
    if Size >= wchar {
        let idx = (Size / wchar - 1) as usize;
        if *Buffer.add(idx) == 0 {
            return 1;
        }
    }
    0
}
