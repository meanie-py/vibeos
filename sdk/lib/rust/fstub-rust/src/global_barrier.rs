//! Per-crate write barrier for fstub-rust's `pub static mut` globals — a visible,
//! un-eliminable write site so LLVM can't const-fold a global that's 0-initialized
//! in Rust but written from C or another crate. NEVER CALLED; `#[no_mangle]` +
//! pulled into the link via kernel-rust's `extern crate fstub_rust`. Relocated
//! from ntoskrnl-rust/src/global_barrier.rs during the crate-split (Phase 1b).
//!
//! Hand-written for the crate-split. When `tools/gen_global_barrier.py` is made
//! crate-aware this becomes generated output — see
//! docs/engineering/ntoskrnl-crate-split-plan.md.
//!
//! NOTE: fstub had ZERO entries in the monolith global_barrier.rs. Its two
//! `pub static mut` globals — `HalDispatchTable` (halstub.rs:586) and
//! `HalPrivateDispatchTable` (halstub.rs:617) — are FULLY struct-initialized (a
//! `HAL_DISPATCH { … }` / `HAL_PRIVATE_DISPATCH { … }` literal), NOT 0-init, so
//! they were never barrier candidates and none was ever emitted. This function is
//! therefore intentionally empty; kept for symmetry with the other extracted
//! crates and so a future 0-init fstub global has an obvious home.
#[allow(unused_unsafe)]
#[allow(dead_code)]
/// Write barrier — touches fstub-rust's pub static mut globals. NEVER CALLED.
/// Currently empty: fstub has no 0-initialized globals (both HAL dispatch tables
/// are fully struct-initialized, so LLVM has nothing to const-fold away).
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _fstub_global_write_barrier() {
    // No entries — see module doc. fstub's HalDispatchTable /
    // HalPrivateDispatchTable are fully-initialized literals, not barrier
    // candidates. Add write_volatile(addr_of_mut!(…)) pairs here if a future
    // fstub global becomes 0-initialized.
}
