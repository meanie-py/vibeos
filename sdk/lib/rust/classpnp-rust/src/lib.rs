//! classpnp-rust — drivers/storage/class/classpnp → Rust (DeepSeek+deepdrive).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#![allow(unused_imports, unused_variables, unused_assignments, unused_mut)]


// PAGED_CODE() is the DDK's "pageable code" IRQL assertion (xdk/kefuncs.h): NOP_FUNCTION in
// a free build, and a no-op here. It is a macro, never a fn -- an out-of-line no-op call is an
// optimization barrier, and an `extern` one resolved by whichever crate happened to export it.
macro_rules! PAGED_CODE {
    () => {};
}

// DBGLOGFLUSHINFO — debug macro that discards arguments in the C free build. A macro, never an
// out-of-line fn (an out-of-line no-op is an optimization barrier).
#[macro_export]
macro_rules! DBGLOGFLUSHINFO {
    ($($arg:tt)*) => {};
}

// Fold-proof trace shims (TracePrint/StorDebugPrint) — see drvtrace-shim/src/lib.rs.
extern crate drvtrace_shim;

// crate-type = staticlib (cohab) → this PE module needs its own panic handler /
// eh personality (each .sys links independently of ntoskrnl). Matches wmilib-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[macro_use]
mod macros;

pub mod types;
pub mod glue;
pub mod shims;
pub mod data;
pub mod guid;
pub mod globals;

pub mod autorun;
pub mod class;
pub mod class_part_b;
pub mod class_part_c;
pub mod class_part_d;
pub mod class_part_e;
pub mod ddk_shims;
pub mod residual_part_b;
pub mod classwmi;
pub mod clntirp;
pub mod create;
pub mod debug;
pub mod dictlib;
pub mod dispatch;
pub mod history;
pub mod lock;
pub mod obsolete;
pub mod power;
pub mod retry;
pub mod srblib;
pub mod utils;
pub mod xferpkt;
