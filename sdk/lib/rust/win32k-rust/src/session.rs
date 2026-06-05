//! Rust translation of `win32ss/user/ntuser/session.c` — USER session init.
//! [ntuser wave 1: opening the USER subsystem — atom-table pair with useratom.rs]
//!
//! Defines the global USER atom table `gAtomTable` (consumed by useratom.rs and
//! still-C class.c / cursoricon.c) and the one-line session initializer.

use ntkrnl_sys::ddk::{PRTL_ATOM_TABLE, RtlCreateAtomTable};

/// `PRTL_ATOM_TABLE gAtomTable` — the global USER atom table. `#[no_mangle]`
/// because class.c / cursoricon.c (still C) reference the symbol directly.
#[no_mangle]
pub static mut gAtomTable: PRTL_ATOM_TABLE = core::ptr::null_mut();

#[inline]
unsafe fn init_session_impl() -> i32 {
    RtlCreateAtomTable(37, core::ptr::addr_of_mut!(gAtomTable))
}

// InitSessionImpl is FASTCALL (`@InitSessionImpl@0`) — dual-cfg.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn InitSessionImpl() -> i32 {
    init_session_impl()
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn InitSessionImpl() -> i32 {
    init_session_impl()
}
