//! classpnp-rust — drivers/storage/class/classpnp → Rust (DeepSeek+deepdrive).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#![allow(unused_imports, unused_variables, unused_assignments, unused_mut)]

// crate-type = staticlib (cohab) → this PE module needs its own panic handler /
// eh personality (each .sys links independently of ntoskrnl). Matches wmilib-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
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
