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

// svcctl.idl + winsvc.h RPC marshalling types (consumed by rpcserver.rs).
pub mod svcctl;

// lock.c — ScmAcquire/Release/QueryServiceLockStatus{W,A}.
pub mod lock;

// controlset.c — Current/LKG control-set management + boot acceptance.
pub mod controlset;

// security.c — SCM security descriptors (SIDs/ACLs/SDs, pPipeSD).
pub mod security;

// groupdb.c — service-group database (first SERVICE-mirror consumer).
pub mod groupdb;

// driver.c — driver control interface (Nt{Load,Unload}Driver + status probe).
pub mod driver;

// config.c — service-config registry I/O + LSA password storage + decrypt.
pub mod config;

// database.c — SCM database core (service/image lists, process spawn, named-pipe
// IPC, control packets, autostart). Owns ServiceListHead/ImageListHead +
// Scm{Lock,Unlock}Database. Translated by rustconv (Opus 4.8).
pub mod database;

// rpcserver.c — the svcctl RPC server (R* interface handlers). Final SCM file.
// Translated by rustconv (Opus 4.8).
pub mod rpcserver;

// services.c — the SCM main controller (wWinMain entry + bootstrap). Owns the
// 4 cross-module SCM state flags. The 8th and final services.exe file flipped.
pub mod services;

// USER-MODE no_std staticlib → own panic handler + eh_personality (same role
// as winsrv-rust / smss-csrss-rust). No privileged instructions; spin.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
