//! lsass-rust — Rust translation of lsass.exe, the Local Security Authority
//! Subsystem host (87-LOC leaf). Successor to the smss/csrss/winsrv/userinit
//! user-mode spine flips. smss/wininit launches lsass.exe; it makes itself
//! process-critical, then inits the LSA server DLL (lsasrv), the SAM server
//! DLL (samsrv), and starts the security services.
//!
//! Per-binary cohabitation: lsass.c is commented out of
//! base/system/lsass/CMakeLists.txt once Rust parity is verified; the C stays
//! on disk (translation philosophy: never delete the C). Substrate is
//! ntdll-sys (Nt*/Rtl*); the three server-DLL exports resolve against the
//! lsasrv/samsrv import libs.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// lsass.c — wWinMain entry.
pub mod lsass;

// USER-MODE no_std staticlib → needs its own panic handler + eh_personality
// (same role as smss-csrss-rust / userinit-rust). Must NOT use privileged
// instructions; spin instead. lsass is process-critical, so reaching this is
// already a fatal-for-the-system condition.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
