#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_macros)]
#![allow(static_mut_refs)]
#![allow(private_interfaces)]
#![allow(clippy::all)]

//! menuband-rust — the CMenuBand shell menu-band class (dll/win32/shell32/shellmenu/
//! CMenuBand.cpp), translated to Rust and built as a standalone staticlib
//! (libmenuband_rust.a) linked into BOTH shell32.dll and rshell.dll. See Cargo.toml for
//! why this is a separate crate rather than a shell32-rust module.

// Wine debug calls are macros in the C++ source. Keep them as macros in Rust too: an
// out-of-line no-op function is an optimization/ABI barrier and is not equivalent.
macro_rules! TRACE {
    ($($arg:tt)*) => {{}};
}
macro_rules! WARN {
    ($($arg:tt)*) => {{}};
}
macro_rules! ERR {
    ($($arg:tt)*) => {{}};
}
macro_rules! FIXME {
    ($($arg:tt)*) => {{}};
}
macro_rules! MESSAGE {
    ($($arg:tt)*) => {{}};
}

// Shared shell/COM type definitions — path-included from shell32-rust (pure #[repr(C)]
// layout, no code), giving this crate its own layout-identical copy without depending on
// the shell32-rust staticlib.
#[path = "../../shell32-rust/src/types.rs"]
pub mod types;
#[path = "../../shell32-rust/src/shellcom.rs"]
pub mod shellcom;

pub mod menu_band;
pub mod menu_desk_bar;
pub mod menu_site;
pub mod merged_folder;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

// Every vibeos crate compiles panic="abort", so the personality routine is never invoked;
// but rustc still references it from the (dead) unwind tables, so lld demands the symbol in
// any DLL that links this staticlib standalone (rshell has no other provider). Stub it, like
// the other standalone staticlibs (user32-rust, atapi-rust, …). `_Unwind_Resume` comes from
// panic-common.
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
