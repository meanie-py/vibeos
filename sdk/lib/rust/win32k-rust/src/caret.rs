//! Rust translation of `win32ss/user/ntuser/caret.c` — text caret.
//! [ntuser wave 3: caret]

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{PsGetCurrentThread, PsGetCurrentThreadWin32Thread};
use ntkrnl_sys::base::NT_SUCCESS;

// MmCopyToCaller(x,y,z) macro -> _MmCopyToCaller cdecl (i386 decorates to __MmCopyToCaller).
extern "C" {
    fn _MmCopyToCaller(target: *mut core::ffi::c_void, source: *mut core::ffi::c_void, bytes: UINT) -> NTSTATUS;
}
// SetLastNtError is FASTCALL.
#[cfg(target_arch = "x86")]
extern "fastcall" { fn SetLastNtError(Status: NTSTATUS); }
#[cfg(not(target_arch = "x86"))]
extern "C" { fn SetLastNtError(Status: NTSTATUS); }

// dtCaretBlink lives in the opaque PERUSERSERVERINFO tail of tagSERVERINFO.
#[inline]
unsafe fn pusi(p: PSERVERINFO) -> *mut _PERUSERSERVERINFO {
    core::ptr::addr_of_mut!((*p).__bindgen_padding_0) as *mut _PERUSERSERVERINFO
}

// win32k-private object helpers: UserReferenceObject/Dereference + ValidateHwndNoErr
// are FASTCALL exports; UserRefObjectCo/Deref + UserHMGetHandle are object.h
// static __inline / macros — reimplemented. head.{h,pti} live in the opaque
// THROBJHEAD base of _THRDESKHEAD, reached via _HEAD/_THROBJHEAD views.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
}

#[inline]
unsafe fn push_entry_list(head: *mut SINGLE_LIST_ENTRY, entry: *mut SINGLE_LIST_ENTRY) {
    (*entry).Next = (*head).Next;
    (*head).Next = entry;
}
#[inline]
unsafe fn pop_entry_list(head: *mut SINGLE_LIST_ENTRY) -> *mut SINGLE_LIST_ENTRY {
    let first = (*head).Next;
    if !first.is_null() {
        (*head).Next = (*first).Next;
    }
    first
}
#[inline]
unsafe fn user_ref_object_co(obj: PVOID, entry: *mut USER_REFERENCE_ENTRY) {
    let w32: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*entry).obj = obj;
    UserReferenceObject(obj);
    push_entry_list(
        core::ptr::addr_of_mut!((*w32).ReferencesList),
        core::ptr::addr_of_mut!((*entry).Entry),
    );
}
#[inline]
unsafe fn user_deref_object_co(obj: PVOID) {
    let w32: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let _ = pop_entry_list(core::ptr::addr_of_mut!((*w32).ReferencesList));
    UserDereferenceObject(obj);
}
#[inline]
unsafe fn user_hm_get_handle(w: PWND) -> HWND {
    (*(core::ptr::addr_of!((*w).head) as *const _HEAD)).h as HWND
}
#[inline]
unsafe fn wnd_pti(w: PWND) -> PTHREADINFO {
    (*(core::ptr::addr_of!((*w).head) as *const _THROBJHEAD)).pti as PTHREADINFO
}
// pEThread is in the _W32THREAD base of _THREADINFO (opaque base, offset 0).
#[inline]
unsafe fn wnd_pethread(w: PWND) -> PETHREAD {
    (*(wnd_pti(w) as *mut _W32THREAD)).pEThread
}

// ---- file-scope preamble (translated) ----
pub const MIN_CARETBLINKRATE: UINT_PTR = 100;
pub const MAX_CARETBLINKRATE: UINT_PTR = 10000;

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntDrawCaret(pWnd: PWND, CaretInfo: PTHRDCARETINFO) {
    co_IntDrawCaret_impl(pWnd, CaretInfo)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntDrawCaret(pWnd: PWND, CaretInfo: PTHRDCARETINFO) {
    co_IntDrawCaret_impl(pWnd, CaretInfo)
}

unsafe fn co_IntDrawCaret_impl(pWnd: PWND, CaretInfo: PTHRDCARETINFO) {
    let hdc: HDC;
    let hdcMem: HDC;
    let hbmOld: HBITMAP;
    let mut rcClient: RECT = core::mem::zeroed();
    let mut bDone: BOOL = 0;

    if pWnd == core::ptr::null_mut() {
        return;
    }

    hdc = UserGetDCEx(pWnd, core::ptr::null_mut(), DCX_USESTYLE as u32);
    if hdc.is_null() {
        return;
    }

    if !(*pWnd).hrgnUpdate.is_null() {
        NtGdiSaveDC(hdc);
    }

    IntGetClientRect(pWnd, core::ptr::addr_of_mut!(rcClient) as *mut _);
    NtGdiIntersectClipRect(
        hdc,
        rcClient.left,
        rcClient.top,
        rcClient.right,
        rcClient.bottom,
    );

    'cleanup: {
        if !(*CaretInfo).Bitmap.is_null() {
            if GreGetBitmapDimension((*CaretInfo).Bitmap, &mut (*CaretInfo).Size) == 0 {
                break 'cleanup;
            }

            hdcMem = NtGdiCreateCompatibleDC(hdc);
            if !hdcMem.is_null() {
                hbmOld = NtGdiSelectBitmap(hdcMem, (*CaretInfo).Bitmap);
                bDone = NtGdiBitBlt(
                    hdc,
                    (*CaretInfo).Pos.x,
                    (*CaretInfo).Pos.y,
                    (*CaretInfo).Size.cx,
                    (*CaretInfo).Size.cy,
                    hdcMem,
                    0,
                    0,
                    SRCINVERT as u32,
                    CLR_INVALID,
                    0,
                );
                NtGdiSelectBitmap(hdcMem, hbmOld);
                GreDeleteObject(hdcMem as _);
            }
        }

        if bDone == 0 {
            NtGdiPatBlt(
                hdc,
                (*CaretInfo).Pos.x,
                (*CaretInfo).Pos.y,
                (*CaretInfo).Size.cx,
                (*CaretInfo).Size.cy,
                DSTINVERT as u32,
            );
        }
    }

    if !(*pWnd).hrgnUpdate.is_null() {
        NtGdiRestoreDC(hdc, -1);
    }

    UserReleaseDC(pWnd, hdc, 0u8 as i32);
}

// CaretSystemTimerProc is a TIMERPROC (CALLBACK = stdcall i386); the type must
// be "stdcall" not "system" to coerce into TIMERPROC at IntSetTimer call sites.
#[no_mangle]
pub unsafe extern "system" fn CaretSystemTimerProc(hwnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD) {
    caret_system_timer_proc(hwnd, uMsg, idEvent, dwTime)
}

unsafe fn caret_system_timer_proc(
    hwnd: HWND,
    uMsg: UINT,
    idEvent: UINT_PTR,
    dwTime: DWORD,
) {
    let pti: PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;
    let pWnd: PWND;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ThreadQueue = (*pti).MessageQueue;

    if (*ThreadQueue).CaretInfo.hWnd != hwnd {
        return;
    }

    if !hwnd.is_null() {
        pWnd = UserGetWindowObject(hwnd);
        if pWnd.is_null() {
            return;
        }
    } else {
        return;
    }

    if idEvent == IDCARETTIMER as UINT_PTR {
        (*ThreadQueue).CaretInfo.Showing =
            if (*ThreadQueue).CaretInfo.Showing != 0 { 0 } else { 1 };
        co_IntDrawCaret(pWnd, core::ptr::addr_of_mut!((*ThreadQueue).CaretInfo));
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntHideCaret(CaretInfo: PTHRDCARETINFO) -> BOOL {
    co_IntHideCaret_impl(CaretInfo)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntHideCaret(CaretInfo: PTHRDCARETINFO) -> BOOL {
    co_IntHideCaret_impl(CaretInfo)
}

unsafe fn co_IntHideCaret_impl(CaretInfo: PTHRDCARETINFO) -> BOOL {
    let pWnd: PWND;
    if (*CaretInfo).hWnd != core::ptr::null_mut() && (*CaretInfo).Visible != 0 && (*CaretInfo).Showing != 0 {
        pWnd = UserGetWindowObject((*CaretInfo).hWnd);
        (*CaretInfo).Showing = 0;

        co_IntDrawCaret(pWnd, CaretInfo);
        IntNotifyWinEvent(EVENT_OBJECT_HIDE, pWnd, OBJID_CARET as LONG, CHILDID_SELF as LONG, 0);
        return TRUE as BOOL;
    }
    FALSE as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn co_IntDestroyCaret(Win32Thread: PTHREADINFO) -> BOOL {
    co_IntDestroyCaret_impl(Win32Thread)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn co_IntDestroyCaret(Win32Thread: PTHREADINFO) -> BOOL {
    co_IntDestroyCaret_impl(Win32Thread)
}

unsafe fn co_IntDestroyCaret_impl(Win32Thread: PTHREADINFO) -> BOOL {
    let ThreadQueue: PUSER_MESSAGE_QUEUE;
    let pWnd: PWND;
    ThreadQueue = (*Win32Thread).MessageQueue;

    if ThreadQueue.is_null() {
        return 0;
    }

    pWnd = ValidateHwndNoErr((*ThreadQueue).CaretInfo.hWnd);
    co_IntHideCaret(core::ptr::addr_of_mut!((*ThreadQueue).CaretInfo));
    (*ThreadQueue).CaretInfo.Bitmap = core::ptr::null_mut::<core::ffi::c_void>() as HBITMAP;
    (*ThreadQueue).CaretInfo.hWnd = core::ptr::null_mut::<core::ffi::c_void>() as HWND;
    (*ThreadQueue).CaretInfo.Size.cy = 0;
    (*ThreadQueue).CaretInfo.Size.cx = (*ThreadQueue).CaretInfo.Size.cy;
    (*ThreadQueue).CaretInfo.Showing = 0;
    (*ThreadQueue).CaretInfo.Visible = 0;
    if !pWnd.is_null() {
        IntNotifyWinEvent(EVENT_OBJECT_DESTROY, pWnd, OBJID_CARET as _, CHILDID_SELF as _, 0);
    }
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntSetCaretBlinkTime(uMSeconds: UINT) -> BOOL {
    (*pusi(gpsi)).dtCaretBlink = uMSeconds;
    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntSetCaretBlinkTime(uMSeconds: UINT) -> BOOL {
    (*pusi(gpsi)).dtCaretBlink = uMSeconds;
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntSetCaretPos(X: core::ffi::c_int, Y: core::ffi::c_int) -> BOOL {
    co_IntSetCaretPos_impl(X, Y)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntSetCaretPos(X: core::ffi::c_int, Y: core::ffi::c_int) -> BOOL {
    co_IntSetCaretPos_impl(X, Y)
}

unsafe fn co_IntSetCaretPos_impl(X: core::ffi::c_int, Y: core::ffi::c_int) -> BOOL {
    let pti: PTHREADINFO;
    let pWnd: PWND;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ThreadQueue = (*pti).MessageQueue;

    if (*ThreadQueue).CaretInfo.hWnd != core::ptr::null_mut() {
        pWnd = UserGetWindowObject((*ThreadQueue).CaretInfo.hWnd);
        if (*ThreadQueue).CaretInfo.Pos.x != X || (*ThreadQueue).CaretInfo.Pos.y != Y {
            co_IntHideCaret(core::ptr::addr_of_mut!((*ThreadQueue).CaretInfo));
            (*ThreadQueue).CaretInfo.Pos.x = X;
            (*ThreadQueue).CaretInfo.Pos.y = Y;
            if (*ThreadQueue).CaretInfo.Visible != 0 {
                (*ThreadQueue).CaretInfo.Showing = 1;
                co_IntDrawCaret(pWnd, core::ptr::addr_of_mut!((*ThreadQueue).CaretInfo));
            }

            IntSetTimer(pWnd, IDCARETTIMER as _, (*pusi(gpsi)).dtCaretBlink, Some(CaretSystemTimerProc), TMRF_SYSTEM as _);
            IntNotifyWinEvent(EVENT_OBJECT_LOCATIONCHANGE as _, pWnd as _, OBJID_CARET as _, CHILDID_SELF as _, 0);
        }
        return 1;
    }

    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_UserHideCaret(Window: PWND) -> BOOL {
    co_UserHideCaret_impl(Window)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_UserHideCaret(Window: PWND) -> BOOL {
    co_UserHideCaret_impl(Window)
}

unsafe fn co_UserHideCaret_impl(Window: PWND) -> BOOL {
    let pti: PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;

    if Window != core::ptr::null_mut() {
    }

    if Window != core::ptr::null_mut() && wnd_pethread(Window) != (PsGetCurrentThread() as PETHREAD) {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return 0;
    }

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ThreadQueue = (*pti).MessageQueue;

    if Window != core::ptr::null_mut()
        && (*ThreadQueue).CaretInfo.hWnd != user_hm_get_handle(Window)
    {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return 0;
    }

    if (*ThreadQueue).CaretInfo.Visible != 0 {
        let pwnd: PWND = UserGetWindowObject((*ThreadQueue).CaretInfo.hWnd);
        IntKillTimer(pwnd, IDCARETTIMER as UINT_PTR, 1);

        co_IntHideCaret(core::ptr::addr_of_mut!((*ThreadQueue).CaretInfo));
        (*ThreadQueue).CaretInfo.Visible = 0;
        (*ThreadQueue).CaretInfo.Showing = 0;
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_UserShowCaret(Window: PWND) -> BOOL {
    co_UserShowCaret_impl(Window)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_UserShowCaret(Window: PWND) -> BOOL {
    co_UserShowCaret_impl(Window)
}

unsafe fn co_UserShowCaret_impl(Window: PWND) -> BOOL {
    let pti: PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;
    let mut pWnd: PWND = core::ptr::null_mut();

    if !Window.is_null() && wnd_pethread(Window) != (PsGetCurrentThread() as PETHREAD) {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return 0;
    }

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ThreadQueue = (*pti).MessageQueue;

    if !Window.is_null() && (*ThreadQueue).CaretInfo.hWnd != user_hm_get_handle(Window) {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return 0;
    }

    if (*ThreadQueue).CaretInfo.Visible == 0 {
        (*ThreadQueue).CaretInfo.Visible = 1;
        pWnd = ValidateHwndNoErr((*ThreadQueue).CaretInfo.hWnd);
        if (*ThreadQueue).CaretInfo.Showing == 0 && !pWnd.is_null() {
            IntNotifyWinEvent(EVENT_OBJECT_SHOW, pWnd, OBJID_CARET as LONG, CHILDID_SELF as LONG, 0);
        }
        if ((*pusi(gpsi)).dtCaretBlink as INT) > 0 {
            IntSetTimer(pWnd, IDCARETTIMER as UINT_PTR, (*pusi(gpsi)).dtCaretBlink, Some(CaretSystemTimerProc), TMRF_SYSTEM as INT);
        } else if (*ThreadQueue).CaretInfo.Visible != 0 {
            (*ThreadQueue).CaretInfo.Showing = 1;
            co_IntDrawCaret(pWnd, core::ptr::addr_of_mut!((*ThreadQueue).CaretInfo));
        }
    }
    1
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCreateCaret(
    hWnd: HWND,
    hBitmap: HBITMAP,
    mut nWidth: core::ffi::c_int,
    mut nHeight: core::ffi::c_int,
) -> BOOL {
    let Window: PWND;
    let pti: PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'exit;
        }

        if wnd_pethread(Window) != (PsGetCurrentThread() as PETHREAD) {
            EngSetLastError(ERROR_ACCESS_DENIED);
            break 'exit;
        }

        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
        ThreadQueue = (*pti).MessageQueue;

        if (*ThreadQueue).CaretInfo.Visible != 0 {
            IntKillTimer(Window, IDCARETTIMER as _, 1u8 as BOOL);
            co_IntHideCaret(core::ptr::addr_of_mut!((*ThreadQueue).CaretInfo));
        }

        (*ThreadQueue).CaretInfo.hWnd = hWnd;
        if !hBitmap.is_null() {
            (*ThreadQueue).CaretInfo.Bitmap = hBitmap;
            (*ThreadQueue).CaretInfo.Size.cy = 0;
            (*ThreadQueue).CaretInfo.Size.cx = 0;
        } else {
            if nWidth == 0 {
                nWidth = UserGetSystemMetrics(SM_CXBORDER as _);
            }
            if nHeight == 0 {
                nHeight = UserGetSystemMetrics(SM_CYBORDER as _);
            }
            (*ThreadQueue).CaretInfo.Bitmap = core::ptr::null_mut();
            (*ThreadQueue).CaretInfo.Size.cx = nWidth;
            (*ThreadQueue).CaretInfo.Size.cy = nHeight;
        }
        (*ThreadQueue).CaretInfo.Visible = 0;
        (*ThreadQueue).CaretInfo.Showing = 0;

        IntSetTimer(
            Window,
            IDCARETTIMER as _,
            (*pusi(gpsi)).dtCaretBlink,
            Some(CaretSystemTimerProc),
            TMRF_SYSTEM as _,
        );

        IntNotifyWinEvent(EVENT_OBJECT_CREATE as _, Window, OBJID_CARET as _, CHILDID_SELF as _, 0);

        Ret = 1u8 as BOOL;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetCaretBlinkTime() -> UINT {
    let ret: UINT;

    UserEnterShared();

    ret = (*pusi(gpsi)).dtCaretBlink;

    UserLeave();

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetCaretPos(lpPoint: LPPOINT) -> BOOL {
    let pti: PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;
    let Status: NTSTATUS;
    let mut Ret: BOOL = 1;

    UserEnterShared();

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ThreadQueue = (*pti).MessageQueue;

    Status = _MmCopyToCaller(
        lpPoint as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*ThreadQueue).CaretInfo.Pos) as *mut core::ffi::c_void,
        core::mem::size_of::<POINT>() as UINT,
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        Ret = 0;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserShowCaret(hWnd: HWND) -> BOOL {
    let mut Window: PWND = core::ptr::null_mut();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut ret: BOOL = 0;

    UserEnterExclusive();

    'block: {
        if !hWnd.is_null() {
            Window = UserGetWindowObject(hWnd);
            if Window.is_null() {
                break 'block;
            }
        }

        if !Window.is_null() {
            user_ref_object_co(Window as PVOID, core::ptr::addr_of_mut!(Ref));
        }

        ret = co_UserShowCaret(Window);

        if !Window.is_null() {
            user_deref_object_co(Window as PVOID);
        }
    }

    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserHideCaret(hWnd: HWND) -> BOOL {
    let mut Window: PWND = core::ptr::null_mut();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut ret: BOOL = 0;

    UserEnterExclusive();

    'block: {
        if !hWnd.is_null() {
            Window = UserGetWindowObject(hWnd);
            if Window.is_null() {
                break 'block;
            }
        }

        if !Window.is_null() {
            user_ref_object_co(Window as PVOID, core::ptr::addr_of_mut!(Ref));
        }

        ret = co_UserHideCaret(Window);

        if !Window.is_null() {
            user_deref_object_co(Window as PVOID);
        }
    }

    UserLeave();
    ret
}
