//! disk-rust — drivers/storage/class/disk → Rust (DeepSeek mechanical + c2rust).
//! SCSI disk class driver built on classpnp's Class* exports.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#![allow(unused_imports, unused_variables, unused_assignments, unused_mut)]

// crate-type = staticlib (cohab) → this PE module needs its own panic handler /
// eh personality (each .sys links independently of ntoskrnl). Matches classpnp-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

pub mod types;
pub mod data;
pub mod glue;
pub mod ddk_shims;

pub mod disk;
pub mod diskwmi;
pub mod geometry;
pub mod pnp;
