//! aclayers — Rust translation of dll/appcompat/shims/layer.
//! staticlib → libaclayers_rust.a, cohab-linked into the cmake-built aclayers
//! (usermode module type `win32dll`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 7 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh aclayers)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs
#[macro_use]
pub mod shim; // shim template framework: setup_shim.inl / implement_shim.inl ported as macros

#[allow(clippy::module_inception)]
pub mod aclayers {
    pub mod dispmode;
    pub mod forcedxsetupsuccess;
    pub mod globalmemorystatus;
    pub mod ignoreloadlibrary;
    pub mod versionlie;
    pub mod vmhorizon;
    pub mod main;
}

// staticlib needs a panic handler + eh personality (no_std cohab); std supplies them under test.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

/// What the C stub (dll/appcompat/shims/layer/rust_cohab_stub.c) references. Nothing in the DLL
/// calls into this crate by name: shimlib finds the shims by walking the `.shm` section the
/// `implement_shim!` records sit in. So without a reference the linker never pulls this crate's
/// object out of the archive and the DLL registers no shim at all. The workspace builds this
/// crate as ONE codegen unit (Cargo.toml), so the anchor's object holds every record.
#[no_mangle]
#[used]
pub static aclayers_rust_anchor: u8 = 0;
