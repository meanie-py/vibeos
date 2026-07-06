//! Per-crate write barrier for dbgk-rust's `pub static mut` globals — a visible,
//! un-eliminable write site so LLVM can't const-fold a global that's 0-initialized
//! in Rust but written from C/another crate. NEVER CALLED; `#[no_mangle]` + pulled
//! into the link via kernel-rust's `extern crate dbgk_rust`. Hand-written for the
//! crate-split; see docs/engineering/ntoskrnl-crate-split-plan.md.
#[allow(unused_unsafe)]
#[allow(dead_code)]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _dbgk_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    write_volatile(addr_of_mut!(crate::dbgkobj::DbgkDebugObjectType), read_volatile(addr_of!(crate::dbgkobj::DbgkDebugObjectType)));
    write_volatile(addr_of_mut!(crate::dbgkobj::DbgkpProcessDebugPortMutex), read_volatile(addr_of!(crate::dbgkobj::DbgkpProcessDebugPortMutex)));
    write_volatile(addr_of_mut!(crate::dbgkobj::DbgkpTraceLevel), read_volatile(addr_of!(crate::dbgkobj::DbgkpTraceLevel)));
    write_volatile(addr_of_mut!(crate::dbgkobj::DbgkDebugObjectMapping), read_volatile(addr_of!(crate::dbgkobj::DbgkDebugObjectMapping)));
}
