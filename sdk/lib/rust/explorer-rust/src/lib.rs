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
    // Tier C — taskband.cpp: CTaskBand, the internal task band. TEMPLATE for the
    // MULTI-INTERFACE COM object (5 QI-reachable interfaces, one vtbl ptr each,
    // CONTAINING_RECORD recovery per interface).
    pub mod taskband;
    // Tier C — startmnusite.cpp: CStartMenuSite (start-menu site COM object,
    // IServiceProvider/ITrayPriv/IOleCommandTarget/IMenuPopup).
    pub mod startmnusite;
    // Tier C — startctxmnu.cpp: CStartMenuBtnCtxMenu (Start-button context menu,
    // single-interface IContextMenu; consumes IShellFolder + PIDLs).
    pub mod startctxmnu;
    // Tier C — tbsite.cpp: CTrayBandSite (tray band site; wraps the shell
    // RebarBandSite, hosts the task band). ITrayBandSite/IBandSite/
    // IBandSiteStreamCallback; consumes taskband + rshell band-site factories.
    pub mod tbsite;
    // Tier C — syspager.cpp: CSysPagerWnd (systray icon pager) + CIconWatcher/
    // CBalloonQueue/CNotifyToolbar helpers. IOleWindow window class; the
    // embedded CNotifyToolbar is a subclassed comctl32 toolbar.
    pub mod syspager;
    // Tier C — taskswnd.cpp: CTaskSwitchWnd (task-switch / window-list toolbar)
    // + CHardErrorThread. IOleWindow window class; embeds a subclassed comctl32
    // toolbar (the task buttons) + a hard-error balloon thread.
    pub mod taskswnd;
    // trayprop.cpp — Taskbar & Start-Menu Properties dialog (3 WTL property
    // pages → flat DLGPROCs). Not boot-path (runs on user action).
    pub mod trayprop;
    // startmnucust.cpp — "Customize Classic Start Menu" dialog (treeview of
    // advanced options → registry Advanced values). Not boot-path.
    pub mod startmnucust;
    // Tier C (mega-cluster) — traywnd.cpp + appbar.cpp + CStartButton: CTrayWindow,
    // the taskbar core (ITrayWindow/IShellDesktopTray/IOleWindow/IContextMenu),
    // its CAppBarManager base, and the embedded start button. Creates everything.
    pub mod traywnd;
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
