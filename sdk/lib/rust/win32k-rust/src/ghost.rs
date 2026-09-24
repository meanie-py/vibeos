//! Rust translation of `win32ss/user/ntuser/ghost.c` — window ghosting.
//! [ntuser wave 5: ghost]

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::PsGetCurrentThreadWin32Thread;
use ntkrnl_sys::base::NT_SUCCESS;
use core::ffi::c_void;

// SafeProbeForRead is NTAPI (stdcall i386, `_SafeProbeForRead@12`).
extern "system" {
    fn SafeProbeForRead(addr: *mut c_void, len: SIZE_T, align: ULONG) -> NTSTATUS;
}

// ValidateHwndNoErr / MsqIsHung / LookupFnIdToiCls are FASTCALL win32k exports
// not covered by win32k-sys bindgen. (UserEnterExclusive/UserLeave/UserGetClassName/
// IntGetAtomFromStringOrAtom/UserGetProp/UserCreateSystemThread/gpsi come from the glob.)
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
    fn MsqIsHung(pti: PTHREADINFO, flags: UINT) -> BOOLEAN;
    fn LookupFnIdToiCls(fnid: INT, picls: *mut INT) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
    fn MsqIsHung(pti: PTHREADINFO, flags: UINT) -> BOOLEAN;
    fn LookupFnIdToiCls(fnid: INT, picls: *mut INT) -> BOOL;
}

// Rtl*UnicodeString are NTAPI (i386 stdcall / else cdecl => extern "system").
// Declared locally with win32k-sys UNICODE_STRING to avoid a cross-crate type clash.
extern "system" {
    fn RtlInitUnicodeString(dst: PUNICODE_STRING, src: *const u16);
    fn RtlEqualUnicodeString(s1: PUNICODE_STRING, s2: PUNICODE_STRING, ci: BOOLEAN) -> BOOLEAN;
    fn RtlFreeUnicodeString(s: PUNICODE_STRING);
}

// GHOST_DATA (win32ss/include/ghostwnd.h) — not in win32k-sys bindgen.
#[repr(C)]
#[derive(Copy, Clone)]
struct GHOST_DATA {
    hwndTarget: HWND,
    hbm32bpp: HBITMAP,
    bDestroyTarget: BOOL,
}

// static UNICODE_STRING GhostClass = RTL_CONSTANT_STRING(L"Ghost");
// static UNICODE_STRING GhostProp  = RTL_CONSTANT_STRING(L"GhostProp");
// (rustconv dropped these — reconstructed from [u16;N] backing buffers.)
static GHOST_CLASS_BUF: [u16; 6] = [
    b'G' as u16, b'h' as u16, b'o' as u16, b's' as u16, b't' as u16, 0,
];
static GHOST_PROP_BUF: [u16; 10] = [
    b'G' as u16, b'h' as u16, b'o' as u16, b's' as u16, b't' as u16,
    b'P' as u16, b'r' as u16, b'o' as u16, b'p' as u16, 0,
];
#[inline]
unsafe fn us_from(buf: &'static [u16]) -> UNICODE_STRING {
    let chars = (buf.len() - 1) as USHORT; // exclude the NUL terminator
    UNICODE_STRING {
        Length: chars * 2,
        MaximumLength: (buf.len() as USHORT) * 2,
        Buffer: buf.as_ptr() as PWCH,
    }
}

// head.pti lives in the opaque _THROBJHEAD base of _THRDESKHEAD (offset 8).
#[inline]
unsafe fn wnd_pti(w: PWND) -> PTHREADINFO {
    (*(core::ptr::addr_of!((*w).head) as *const _THROBJHEAD)).pti as PTHREADINFO
}

pub static mut gptiGhostThread: PTHREADINFO = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn UserGhostThreadEntry() {
    UserGhostThreadEntry_impl()
}
unsafe fn UserGhostThreadEntry_impl() {
    UserEnterExclusive();
    gptiGhostThread = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    // TODO (matches C): handle all ghost windows; exit when none needed.
    gptiGhostThread = core::ptr::null_mut();
    UserLeave();
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntIsGhostWindow(Window: PWND) -> BOOL {
    IntIsGhostWindow_impl(Window)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntIsGhostWindow(Window: PWND) -> BOOL {
    IntIsGhostWindow_impl(Window)
}
unsafe fn IntIsGhostWindow_impl(Window: PWND) -> BOOL {
    let mut Ret: BOOLEAN = 0u8;
    let mut ClassName: UNICODE_STRING = core::mem::zeroed();
    let mut iCls: INT = 0;
    let mut Atom: RTL_ATOM = 0;

    if Window.is_null() {
        return 0;
    }

    if (*Window).fnid != 0 && ((*Window).fnid & FNID_DESTROY as u32) == 0 {
        if LookupFnIdToiCls((*Window).fnid as INT, &mut iCls) != 0 {
            Atom = (*gpsi).atomSysClass[iCls as usize];
        }
    }

    RtlInitUnicodeString(&mut ClassName, core::ptr::null());
    let Len: INT = UserGetClassName((*Window).pcls, &mut ClassName, Atom, 0);
    if Len > 0 {
        let mut ghost_class = us_from(&GHOST_CLASS_BUF);
        Ret = RtlEqualUnicodeString(&mut ClassName, &mut ghost_class, 1u8);
    }
    RtlFreeUnicodeString(&mut ClassName);

    Ret as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGhostWindowFromHungWindow(pHungWnd: PWND) -> HWND {
    IntGhostWindowFromHungWindow_impl(pHungWnd)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGhostWindowFromHungWindow(pHungWnd: PWND) -> HWND {
    IntGhostWindowFromHungWindow_impl(pHungWnd)
}
unsafe fn IntGhostWindowFromHungWindow_impl(pHungWnd: PWND) -> HWND {
    let mut Atom: RTL_ATOM = 0;
    let mut ghost_prop = us_from(&GHOST_PROP_BUF);

    if IntGetAtomFromStringOrAtom(&mut ghost_prop, core::ptr::addr_of_mut!(Atom)) == 0 {
        return core::ptr::null_mut();
    }

    let hwndGhost = UserGetProp(pHungWnd, Atom as ATOM, 1u8) as HWND;
    if !hwndGhost.is_null() {
        if !ValidateHwndNoErr(hwndGhost).is_null() {
            return hwndGhost;
        }
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGhostWindowFromHungWindow(hwndHung: HWND) -> HWND {
    UserGhostWindowFromHungWindow_impl(hwndHung)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGhostWindowFromHungWindow(hwndHung: HWND) -> HWND {
    UserGhostWindowFromHungWindow_impl(hwndHung)
}
unsafe fn UserGhostWindowFromHungWindow_impl(hwndHung: HWND) -> HWND {
    let pHungWnd = ValidateHwndNoErr(hwndHung);
    if pHungWnd.is_null() {
        return core::ptr::null_mut();
    }
    IntGhostWindowFromHungWindow(pHungWnd)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntHungWindowFromGhostWindow(pGhostWnd: PWND) -> HWND {
    IntHungWindowFromGhostWindow_impl(pGhostWnd)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntHungWindowFromGhostWindow(pGhostWnd: PWND) -> HWND {
    IntHungWindowFromGhostWindow_impl(pGhostWnd)
}
unsafe fn IntHungWindowFromGhostWindow_impl(pGhostWnd: PWND) -> HWND {
    let UserData: *const GHOST_DATA;
    let mut hwndTarget: HWND = core::ptr::null_mut();

    if IntIsGhostWindow(pGhostWnd) == 0 {
        return core::ptr::null_mut();
    }

    UserData = (*pGhostWnd).dwUserData as *const GHOST_DATA;
    if !UserData.is_null() {
        let Status: NTSTATUS = SafeProbeForRead(
            UserData as *mut c_void,
            core::mem::size_of::<GHOST_DATA>() as _,
            1,
        );
        if !NT_SUCCESS(Status) {
            hwndTarget = core::ptr::null_mut();
        } else if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(hwndTarget) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*UserData).hwndTarget) as *const core::ffi::c_void,
            core::mem::size_of::<HWND>(),
        ) {
            // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
            hwndTarget = core::ptr::null_mut();
        }
    } else {
        hwndTarget = core::ptr::null_mut();
    }

    if !hwndTarget.is_null() {
        if !ValidateHwndNoErr(hwndTarget).is_null() {
            return hwndTarget;
        }
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserHungWindowFromGhostWindow(hwndGhost: HWND) -> HWND {
    UserHungWindowFromGhostWindow_impl(hwndGhost)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserHungWindowFromGhostWindow(hwndGhost: HWND) -> HWND {
    UserHungWindowFromGhostWindow_impl(hwndGhost)
}
unsafe fn UserHungWindowFromGhostWindow_impl(hwndGhost: HWND) -> HWND {
    let pGhostWnd: PWND = ValidateHwndNoErr(hwndGhost);
    IntHungWindowFromGhostWindow(pGhostWnd)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntMakeHungWindowGhosted(hwndHung: HWND) -> BOOL {
    IntMakeHungWindowGhosted_impl(hwndHung)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntMakeHungWindowGhosted(hwndHung: HWND) -> BOOL {
    IntMakeHungWindowGhosted_impl(hwndHung)
}
unsafe fn IntMakeHungWindowGhosted_impl(hwndHung: HWND) -> BOOL {
    let pHungWnd: PWND = ValidateHwndNoErr(hwndHung);
    if pHungWnd.is_null() {
        return 0; // not a window
    }

    if MsqIsHung(wnd_pti(pHungWnd), MSQ_HUNG as UINT) == 0 {
        return 0; // not hung
    }

    if ((*pHungWnd).style & WS_VISIBLE as u32) == 0 {
        return 0; // invisible
    }

    if ((*pHungWnd).style & WS_CHILD as u32) != 0 {
        return 0; // child
    }

    if IntIsGhostWindow(pHungWnd) != 0 {
        return 0; // a ghost window cannot be ghosted
    }

    if !IntGhostWindowFromHungWindow(pHungWnd).is_null() {
        return 0; // already ghosting
    }

    if gptiGhostThread.is_null() {
        UserCreateSystemThread(ST_GHOST_THREAD as DWORD);
    }

    1
}
