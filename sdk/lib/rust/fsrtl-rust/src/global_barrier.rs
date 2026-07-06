//! Per-crate write barrier for fsrtl-rust's `pub static mut` globals — a visible,
//! un-eliminable write site so LLVM can't const-fold a global that's 0-initialized
//! in Rust but written from C or another crate. NEVER CALLED; `#[no_mangle]` +
//! pulled into the link via kernel-rust's `extern crate fsrtl_rust`.
//!
//! Hand-written for the crate-split. When `tools/gen_global_barrier.py` is made
//! crate-aware this becomes generated output — see
//! docs/engineering/ntoskrnl-crate-split-plan.md.
#[allow(unused_unsafe)]
#[allow(dead_code)]
/// Write barrier — touches fsrtl-rust's pub static mut globals. NEVER CALLED.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _fsrtl_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    write_volatile(addr_of_mut!(crate::filelock::FsRtlFileLockLookasideList), read_volatile(addr_of!(crate::filelock::FsRtlFileLockLookasideList)));
    write_volatile(addr_of_mut!(crate::unc::FsRtlpUncSemaphore), read_volatile(addr_of!(crate::unc::FsRtlpUncSemaphore)));
}
