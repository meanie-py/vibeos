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

// ============================================================================
// Universal LIST_ENTRY inlines (winnt.h/ntddk FORCEINLINE macros — not
// bindgen-visible). Ported verbatim from ntkrnl-sys/src/base.rs; every
// user-mode component (csrsrv/smss/basesrv) iterates intrusive lists.
// Re-exported via `ntdll_sys::nt::*`.
// ============================================================================
use crate::nt::LIST_ENTRY;

#[inline]
pub unsafe fn InitializeListHead(head: *mut LIST_ENTRY) {
    (*head).Flink = head;
    (*head).Blink = head;
}
#[inline]
pub unsafe fn IsListEmpty(head: *const LIST_ENTRY) -> bool {
    (*head).Flink == head as *mut LIST_ENTRY
}
#[inline]
pub unsafe fn InsertHeadList(head: *mut LIST_ENTRY, entry: *mut LIST_ENTRY) {
    let flink = (*head).Flink;
    (*entry).Flink = flink;
    (*entry).Blink = head;
    (*flink).Blink = entry;
    (*head).Flink = entry;
}
#[inline]
pub unsafe fn InsertTailList(head: *mut LIST_ENTRY, entry: *mut LIST_ENTRY) {
    let blink = (*head).Blink;
    (*entry).Flink = head;
    (*entry).Blink = blink;
    (*blink).Flink = entry;
    (*head).Blink = entry;
}
#[inline]
pub unsafe fn RemoveEntryList(entry: *mut LIST_ENTRY) -> bool {
    let flink = (*entry).Flink;
    let blink = (*entry).Blink;
    (*blink).Flink = flink;
    (*flink).Blink = blink;
    flink == blink
}
#[inline]
pub unsafe fn RemoveHeadList(list_head: *mut LIST_ENTRY) -> *mut LIST_ENTRY {
    let entry = (*list_head).Flink;
    let flink = (*entry).Flink;
    (*list_head).Flink = flink;
    (*flink).Blink = list_head;
    entry
}
#[inline]
pub unsafe fn RemoveTailList(list_head: *mut LIST_ENTRY) -> *mut LIST_ENTRY {
    let entry = (*list_head).Blink;
    let blink = (*entry).Blink;
    (*list_head).Blink = blink;
    (*blink).Flink = list_head;
    entry
}

// ============================================================================
// ndk/rtlfuncs.h FORCEINLINE string initializers. bindgen declares them as externs, and
// no DLL exports them: autochk's two calls linked against nothing (undefined
// `_RtlInitEmptyUnicodeString` on i386).
// ============================================================================
use crate::nt::{PUNICODE_STRING, PWCHAR, USHORT};

/// `RtlInitEmptyUnicodeString(UnicodeString, Buffer, BufferSize)`: an empty string over
/// `Buffer`, `BufferSize` bytes long.
#[inline]
pub unsafe fn RtlInitEmptyUnicodeString(
    UnicodeString: PUNICODE_STRING,
    Buffer: PWCHAR,
    BufferSize: USHORT,
) {
    (*UnicodeString).Length = 0;
    (*UnicodeString).MaximumLength = BufferSize;
    (*UnicodeString).Buffer = Buffer;
}
