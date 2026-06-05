//! services-rust — Rust translation of services.exe, the Service Control
//! Manager (SCM). Next boot-spine subsystem after the smss/csrss/winsrv/lsass
//! flips: winlogon/smss launches services.exe, which starts the boot/system
//! drivers and the autostart services. On the path to `shell ready`.
//!
//! Flipped file-by-file (per-binary cohabitation, mirrors winsrv-rust): each
//! `.c` is commented out of base/system/services/CMakeLists.txt once Rust
//! parity is verified; the C stays on disk (translation philosophy). Substrate
//! is winsrv-sys (kernel32/advapi32 superset over ntdll-sys); services-private
//! and svcctl RPC types live in `types`.
//!
//! Wave 1: lock.c (the SCM start-serialization lock).

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

// services.h private types + svcctl RPC lock-status structs + extern Scm*
// helpers. Shared by all flipped modules.
pub mod types;

// lock.c — ScmAcquire/Release/QueryServiceLockStatus{W,A}.
pub mod lock;

// controlset.c — Current/LKG control-set management + boot acceptance.
pub mod controlset;

// USER-MODE no_std staticlib → own panic handler + eh_personality (same role
// as winsrv-rust / smss-csrss-rust). No privileged instructions; spin.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
