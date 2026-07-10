#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(clippy::all)]

//! mup.sys — Rust translation of the Multiple UNC Provider driver
//! (drivers/filesystems/mup). staticlib -> libmup_rust.a, cohab-linked into the cmake
//! mup.sys with mup.c/dfs.c commented out. NOT boot-loaded (not in txtsetup) → the bar
//! is link-clean ×3 + no smoke regression. WDM, imports ntoskrnl + hal only.
//!
//! Hand-translated faithfully from the C: constant values from the SDK headers, union
//! field paths from the ntkrnl-sys bindgen substrate. PSEH2 SEH is dead under
//! USE_DUMMY_PSEH (see helpers::seh notes in mup.rs).

pub mod glue;
pub mod helpers;
pub mod types;
pub mod dfs;
pub mod mup;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
