//! [correctness-review 2026-06-07 · sonnet] Reviewed vs sdk/lib/rossym/initkm.c.
//! Risk: MED. Behavior: 2 issues found.
//! Functions reviewed: RosSymAllocMemKM, RosSymFreeMemKM, RosSymInitKernelMode (3/3).
//! Findings:
//!   BUG — TAG_ROSSYM constant value wrong. C: `'MYSR'` = 0x4D595352 (multichar literal,
//!     most-significant byte = 'M'). Rust: 0x5253594D. These are NOT equal (0x4D595352 ≠
//!     0x5253594D). ExAllocatePoolWithTag is called with the wrong tag value. Pool allocations
//!     from rossym will show under tag 'MYSR' in debuggers (Rust value, stored as bytes
//!     4D 59 53 52) vs the C tag (stored as bytes 52 53 59 4D = 'RSYM'). The allocation
//!     itself succeeds, but pool tag mismatches confuse kernel debugger pool inspection.
//!     Fix: `pub const TAG_ROSSYM: u32 = 0x4D595352;`
//!   NOTE — `extern "system" { fn ExFreePool(...) }` is a redundant redeclaration alongside
//!     `use ntkrnl_sys::ex::*` which already imports ExFreePool as `extern "stdcall"`.
//!     On all architectures "system" and "stdcall" resolve identically, so no functional
//!     bug, but the duplicate can be removed.
//!   NOTE — MemGetProc: None. C has only 4 fields in the static KmCallbacks initializer
//!     (AllocMem, FreeMem, ReadFile, SeekFile). MemGetProc=None is correct: kernel-mode
//!     rossym doesn't use MemGet.
//! Unverified: whether ntkrnl_sys::ex::ExAllocatePoolWithTag and the local ExFreePool
//!   will collide at link time if both are emitted into the same object.
use crate::types::*;
use crate::init::RosSymInit;
use crate::zwfile::{RosSymZwReadFile, RosSymZwSeekFile};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
// (all imports from ddk moved to local definitions)
use core::ffi::c_void;

extern "system" {
    fn ExFreePool(P: *mut core::ffi::c_void);
}

// ---- file-scope preamble (translated) ----
pub const TAG_ROSSYM: u32 = 0x4D595352; // 'MYSR' C multichar literal: 'M'<<24|'Y'<<16|'S'<<8|'R'

unsafe extern "C" fn RosSymAllocMemKM(Size: usize) -> *mut core::ffi::c_void {
    ExAllocatePoolWithTag(NonPagedPool, Size, TAG_ROSSYM)
}

unsafe extern "C" fn RosSymFreeMemKM(Area: *mut core::ffi::c_void) {
    ExFreePool(Area);
}
#[no_mangle]
pub unsafe extern "C" fn RosSymInitKernelMode() {
    static mut KmCallbacks: ROSSYM_CALLBACKS = ROSSYM_CALLBACKS {
        AllocMemProc: Some(RosSymAllocMemKM),
        FreeMemProc: Some(RosSymFreeMemKM),
        ReadFileProc: Some(RosSymZwReadFile),
        SeekFileProc: Some(RosSymZwSeekFile),
        MemGetProc: None,
    };
    RosSymInit(core::ptr::addr_of_mut!(KmCallbacks));
}
