//! [correctness-review 2026-06-07 · sonnet] Reviewed vs sdk/lib/rossym/initum.c.
//! Risk: HIGH. Behavior: 1 real bug — file I/O callbacks silently dropped.
//! Functions reviewed: RosSymAllocMemUM, RosSymFreeMemUM, RosSymInitUserMode (3/3).
//! Findings:
//!   BUG — ReadFileProc and SeekFileProc are set to None in the static KmCallbacks.
//!     C initum.c explicitly wires both to RosSymZwReadFile and RosSymZwSeekFile:
//!       `RosSymZwReadFile, RosSymZwSeekFile`
//!     (these are real functions in zwfile.c, accessible to user mode via NT syscalls).
//!     With None, any call to RosSymCreateFromFile in user-mode context will dereference a
//!     null function pointer through the RosSymReadFile/RosSymSeekFile macros → crash.
//!     Fix: wire ReadFileProc: Some(RosSymZwReadFile), SeekFileProc: Some(RosSymZwSeekFile)
//!     (import them from crate::zwfile).
//!   NOTE — MemGetProc: None. Correct; user-mode rossym doesn't use MemGet.
//!   NOTE — Local variable name `KmCallbacks` (copy of C's local name) is misleading in
//!     user-mode context. Cosmetic only.
//! Unverified: Whether user-mode rossym is currently exercised through RosSymCreateFromFile
//!   (if only RosSymCreateFromMem is used in practice, the None bug is latent rather than
//!   immediately crashing).
use crate::types::*;
use crate::init::RosSymInit;
use crate::zwfile::{RosSymZwReadFile, RosSymZwSeekFile};
use core::ffi::c_void;

extern "system" {
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
}

unsafe extern "C" fn RosSymAllocMemUM(Size: usize) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), 0, Size)
}

unsafe extern "C" fn RosSymFreeMemUM(Area: *mut core::ffi::c_void) {
    unsafe {
        HeapFree(GetProcessHeap(), 0, Area);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RosSymInitUserMode() {
    static mut KmCallbacks: ROSSYM_CALLBACKS = ROSSYM_CALLBACKS {
        AllocMemProc: Some(RosSymAllocMemUM),
        FreeMemProc: Some(RosSymFreeMemUM),
        ReadFileProc: Some(RosSymZwReadFile),   // BUG-FIX: was None; C wires RosSymZwReadFile
        SeekFileProc: Some(RosSymZwSeekFile),   // BUG-FIX: was None; C wires RosSymZwSeekFile
        MemGetProc: None,
    };
    RosSymInit(core::ptr::addr_of_mut!(KmCallbacks));
}
