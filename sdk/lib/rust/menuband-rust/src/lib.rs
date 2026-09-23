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
macro_rules! MESSAGE {
    ($($arg:tt)*) => {{}};
}

// Wine's ERR/FIXME/WARN/TRACE for the whole crate; `#[macro_use]` must precede the
// modules that call them (macro_rules! is textually scoped).
#[macro_use]
mod winedebug;

// Shared shell/COM type definitions — path-included from shell32-rust (pure #[repr(C)]
// layout, no code), giving this crate its own layout-identical copy without depending on
// the shell32-rust staticlib.
#[path = "../../shell32-rust/src/types.rs"]
pub mod types;
// The bindgen island, path-included for the same reason types.rs and shellcom.rs are: this
// crate needs its own layout-identical copy without depending on the shell32-rust staticlib.
// `shellcom` re-exports its DATA types from here (the island owns layouts, shellcom owns
// vtable ABI -- shell32-rust/src/prelude.rs states the boundary), so shellcom cannot compile
// in a crate that has no island. The nested `#[path]`s inside island.rs resolve relative to
// island.rs's own directory, so all three per-arch files come from shell32-rust/src/.
#[path = "../../shell32-rust/src/island.rs"]
pub mod island;
#[path = "../../shell32-rust/src/shellcom.rs"]
pub mod shellcom;
// The ONE import surface for translated modules here — island glob plus the few hand-written
// names that have no island equivalent. See its header for why nothing scalar or PIDL-shaped
// crosses over from `shellcom`.
pub mod prelude;

pub mod menu_band;
pub mod menu_desk_bar;
pub mod menu_site;
/// `CMenuFocusManager.cpp`, the whole TU — a PLAIN class (`--plain`): no COM map, no vtable,
/// no activation, reached only by direct call from this library.
/// Hand-written substrate for the module below: the class-static TlsIndex and the two
/// HOOKPROCs, neither of which a worker body may own. Its header says why.
pub mod menu_focus_manager_support;
pub mod menu_focus_manager;
pub mod merged_folder_support;
pub mod merged_folder;
/// `CMergedFolder.cpp`'s SECOND class. CMergedFolder itself has been Rust for a while, but
/// CEnumMergedFolder stayed C++ behind six `_RustBridge` shims because its activation is
/// `new CComObject<T>` — mapped now. Wired ahead of the flip so it is compiled and visible.
/// Hand-written adapter for the module below: the DSA callback trampoline, which needs an
/// ABI a worker body may not declare. Its header says why.
pub mod enum_merged_folder_support;
pub mod enum_merged_folder;
/// `CStartMenu.cpp`'s five free functions (`--free-functions`). Its ONE class,
/// CShellMenuCallback, has its methods defined inline in the .cpp and is a separate run —
/// so the .cpp stays in CMake until that lands too.
/// `CStartMenu.cpp`'s ONE class — its methods are defined inline in the .cpp, which is why a
/// `^Class::method` survey of that TU finds nothing and reports it as free-functions-only.
/// Activated by ATL's static `CComObject<T>::CreateInstance(&p)`, at refcount 0.
pub mod shell_menu_callback;
pub mod start_menu;
/// `CMenuToolbars.cpp` — CMenuToolbarBase and its two derived toolbars, hand-translated:
/// the bespoke (non-COM) vtable and the constructors on BOTH base and derived put it outside
/// what the deterministic scaffold can hold. IN PROGRESS: the module is empty and
/// CMenuToolbars.cpp stays in CMake until it publishes all 29 `*_RustBridge` entry points
/// `menu_band.rs` declares.
pub mod menu_toolbars;

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
