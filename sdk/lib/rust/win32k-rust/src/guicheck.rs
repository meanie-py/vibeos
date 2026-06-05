//! Rust translation of `win32ss/user/ntuser/guicheck.c` — GUI state check.
//! [ntuser wave 2: easy leaves]
//!
//! Delayed GUI setup: assigns a window station, creates the thread message
//! queue, and registers the system window classes on first GUI use.

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::PsGetCurrentProcessWin32Process;
use ntkrnl_sys::interlocked::{InterlockedIncrement, InterlockedDecrement};

// External desktop-graphics helpers (still C, in desktop.c) — all FASTCALL
// (`@Name@0`): dual-cfg so i386 gets the `@0` decoration, C elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn co_IntInitializeDesktopGraphics() -> BOOL;
    fn co_IntDeferredCursorsAndIcons();
    fn IntEndDesktopGraphics();
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn co_IntInitializeDesktopGraphics() -> BOOL;
    fn co_IntDeferredCursorsAndIcons();
    fn IntEndDesktopGraphics();
}

static mut NrGuiAppsRunning: LONG = 0;

// W32PF_flags lives in the W32PROCESS base, which is opaque padding in
// _PROCESSINFO — reach it via a `*mut _W32PROCESS` cast (offset 0), same as
// init.rs.
#[inline]
unsafe fn pw32(w32data: PPROCESSINFO) -> *mut _W32PROCESS {
    w32data as *mut _W32PROCESS
}

// File-internal (C `static FASTCALL`) — no export, plain Rust fns.
unsafe fn co_add_gui_app(w32data: PPROCESSINFO) -> BOOL {
    (*pw32(w32data)).W32PF_flags |= W32PF_CREATEDWINORDC;
    if InterlockedIncrement(core::ptr::addr_of_mut!(NrGuiAppsRunning)) == 1 {
        let initialized: BOOL = co_IntInitializeDesktopGraphics();
        if initialized == 0 {
            (*pw32(w32data)).W32PF_flags &= !W32PF_CREATEDWINORDC;
            InterlockedDecrement(core::ptr::addr_of_mut!(NrGuiAppsRunning));
            return 0;
        }
    } else {
        // Desktop graphics already initialized. Try to load deferred
        // cursors/icons if a previous process didn't have user32 ready.
        co_IntDeferredCursorsAndIcons();
    }
    1
}

unsafe fn remove_gui_app(w32data: PPROCESSINFO) {
    (*pw32(w32data)).W32PF_flags &= !W32PF_CREATEDWINORDC;
    if InterlockedDecrement(core::ptr::addr_of_mut!(NrGuiAppsRunning)) == 0 {
        IntEndDesktopGraphics();
    }
}

// co_IntGraphicsCheck / co_IntUserManualGuiCheck are FASTCALL cross-file
// exports (windc.c/main.c, window.c/window.split3.c) — dual-cfg + #[no_mangle].
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntGraphicsCheck(create: BOOL) -> BOOL {
    co_int_graphics_check(create)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntGraphicsCheck(create: BOOL) -> BOOL {
    co_int_graphics_check(create)
}

unsafe fn co_int_graphics_check(create: BOOL) -> BOOL {
    let w32data: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    let flags = (*pw32(w32data)).W32PF_flags;
    if create != 0 {
        if flags & W32PF_CREATEDWINORDC == 0 && flags & W32PF_MANUALGUICHECK == 0 {
            return co_add_gui_app(w32data);
        }
    } else {
        if flags & W32PF_CREATEDWINORDC != 0 && flags & W32PF_MANUALGUICHECK == 0 {
            remove_gui_app(w32data);
        }
    }
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntUserManualGuiCheck(create: BOOL) {
    co_int_user_manual_gui_check(create)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntUserManualGuiCheck(create: BOOL) {
    co_int_user_manual_gui_check(create)
}

unsafe fn co_int_user_manual_gui_check(create: BOOL) {
    let w32data: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    (*pw32(w32data)).W32PF_flags |= W32PF_MANUALGUICHECK;
    if create != 0 {
        co_add_gui_app(w32data);
    } else {
        remove_gui_app(w32data);
    }
}
