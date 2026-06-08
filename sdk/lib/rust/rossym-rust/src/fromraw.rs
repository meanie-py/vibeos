//! [correctness-review 2026-06-07 · sonnet; FIXED 2026-06-07 · opus] Reviewed vs sdk/lib/rossym/fromraw.c.
//! Risk: MED → RESOLVED. FIXED: RosSymAllocMem is now a local wrapper fn (the rossympriv.h macro
//! expansion over RosSymCallbacks.AllocMemProc) — kills both the i386 `extern "system"` ABI bug and
//! the phantom-symbol linkage bug; return type i32 → u8 (BOOLEAN).
//! Behavior: logic correct; two ABI bugs will surface at integration. [original finding]
//! Functions reviewed: RosSymCreateFromRaw (1/1).
//! Findings:
//!   BUG ABI — `extern "system"` for `RosSymAllocMem` (line 7). rossym is a cdecl library
//!     (verified: `_RosSymDelete` has no `@N` decoration per the Cargo.toml comment).
//!     On i386, `extern "system"` = stdcall (_Name@N), so the call site emits
//!     `call _RosSymAllocMem@4` while any providing symbol is `_RosSymAllocMem` (cdecl).
//!     Undefined symbol on i386 link. On amd64/arm64 both resolve identically (no decoration),
//!     so the bug is i386-only but real. Fix: change to `extern "C"`.
//!   BUG LINKAGE — `extern "system" fn RosSymAllocMem` (like `RosSymFreeMem` in delete.rs).
//!     `RosSymAllocMem` is a C preprocessor macro in rossympriv.h:
//!     `#define RosSymAllocMem(Size) (*RosSymCallbacks.AllocMemProc)(Size)`.
//!     No exported C function with this name exists. Link error at integration.
//!     Fix: call `crate::init::RosSymCallbacks.AllocMemProc.unwrap()(size)` directly.
//!   BUG RETURN TYPE — Return type `i32` (line 12). C declares `BOOLEAN` (unsigned char, u8).
//!     Both use the same return register (AL within EAX/RAX), so the actual return value
//!     is observable by the caller, but the ABI contract is wrong. Fix: change to `-> u8`.
//!   NOTE — 4-condition ROSSYM_HEADER validation (lines 18-27) matches fromraw.c exactly,
//!     including the `DataSize < StringsOffset+StringsLength` check absent from fromfile.c.
//!   NOTE — Allocation size `sizeof(ROSSYM_INFO) + SymbolsLength + StringsLength + 1` matches C.
//!   NOTE — `copy_nonoverlapping` (memcpy) for Symbols and Strings matches C `memcpy` exactly.
//!   NOTE — Null-terminator write `Strings.add(StringsLength).write(0i8)` matches C correctly.
//!   NOTE — `DataSize: usize` in the Rust signature matches `ULONG_PTR DataSize` in C
//!     (ULONG_PTR = usize). This is CORRECT (contrast: frommem.rs's extern decl uses u32 wrong).
//! Unverified: Whether a cmake-level C shim will define `RosSymAllocMem` as a real function.

use crate::types::*;
use core::ffi::c_void;

// RosSymAllocMem is a function-like macro in rossympriv.h over the global RosSymCallbacks,
// not an exported symbol — translated as a local wrapper (the original "extern" decl was a
// mechanical-translation artifact, and "extern system" would also have been the wrong ABI on
// i386). Call site below is unchanged.
#[inline]
unsafe fn RosSymAllocMem(size: usize) -> *mut ROSSYM_INFO {
    (crate::init::RosSymCallbacks.AllocMemProc.unwrap())(size) as *mut ROSSYM_INFO
}

// ---- file-scope preamble (translated) ----
// No Rust declarations for #define NDEBUG (preprocessor symbol).
#[no_mangle]
pub unsafe extern "C" fn RosSymCreateFromRaw(
    RawData: *mut core::ffi::c_void,
    DataSize: usize,
    RosSymInfo: *mut *mut ROSSYM_INFO,
) -> u8 {
    let RosSymHeader: *mut ROSSYM_HEADER = RawData as *mut ROSSYM_HEADER;
    if ((*RosSymHeader).SymbolsOffset as usize) < (core::mem::size_of::<ROSSYM_HEADER>())
        || ((*RosSymHeader).StringsOffset as usize)
            < (((*RosSymHeader).SymbolsOffset + (*RosSymHeader).SymbolsLength) as usize)
        || DataSize
            < (((*RosSymHeader).StringsOffset + (*RosSymHeader).StringsLength) as usize)
        || ((*RosSymHeader).SymbolsLength as usize % core::mem::size_of::<ROSSYM_ENTRY>()) != 0
    {
        // DPRINT removed
        return 0;
    }
    *RosSymInfo = RosSymAllocMem(
        core::mem::size_of::<ROSSYM_INFO>()
            + (*RosSymHeader).SymbolsLength as usize
            + (*RosSymHeader).StringsLength as usize
            + 1,
    );
    if (*RosSymInfo).is_null() {
        // DPRINT removed
        return 0;
    }
    let info = *RosSymInfo;
    (*info).Symbols = (info as *mut u8).add(core::mem::size_of::<ROSSYM_INFO>())
        as *mut ROSSYM_ENTRY;
    (*info).SymbolsCount = (*RosSymHeader).SymbolsLength
        / core::mem::size_of::<ROSSYM_ENTRY>() as u32;
    (*info).Strings = (info as *mut u8)
        .add(core::mem::size_of::<ROSSYM_INFO>() + (*RosSymHeader).SymbolsLength as usize)
        as *mut i8;
    (*info).StringsLength = (*RosSymHeader).StringsLength;
    core::ptr::copy_nonoverlapping(
        (RosSymHeader as *mut u8).add((*RosSymHeader).SymbolsOffset as usize) as *const u8,
        (*info).Symbols as *mut u8,
        (*RosSymHeader).SymbolsLength as usize,
    );
    core::ptr::copy_nonoverlapping(
        (RosSymHeader as *mut u8).add((*RosSymHeader).StringsOffset as usize) as *const u8,
        (*info).Strings as *mut u8,
        (*RosSymHeader).StringsLength as usize,
    );
    (*info).Strings.add((*info).StringsLength as usize).write(0i8);
    return 1;
}
