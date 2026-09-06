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
        // LOCAL FIX (not upstream): retry the deferred cursor/icon load on every GUI
        // check, and do it AFTER co_add_gui_app.
        //
        // co_IntInitializeDesktopGraphics' cursor/icon block is compiled out here by
        // this tree's `#if !defined(_USE_DUMMY_PSEH)` (and -D_USE_DUMMY_PSEH=1 is on
        // every i386 TU, so the comment claiming i386 keeps it is wrong), leaving
        // co_IntDeferredCursorsAndIcons as the only path that ever loads them. It was
        // reached EXACTLY ONCE per boot, on the desktop thread, where its own guard
        // makes it return immediately — so gasyscur stayed NULL for the whole boot and
        // every system class registered with hCursor = NULL.
        //
        // Order matters: NtUserSetSystemCursor only stores into gasyscur for a process
        // that already has W32PF_CREATEDWINORDC (cursoricon.c), which co_add_gui_app is
        // what sets. Calling before it loads every cursor in user mode and then throws
        // them away in the kernel — silently, since the syscall just returns FALSE.
        //
        // The call is idempotent and self-guarded (gbDesktopCursorsAndIconsLoaded, the
        // desktop-thread check, and a NULL KernelCallbackTable all bail), so attempting
        // it from each GUI thread costs one predictable-branch test once loaded.
        let ret = if flags & W32PF_CREATEDWINORDC == 0 && flags & W32PF_MANUALGUICHECK == 0 {
            co_add_gui_app(w32data)
        } else {
            1
        };
        // Only once the process really is a GUI app: NtUserSetSystemCursor stores
        // nothing without W32PF_CREATEDWINORDC, so on a failed co_add_gui_app this
        // would load every cursor in user mode just to have the kernel drop it.
        if ret != 0 {
            co_IntDeferredCursorsAndIcons();
        }
        return ret;
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
