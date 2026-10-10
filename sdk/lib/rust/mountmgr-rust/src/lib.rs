//! mountmgr-rust — drivers/storage/mountmgr → Rust (DeepSeek mechanical + c2rust).
//! The mount manager: drive letters, mount points, volume symlinks (MOUNTDEV/
//! MOUNTMGR ioctls). Boot-start WDM driver, DriverEntry-only cohab.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#![allow(unused_imports, unused_variables, unused_assignments, unused_mut)]

// UNREFERENCED_PARAMETER is a macro in the DDK headers; an out-of-line no-op fn is an
// optimization barrier that can uncloak an ABI mismatch.
#[allow(unused_macros)]
macro_rules! UNREFERENCED_PARAMETER {
    ($($p:expr),* $(,)?) => {{ $( let _ = $p; )* }};
}

// crate-type = staticlib (cohab) → this PE module needs its own panic handler /
// eh personality (each .sys links independently of ntoskrnl). Matches disk-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

pub mod types;
pub mod glue;
pub mod ddk_shims;

// Translated modules (one per C source).
pub mod database;
pub mod device;
pub mod mountmgr;
pub mod notify;
pub mod point;
pub mod symlink;
pub mod uniqueid;
