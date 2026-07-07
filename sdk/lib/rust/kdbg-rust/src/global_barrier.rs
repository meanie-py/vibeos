//! Per-crate write barrier for kdbg-rust's `pub static mut` globals — a visible,
//! un-eliminable write site so LLVM can't const-fold a global that's 0-initialized
//! in Rust but written from C or another crate. NEVER CALLED; `#[no_mangle]` +
//! pulled into the link via kernel-rust's `extern crate kdbg_rust`. Relocated from
//! ntoskrnl-rust/src/global_barrier.rs during the crate-split (Phase 1b).
//!
//! x86-ONLY: the whole crate is gated `#![cfg(any(target_arch="x86","x86_64"))]` in
//! lib.rs, so this module (and every symbol it names) simply does not exist on arm64
//! — no separate arm64 guard is needed here. The barrier is only meaningful on x86,
//! where these statics live.
//!
//! Hand-written for the crate-split. When `tools/gen_global_barrier.py` is made
//! crate-aware this becomes generated output — see
//! docs/engineering/ntoskrnl-crate-split-plan.md.
//!
//! COVERAGE: 25 entries. The recipe's §6 lists 27 barrier CANDIDATES; two are
//! deliberately NOT included:
//!   - `KdbEnteredOnSingleStep` (kdb.rs:151) is a PRIVATE `static mut` (not `pub`,
//!     not `#[no_mangle]`) — unreachable from this module (privacy) AND fully
//!     observed by LLVM within the crate (all writes are in kdb.rs), so it neither
//!     can be nor needs to be barrier-protected.
//!   - `KdpDmesgBufferSize` (tables.rs:367) is an IMMUTABLE `pub static` (= 128*1024)
//!     — a genuine compile-time constant that is never written; `addr_of_mut!` is
//!     illegal on it and there is no folding hazard.
#[allow(unused_unsafe)]
#[allow(dead_code)]
/// Write barrier — touches kdbg-rust's pub static mut globals. NEVER CALLED.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _kdbg_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    // kdb.rs (8)
    write_volatile(addr_of_mut!(crate::kdb::KdbLastBreakPointNr), read_volatile(addr_of!(crate::kdb::KdbLastBreakPointNr)));
    write_volatile(addr_of_mut!(crate::kdb::KdbNumSingleSteps), read_volatile(addr_of!(crate::kdb::KdbNumSingleSteps)));
    write_volatile(addr_of_mut!(crate::kdb::KdbSingleStepOver), read_volatile(addr_of!(crate::kdb::KdbSingleStepOver)));
    write_volatile(addr_of_mut!(crate::kdb::KdbCurrentProcess), read_volatile(addr_of!(crate::kdb::KdbCurrentProcess)));
    write_volatile(addr_of_mut!(crate::kdb::KdbOriginalProcess), read_volatile(addr_of!(crate::kdb::KdbOriginalProcess)));
    write_volatile(addr_of_mut!(crate::kdb::KdbCurrentThread), read_volatile(addr_of!(crate::kdb::KdbCurrentThread)));
    write_volatile(addr_of_mut!(crate::kdb::KdbOriginalThread), read_volatile(addr_of!(crate::kdb::KdbOriginalThread)));
    write_volatile(addr_of_mut!(crate::kdb::KdbCurrentTrapFrame), read_volatile(addr_of!(crate::kdb::KdbCurrentTrapFrame)));
    // tables.rs (17)
    write_volatile(addr_of_mut!(crate::tables::ComponentTable), read_volatile(addr_of!(crate::tables::ComponentTable)));
    write_volatile(addr_of_mut!(crate::tables::KdbDebuggerCommands), read_volatile(addr_of!(crate::tables::KdbDebuggerCommands)));
    write_volatile(addr_of_mut!(crate::tables::KdbCliCallbacks), read_volatile(addr_of!(crate::tables::KdbCliCallbacks)));
    write_volatile(addr_of_mut!(crate::tables::KdbUseIntelSyntax), read_volatile(addr_of!(crate::tables::KdbUseIntelSyntax)));
    write_volatile(addr_of_mut!(crate::tables::KdbBreakOnModuleLoad), read_volatile(addr_of!(crate::tables::KdbBreakOnModuleLoad)));
    write_volatile(addr_of_mut!(crate::tables::KdbNumberOfRowsPrinted), read_volatile(addr_of!(crate::tables::KdbNumberOfRowsPrinted)));
    write_volatile(addr_of_mut!(crate::tables::KdbNumberOfColsPrinted), read_volatile(addr_of!(crate::tables::KdbNumberOfColsPrinted)));
    write_volatile(addr_of_mut!(crate::tables::KdbOutputAborted), read_volatile(addr_of!(crate::tables::KdbOutputAborted)));
    write_volatile(addr_of_mut!(crate::tables::KdbRepeatLastCommand), read_volatile(addr_of!(crate::tables::KdbRepeatLastCommand)));
    write_volatile(addr_of_mut!(crate::tables::KdbInitFileBuffer), read_volatile(addr_of!(crate::tables::KdbInitFileBuffer)));
    write_volatile(addr_of_mut!(crate::tables::KdbpBugCheckRequested), read_volatile(addr_of!(crate::tables::KdbpBugCheckRequested)));
    write_volatile(addr_of_mut!(crate::tables::KdpDmesgBuffer), read_volatile(addr_of!(crate::tables::KdpDmesgBuffer)));
    write_volatile(addr_of_mut!(crate::tables::KdpDmesgCurrentPosition), read_volatile(addr_of!(crate::tables::KdpDmesgCurrentPosition)));
    write_volatile(addr_of_mut!(crate::tables::KdpDmesgFreeBytes), read_volatile(addr_of!(crate::tables::KdpDmesgFreeBytes)));
    write_volatile(addr_of_mut!(crate::tables::KdbDmesgTotalWritten), read_volatile(addr_of!(crate::tables::KdbDmesgTotalWritten)));
    write_volatile(addr_of_mut!(crate::tables::KdbpIsInDmesgMode), read_volatile(addr_of!(crate::tables::KdbpIsInDmesgMode)));
    write_volatile(addr_of_mut!(crate::tables::KdpDmesgLogSpinLock), read_volatile(addr_of!(crate::tables::KdpDmesgLogSpinLock)));
}
