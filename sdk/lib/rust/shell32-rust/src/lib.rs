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
macro_rules! MESSAGE {
    ($($arg:tt)*) => {{}};
}

// Foundation: minimal Win32 type aliases (grows as waves land).
// Wine's ERR/FIXME/WARN/TRACE for the whole crate; `#[macro_use]` must precede the
// modules that call them (macro_rules! is textually scoped).
#[macro_use]
mod winedebug;

pub mod types;

// ATL smart pointers and containers — the shared substrate in front of the remaining
// C++ TUs (CComPtr appears in 25 of the 38 still-C files). See its header for the three
// invariants the translated call sites depend on.
pub mod atl;

// The ATL CWindowImpl substrate (GWLP_USERDATA in place of the per-instance machine-code
// thunk). Shared by every translated ATL window class; see its header for the mechanism
// decision and the one behavioural difference it carries.
pub mod atlwin;

/// `ui/rosctrls.h`'s CListView + the CWindow methods it inherits — header-only C++ classes, so
/// the island renders the struct and none of the methods. Any TU driving a list view needs this.
pub mod rosctrls;

// The C++ header inlines from shellutils.h / utils.h. They live in HEADERS, so no translated
// TU carries their bodies, yet several TUs call them.
/// `ui/layout.h`'s resizable-dialog inlines — see that module's header for why they are here.
pub mod layout;
/// `shfldr.h`'s header-inline `CRegKeyHandleArray` — one definition, because two
/// independently re-derived copies both lost its destructor. See the module header.
pub mod shfldr;
pub mod shellutils;

// Wave 4: canonical shared shell/COM types (PIDL on-disk format, STRRET, GUID family, COM
// vtables) used by the PIDL core (pidl/shellpath/shellord). Promoted from pidl.rs.
pub mod shellcom;
pub mod active_desktop;
pub mod admin_tools_folder;
pub mod cfs_folder;
// The rest of `folders/CFSFolder.cpp`: its free helper closure plus the bridges it published,
// and its file-local enumerator. With these the whole TU leaves CMake — `cfs_folder` was only
// ever the activated class half of it.
pub mod cfs_folder_tu;
pub mod file_sys_enum;
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
// Assembled at the C tree's path (droptargets/) by rustconv; keep that path stable for re-runs.
#[path = "droptargets/CFSDropTarget.rs"]
pub mod fs_drop_target;
#[path = "droptargets/CRecyclerDropTarget.rs"]
pub mod recycler_drop_target;
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
// The filesystem property-sheet entry points. `ShellPropSheetDialog` is a C++ `struct` used as
// a namespace — no vtable, no activation — so its four statics are hand-written free functions
// here; see the module header.
#[path = "dialogs/item_prop.rs"]
pub mod dialogs_item_prop;

// The shell's own dialogs: Run, Pick-Icon, Restart/Logoff/Exit-Windows.
#[path = "dialogs/dialogs.rs"]
pub mod dialogs_dialogs;

// CDefViewDual — the IShellFolderViewDual automation object. Assembled at the C tree's path
// (shell32/) so re-runs stay stable, but it is a COM class rather than one of that module's
// free-function TUs, so it is wired here instead of from shell32/mod.rs.
#[path = "shell32/CDefViewDual.rs"]
pub mod def_view_dual;

// CDefViewBckgrndMenu — the folder-background context menu. Same wiring rationale as
// CDefViewDual above: assembled under shell32/ to keep the re-run path stable, wired here
// because it is a COM class and not one of that module's free-function TUs.
#[path = "shell32/CDefViewBckgrndMenu.rs"]
pub mod def_view_bckgrnd_menu;

// CDefView — the shell's default folder view (IShellView3 + 7 more interfaces), the largest
// single class in shell32. Wired and flipped together, which is the only safe order: wiring
// alone, with CDefView.cpp still compiled, is a PARTIAL cohab flip — the Rust `#[no_mangle]`
// names shadow the live C++ ones with no duplicate-symbol error and live entry points reroute
// into Rust that never ran (measured 2026-09-19: booted, passed smoke on all three arches,
// then froze after `[EXPLORER] startup: shell window ready`, stopping ProcessStartupItems and
// the kmtest sweep at launched=0). What made the flip safe is that the TU is a LEAF:
// `llvm-nm --undefined-only` over every other object linked into shell32.dll references
// nothing defined in CDefView.cpp.obj. Field manual, "Cohab link plumbing".
#[path = "shell32/cdefview_support.rs"]
pub mod def_view_support;
#[path = "shell32/CDefView.rs"]
pub mod def_view;

// CDefaultContextMenu — the shell's default item context menu. Two rustconv runs, because
// `--cpp-com --class` translates the class only and the TU also carries 22 free functions
// (`DCMA_*`, `SHCreateDefaultContextMenu`, the `*_RustBridge` entry points). The support
// module is HAND-WRITTEN — ports of the utils.h/shellutils.h inlines the TU calls, plus the
// constants and externs outside the bindgen island's closure. It lives in its own file
// because a re-run overwrites each generated half wholesale, so hand-written code parked
// inside either of them is lost the next time the TU is re-assembled.
// COpenWithMenu.cpp. The COM class itself is `open_with_menu` (landed earlier); these are
// the retained C++ closure that kept the .cpp compiled — two PLAIN helper classes (no vtable,
// no QI, no activation: `--cpp-com --class X --plain`) and the free/bridge TU. `SApp` is
// hand-written because it is a POD nested inside COpenWithList and reached only by pointer.
#[path = "shell32/copenwithmenu_support.rs"]
pub mod open_with_menu_support;
#[path = "shell32/COpenWithList.rs"]
pub mod open_with_list;
#[path = "shell32/COpenWithDialog.rs"]
pub mod open_with_dialog;
#[path = "shell32/COpenWithMenu_tu.rs"]
pub mod open_with_menu_tu;

#[path = "shell32/cdefaultcontextmenu_support.rs"]
pub mod default_context_menu_support;
#[path = "shell32/CDefaultContextMenu.rs"]
pub mod default_context_menu;
#[path = "shell32/CDefaultContextMenu_tu.rs"]
pub mod default_context_menu_tu;

// The rest of folders/CNetFolder.cpp. `net_folder` already owns the CNetFolder class itself;
// these two are the retained closure that kept the .cpp compiled — its free helpers and
// `*_RustBridge` entry points, and the CNetFolderEnum enumerator. Two files because one TU
// needs two rustconv runs (--free-functions and --cpp-com) and each assembles its own output.
#[path = "folders/CNetFolder.rs"]
pub mod net_folder_tu;
#[path = "folders/CNetFolderEnum.rs"]
pub mod net_folder_enum;

// The same peel for three more hybrids. In each case the activated class was already Rust and
// what kept the `.cpp` compiled was its free helpers plus a file-local helper class, so the
// slices come in pairs: the `--free-functions` assembly and one `--cpp-com --class` assembly
// per class. The `.cpp` files stay in CMake until each pair compiles and its retained closure
// is published from Rust.
#[path = "dialogs/drvdefext_tu.rs"]
pub mod drv_def_ext_tu;
// filedefext.cpp needed no paired class slice: CFileDefExt and CFileVersionInfo were already
// `dialogs/filedefext.rs`, and its whole retained closure is free-function `_RustBridge`
// entries. SH_FormatByteSize_RustBridge moves here from the .cpp, which is why the two must
// land together.
#[path = "dialogs/filedefext_tu.rs"]
pub mod file_def_ext_tu;
// CDrivesFolder.cpp: the CDrivesFolder class itself was already `drives_folder.rs`; these are
// its free helpers and the one file-local COM class (CDrivesFolderEnum, an internal
// ShellObjectCreator object with no OBJECT_ENTRY — bought with `--cpp-com --internal`).
#[path = "folders/CDrivesFolder.rs"]
pub mod drives_folder_tu;
#[path = "folders/CDrivesFolderEnum.rs"]
pub mod drives_folder_enum;
// folders/CRecycleBin.cpp: CRecycleBin itself was already `recycle_bin.rs`. These are its free
// helpers plus BOTH of its file-local COM classes. The two companions that this TU only *calls*
// — CRecyclerDropTarget and CRecycleBinFolderViewCB — stay C++; see
// dll/win32/shell32/rust_recyclebin_bridges_stub.cpp for why neither could come across.
pub mod def_view_util;
// folders/CRegFolder.cpp, cohab-flipped. Three modules for one TU: the two COM classes and
// the file-scope helpers are separate rustconv runs, each overwriting its own file.
#[path = "folders/CRegFolder.rs"]
pub mod reg_folder;
#[path = "folders/CRegFolderEnum.rs"]
pub mod reg_folder_enum;
#[path = "folders/regfolder_free.rs"]
pub mod reg_folder_free;
#[path = "folders/CRecycleBin.rs"]
pub mod recycle_bin_tu;
#[path = "folders/CRecycleBinEnum.rs"]
pub mod recycle_bin_enum;
#[path = "folders/CRecycleBinItemContextMenu.rs"]
pub mod recycle_bin_item_context_menu;
#[path = "folders/CDesktopFolder.rs"]
pub mod desktop_folder_tu;
#[path = "folders/CDesktopFolderEnum.rs"]
pub mod desktop_folder_enum;
#[path = "folders/CControlPanelFolder.rs"]
pub mod control_panel_folder_tu;
#[path = "folders/CControlPanelEnum.rs"]
pub mod control_panel_enum;
#[path = "folders/CCPLItemMenu.rs"]
pub mod cpl_item_menu;


// Wave 5, the shell32/ directory — free-function TUs from the root of dll/win32/shell32.
pub mod shelldesktop;
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
