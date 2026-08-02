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

// ATL smart pointers and containers — the shared substrate in front of the remaining
// C++ TUs (CComPtr appears in 25 of the 38 still-C files). See its header for the three
// invariants the translated call sites depend on.
pub mod atl;

// The C++ header inlines from shellutils.h / utils.h. They live in HEADERS, so no translated
// TU carries their bodies, yet several TUs call them.
/// `ui/layout.h`'s resizable-dialog inlines — see that module's header for why they are here.
pub mod layout;
pub mod shellutils;

// Wave 4: canonical shared shell/COM types (PIDL on-disk format, STRRET, GUID family, COM
// vtables) used by the PIDL core (pidl/shellpath/shellord). Promoted from pidl.rs.
pub mod shellcom;
pub mod active_desktop;
pub mod admin_tools_folder;
pub mod cfs_folder;
pub mod control_panel_folder;
pub mod copy_as_path_menu;
pub mod copy_to_menu;
pub mod desktop_folder;
pub mod drives_folder;
// Assembled at the C tree's path (dialogs/) by rustconv; keep that path stable for re-runs.
#[path = "dialogs/drvdefext.rs"]
pub mod drv_def_ext;
#[path = "dialogs/filedefext.rs"]
pub mod file_def_ext;
#[path = "dialogs/view.rs"]
pub mod dialogs_view;
pub mod drop_target_helper;
pub mod exe_drop_handler;
pub mod fonts_folder;
pub mod folder_options;
pub mod move_to_menu;
pub mod my_docs_folder;
pub mod net_folder;
pub mod new_menu;
pub mod open_control_panel;
pub mod open_with_menu;
pub mod printer_folder;
pub mod printers_enum;
pub mod query_associations;
pub mod recycle_bin;
pub mod recycle_bin_cleaner;
pub mod send_to_menu;
pub mod shell_item;
pub mod shell_item_array;
pub mod shell_dispatch;
pub mod shell_link;
pub mod start_menu_dummy;
pub mod user_notification;

// wine/ — wine-derived pure-C bodies. Wave 0: shellreg (SHReg* registry wrappers).
// Wave 1: shpolicy, shellstring, clipboard.
pub mod wine;

// Top-level (mirrors the C tree: appbar.c / vista.c live at dll/win32/shell32/ root).
// Wave 1: appbar (SHAppBarMessage), vista (RegLoadMUIString A/W).
pub mod appbar;
pub mod vista;

// Shared Win32/shell substrate for the free-function TUs: SDK facts plus definitions
// promoted out of modules that already proved them. See win32.rs.
pub mod win32;

// The bindgen island — per-arch struct layouts and constants derived from shell32's REAL
// include closure (tools/rustconv/gen_driver_types.sh shell32; profile from
// rustconv.profile_derive). shell32-rust was the one substantial translated module with no
// island, which is why `verify_ffi` — whose oracle is `bindgen_i386.rs` — could not see this
// crate at all, and why it appears in none of the 40 crates the clashing-extern passes
// converged. Declared but not yet globbed by any module: adopting it wholesale means retiring
// the hand-written types.rs/shellcom.rs that 40 landed COM-class modules compile against, and
// that is gated on the vtable-representation conflict documented in the generator's shell32
// cell. Compiling it here proves all three per-arch islands build before anything depends on
// them. Measured against the parked Wave-5 TUs: 49 of their 60 unresolved types resolve from
// here plus ntdll-sys, versus 2,768 lines of hand-written substrate today.
pub mod island;

// Island-first substrate for translated TUs. See prelude.rs for why it REPLACES the
// types/win32 globs rather than joining them.
pub mod prelude;

// Wave 4 free-function TUs: whole plain-C++ translation units with no COM class of their
// own, translated with --free-functions once that path learned the C++ language layer.
//
// propsheet.cpp reaches CStubWindow32 (an ATL CWindowImpl<> that utils.cpp and
// CDrivesFolder.cpp still own) through a create/destroy bridge, so the class keeps a
// single implementation and one registered window class.
pub mod debughlp;
pub mod folders;
pub mod propsheet;
pub mod systray;

// Wave 5, first two leaves — LANDED, and the proof that the island-first recipe closes this
// class. Both glob `crate::prelude` (the island) INSTEAD of `types`/`win32`; see prelude.rs.
#[path = "dialogs/general.rs"]
pub mod dialogs_general;
#[path = "dialogs/recycler_prop.rs"]
pub mod dialogs_recycler_prop;
// The format-drive dialog. It is the TU that made the island grow a disk IOCTL surface — see
// its module header, and `gen_driver_types.sh`'s shell32 cell for the two headers it added.
#[path = "dialogs/drive.rs"]
pub mod dialogs_drive;
// Folder Options. Its one ATL `CWindowImpl<>` stays C++ in `rust_folder_options_stub.cpp`
// behind a create/destroy bridge — the class NAME is what `FindWindow` keys on, so a second
// implementation would split the already-open lookup rather than replace it.
#[path = "dialogs/folder_options.rs"]
pub mod dialogs_folder_options;

// Wave 5, the shell32/ directory — free-function TUs from the root of dll/win32/shell32.
pub mod shell32;
// `stubs` is wired; the other six are parked on the C++ substrate, and shell32/mod.rs records
// each one's remaining error count so the next lane can pick the cheapest. Do NOT delete the
// parked files — they are paid artifacts.

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
