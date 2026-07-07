//! Per-crate write barrier for config-rust's `pub static mut` globals — a visible,
//! un-eliminable write site so LLVM can't const-fold a global that's 0-initialized in
//! Rust but written from C or another crate. NEVER CALLED; `#[no_mangle]` + pulled
//! into the link via kernel-rust's `extern crate config_rust`. Relocated from
//! ntoskrnl-rust/src/global_barrier.rs during the crate-split (Phase 1b).
//!
//! 1 entry — IopAutoReboot (the only config static the monolith barrier covered; it
//! lives in cmdata.rs, written via the CmControlVector registry apply, read by
//! ke/bug.rs).
#[allow(unused_unsafe)]
#[allow(dead_code)]
/// Write barrier — touches config-rust's pub static mut globals. NEVER CALLED.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _config_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    write_volatile(addr_of_mut!(crate::cmdata::IopAutoReboot), read_volatile(addr_of!(crate::cmdata::IopAutoReboot)));
}
