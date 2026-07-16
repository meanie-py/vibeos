#![no_std]
// ShellMessageBoxA (shellord.rs) is a genuine C-variadic export (forwards a va_list to
// FormatMessageA). Stable Rust cannot DEFINE a variadic fn, so its variadic entry point
// is a trampoline in dll/win32/shell32/rust_varargs_shim.c that calls the fixed-arity
// `ShellMessageBoxA_v` twin — this crate needs no nightly feature gate.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_macros)]
#![allow(static_mut_refs)]
#![allow(private_interfaces)]
#![allow(clippy::all)]

//! shell32.dll — Rust translation of dll/win32/shell32 (the usermode Shell client DLL).
//! staticlib → libshell32_rust.a, cohab-linked into the cmake shell32.dll with the
//! translated .c commented out. The wine-derived pure-C half of the Shell API (registry
//! helpers, PIDL, shell paths, ordinal exports, policy, string utils, control-panel,
//! menus and OLE glue). The C++/ATL COM-class tail is now entering class-atomic
//! translation; `active_desktop`, `drop_target_helper`, `open_control_panel`, and
//! `exe_drop_handler` are the first Rust-owned activated objects.
//! Most bodies forward to other usermode DLLs (advapi32/shlwapi/ole32/kernel32) resolved
//! via shell32's existing import libs. See docs/engineering/shell32-translation-audit.md.

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

// Foundation: minimal Win32 type aliases (grows as waves land).
pub mod types;

// Wave 4: canonical shared shell/COM types (PIDL on-disk format, STRRET, GUID family, COM
// vtables) used by the PIDL core (pidl/shellpath/shellord). Promoted from pidl.rs.
pub mod shellcom;
pub mod active_desktop;
pub mod drop_target_helper;
pub mod exe_drop_handler;
pub mod open_control_panel;

// wine/ — wine-derived pure-C bodies. Wave 0: shellreg (SHReg* registry wrappers).
// Wave 1: shpolicy, shellstring, clipboard.
pub mod wine;

// Top-level (mirrors the C tree: appbar.c / vista.c live at dll/win32/shell32/ root).
// Wave 1: appbar (SHAppBarMessage), vista (RegLoadMUIString A/W).
pub mod appbar;
pub mod vista;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
