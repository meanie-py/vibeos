//! Per-crate write barrier for lpc-rust's `pub static mut` globals — same purpose
//! as ntoskrnl-rust's `_global_write_barrier`: a visible, un-eliminable write site
//! so LLVM can't const-fold a global that's 0-initialized in Rust but written from
//! C or another crate. NEVER CALLED; `#[no_mangle]` + pulled into the link via
//! kernel-rust's `extern crate lpc_rust`.
//!
//! Hand-written for the crate-split PoC. When `tools/gen_global_barrier.py` is made
//! crate-aware this becomes generated output — see
//! docs/engineering/ntoskrnl-crate-split-plan.md.
#[allow(unused_unsafe)]
#[allow(dead_code)]
/// Write barrier — touches lpc-rust's pub static mut globals. NEVER CALLED.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _lpc_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    // port.rs
    write_volatile(addr_of_mut!(crate::port::LpcPortObjectType), read_volatile(addr_of!(crate::port::LpcPortObjectType)));
    write_volatile(addr_of_mut!(crate::port::LpcWaitablePortObjectType), read_volatile(addr_of!(crate::port::LpcWaitablePortObjectType)));
    write_volatile(addr_of_mut!(crate::port::LpcpMaxMessageSize), read_volatile(addr_of!(crate::port::LpcpMaxMessageSize)));
    write_volatile(addr_of_mut!(crate::port::LpcpMessagesLookaside), read_volatile(addr_of!(crate::port::LpcpMessagesLookaside)));
    write_volatile(addr_of_mut!(crate::port::LpcpLock), read_volatile(addr_of!(crate::port::LpcpLock)));
    write_volatile(addr_of_mut!(crate::port::LpcpTraceLevel), read_volatile(addr_of!(crate::port::LpcpTraceLevel)));
    write_volatile(addr_of_mut!(crate::port::LpcpNextMessageId), read_volatile(addr_of!(crate::port::LpcpNextMessageId)));
    write_volatile(addr_of_mut!(crate::port::LpcpNextCallbackId), read_volatile(addr_of!(crate::port::LpcpNextCallbackId)));
    write_volatile(addr_of_mut!(crate::port::LpcpPortMapping), read_volatile(addr_of!(crate::port::LpcpPortMapping)));
}
