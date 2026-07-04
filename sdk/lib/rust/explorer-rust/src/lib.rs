//! explorer-rust — Rust translation of explorer.exe, the desktop shell
//! (taskbar/tray/start-menu host) and the last boot-path component still
//! C++. Substrate: winsrv-sys (Win32 superset) + explorer-sys (COM vtables,
//! shlwapi/version externs, cohab globals).
//!
//! Per-file cohabitation into the cmake-built explorer.exe; tier ordering
//! (leaf-procedural → OO → ATL/WTL monsters) and the COM/WTL→Rust design
//! live in docs/engineering/explorer-rust-translation-plan.md.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

pub mod explorer {
    // util.cpp — menu/registry/version-info helpers. First swap (PoC).
    pub mod util;
    // Tier A (procedural) — translated + integrated 2026-07-04.
    pub mod notifyiconscust;
    pub mod rshell;
    pub mod shellservice;
    pub mod startmnu;
    pub mod startup;
    // Tier B — settings.cpp: TaskbarSettings::Save/Load + the shared
    // g_TaskbarSettings global (referenced by 8 still-C++ files). The mangled
    // ::Save/::Load entry points forward from settings_shim.cpp.
    pub mod settings;
    // Tier B — desktop.cpp: CDesktopThread (own desktop message-loop thread) +
    // the extern "C" DesktopCreateWindow/DesktopDestroyShellWindow entry points.
    pub mod desktop;
    // Tier C (FIRST) — trayclock.cpp: CTrayClockWnd, the taskbar clock. The
    // template class exercising the whole explorer_sys::server ATL/WTL
    // substrate (CWindowImpl→WndProc, CComObjectRootEx→refcount+QI,
    // ShellObjectCreatorInit→factory, CTooltips→TTM_* wrappers).
    pub mod trayclock;
    // TODO(next round): startmnucust.rs is translated (.rs present) but NOT
    // yet integrated — it needs the TaskbarSettings/TW_STUCKRECTS2 bitfield
    // mirror + comctl32 treeview substrate in explorer-sys. startmnucust.cpp
    // stays compiled until then (phased swap; NOT a dodge — never yet green).
    // pub mod startmnucust;
}

// USER-MODE no_std staticlib linked into a C++ exe that has its own CRT/EH;
// Rust-side panics only ever come from translated-code bugs. Same handler
// shape as userinit-rust/winsrv-rust (no privileged instructions).
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
