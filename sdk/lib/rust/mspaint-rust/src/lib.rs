//! mspaint — Rust translation of base/applications/mspaint.
//! staticlib → libmspaint_rust.a, cohab-linked into the cmake-built mspaint
//! (usermode module type `win32gui`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 18 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh mspaint)
#[macro_use]
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod mspaint {
    pub mod cpp_classes; // generated: rustconv port cppapp
    pub mod atlwin_h_h;
    pub mod fullscreen_cpp_h;
    pub mod fullscreen_h_h;
    pub mod history_cpp_h;
    pub mod history_h_h;
    pub mod miniature_cpp_h;
    pub mod palette_cpp_h;
    pub mod palettemodel_cpp_h;
    pub mod registry_cpp_h;
    pub mod registry_h_h;
    pub mod selectionmodel_cpp_h;
    pub mod selectionmodel_h_h;
    pub mod canvas_h_h;
    pub mod canvas_cpp_h;
    pub mod textedit_cpp_h;
    pub mod textedit_h_h;
    pub mod toolbox_cpp_h;
    pub mod toolbox_h_h;
    pub mod toolsettings_cpp_h;
    pub mod toolsettings_h_h;
    pub mod toolsmodel_h_h;
    pub mod mouse_cpp_h;
    pub mod toolsmodel_cpp_h;
    pub mod main_h_h;
    pub mod main_cpp_h;
    pub mod dialogs_cpp_h;
    pub mod dialogs_h_h;
    pub mod atlimagedx_h_h;
    pub mod dib_cpp_h;
    pub mod sizebox_cpp_h;
    pub mod drawing_cpp_h;
    pub mod atlbase_h_h;
    pub mod atltypes_h_h;
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
