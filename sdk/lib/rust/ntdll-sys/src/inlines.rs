//! Hand-written user-mode FORCEINLINE accessors — the user-mode
//! analog of ntoskrnl-rust's `ke-inlines` (see memory
//! `feedback_ke_inlines_load_bearing`).
//!
//! `NtCurrentPeb`/`NtCurrentTeb`/`NtCurrentProcess`/`NtCurrentThread`
//! are FORCEINLINE macros in winnt.h/ntddk — they are NOT ntdll
//! exports, so they never appear in the bindgen surface. Translated
//! user-mode code (csrss.exe, …) calls them ubiquitously; rustconv
//! would otherwise mis-emit them as `extern "system" fn` (link
//! failure) plus a hand-rolled hardcoded-offset `struct PEB`
//! (arch-wrong). This module provides the correct inlines over the
//! genuine bindgen `_TEB`/`_PEB`, re-exported from `ntdll_sys::nt`.

use crate::nt::{HANDLE, PPEB, _TEB};

/// `NtCurrentTeb()` — the per-thread TEB self pointer.
///
/// Reads `NtTib.Self`: `fs:[0x18]` (i386), `gs:[0x30]` (amd64), or the
/// dedicated TEB register `x18` (ARM64 Windows). Stable for the
/// thread's lifetime, hence `pure`.
#[inline]
pub unsafe fn NtCurrentTeb() -> *mut _TEB {
    let teb: *mut _TEB;
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!(
        "mov {}, gs:[0x30]", out(reg) teb,
        options(nostack, pure, readonly, preserves_flags),
    );
    #[cfg(target_arch = "x86")]
    core::arch::asm!(
        "mov {}, fs:[0x18]", out(reg) teb,
        options(nostack, pure, readonly, preserves_flags),
    );
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!(
        "mov {}, x18", out(reg) teb,
        options(nostack, pure, nomem, preserves_flags),
    );
    teb
}

/// `NtCurrentPeb()` — `NtCurrentTeb()->ProcessEnvironmentBlock`.
#[inline]
pub unsafe fn NtCurrentPeb() -> PPEB {
    (*NtCurrentTeb()).ProcessEnvironmentBlock
}

/// `NtCurrentProcess()` — the current-process pseudo-handle
/// `(HANDLE)(LONG_PTR)-1`.
#[inline]
pub fn NtCurrentProcess() -> HANDLE {
    (-1isize) as HANDLE
}

/// `NtCurrentThread()` — the current-thread pseudo-handle
/// `(HANDLE)(LONG_PTR)-2`.
#[inline]
pub fn NtCurrentThread() -> HANDLE {
    (-2isize) as HANDLE
}

/// `ZwCurrentProcess()` / `ZwCurrentThread()` aliases (same values).
#[inline]
pub fn ZwCurrentProcess() -> HANDLE {
    NtCurrentProcess()
}
#[inline]
pub fn ZwCurrentThread() -> HANDLE {
    NtCurrentThread()
}
