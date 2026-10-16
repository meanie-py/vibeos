//! rapps — Rust translation of base/applications/rapps.
//! staticlib → librapps_rust.a, cohab-linked into the cmake-built rapps
//! (usermode module type `win32gui`).
//!
//! Module map mirrors the CMakeLists source list exactly — the 15 TU(s)
//! the build actually compiles. Generated sources (none)
//! stay C and link alongside.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh rapps)
#[macro_use]
pub mod gdiplus_flat; // GDI+ flat API the gdiplus*.h wrappers call (see the file)
pub mod prelude; // the ONE import surface: types glob + debug macros + CRT externs

#[allow(clippy::module_inception)]
pub mod rapps {
    pub mod cpp_classes; // generated: rustconv port cppapp
    pub mod atlbase_h_h;
    pub mod settings_h_h;
    pub mod appinfo_cpp_h;
    pub mod appinfo_h_h;
    pub mod appdb_cpp_h;
    pub mod appdb_h_h;
    pub mod misc_h_h;
    pub mod configparser_cpp_h;
    pub mod rapps_h_h;
    pub mod atlwin_h_h;
    pub mod rosui_h_h;
    pub mod rosctrls_h_h;
    pub mod crichedit_h_h;
    pub mod asyncinet_h_h;
    pub mod atltypes_h_h;
    pub mod appview_h_h;
    pub mod appview_cpp_h;
    pub mod gui_cpp_h;
    pub mod geninst_cpp_h;
    pub mod settings_cpp_h;
    pub mod settingsdlg_cpp_h;
    pub mod winmain_cpp_h;
    pub mod integrity_cpp_h;
    pub mod unattended_cpp_h;
    pub mod loaddlg_cpp_h;
    pub mod misc_cpp_h;
    pub mod asyncinet_cpp_h;
    pub mod cabinet_cpp_h;
    pub mod gdiplusinit_h_h;
    pub mod gdipluscolor_h_h;
    pub mod gdiplusbase_h_h;
    pub mod gdiplusbrush_h_h;
    pub mod gdiplusgraphics_h_h;
    pub mod gdiplusheaders_h_h;
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
