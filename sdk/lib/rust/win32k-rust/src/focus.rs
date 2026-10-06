#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::ddk::{PsGetCurrentProcessWin32Process, PsGetCurrentThreadWin32Thread, PsGetProcessWin32Process, PsGetThreadId, PsLookupProcessByProcessId};
use ntkrnl_sys::ddk::ExFreePoolWithTag;
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::interlocked::{InterlockedIncrement, InterlockedDecrement};

// USERTAG_WINDOWLIST pool tag = 'lwsU' (tags.h). C 4-char literal is big-endian;
// the LE byte order is the literal reversed -> "Uswl".
const USERTAG_WINDOWLIST: u32 = u32::from_le_bytes(*b"Uswl");

// HWND_TOP = (HWND)0 / HWND_BROADCAST = (HWND)0xffff / HWND_BOTTOM = (HWND)1.
const HWND_TOP: HWND = 0usize as HWND;
const HWND_BROADCAST: HWND = 0xffff as HWND;

// ASFW_ANY = ((DWORD)-1) (winuser.h).
const ASFW_ANY: DWORD = 0xffff_ffff;

// MAKELONG / MAKEWPARAM / MAKELPARAM (minwindef.h / winuser.h macros).
#[inline(always)]
fn MAKELONG(low: u32, high: u32) -> LONG {
    ((low & 0xffff) | ((high & 0xffff) << 16)) as LONG
}
#[inline(always)]
fn MAKEWPARAM(low: u32, high: u32) -> WPARAM {
    MAKELONG(low, high) as u32 as WPARAM
}
#[inline(always)]
fn MAKELPARAM(low: i32, high: i32) -> LPARAM {
    MAKELONG(low as u32, high as u32) as LPARAM
}

// UlongToHandle(ul) = (HANDLE)(ULONG_PTR)(ULONG)ul (basetsd.h macro).
#[inline(always)]
fn UlongToHandle(ul: u32) -> HANDLE {
    ul as ULONG_PTR as HANDLE
}

// UserHMGetHandle(obj) = obj->head.h (HANDLE at offset 0 of the object HEAD).
// FORCEINLINE macro — no exported symbol. The C macro yields HANDLE; for PWND it is
// always used as HWND, so return HWND (which also casts to WPARAM/LPARAM at use).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(pobj: *mut T) -> HWND {
    *(pobj as *const HWND)
}

// _THRDESKHEAD/_THROBJHEAD.pti: bindgen flattens HEAD{h, cLockObj} + pti into
// __bindgen_padding_0. pti follows HEAD{HANDLE, DWORD} and is ptr-aligned, so it
// sits at 2*usize (8 i386 / 16 amd64/arm64) — HANDLE+DWORD (=12 on 64-bit) would
// MISS the tail padding. (Matches simplecall.rs/class.rs head_pti.)
#[inline(always)]
unsafe fn head_pti(head: *const _THRDESKHEAD) -> PTHREADINFO {
    let off = 2 * core::mem::size_of::<usize>();
    *((head as *const u8).add(off) as *const PTHREADINFO)
}

// _THREADINFO.pEThread: bindgen flattens the embedded W32THREAD prefix into
// __bindgen_padding_0; pEThread is the very first W32THREAD member, so it sits at
// offset 0 of _THREADINFO.
// Returns the ddk ETHREAD ptr type (PsGetThreadId's param) — identical layout to
// win32k_sys::PETHREAD but a distinct nominal type.
#[inline(always)]
unsafe fn pti_pEThread(pti: PTHREADINFO) -> *mut ntkrnl_sys::ddk::_ETHREAD {
    *(pti as *const *mut ntkrnl_sys::ddk::_ETHREAD)
}

// PROCESSINFO->W32PF_flags lives in the flattened W32PROCESS prefix
// (peProcess ptr + RefCount DWORD), so W32PF_flags (ULONG) sits at
// size_of::<usize>()+size_of::<u32>() = 8 (i386) / 12 (amd64/arm64).
#[inline(always)]
unsafe fn ppi_w32pf_flags(ppi: PPROCESSINFO) -> *mut DWORD {
    let off = core::mem::size_of::<usize>() + core::mem::size_of::<u32>();
    (ppi as *mut u8).add(off) as *mut DWORD
}

// gpsi->{ptCursor,PUSIFlags}: tagSERVERINFO ends with __bindgen_padding_0
// representing the anonymous PERUSERSERVERINFO member; cast that tail to access
// its fields (matches mouse.rs/timer.rs gpsi_per_user).
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// IS_IMM_MODE() = (gpsi && (gpsi->dwSRVIFlags & SRVINFO_IMM32)) (ntuser.h macro).
#[inline(always)]
unsafe fn IS_IMM_MODE() -> BOOL {
    (!gpsi.is_null() && ((*gpsi).dwSRVIFlags & SRVINFO_IMM32) != 0) as BOOL
}

// IS_WND_IMELIKE(pWnd) (window.h macro): pcls->style & CS_IME ||
// pcls->atomClassName == gpsi->atomSysClass[ICLS_IME].
#[inline(always)]
unsafe fn IS_WND_IMELIKE(pWnd: PWND) -> BOOL {
    (((*(*pWnd).pcls).style & CS_IME) != 0
        || (*(*pWnd).pcls).atomClassName == (*gpsi).atomSysClass[ICLS_IME as usize]) as BOOL
}

// IntGetWndThreadId(WndObj) = PsGetThreadId(WndObj->head.pti->pEThread) (window.h).
#[inline(always)]
unsafe fn IntGetWndThreadId(WndObj: PWND) -> HANDLE {
    PsGetThreadId(pti_pEThread(head_pti(core::ptr::addr_of!((*WndObj).head))))
}

// IntGetNonChildAncestor(pWnd) (window.c): walk up while CHILD-not-POPUP.
#[inline(always)]
unsafe fn IntGetNonChildAncestor(mut pWnd: PWND) -> PWND {
    while !pWnd.is_null() && ((*pWnd).style & (WS_CHILD | WS_POPUP)) == WS_CHILD {
        pWnd = (*pWnd).spwndParent;
    }
    pWnd
}

// ReplaceWndPtr(ppwnd, pwndNew) (window.h static inline): ref new, deref old, set.
#[inline(always)]
unsafe fn ReplaceWndPtr(ppwnd: *mut PWND, pwndNew: PWND) {
    if !pwndNew.is_null() {
        UserReferenceObject(pwndNew as PVOID);
    }
    if !(*ppwnd).is_null() {
        UserDereferenceObject((*ppwnd) as PVOID);
    }
    *ppwnd = pwndNew;
}

// WndSetLastActive(pwnd, pwndLastActive) (window.h static inline).
#[inline(always)]
unsafe fn WndSetLastActive(pwnd: PWND, pwndLastActive: PWND) {
    ReplaceWndPtr(core::ptr::addr_of_mut!((*pwnd).spwndLastActive), pwndLastActive);
}

// ASSERT_REFS_CO — debug-only assertion, no-op in release (matches scrollex.rs).
#[inline(always)]
unsafe fn ASSERT_REFS_CO(_Window: PWND) {}

// IntReferenceThreadInfo / IntDereferenceThreadInfo (win32.h macros): inc/dec the
// RefCount in the flattened W32THREAD prefix; on 0 delete (matches object.rs).
#[inline(always)]
unsafe fn IntReferenceThreadInfo(pti: PTHREADINFO) {
    InterlockedIncrement(core::ptr::addr_of_mut!((*(pti as *mut W32THREAD)).RefCount) as *mut i32);
}
#[inline(always)]
unsafe fn IntDereferenceThreadInfo(pti: PTHREADINFO) {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*(pti as *mut W32THREAD)).RefCount) as *mut i32) == 0 {
        UserDeleteW32Thread(pti);
    }
}

// UserRefObjectCo / UserDerefObjectCo — static __inline (object.h), no exported
// symbol. ReferencesList is a real bindgen field on _THREADINFO; USER_REFERENCE_ENTRY
// has Entry (SINGLE_LIST_ENTRY) as its first field so the entry ptr == struct ptr.
#[inline(always)]
unsafe fn UserRefObjectCo<T>(obj: *mut T, UserReferenceEntry: *mut USER_REFERENCE_ENTRY) {
    let obj = obj as PVOID;
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*UserReferenceEntry).obj = obj;
    UserReferenceObject(obj);
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let entry: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*UserReferenceEntry).Entry);
    (*entry).Next = (*list_head).Next;
    (*list_head).Next = entry;
}
#[inline(always)]
unsafe fn UserDerefObjectCo<T>(obj: *mut T) {
    let obj = obj as PVOID;
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let reference_entry: *mut SINGLE_LIST_ENTRY = (*list_head).Next;
    if !reference_entry.is_null() {
        (*list_head).Next = (*reference_entry).Next;
    }
    let _user_reference_entry = reference_entry as *mut USER_REFERENCE_ENTRY;
    UserDereferenceObject(obj);
}

// ---- cross-file win32k symbols (decoration verified via llvm-nm in Phase 2) ----
// FASTCALL (`@Name@N`) on i386, cdecl elsewhere.
// RECTL_vSetRect (rect.h FORCEINLINE — no symbol) reimplemented inline.
#[inline(always)]
unsafe fn RECTL_vSetRect(prc: LPRECTL, left: i32, top: i32, right: i32, bottom: i32) {
    (*prc).left = left;
    (*prc).top = top;
    (*prc).right = right;
    (*prc).bottom = bottom;
}

// RtlEqualMemory(d, s, n) = !memcmp(d, s, n) (rtlfuncs.h macro — no symbol).
#[inline(always)]
unsafe fn RtlEqualMemory(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, len: usize) -> u8 {
    let a = core::slice::from_raw_parts(s1 as *const u8, len);
    let b = core::slice::from_raw_parts(s2 as *const u8, len);
    (a == b) as u8
}

#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_bIntersectRect(prcDst: LPRECTL, prcSrc1: LPRECTL, prcSrc2: LPRECTL) -> u8;
    fn ClearMsgBitsMask(pti: PTHREADINFO, MessageBits: UINT);
    fn MsqDestroyMessage(Message: PUSER_MESSAGE);
    fn MsqSetStateWindow(pti: PTHREADINFO, Type: u32, hWnd: HWND) -> HWND;
    fn co_IntSendMessage(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
    fn VerifyWnd(Window: PWND) -> PWND;
    fn MsqPostMessage(pti: PTHREADINFO, Msg: *mut MSG, Hardware: u8, MessageBits: DWORD, dwQEvent: DWORD, ExtraInfo: isize) -> BOOL;
    fn co_WinPosSetWindowPos(Wnd: PWND, WndInsertAfter: HWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> BOOL;
    fn co_WinPosActivateOtherWindow(Wnd: PWND);
    fn ActivateOtherWindowMin(Wnd: PWND) -> BOOL;
    fn MsqReleaseModifierKeys(MessageQueue: PUSER_MESSAGE_QUEUE);
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(obj: PVOID) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_bIntersectRect(prcDst: LPRECTL, prcSrc1: LPRECTL, prcSrc2: LPRECTL) -> u8;
    fn ClearMsgBitsMask(pti: PTHREADINFO, MessageBits: UINT);
    fn MsqDestroyMessage(Message: PUSER_MESSAGE);
    fn MsqSetStateWindow(pti: PTHREADINFO, Type: u32, hWnd: HWND) -> HWND;
    fn co_IntSendMessage(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
    fn VerifyWnd(Window: PWND) -> PWND;
    fn MsqPostMessage(pti: PTHREADINFO, Msg: *mut MSG, Hardware: u8, MessageBits: DWORD, dwQEvent: DWORD, ExtraInfo: isize) -> BOOL;
    fn co_WinPosSetWindowPos(Wnd: PWND, WndInsertAfter: HWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> BOOL;
    fn co_WinPosActivateOtherWindow(Wnd: PWND);
    fn ActivateOtherWindowMin(Wnd: PWND) -> BOOL;
    fn MsqReleaseModifierKeys(MessageQueue: PUSER_MESSAGE_QUEUE);
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(obj: PVOID) -> BOOL;
}

// UserDeleteW32Thread — exported NTAPI (`_UserDeleteW32Thread`, cdecl on i386).
extern "C" {
    fn UserDeleteW32Thread(pti: PTHREADINFO);
}

// ---- file-scope preamble (translated) ----
// Exported data globals — referenced by other C TUs (decoration `_name` on i386).
#[no_mangle]
pub static mut gpqForeground: PUSER_MESSAGE_QUEUE = core::ptr::null_mut();
#[no_mangle]
pub static mut gpqForegroundPrev: PUSER_MESSAGE_QUEUE = core::ptr::null_mut();
#[no_mangle]
pub static mut gptiForeground: PTHREADINFO = core::ptr::null_mut();
#[no_mangle]
pub static mut gppiLockSFW: PPROCESSINFO = core::ptr::null_mut();
#[no_mangle]
pub static mut guSFWLockCount: ULONG = 0;
#[no_mangle]
pub static mut ptiLastInput: PTHREADINFO = core::ptr::null_mut();
#[no_mangle]
pub static mut ghwndOldFullscreen: HWND = core::ptr::null_mut();

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IsFGLocked() -> BOOL {
    (!gppiLockSFW.is_null() || guSFWLockCount != 0) as BOOL
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IsFGLocked() -> BOOL {
    (!gppiLockSFW.is_null() || guSFWLockCount != 0) as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetCaptureWindow() -> HWND {
    IntGetCaptureWindow_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetCaptureWindow() -> HWND {
    IntGetCaptureWindow_impl()
}

unsafe fn IntGetCaptureWindow_impl() -> HWND {

    let ForegroundQueue: PUSER_MESSAGE_QUEUE = IntGetFocusMessageQueue();
    if ForegroundQueue.is_null() {
        return core::ptr::null_mut();
    }
    if !(*ForegroundQueue).spwndCapture.is_null() {
        UserHMGetHandle((*ForegroundQueue).spwndCapture as *mut core::ffi::c_void)
    } else {
        core::ptr::null_mut()
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetThreadFocusWindow() -> HWND {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;
    if ThreadQueue.is_null() {
        return core::ptr::null_mut();
    }
    if !(*ThreadQueue).spwndFocus.is_null() {
        UserHMGetHandle((*ThreadQueue).spwndFocus)
    } else {
        core::ptr::null_mut()
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetThreadFocusWindow() -> HWND {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;
    if ThreadQueue.is_null() {
        return core::ptr::null_mut();
    }
    if !(*ThreadQueue).spwndFocus.is_null() {
        UserHMGetHandle((*ThreadQueue).spwndFocus)
    } else {
        core::ptr::null_mut()
    }
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn IntIsWindowFullscreen(Window: PWND) -> i32 {
    IntIsWindowFullscreen_impl(Window)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe extern "fastcall" fn IntIsWindowFullscreen(Window: PWND) -> i32 {
    IntIsWindowFullscreen_impl(Window)
}

#[inline]
unsafe fn IntIsWindowFullscreen_impl(Window: PWND) -> i32 {
    let mut rclAnd: RECTL = core::mem::zeroed();
    let mut rclMonitor: RECTL = core::mem::zeroed();
    let mut rclWindow: RECTL = core::mem::zeroed();
    let pMonitor: PMONITOR;

    if Window.is_null()
        || ((*Window).style & WS_VISIBLE) == 0
        || ((*Window).style & WS_CHILD) != 0
        || IntGetWindowRect(Window, &mut rclWindow) == 0
    {
        return 0;
    }

    pMonitor = UserGetPrimaryMonitor();
    if pMonitor.is_null() {
        RECTL_vSetRect(
            &mut rclMonitor,
            0,
            0,
            UserGetSystemMetrics(SM_CXSCREEN as u32),
            UserGetSystemMetrics(SM_CYSCREEN as u32),
        );
    } else {
        rclMonitor = *(&(*pMonitor).rcMonitor as *const _ as LPRECTL);
    }

    RECTL_bIntersectRect(&mut rclAnd, &mut rclMonitor, &mut rclWindow);
    RtlEqualMemory(
        &rclAnd as *const _ as *const core::ffi::c_void,
        &rclMonitor as *const _ as *const core::ffi::c_void,
        core::mem::size_of::<RECTL>(),
    ) as i32
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub unsafe extern "C" fn IntCheckFullscreen(Window: PWND) -> i32 {
    IntCheckFullscreen_impl(Window)
}

#[cfg(not(target_pointer_width = "64"))]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCheckFullscreen(Window: PWND) -> i32 {
    IntCheckFullscreen_impl(Window)
}

unsafe fn IntCheckFullscreen_impl(Window: PWND) -> i32 {
    let hWnd: HWND;

    if !ghwndOldFullscreen.is_null() && IntIsWindowFullscreen(ValidateHwndNoErr(ghwndOldFullscreen)) == 0 {
        ghwndOldFullscreen = core::ptr::null_mut();
    }

    if IntIsWindowFullscreen(Window) == 0 {
        return 0;
    }

    hWnd = UserHMGetHandle(Window);
    if ghwndOldFullscreen != hWnd {
        co_IntShellHookNotify(HSHELL_RUDEAPPACTIVATED as WPARAM, hWnd as WPARAM, 1);
        ghwndOldFullscreen = hWnd;
    }
    1
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub unsafe extern "C" fn UpdateShellHook(Window: PWND) {
    UpdateShellHook_impl(Window)
}

#[cfg(not(target_pointer_width = "64"))]
#[no_mangle]
pub unsafe extern "fastcall" fn UpdateShellHook(Window: PWND) {
    UpdateShellHook_impl(Window)
}

unsafe fn UpdateShellHook_impl(Window: PWND) {
    if IntCheckFullscreen(Window) != 0 {
        return;
    }

    if (*Window).spwndParent == UserGetDesktopWindow()
        && ((((*Window).ExStyle & WS_EX_TOOLWINDOW) == 0)
            || ((*Window).ExStyle & WS_EX_APPWINDOW) != 0)
    {
        co_IntShellHookNotify(
            HSHELL_WINDOWACTIVATED as WPARAM,
            UserHMGetHandle(Window) as WPARAM,
            0,
        );
    } else {
        co_IntShellHookNotify(HSHELL_WINDOWACTIVATED as WPARAM, 0, 0);
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntSendDeactivateMessages(hWndPrev: HWND, hWnd: HWND, Clear: BOOL) -> BOOL {
    co_IntSendDeactivateMessages_impl(hWndPrev, hWnd, Clear)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntSendDeactivateMessages(hWndPrev: HWND, hWnd: HWND, Clear: BOOL) -> BOOL {
    co_IntSendDeactivateMessages_impl(hWndPrev, hWnd, Clear)
}

unsafe fn co_IntSendDeactivateMessages_impl(hWndPrev: HWND, hWnd: HWND, Clear: BOOL) -> BOOL {
    let mut RefPrev: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let WndPrev: PWND;
    let mut Ret: BOOL = 1;
    let lParam: LPARAM = if !hWnd.is_null() { hWnd as LPARAM } else { 0 };

    if !hWndPrev.is_null() {
        WndPrev = ValidateHwndNoErr(hWndPrev);
        if !WndPrev.is_null() {
            UserRefObjectCo(WndPrev, &mut RefPrev);

            if co_IntSendMessage(hWndPrev, WM_NCACTIVATE, 0, lParam) != 0 {
                co_IntSendMessage(
                    hWndPrev,
                    WM_ACTIVATE,
                    MAKEWPARAM(WA_INACTIVE, (((*WndPrev).style & WS_MINIMIZE) != 0) as u32),
                    hWnd as LPARAM,
                );

                if !WndPrev.is_null() && Clear != 0 {
                    (*WndPrev).state &= !(WNDS_ACTIVEFRAME | WNDS_HASCAPTION);
                }
            } else {
                ERR!("Application is keeping itself Active to prevent the change!\n");
                Ret = 0;
            }

            UserDerefObjectCo(WndPrev);
        }
    }
    Ret
}

unsafe fn IntFocusSetInputContext(pWnd: PWND, bActivate: i32, bCallback: i32) {
    let pti: PTHREADINFO;
    let pImeWnd: PWND;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let hImeWnd: HWND;
    let wParam: WPARAM;
    let lParam: LPARAM;

    if pWnd.is_null() || (*pWnd).pcls.is_null() || IS_WND_IMELIKE(pWnd) != 0 {
        return;
    }

    pti = head_pti(core::ptr::addr_of!((*pWnd).head));
    if pti.is_null() || ((*pti).TIF_flags & TIF_INCLEANUP) != 0 {
        return;
    }

    pImeWnd = (*pti).spwndDefaultIme;
    if pImeWnd.is_null() {
        return;
    }

    UserRefObjectCo(pImeWnd, &mut Ref);

    hImeWnd = UserHMGetHandle(pImeWnd);
    wParam = (if bActivate != 0 { IMS_IMEACTIVATE } else { IMS_IMEDEACTIVATE }) as WPARAM;
    lParam = UserHMGetHandle(pWnd) as LPARAM;

    if bCallback != 0 {
        co_IntSendMessageWithCallBack(
            hImeWnd,
            WM_IME_SYSTEM,
            wParam,
            lParam,
            None,
            0,
            core::ptr::null_mut(),
        );
    } else {
        co_IntSendMessage(hImeWnd, WM_IME_SYSTEM, wParam, lParam);
    }

    UserDerefObjectCo(pImeWnd);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntDeactivateWindow(pti: PTHREADINFO, tid: HANDLE) -> BOOL {
    IntDeactivateWindow_impl(pti, tid)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntDeactivateWindow(pti: PTHREADINFO, tid: HANDLE) -> BOOL {
    IntDeactivateWindow_impl(pti, tid)
}

unsafe fn IntDeactivateWindow_impl(pti: PTHREADINFO, tid: HANDLE) -> BOOL {
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut ptiPrev: PTHREADINFO;
    let pwndPrev: PWND;
    let mut InAAPM: BOOL = 0;
    let ptiCurrent: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if (*(*pti).MessageQueue).spwndActive.is_null() {
        return 1;
    }

    if ptiCurrent != pti {
        IntReferenceThreadInfo(pti);
        IntReferenceThreadInfo(ptiCurrent);
    }

    if ((*pti).TIF_flags & TIF_INACTIVATEAPPMSG as u32) == 0 {
        (*pti).TIF_flags |= TIF_INACTIVATEAPPMSG as u32;
        InAAPM = 1;
    }

    //
    // Check for Capture and release it.
    //
    if !(*(*pti).MessageQueue).spwndCapture.is_null() {
        let mut msg: MSG = core::mem::zeroed();
        let pwndCapture: PWND = (*(*pti).MessageQueue).spwndCapture;

        UserRefObjectCo(pwndCapture, &mut Ref);
        co_IntSendMessage(UserHMGetHandle(pwndCapture), WM_CANCELMODE, 0, 0);
        UserDerefObjectCo(pwndCapture);

        msg.message = WM_MOUSEMOVE;
        msg.wParam = UserGetMouseButtonsState() as WPARAM;
        msg.lParam = MAKELPARAM((*gpsi_per_user()).ptCursor.x, (*gpsi_per_user()).ptCursor.y);
        msg.pt = (*gpsi_per_user()).ptCursor;
        co_MsqInsertMouseMessage(&mut msg, 0, 0, 1);
    }

    //
    // Check for Active and release it.
    //
    if !(*(*pti).MessageQueue).spwndActive.is_null() {
        pwndPrev = (*(*pti).MessageQueue).spwndActive;
        ptiPrev = head_pti(core::ptr::addr_of!((*pwndPrev).head));

        if co_IntSendDeactivateMessages(UserHMGetHandle(pwndPrev), core::ptr::null_mut(), 1) == 0 {
            if InAAPM != 0 {
                (*pti).TIF_flags &= !(TIF_INACTIVATEAPPMSG as u32);
            }
            if ptiCurrent != pti {
                IntDereferenceThreadInfo(pti);
                IntDereferenceThreadInfo(ptiCurrent);
            }
            return 0;
        }

        if (*(*pti).MessageQueue).spwndActive == pwndPrev {
            (*(*pti).MessageQueue).spwndActivePrev = pwndPrev;
            (*(*pti).MessageQueue).spwndActive = core::ptr::null_mut();
        }

        if ((*ptiPrev).TIF_flags & TIF_INCLEANUP as u32) != 0 {
            ptiPrev = core::ptr::null_mut();
        }
    } else {
        ptiPrev = pti;
        pwndPrev = (-1isize) as PWND;
    }

    if !ptiPrev.is_null() {
        let OldTID: HANDLE = PsGetThreadId(pti_pEThread(ptiPrev));
        let cWindow: PWND;
        let List: *mut HWND;
        let mut phWnd: *mut HWND;

        List = IntWinListChildren(UserGetDesktopWindow());
        if !List.is_null() {
            if !OldTID.is_null() {
                phWnd = List;
                while !(*phWnd).is_null() {
                    let cWindow = ValidateHwndNoErr(*phWnd);
                    if !cWindow.is_null() && head_pti(core::ptr::addr_of!((*cWindow).head)) == ptiPrev {
                        UserRefObjectCo(cWindow, &mut Ref);
                        co_IntSendMessage(*phWnd, WM_ACTIVATEAPP, 0, OldTID as LPARAM);
                        UserDerefObjectCo(cWindow);
                    }
                    phWnd = phWnd.add(1);
                }
            }
            let _ = cWindow;
            ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
        }
        let _ = tid;
    }

    //
    // Now check for a change (Bounce).
    //
    if (*(*pti).MessageQueue).spwndActive == pwndPrev {
        if co_IntSendDeactivateMessages(UserHMGetHandle(pwndPrev), core::ptr::null_mut(), 0) == 0 {
            if InAAPM != 0 {
                (*pti).TIF_flags &= !(TIF_INACTIVATEAPPMSG as u32);
            }
            if ptiCurrent != pti {
                IntDereferenceThreadInfo(pti);
                IntDereferenceThreadInfo(ptiCurrent);
            }
            return 0;
        }

        if (*(*pti).MessageQueue).spwndActive == pwndPrev {
            (*(*pti).MessageQueue).spwndActivePrev = pwndPrev;
            (*(*pti).MessageQueue).spwndActive = core::ptr::null_mut();
        }
    }

    //
    // Check for Focus and release it.
    //
    if !(*(*pti).MessageQueue).spwndFocus.is_null() {
        let pwndFocus: PWND = (*(*pti).MessageQueue).spwndFocus;

        (*(*pti).MessageQueue).spwndFocus = core::ptr::null_mut();

        UserRefObjectCo(pwndFocus, &mut Ref);
        co_IntSendMessage(UserHMGetHandle(pwndFocus), WM_KILLFOCUS, 0, 0);
        if IS_IMM_MODE() != 0 {
            IntFocusSetInputContext(pwndFocus, 0, 0);
        }
        UserDerefObjectCo(pwndFocus);
    }

    MsqReleaseModifierKeys((*pti).MessageQueue);

    if InAAPM != 0 {
        (*pti).TIF_flags &= !(TIF_INACTIVATEAPPMSG as u32);
    }
    if ptiCurrent != pti {
        IntDereferenceThreadInfo(pti);
        IntDereferenceThreadInfo(ptiCurrent);
    }
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntActivateWindow(
    Wnd: PWND,
    pti: PTHREADINFO,
    tid: HANDLE,
    Type: DWORD,
) {
    IntActivateWindow_impl(Wnd, pti, tid, Type)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntActivateWindow(
    Wnd: PWND,
    pti: PTHREADINFO,
    tid: HANDLE,
    Type: DWORD,
) {
    IntActivateWindow_impl(Wnd, pti, tid, Type)
}

unsafe fn IntActivateWindow_impl(
    mut Wnd: PWND,
    pti: PTHREADINFO,
    tid: HANDLE,
    Type: DWORD,
) {

    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let pmq: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;

    if !Wnd.is_null() {
        Wnd = VerifyWnd(Wnd);

        if Wnd.is_null() {
            return;
        }

        UserRefObjectCo(Wnd as *mut core::ffi::c_void, &mut Ref);

        if gpqForeground.is_null() {
            co_IntSetForegroundMessageQueue(Wnd, pti, Type as BOOL, 0);
        } else {
            if (*pmq).spwndActive == Wnd {
                let wParam: WPARAM =
                    ((*head_pti(core::ptr::addr_of!((*Wnd).head))).MessageQueue == gpqForeground) as WPARAM;

                co_IntSendMessage(UserHMGetHandle(Wnd), WM_NCACTIVATE, wParam, 0);

                if wParam != 0 {
                    UpdateShellHook(Wnd);

                    co_WinPosSetWindowPos(
                        Wnd,
                        0 as HWND,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOMOVE,
                    );
                }
            } else {
                co_IntSetActiveWindow(Wnd, Type as BOOL, 1 as BOOL, 1 as BOOL);
            }
        }
        UserDerefObjectCo(Wnd as *mut core::ffi::c_void);
    } else {
        if !tid.is_null() && !(*pmq).spwndActive.is_null() && gpqForeground == pmq {
            Wnd = (*pmq).spwndActive;

            UserRefObjectCo(Wnd as *mut core::ffi::c_void, &mut Ref);

            co_IntSendMessage(UserHMGetHandle(Wnd), WM_NCACTIVATE, 1 as WPARAM, 0);

            UpdateShellHook(Wnd);

            co_WinPosSetWindowPos(Wnd, 0 as HWND, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);

            UserDerefObjectCo(Wnd as *mut core::ffi::c_void);
        } else if gpqForeground != pmq {
            (*pti).TIF_flags &= !TIF_ALLOWFOREGROUNDACTIVATE;
            (*ppi_w32pf_flags((*pti).ppi)) &= !W32PF_ALLOWFOREGROUNDACTIVATE;
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntMakeWindowActive(Window: PWND) -> BOOL {
    co_IntMakeWindowActive_impl(Window)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntMakeWindowActive(Window: PWND) -> BOOL {
    co_IntMakeWindowActive_impl(Window)
}

unsafe fn co_IntMakeWindowActive_impl(Window: PWND) -> BOOL {
    let mut spwndOwner: PWND;
    if !VerifyWnd(Window).is_null() {
        spwndOwner = Window;
        while !(*spwndOwner).spwndOwner.is_null() {
            spwndOwner = (*spwndOwner).spwndOwner;
        }
        WndSetLastActive(spwndOwner, Window);
        return 1;
    }
    ERR!("MakeWindowActive Failed!\n");
    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntSendActivateMessages(WindowPrev: PWND, Window: PWND, MouseActivate: BOOL, Async: BOOL) -> BOOL {
    co_IntSendActivateMessages_impl(WindowPrev, Window, MouseActivate, Async)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntSendActivateMessages(WindowPrev: PWND, Window: PWND, MouseActivate: BOOL, Async: BOOL) -> BOOL {
    co_IntSendActivateMessages_impl(WindowPrev, Window, MouseActivate, Async)
}

unsafe fn co_IntSendActivateMessages_impl(WindowPrev: PWND, Window: PWND, MouseActivate: BOOL, Async: BOOL) -> BOOL {
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut RefPrev: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut RefCall: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let OldTID: HANDLE;
    let NewTID: HANDLE;
    let ptiOld: PTHREADINFO;
    let ptiNew: PTHREADINFO;
    let mut InAAPM: BOOL = 0;

    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if !Window.is_null() {
        UserRefObjectCo(Window, core::ptr::addr_of_mut!(Ref));

        if !WindowPrev.is_null() {
            UserRefObjectCo(WindowPrev, core::ptr::addr_of_mut!(RefPrev));
        }

        (*(*pti).MessageQueue).QF_flags &= !QF_EVENTDEACTIVATEREMOVED;

        if (*gpsi_per_user()).PUSIFlags & PUSIF_PALETTEDISPLAY != 0 &&
            co_IntSendMessage(UserHMGetHandle(Window), WM_QUERYNEWPALETTE as u32, 0, 0) != 0
        {
            UserSendNotifyMessage(HWND_BROADCAST,
                                  WM_PALETTEISCHANGING as u32,
                                  UserHMGetHandle(Window) as WPARAM,
                                  0);
        }

        if (*Window).style & WS_CHILD as u32 == 0 {
            let mut pwndTemp: PWND = (*co_GetDesktopWindow(Window)).spwndChild;

            while !pwndTemp.is_null() && (*pwndTemp).style & WS_VISIBLE as u32 == 0 {
                pwndTemp = (*pwndTemp).spwndNext;
            }

            if Window != pwndTemp || (!WindowPrev.is_null() && IntIsWindowVisible(WindowPrev) == 0) {
                if Async == 0 || (*pti).MessageQueue == gpqForeground {
                    let mut flags: UINT = (SWP_NOSIZE | SWP_NOMOVE) as UINT;
                    if Window == pwndTemp {
                        flags |= SWP_NOACTIVATE as UINT;
                    }
                    co_WinPosSetWindowPos(Window, HWND_TOP, 0, 0, 0, 0, flags);
                }
            }
        }

        if !(*Window).spwndPrev.is_null() {
            let phwndTopLevel: *mut HWND;
            let mut phwndCurrent: *mut HWND;
            let mut pwndCurrent: PWND;
            let pwndDesktop: PWND;

            pwndDesktop = co_GetDesktopWindow(Window);
            if (*Window).spwndParent == pwndDesktop {
                phwndTopLevel = IntWinListChildren(pwndDesktop);
                phwndCurrent = phwndTopLevel;
                while !(*phwndCurrent).is_null() {
                    pwndCurrent = UserGetWindowObject(*phwndCurrent);

                    if !pwndCurrent.is_null() && (*pwndCurrent).spwndOwner == Window {
                        co_WinPosSetWindowPos(pwndCurrent, HWND_TOP, 0, 0, 0, 0,
                            (SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE) as UINT);
                    }
                    phwndCurrent = phwndCurrent.add(1);
                }
                ExFreePoolWithTag(phwndTopLevel as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
            }
        }
    }

    OldTID = if !WindowPrev.is_null() { IntGetWndThreadId(WindowPrev) } else { core::ptr::null_mut() };
    NewTID = if !Window.is_null() { IntGetWndThreadId(Window) } else { core::ptr::null_mut() };
    ptiOld = if !WindowPrev.is_null() { head_pti(core::ptr::addr_of!((*WindowPrev).head)) } else { core::ptr::null_mut() };
    ptiNew = if !Window.is_null() { head_pti(core::ptr::addr_of!((*Window).head)) } else { core::ptr::null_mut() };

    if (*pti).TIF_flags & TIF_INACTIVATEAPPMSG == 0 && OldTID != NewTID {
        let mut cWindow: PWND;
        let List: *mut HWND;
        let mut phWnd: *mut HWND;

        List = IntWinListChildren(UserGetDesktopWindow());
        if !List.is_null() {
            if !OldTID.is_null() {
                (*ptiOld).TIF_flags |= TIF_INACTIVATEAPPMSG;
                phWnd = List;
                while !(*phWnd).is_null() {
                    cWindow = ValidateHwndNoErr(*phWnd);
                    if !cWindow.is_null() && head_pti(core::ptr::addr_of!((*cWindow).head)) == ptiOld {
                        UserRefObjectCo(cWindow, core::ptr::addr_of_mut!(RefCall));
                        co_IntSendMessage(*phWnd, WM_ACTIVATEAPP as u32, 0, NewTID as LPARAM);
                        UserDerefObjectCo(cWindow);
                    }
                    phWnd = phWnd.add(1);
                }
                (*ptiOld).TIF_flags &= !TIF_INACTIVATEAPPMSG;
            }
            if !NewTID.is_null() {
                InAAPM = 1;
                (*pti).TIF_flags |= TIF_INACTIVATEAPPMSG;
                phWnd = List;
                while !(*phWnd).is_null() {
                    cWindow = ValidateHwndNoErr(*phWnd);
                    if !cWindow.is_null() && head_pti(core::ptr::addr_of!((*cWindow).head)) == ptiNew {
                        UserRefObjectCo(cWindow, core::ptr::addr_of_mut!(RefCall));
                        co_IntSendMessage(*phWnd, WM_ACTIVATEAPP as u32, 1, OldTID as LPARAM);
                        UserDerefObjectCo(cWindow);
                    }
                    phWnd = phWnd.add(1);
                }
            }
            ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
        }
    }

    if !Window.is_null() {
        if !WindowPrev.is_null() {
            UserDerefObjectCo(WindowPrev);
        }

        if (*Window).state & WNDS_ACTIVEFRAME != 0 {
            (*Window).state |= WNDS_NONCPAINT;
        }

        if (*Window).style & WS_MINIMIZE as u32 != 0 {
            TRACE!(b"Widow was minimized\n\0".as_ptr() as PCHAR);
        }

        co_IntMakeWindowActive(Window);

        co_IntSendMessage(UserHMGetHandle(Window),
                          WM_NCACTIVATE as u32,
                          (Window == (if !gpqForeground.is_null() { (*gpqForeground).spwndActive } else { core::ptr::null_mut() })) as WPARAM,
                          0);

        co_IntSendMessage(UserHMGetHandle(Window),
                          WM_ACTIVATE as u32,
                          MAKEWPARAM(if MouseActivate != 0 { WA_CLICKACTIVE as u32 } else { WA_ACTIVE as u32 },
                                     ((*Window).style & WS_MINIMIZE as u32 != 0) as u32),
                          if !WindowPrev.is_null() { UserHMGetHandle(WindowPrev) as LPARAM } else { 0 });

        if (*Window).style & WS_VISIBLE as u32 != 0 {
            UpdateShellHook(Window);
        }

        (*Window).state &= !WNDS_NONCPAINT;

        UserDerefObjectCo(Window);
    }
    InAAPM
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSendFocusMessages(pti: PTHREADINFO, pWnd: PWND) {
    IntSendFocusMessages_impl(pti, pWnd)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSendFocusMessages(pti: PTHREADINFO, pWnd: PWND) {
    IntSendFocusMessages_impl(pti, pWnd)
}

unsafe fn IntSendFocusMessages_impl(pti: PTHREADINFO, pWnd: PWND) {
    let pWndPrev: PWND;
    let ThreadQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;
    let hwndPrev: HWND;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    (*ThreadQueue).QF_flags &= !(QF_FOCUSNULLSINCEACTIVE as u32);
    if pWnd.is_null() && !(*ThreadQueue).spwndActive.is_null() {
        (*ThreadQueue).QF_flags |= QF_FOCUSNULLSINCEACTIVE as u32;
    }

    pWndPrev = (*ThreadQueue).spwndFocus;
    if !pWndPrev.is_null() {
        UserRefObjectCo(pWndPrev, &mut Ref);
    }

    if pWnd.is_null() || ThreadQueue == (*head_pti(core::ptr::addr_of!((*pWnd).head))).MessageQueue {
        (*ThreadQueue).spwndFocus = pWnd;
    }

    if !pWnd.is_null() {
        if !pWndPrev.is_null() {
            co_IntSendMessage(UserHMGetHandle(pWndPrev), WM_KILLFOCUS, UserHMGetHandle(pWnd) as WPARAM, 0);
            if IS_IMM_MODE() != 0 {
                IntFocusSetInputContext(pWndPrev, 0, 0);
            }
        }
        if (*ThreadQueue).spwndFocus == pWnd {
            if IS_IMM_MODE() != 0 {
                IntFocusSetInputContext(pWnd, 1, 0);
            }

            IntNotifyWinEvent(EVENT_OBJECT_FOCUS, pWnd, OBJID_CLIENT as i32, CHILDID_SELF as i32, 0);

            hwndPrev = if !pWndPrev.is_null() { UserHMGetHandle(pWndPrev) } else { core::ptr::null_mut() };
            co_IntSendMessage(UserHMGetHandle(pWnd), WM_SETFOCUS, hwndPrev as WPARAM, 0);
        }
    } else {
        if !pWndPrev.is_null() {
            IntNotifyWinEvent(EVENT_OBJECT_FOCUS, core::ptr::null_mut(), OBJID_CLIENT as i32, CHILDID_SELF as i32, 0);

            co_IntSendMessage(UserHMGetHandle(pWndPrev), WM_KILLFOCUS, 0, 0);
            if IS_IMM_MODE() != 0 {
                IntFocusSetInputContext(pWndPrev, 0, 0);
            }
        }
    }

    if !pWndPrev.is_null() {
        UserDerefObjectCo(pWndPrev);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn FindRemoveEventMsg(
    pti: PTHREADINFO,
    Event: DWORD,
    EventLast: DWORD,
) -> BOOL {
    FindRemoveEventMsg_impl(pti, Event, EventLast)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn FindRemoveEventMsg(
    pti: PTHREADINFO,
    Event: DWORD,
    EventLast: DWORD,
) -> BOOL {
    FindRemoveEventMsg_impl(pti, Event, EventLast)
}

unsafe fn FindRemoveEventMsg_impl(pti: PTHREADINFO, Event: DWORD, EventLast: DWORD) -> BOOL {

    let mut Message: PUSER_MESSAGE;
    let mut Entry: PLIST_ENTRY;
    let mut Ret: BOOL = 0;

    Entry = (*pti).PostedMessagesListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*pti).PostedMessagesListHead) {
        Message = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(USER_MESSAGE, ListEntry) as isize))
            as *mut USER_MESSAGE;
        Entry = (*Entry).Flink;

        if (*Message).dwQEvent == EventLast {
            return Ret;
        }

        if (*Message).dwQEvent == Event {
            ClearMsgBitsMask(pti, (*Message).QS_Flags);
            MsqDestroyMessage(Message);
            Ret = 1;
        }
    }
    Ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn ToggleFGActivate(pti: PTHREADINFO) -> BOOL {
    ToggleFGActivate_impl(pti)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn ToggleFGActivate(pti: PTHREADINFO) -> BOOL {
    ToggleFGActivate_impl(pti)
}

#[inline]
unsafe fn ToggleFGActivate_impl(pti: PTHREADINFO) -> BOOL {
    let ppi: PPROCESSINFO = (*pti).ppi;

    let mut Ret: BOOL = ((*pti).TIF_flags & TIF_ALLOWFOREGROUNDACTIVATE != 0) as BOOL;
    if Ret != 0 {
        (*pti).TIF_flags &= !TIF_ALLOWFOREGROUNDACTIVATE;
    } else {
        Ret = ((*ppi_w32pf_flags(ppi)) & W32PF_ALLOWFOREGROUNDACTIVATE != 0) as BOOL;
    }

    if Ret != 0 {
        (*ppi_w32pf_flags(ppi)) &= !W32PF_ALLOWFOREGROUNDACTIVATE;
    }
    Ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IsAllowedFGActive(pti: PTHREADINFO, Wnd: PWND) -> BOOL {
    IsAllowedFGActive_impl(pti, Wnd)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IsAllowedFGActive(pti: PTHREADINFO, Wnd: PWND) -> BOOL {
    IsAllowedFGActive_impl(pti, Wnd)
}

#[inline]
unsafe fn IsAllowedFGActive_impl(pti: PTHREADINFO, Wnd: PWND) -> BOOL {
    if ToggleFGActivate(pti) == 0
        || (*pti).rpdesk != gpdeskInputDesktop
        || (*pti).MessageQueue == gpqForeground
        || IsFGLocked() != 0
        || ((*Wnd).ExStyle & WS_EX_NOACTIVATE) != 0
    {
        return 0;
    }
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn CanForceFG(ppi: PPROCESSINFO) -> BOOL {
    CanForceFG_impl(ppi)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn CanForceFG(ppi: PPROCESSINFO) -> BOOL {
    CanForceFG_impl(ppi)
}

unsafe fn CanForceFG_impl(ppi: PPROCESSINFO) -> BOOL {
    if ptiLastInput.is_null()
        || (*ptiLastInput).ppi == ppi
        || gptiForeground.is_null()
        || (*gptiForeground).ppi == ppi
        || ((*ppi_w32pf_flags(ppi)) & (W32PF_ALLOWFOREGROUNDACTIVATE | W32PF_SETFOREGROUNDALLOWED)) != 0
        || gppiInputProvider == ppi
        || gpqForeground.is_null()
    {
        return 1;
    }
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntSetForegroundMessageQueue(
    Wnd: PWND,
    pti: PTHREADINFO,
    MouseActivate: BOOL,
    Type: DWORD,
) -> BOOL {
    co_IntSetForegroundMessageQueue_impl(Wnd, pti, MouseActivate, Type)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntSetForegroundMessageQueue(
    Wnd: PWND,
    pti: PTHREADINFO,
    MouseActivate: BOOL,
    Type: DWORD,
) -> BOOL {
    co_IntSetForegroundMessageQueue_impl(Wnd, pti, MouseActivate, Type)
}

unsafe fn co_IntSetForegroundMessageQueue_impl(
    Wnd: PWND,
    pti: PTHREADINFO,
    MouseActivate: BOOL,
    Type: DWORD,
) -> BOOL {

    let ptiChg: PTHREADINFO;
    let mut ptiPrev: PTHREADINFO;
    let mut pumq: PUSER_MESSAGE_QUEUE;
    let pumqChg: PUSER_MESSAGE_QUEUE;
    let mut pumqPrev: PUSER_MESSAGE_QUEUE;
    let Removed: BOOL;
    let Ret: BOOL = 1;

    if !Wnd.is_null() && VerifyWnd(Wnd).is_null() {
        return 0;
    }

    if gptiForeground.is_null() || ((*gptiForeground).TIF_flags & TIF_INCLEANUP) != 0 {
        ptiPrev = core::ptr::null_mut();
    } else {
        ptiPrev = gptiForeground;
    }

    if !Wnd.is_null() {
        ptiChg = head_pti(core::ptr::addr_of!((*Wnd).head));
        IntSetFocusMessageQueue((*head_pti(core::ptr::addr_of!((*Wnd).head))).MessageQueue);
        gptiForeground = head_pti(core::ptr::addr_of!((*Wnd).head));
    } else {
        ptiChg = core::ptr::null_mut();
        IntSetFocusMessageQueue(core::ptr::null_mut());
        gptiForeground = core::ptr::null_mut();
    }

    if gpqForegroundPrev != gpqForeground {
        pumqPrev = core::ptr::null_mut();
        if !ptiPrev.is_null() && ((*ptiPrev).TIF_flags & TIF_INCLEANUP) == 0 {
            pumqPrev = (*ptiPrev).MessageQueue;
        }

        pumq = if !pti.is_null() {
            (*pti).MessageQueue
        } else {
            core::ptr::null_mut()
        };

        if !pumqPrev.is_null() {
            if pumq != pumqPrev {
                let mut Msg: MSG = core::mem::zeroed();
                let hWndPrev: HWND = if !(*pumqPrev).spwndActive.is_null() {
                    UserHMGetHandle((*pumqPrev).spwndActive)
                } else {
                    core::ptr::null_mut()
                };
                let tid = if !gptiForeground.is_null() {
                    PsGetThreadId(pti_pEThread(gptiForeground))
                } else {
                    core::ptr::null_mut()
                };

                Msg.message = WM_ASYNC_SETACTIVEWINDOW as u32;
                Msg.hwnd = hWndPrev;
                Msg.wParam = (*pumqPrev).spwndActive as WPARAM;
                Msg.lParam = 0;
                Msg.time = 0;
                MsqPostMessage(
                    ptiPrev,
                    &mut Msg,
                    0,
                    QS_EVENT,
                    POSTEVENT_DAW,
                    tid as isize,
                );
            }
        }

        pumqChg = if !ptiChg.is_null() && ((*ptiChg).TIF_flags & TIF_INCLEANUP) == 0 {
            (*ptiChg).MessageQueue
        } else {
            core::ptr::null_mut()
        };

        pumq = if !pti.is_null() {
            (*pti).MessageQueue
        } else {
            core::ptr::null_mut()
        };

        if !pumqChg.is_null() {
            Removed = FindRemoveEventMsg(ptiChg, POSTEVENT_DAW, POSTEVENT_NONE);

            if pumqChg != pumq {
                let mut Msg: MSG = core::mem::zeroed();
                let hWnd: HWND = if !Wnd.is_null() {
                    UserHMGetHandle(Wnd)
                } else {
                    core::ptr::null_mut()
                };
                let tid = if !ptiPrev.is_null() {
                    PsGetThreadId(pti_pEThread(ptiPrev))
                } else {
                    core::ptr::null_mut()
                };

                if Removed != 0 {
                    (*pumqChg).QF_flags |= QF_EVENTDEACTIVATEREMOVED;
                }

                Msg.message = WM_ASYNC_SETACTIVEWINDOW as u32;
                Msg.hwnd = hWnd;
                Msg.wParam = Wnd as WPARAM;
                Msg.lParam = tid as LPARAM;
                Msg.time = 0;
                MsqPostMessage(
                    ptiChg,
                    &mut Msg,
                    0,
                    QS_EVENT,
                    POSTEVENT_SAW,
                    (Type | MouseActivate as DWORD) as isize,
                );
            } else {
                if (*pumq).spwndActive == Wnd {
                    co_IntSendMessage(
                        UserHMGetHandle(Wnd),
                        WM_NCACTIVATE,
                        1 as WPARAM,
                        UserHMGetHandle(Wnd) as LPARAM,
                    );

                    UpdateShellHook(Wnd);

                    co_WinPosSetWindowPos(
                        Wnd,
                        0 as HWND,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOMOVE,
                    );
                } else {
                    return co_IntSetActiveWindow(Wnd, MouseActivate, 1, 0);
                }
            }
        }

        pumqPrev = core::ptr::null_mut();
        if !ptiPrev.is_null() && ((*ptiPrev).TIF_flags & TIF_INCLEANUP) == 0 {
            pumqPrev = (*ptiPrev).MessageQueue;
        }
        pumq = if !pti.is_null() {
            (*pti).MessageQueue
        } else {
            core::ptr::null_mut()
        };

        if !pumqPrev.is_null() && pumq == pumqPrev {
            let tid = if !Wnd.is_null() {
                PsGetThreadId(pti_pEThread(head_pti(core::ptr::addr_of!((*Wnd).head))))
            } else {
                core::ptr::null_mut()
            };
            IntDeactivateWindow(pti, tid);
        }
    }
    Ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntSetForegroundAndFocusWindow(
    Wnd: PWND,
    MouseActivate: BOOL,
    bFlash: BOOL,
) -> BOOL {
    co_IntSetForegroundAndFocusWindow_impl(Wnd, MouseActivate, bFlash)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntSetForegroundAndFocusWindow(
    Wnd: PWND,
    MouseActivate: BOOL,
    bFlash: BOOL,
) -> BOOL {
    co_IntSetForegroundAndFocusWindow_impl(Wnd, MouseActivate, bFlash)
}

unsafe fn co_IntSetForegroundAndFocusWindow_impl(
    Wnd: PWND,
    MouseActivate: BOOL,
    _bFlash: BOOL,
) -> BOOL {
    let hWnd: HWND = if !Wnd.is_null() {
        UserHMGetHandle(Wnd)
    } else {
        core::ptr::null_mut()
    };
    let PrevForegroundQueue: PUSER_MESSAGE_QUEUE;
    let pti: PTHREADINFO;
    let mut Ret: BOOL = 0;

    if !Wnd.is_null() {
        ASSERT_REFS_CO(Wnd);
    }

    PrevForegroundQueue = IntGetFocusMessageQueue();
    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if !Wnd.is_null() && !PrevForegroundQueue.is_null() {
        if (*head_pti(core::ptr::addr_of!((*Wnd).head))).MessageQueue == PrevForegroundQueue {
            if (*pti).MessageQueue == PrevForegroundQueue {
                Ret = IntUserSetActiveWindow(Wnd, MouseActivate, 1, 0);
            } else if (*(*head_pti(core::ptr::addr_of!((*Wnd).head))).MessageQueue).spwndActive == Wnd {
                Ret = 1;
            } else {
                let mut Msg: MSG = core::mem::zeroed();
                let ptiNew: PTHREADINFO = head_pti(core::ptr::addr_of!((*Wnd).head));

                Msg.message = WM_ASYNC_SETACTIVEWINDOW;
                Msg.hwnd = hWnd;
                Msg.wParam = Wnd as WPARAM;
                Msg.lParam = 0;
                Msg.time = 0;
                MsqPostMessage(
                    ptiNew,
                    &mut Msg,
                    0,
                    QS_EVENT,
                    POSTEVENT_SAW,
                    MouseActivate as isize,
                );

                Ret = 1;
            }
            return Ret;
        }
    }

    if (((IsFGLocked() == 0) || (*pti).ppi == gppiInputProvider)
        && ((CanForceFG((*pti).ppi) != 0)
            || ((*pti).TIF_flags
                & (TIF_SYSTEMTHREAD | TIF_CSRSSTHREAD | TIF_ALLOWFOREGROUNDACTIVATE)
                != 0)))
        || (*pti).ppi == ppiScrnSaver
    {
        ToggleFGActivate(pti);

        return co_IntSetForegroundMessageQueue(Wnd, pti, MouseActivate, 0);
    }

    if Wnd.is_null() {
        return 0;
    }

    if (*pti).MessageQueue == (*head_pti(core::ptr::addr_of!((*Wnd).head))).MessageQueue {
        Ret = IntUserSetActiveWindow(Wnd, MouseActivate, 1, 0);
    } else if (*(*head_pti(core::ptr::addr_of!((*Wnd).head))).MessageQueue).spwndActive == Wnd {
    } else {
        let mut Msg: MSG = core::mem::zeroed();
        let ptiNew: PTHREADINFO = head_pti(core::ptr::addr_of!((*Wnd).head));

        Msg.message = WM_ASYNC_SETACTIVEWINDOW;
        Msg.hwnd = hWnd;
        Msg.wParam = Wnd as WPARAM;
        Msg.lParam = 0;
        Msg.time = 0;
        MsqPostMessage(
            ptiNew,
            &mut Msg,
            0,
            QS_EVENT,
            POSTEVENT_SAW,
            MouseActivate as isize,
        );
    }
    let _ = Ret;
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntSetActiveWindow(
    Wnd: PWND,
    bMouse: BOOL,
    bFocus: BOOL,
    Async: BOOL,
) -> BOOL {
    co_IntSetActiveWindow_impl(Wnd, bMouse, bFocus, Async)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntSetActiveWindow(
    Wnd: PWND,
    bMouse: BOOL,
    bFocus: BOOL,
    Async: BOOL,
) -> BOOL {
    co_IntSetActiveWindow_impl(Wnd, bMouse, bFocus, Async)
}

unsafe fn co_IntSetActiveWindow_impl(
    Wnd: PWND,
    bMouse: BOOL,
    bFocus: BOOL,
    Async: BOOL,
) -> BOOL {
    let pti: PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;
    let pWndChg: PWND;
    let mut WndPrev: PWND;
    let hWndPrev: HWND;
    let mut hWnd: HWND = core::ptr::null_mut();
    let InAAPM: BOOL;
    let mut cbt: CBTACTIVATESTRUCT = core::mem::zeroed();

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ThreadQueue = (*pti).MessageQueue;

    pWndChg = (*ThreadQueue).spwndActive;
    hWndPrev = if !pWndChg.is_null() {
        UserHMGetHandle(pWndChg)
    } else {
        core::ptr::null_mut()
    };

    if Wnd.is_null() || Wnd == UserGetDesktopWindow() {
        return 0;
    }

    ASSERT_REFS_CO(Wnd);
    hWnd = UserHMGetHandle(Wnd);

    if (*Wnd).ExStyle & WS_EX_NOACTIVATE != 0 {
        return 1;
    }

    if ThreadQueue != (*head_pti(core::ptr::addr_of!((*Wnd).head))).MessageQueue {
        return 0;
    }

    if VerifyWnd(Wnd).is_null() {
        return 0;
    }

    if Wnd == pWndChg {
        return 1;
    }

    if (*Wnd).state & WNDS_BEINGACTIVATED != 0 {
        return 1;
    }

    cbt.fMouse = bMouse;
    cbt.hWndActive = hWndPrev;
    if co_HOOK_CallHooks(
        WH_CBT as i32,
        HCBT_ACTIVATE as i32,
        hWnd as WPARAM,
        core::ptr::addr_of_mut!(cbt) as LPARAM,
    ) != 0
    {
        ERR!("SetActiveWindow: WH_CBT Call Hook return!\n");
        return 0;
    }

    (*ThreadQueue).QF_flags &= !QF_EVENTDEACTIVATEREMOVED;

    if !(*ThreadQueue).spwndActive.is_null()
        && (*(*ThreadQueue).spwndActive).state & WNDS_DESTROYED != 0
    {
        (*ThreadQueue).spwndActive = core::ptr::null_mut();
    } else {
        (*ThreadQueue).spwndActivePrev = (*ThreadQueue).spwndActive;
    }

    WndPrev = (*ThreadQueue).spwndActive;

    if !WndPrev.is_null() {
        if ThreadQueue == gpqForeground {
            gpqForegroundPrev = ThreadQueue;
        }
        if co_IntSendDeactivateMessages(UserHMGetHandle(WndPrev), hWnd, 1) == 0 {
            return 0;
        }
    }

    WndPrev = (*ThreadQueue).spwndActive;

    if (*ThreadQueue).spwndActivePrev != (*ThreadQueue).spwndActive
        || pWndChg != WndPrev
        || (!Wnd.is_null() && VerifyWnd(Wnd).is_null())
        || ThreadQueue != (*pti).MessageQueue
    {
        ERR!("SetActiveWindow: Summary ERROR, active state changed!\n");
        return 0;
    }

    if WndPrev.is_null() {
        (*ThreadQueue).QF_flags &= !QF_FOCUSNULLSINCEACTIVE;
    }

    (*ThreadQueue).spwndActive = Wnd;

    (*Wnd).state |= WNDS_BEINGACTIVATED;

    IntNotifyWinEvent(
        EVENT_SYSTEM_FOREGROUND,
        Wnd,
        OBJID_WINDOW as i32,
        CHILDID_SELF as i32,
        WEF_SETBYWNDPTI,
    );

    FindRemoveEventMsg(pti, POSTEVENT_SAW, POSTEVENT_DAW);

    WndPrev = VerifyWnd((*ThreadQueue).spwndActivePrev);

    InAAPM = co_IntSendActivateMessages(WndPrev, Wnd, bMouse, Async);

    if bFocus != 0 && (*ThreadQueue).QF_flags & QF_FOCUSNULLSINCEACTIVE == 0 {
        if (*(*pti).MessageQueue).spwndActive
            != IntGetNonChildAncestor((*(*pti).MessageQueue).spwndFocus)
        {
            let mut pWndSend: PWND = (*(*pti).MessageQueue).spwndActive;
            if !pWndSend.is_null()
                && (*(*(*pti).MessageQueue).spwndActive).style & WS_MINIMIZE != 0
            {
                pWndSend = core::ptr::null_mut();
            }
            IntSendFocusMessages(pti, pWndSend);
        }
    }

    if InAAPM != 0 {
        (*pti).TIF_flags &= !TIF_INACTIVATEAPPMSG;
    }

    (*ThreadQueue).QF_flags |= QF_ACTIVATIONCHANGE;

    (*Wnd).state &= !WNDS_BEINGACTIVATED;
    ((*ThreadQueue).spwndActive == Wnd) as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntUserSetActiveWindow(
    Wnd: PWND,
    bMouse: BOOL,
    bFocus: BOOL,
    Async: BOOL,
) -> BOOL {
    IntUserSetActiveWindow_impl(Wnd, bMouse, bFocus, Async)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntUserSetActiveWindow(
    Wnd: PWND,
    bMouse: BOOL,
    bFocus: BOOL,
    Async: BOOL,
) -> BOOL {
    IntUserSetActiveWindow_impl(Wnd, bMouse, bFocus, Async)
}

#[inline]
unsafe fn IntUserSetActiveWindow_impl(
    Wnd: PWND,
    bMouse: BOOL,
    bFocus: BOOL,
    Async: BOOL,
) -> BOOL {
    let pti: PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;

    if ((*Wnd).style & (WS_POPUP | WS_CHILD)) == WS_CHILD {
        return 0;
    }

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ThreadQueue = (*pti).MessageQueue;

    loop {
        if Wnd.is_null() {
            break;
        }

        let Ret: BOOL;
        let DoFG: BOOL;
        let AllowFG: BOOL;

        if ThreadQueue == (*head_pti(core::ptr::addr_of!((*Wnd).head))).MessageQueue {
            if IsAllowedFGActive(pti, Wnd) != 0 {
                DoFG = 1;
            } else {
                break;
            }
            AllowFG = if (*pti).cVisWindows != 0 { 0 } else { 1 };
        } else {
            if gpqForeground.is_null() || gpqForeground == ThreadQueue {
                DoFG = 1;
            } else {
                DoFG = 0;
            }
            if DoFG != 0 {
                if ((*pti).TIF_flags & TIF_ALLOWFOREGROUNDACTIVATE) != 0 || (*pti).cVisWindows != 0 {
                    AllowFG = 1;
                } else {
                    AllowFG = 0;
                }
            } else {
                AllowFG = 0;
            }
        }
        Ret = if DoFG != 0 {
            (*pti).TIF_flags |= TIF_ALLOWFOREGROUNDACTIVATE;
            let r = co_IntSetForegroundAndFocusWindow(Wnd, bMouse, 1);
            if AllowFG != 0 {
                (*pti).TIF_flags |= TIF_ALLOWFOREGROUNDACTIVATE;
            } else {
                (*pti).TIF_flags &= !TIF_ALLOWFOREGROUNDACTIVATE;
            }
            r
        } else {
            0
        };
        return Ret;
    }

    co_IntSetActiveWindow(Wnd, bMouse, bFocus, Async)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntMouseActivateWindow(Wnd: PWND) -> BOOL {
    co_IntMouseActivateWindow_impl(Wnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntMouseActivateWindow(Wnd: PWND) -> BOOL {
    co_IntMouseActivateWindow_impl(Wnd)
}

unsafe fn co_IntMouseActivateWindow_impl(Wnd: PWND) -> BOOL {
    TRACE!(b"Mouse Active\n\0".as_ptr() as *const i8);
    if !Wnd.is_null() && ((*Wnd).ExStyle & WS_EX_NOACTIVATE) != 0 {
        return 1;
    }
    co_IntSetForegroundAndFocusWindow(Wnd, 1, 1)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserSetActiveWindow(Wnd: PWND) -> BOOL {
    UserSetActiveWindow_impl(Wnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserSetActiveWindow(Wnd: PWND) -> BOOL {
    UserSetActiveWindow_impl(Wnd)
}

unsafe fn UserSetActiveWindow_impl(mut Wnd: PWND) -> BOOL {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if !Wnd.is_null() {
        if ((*Wnd).style & (WS_POPUP | WS_CHILD)) == WS_CHILD {
            return 0;
        }

        return IntUserSetActiveWindow(Wnd, 0, 1, 0);
    }

    if !gpqForegroundPrev.is_null()
        && !(*gpqForegroundPrev).spwndActivePrev.is_null()
        && ((*(*gpqForegroundPrev).spwndActivePrev).style & (WS_VISIBLE | WS_DISABLED)) == WS_VISIBLE
        && ((*(*gpqForegroundPrev).spwndActivePrev).state2 & WNDS2_BOTTOMMOST) == 0
        && {
            Wnd = VerifyWnd((*gpqForegroundPrev).spwndActivePrev);
            !Wnd.is_null()
        }
    {
        return IntUserSetActiveWindow(Wnd, 0, 1, 0);
    }

    if !(*(*pti).MessageQueue).spwndActive.is_null()
        && {
            Wnd = VerifyWnd((*(*pti).MessageQueue).spwndActive);
            !Wnd.is_null()
        }
    {
        if ActivateOtherWindowMin(Wnd) == 0 {
            co_WinPosActivateOtherWindow(Wnd);
        }
        return 1;
    }

    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_UserSetFocus(Window: PWND) -> HWND {
    co_UserSetFocus_impl(Window)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_UserSetFocus(Window: PWND) -> HWND {
    co_UserSetFocus_impl(Window)
}

unsafe fn co_UserSetFocus_impl(Window: PWND) -> HWND {
    let mut hWndPrev: HWND = core::ptr::null_mut();
    let mut pwndTop: PWND;
    let pti: PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ThreadQueue = (*pti).MessageQueue;

    hWndPrev = if !(*ThreadQueue).spwndFocus.is_null() {
        UserHMGetHandle((*ThreadQueue).spwndFocus)
    } else {
        core::ptr::null_mut()
    };

    if !Window.is_null() {
        if hWndPrev == UserHMGetHandle(Window) {
            return if !hWndPrev.is_null() {
                if IntIsWindow(hWndPrev) != 0 {
                    hWndPrev
                } else {
                    core::ptr::null_mut()
                }
            } else {
                core::ptr::null_mut()
            };
        }

        if (*head_pti(core::ptr::addr_of!((*Window).head))).MessageQueue != ThreadQueue {
            ERR!("SetFocus Must have the same Q!\n");
            return core::ptr::null_mut();
        }

        pwndTop = Window;
        while !pwndTop.is_null() {
            if (*pwndTop).style & (WS_MINIMIZED | WS_DISABLED) != 0 {
                return core::ptr::null_mut();
            }
            if (*pwndTop).style & (WS_POPUP | WS_CHILD) != WS_CHILD {
                break;
            }
            if (*pwndTop).spwndParent.is_null() {
                break;
            }
            pwndTop = (*pwndTop).spwndParent;
        }

        if co_HOOK_CallHooks(
            WH_CBT as i32,
            HCBT_SETFOCUS as i32,
            UserHMGetHandle(Window) as WPARAM,
            hWndPrev as LPARAM,
        ) != 0
        {
            ERR!("SetFocus 1 WH_CBT Call Hook return!\n");
            return core::ptr::null_mut();
        }

        if pwndTop != (*ThreadQueue).spwndActive {
            let ForegroundQueue: PUSER_MESSAGE_QUEUE = IntGetFocusMessageQueue();
            if ThreadQueue != ForegroundQueue && IsAllowedFGActive(pti, pwndTop) != 0 {
                if (*pwndTop).style & WS_VISIBLE == 0 {
                    (*ppi_w32pf_flags((*pti).ppi)) |= W32PF_ALLOWFOREGROUNDACTIVATE;
                }
                if co_IntSetForegroundAndFocusWindow(pwndTop, 0, 1) == 0 {
                    ERR!("SetFocus: Set Foreground and Focus Failed!\n");
                    return core::ptr::null_mut();
                }
            }

            if pwndTop != (*ThreadQueue).spwndActive {
                if co_IntSetActiveWindow(pwndTop, 0, 0, 0) == 0 {
                    ERR!("SetFocus: Set Active Failed!\n");
                    return core::ptr::null_mut();
                }
            }

            if (*Window).state2 & WNDS2_INDESTROY != 0 {
                return core::ptr::null_mut();
            }
            if pwndTop != (*ThreadQueue).spwndActive {
                ERR!("SetFocus: Top window did not go active!\n");
                return core::ptr::null_mut();
            }
        }

        hWndPrev = if !(*ThreadQueue).spwndFocus.is_null() {
            UserHMGetHandle((*ThreadQueue).spwndFocus)
        } else {
            core::ptr::null_mut()
        };

        IntSendFocusMessages(pti, Window);
    } else {
        if co_HOOK_CallHooks(
            WH_CBT as i32,
            HCBT_SETFOCUS as i32,
            0 as WPARAM,
            hWndPrev as LPARAM,
        ) != 0
        {
            ERR!("SetFocus: 2 WH_CBT Call Hook return!\n");
            return core::ptr::null_mut();
        }
        IntSendFocusMessages(pti, core::ptr::null_mut());
    }

    if !hWndPrev.is_null() {
        if IntIsWindow(hWndPrev) != 0 {
            hWndPrev
        } else {
            core::ptr::null_mut()
        }
    } else {
        core::ptr::null_mut()
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetForegroundWindow() -> HWND {
    UserGetForegroundWindowImpl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetForegroundWindow() -> HWND {
    UserGetForegroundWindowImpl()
}

unsafe fn UserGetForegroundWindowImpl() -> HWND {

    let ForegroundQueue: PUSER_MESSAGE_QUEUE = IntGetFocusMessageQueue();
    if ForegroundQueue.is_null() {
        return core::ptr::null_mut();
    }
    if !(*ForegroundQueue).spwndActive.is_null() {
        UserHMGetHandle((*ForegroundQueue).spwndActive as *mut core::ffi::c_void)
    } else {
        core::ptr::null_mut()
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetActiveWindow() -> HWND {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;
    if ThreadQueue.is_null() {
        return core::ptr::null_mut();
    }
    if !(*ThreadQueue).spwndActive.is_null() {
        UserHMGetHandle((*ThreadQueue).spwndActive)
    } else {
        core::ptr::null_mut()
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetActiveWindow() -> HWND {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;
    if ThreadQueue.is_null() {
        return core::ptr::null_mut();
    }
    if !(*ThreadQueue).spwndActive.is_null() {
        UserHMGetHandle((*ThreadQueue).spwndActive)
    } else {
        core::ptr::null_mut()
    }
}
#[no_mangle]
pub unsafe extern "system" fn IntGetCapture() -> HWND {
    let pti: PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;
    let Ret: HWND;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ThreadQueue = (*pti).MessageQueue;
    Ret = if !ThreadQueue.is_null() && !(*ThreadQueue).spwndCapture.is_null() {
        UserHMGetHandle((*ThreadQueue).spwndCapture)
    } else {
        core::ptr::null_mut()
    };

    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_UserSetCapture(hWnd: HWND) -> HWND {
    co_UserSetCapture_impl(hWnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_UserSetCapture(hWnd: HWND) -> HWND {
    co_UserSetCapture_impl(hWnd)
}

unsafe fn co_UserSetCapture_impl(hWnd: HWND) -> HWND {
    let pti: PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;
    let pWnd: PWND;
    let mut Window: PWND = core::ptr::null_mut();
    let hWndPrev: HWND;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ThreadQueue = (*pti).MessageQueue;

    if (*ThreadQueue).QF_flags & QF_CAPTURELOCKED != 0 {
        return core::ptr::null_mut();
    }

    if !hWnd.is_null() {
        Window = UserGetWindowObject(hWnd);
        if !Window.is_null() {
            if (*head_pti(core::ptr::addr_of!((*Window).head))).MessageQueue != ThreadQueue {
                ERR!("Window Thread does not match Current!\n");
                return core::ptr::null_mut();
            }
        }
    }

    hWndPrev = MsqSetStateWindow(pti, MSQ_STATE_CAPTURE, hWnd);

    if !hWndPrev.is_null() {
        pWnd = UserGetWindowObject(hWndPrev);
        if !pWnd.is_null() {
            IntNotifyWinEvent(EVENT_SYSTEM_CAPTUREEND, pWnd, OBJID_WINDOW as LONG, CHILDID_SELF as LONG, WEF_SETBYWNDPTI);
        }
    }

    if !Window.is_null() {
        IntNotifyWinEvent(EVENT_SYSTEM_CAPTURESTART, Window, OBJID_WINDOW as LONG, CHILDID_SELF as LONG, WEF_SETBYWNDPTI);
    }

    if !hWndPrev.is_null() {
        if !(*ThreadQueue).MenuOwner.is_null() && !Window.is_null() {
            (*ThreadQueue).QF_flags |= QF_CAPTURELOCKED;
        }

        co_IntSendMessage(hWndPrev, WM_CAPTURECHANGED, 0, hWnd as LPARAM);

        (*ThreadQueue).QF_flags &= !QF_CAPTURELOCKED;
    }

    if hWnd.is_null() {
        let mut mi: MOUSEINPUT = core::mem::zeroed();
        MsqSetStateWindow(pti, MSQ_STATE_MENUOWNER, core::ptr::null_mut());
        MsqSetStateWindow(pti, MSQ_STATE_MOVESIZE, core::ptr::null_mut());
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = MOUSEEVENTF_MOVE;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        UserSendMouseInput(&mut mi, 0);
    }
    hWndPrev
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntReleaseCapture() -> BOOL {
    IntReleaseCapture_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntReleaseCapture() -> BOOL {
    IntReleaseCapture_impl()
}

unsafe fn IntReleaseCapture_impl() -> BOOL {
    let pti: PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ThreadQueue = (*pti).MessageQueue;

    if (*ThreadQueue).QF_flags & QF_CAPTURELOCKED != 0 {
        return 0;
    }

    co_UserSetCapture(core::ptr::null_mut());

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntSetForegroundWindow(Window: PWND) -> BOOL {
    co_IntSetForegroundWindow_impl(Window)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntSetForegroundWindow(Window: PWND) -> BOOL {
    co_IntSetForegroundWindow_impl(Window)
}

unsafe fn co_IntSetForegroundWindow_impl(Window: PWND) -> BOOL {
    if !Window.is_null() {
        ASSERT_REFS_CO(Window);
    }

    co_IntSetForegroundAndFocusWindow(Window, 0, 1)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntSetForegroundWindowMouse(Window: PWND) -> BOOL {
    co_IntSetForegroundWindowMouse_impl(Window)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntSetForegroundWindowMouse(Window: PWND) -> BOOL {
    co_IntSetForegroundWindowMouse_impl(Window)
}

unsafe fn co_IntSetForegroundWindowMouse_impl(Window: PWND) -> BOOL {
    if !Window.is_null() {
        ASSERT_REFS_CO(Window);
    }

    co_IntSetForegroundAndFocusWindow(Window, 1 as BOOL, 0 as BOOL)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntLockSetForegroundWindow(uLockCode: UINT) -> BOOL {
    IntLockSetForegroundWindow_impl(uLockCode)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntLockSetForegroundWindow(uLockCode: UINT) -> BOOL {
    IntLockSetForegroundWindow_impl(uLockCode)
}

unsafe fn IntLockSetForegroundWindow_impl(uLockCode: UINT) -> BOOL {
    let mut Error: ULONG = ERROR_ACCESS_DENIED;
    let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    match uLockCode {
        LSFW_LOCK => {
            if CanForceFG(ppi) != 0 && gppiLockSFW.is_null() {
                gppiLockSFW = ppi;
                return 1;
            }
        }
        LSFW_UNLOCK => {
            if gppiLockSFW == ppi {
                gppiLockSFW = core::ptr::null_mut();
                return 1;
            }
        }
        _ => {
            Error = ERROR_INVALID_PARAMETER;
        }
    }
    EngSetLastError(Error);
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntAllowSetForegroundWindow(dwProcessId: DWORD) -> BOOL {
    IntAllowSetForegroundWindow_impl(dwProcessId)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntAllowSetForegroundWindow(dwProcessId: DWORD) -> BOOL {
    IntAllowSetForegroundWindow_impl(dwProcessId)
}

unsafe fn IntAllowSetForegroundWindow_impl(dwProcessId: DWORD) -> BOOL {
    let mut ppi: PPROCESSINFO;
    let ppiCur: PPROCESSINFO;
    let mut Process: PEPROCESS = core::ptr::null_mut();

    ppi = core::ptr::null_mut();
    if dwProcessId != ASFW_ANY {
        if !NT_SUCCESS(PsLookupProcessByProcessId(
            UlongToHandle(dwProcessId),
            &mut Process as *mut _ as *mut _,
        )) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return FALSE as BOOL;
        }
        ppi = PsGetProcessWin32Process(Process as _) as PPROCESSINFO;
        if ppi.is_null() {
            ObDereferenceObject(Process as _);
            return FALSE as BOOL;
        }
    }
    ppiCur = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    if CanForceFG(ppiCur) == 0 {
        if !Process.is_null() {
            ObDereferenceObject(Process as _);
        }
        EngSetLastError(ERROR_ACCESS_DENIED);
        return FALSE as BOOL;
    }
    if dwProcessId == ASFW_ANY {
        ptiLastInput = core::ptr::null_mut();
    } else {
        ObDereferenceObject(Process as _);
    }
    TRUE as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetForegroundWindow() -> HWND {
    UserEnterExclusive();

    let Ret = UserGetForegroundWindow();

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetActiveWindow(hWnd: HWND) -> HWND {
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let hWndPrev: HWND;
    let mut Window: PWND;
    let pwndPrev: PWND;
    let mut Ret: HWND = core::ptr::null_mut();

    UserEnterExclusive();

    'exit: {
        Window = core::ptr::null_mut();
        if !hWnd.is_null() {
            Window = UserGetWindowObject(hWnd);
            if Window.is_null() {
                break 'exit;
            }
        }

        if Window.is_null()
            || (*head_pti(core::ptr::addr_of!((*Window).head))).MessageQueue == (*gptiCurrent).MessageQueue
        {
            pwndPrev = (*(*gptiCurrent).MessageQueue).spwndActive;
            hWndPrev = if !pwndPrev.is_null() {
                UserHMGetHandle(pwndPrev)
            } else {
                core::ptr::null_mut()
            };
            if !Window.is_null() {
                UserRefObjectCo(Window, core::ptr::addr_of_mut!(Ref));
            }
            UserSetActiveWindow(Window);
            if !Window.is_null() {
                UserDerefObjectCo(Window);
            }
            Ret = if !hWndPrev.is_null() && IntIsWindow(hWndPrev) != 0 {
                hWndPrev
            } else {
                core::ptr::null_mut()
            };
        }
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetCapture(hWnd: HWND) -> HWND {
    let Ret: HWND;

    UserEnterExclusive();

    Ret = co_UserSetCapture(hWnd);

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetFocus(hWnd: HWND) -> HWND {
    let mut Window: PWND;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut ret: HWND = core::ptr::null_mut();

    UserEnterExclusive();

    'exit: {
        if !hWnd.is_null() {
            Window = UserGetWindowObject(hWnd);
            if Window.is_null() {
                break 'exit;
            }

            UserRefObjectCo(Window, &mut Ref);
            ret = co_UserSetFocus(Window);
            UserDerefObjectCo(Window);
        } else {
            ret = co_UserSetFocus(core::ptr::null_mut());
        }
    }

    UserLeave();
    ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          Focus functions
//  * FILE:             win32ss/user/ntuser/focus.c
//  * PROGRAMER:        ReactOS Team
//  */
//
// #include <win32k.h>
// #include <immdev.h>
// DBG_DEFAULT_CHANNEL(UserFocus);
//
// PUSER_MESSAGE_QUEUE gpqForeground = NULL;
// PUSER_MESSAGE_QUEUE gpqForegroundPrev = NULL;
// PTHREADINFO gptiForeground = NULL;
// PPROCESSINFO gppiLockSFW = NULL;
// ULONG guSFWLockCount = 0; // Rule #8, No menus are active. So should be zero.
// PTHREADINFO ptiLastInput = NULL;
// HWND ghwndOldFullscreen = NULL;
//
// /*
//  * Check locking of a process or one or more menus are active.
//  */
// BOOL FASTCALL
// IsFGLocked(VOID)
// {
//     return (gppiLockSFW || guSFWLockCount);
// }
//
// /*
//  * Get capture window via foreground Queue.
//  */
// HWND FASTCALL
// IntGetCaptureWindow(VOID)
// {
//     PUSER_MESSAGE_QUEUE ForegroundQueue = IntGetFocusMessageQueue();
//     if (!ForegroundQueue)
//         return NULL;
//     return (ForegroundQueue->spwndCapture ? UserHMGetHandle(ForegroundQueue->spwndCapture) : NULL);
// }
//
// HWND FASTCALL
// IntGetThreadFocusWindow(VOID)
// {
//     PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//     PUSER_MESSAGE_QUEUE ThreadQueue = pti->MessageQueue;
//     if (!ThreadQueue)
//         return NULL;
//     return (ThreadQueue->spwndFocus ? UserHMGetHandle(ThreadQueue->spwndFocus) : NULL);
// }
//
// BOOL FASTCALL IntIsWindowFullscreen(PWND Window)
// {
//     RECTL rclAnd, rclMonitor, rclWindow;
//     PMONITOR pMonitor;
//
//     if (!Window || !(Window->style & WS_VISIBLE) || (Window->style & WS_CHILD) ||
//         !IntGetWindowRect(Window, &rclWindow))
//     {
//         return FALSE;
//     }
//
//     pMonitor = UserGetPrimaryMonitor();
//     if (!pMonitor)
//     {
//         RECTL_vSetRect(&rclMonitor, 0, 0,
//                        UserGetSystemMetrics(SM_CXSCREEN), UserGetSystemMetrics(SM_CYSCREEN));
//     }
//     else
//     {
//         rclMonitor = *(LPRECTL)&pMonitor->rcMonitor;
//     }
//
//     RECTL_bIntersectRect(&rclAnd, &rclMonitor, &rclWindow);
//     return RtlEqualMemory(&rclAnd, &rclMonitor, sizeof(RECTL));
// }
//
// BOOL FASTCALL IntCheckFullscreen(PWND Window)
// {
//     HWND hWnd;
//
//     if (ghwndOldFullscreen && !IntIsWindowFullscreen(ValidateHwndNoErr(ghwndOldFullscreen)))
//         ghwndOldFullscreen = NULL;
//
//     if (!IntIsWindowFullscreen(Window))
//         return FALSE;
//
//     hWnd = UserHMGetHandle(Window);
//     if (ghwndOldFullscreen != hWnd)
//     {
//         co_IntShellHookNotify(HSHELL_RUDEAPPACTIVATED, (WPARAM)hWnd, TRUE);
//         ghwndOldFullscreen = hWnd;
//     }
//     return TRUE;
// }
//
// VOID FASTCALL
// UpdateShellHook(PWND Window)
// {
//    if (IntCheckFullscreen(Window))
//        return;
//
//    if ( Window->spwndParent == UserGetDesktopWindow() &&
//        (!(Window->ExStyle & WS_EX_TOOLWINDOW) ||
//          (Window->ExStyle & WS_EX_APPWINDOW)))
//    {
//        // FIXME lParam; The value is TRUE if the window is in full-screen mode, or FALSE otherwise.
//        co_IntShellHookNotify(HSHELL_WINDOWACTIVATED, (WPARAM) UserHMGetHandle(Window), FALSE);
//    }
//    else
//    {
//        co_IntShellHookNotify(HSHELL_WINDOWACTIVATED, 0, FALSE);
//    }
// }
//
// BOOL FASTCALL
// co_IntSendDeactivateMessages(HWND hWndPrev, HWND hWnd, BOOL Clear)
// {
//    USER_REFERENCE_ENTRY RefPrev;
//    PWND WndPrev;
//    BOOL Ret = TRUE;
//    LPARAM lParam = hWnd ? (LPARAM)hWnd : 0;
//
//    if (hWndPrev && (WndPrev = ValidateHwndNoErr(hWndPrev)))
//    {
//       UserRefObjectCo(WndPrev, &RefPrev);
//
//       if (co_IntSendMessage(hWndPrev, WM_NCACTIVATE, FALSE, lParam))
//       {
//          co_IntSendMessage(hWndPrev, WM_ACTIVATE,
//                     MAKEWPARAM(WA_INACTIVE, (WndPrev->style & WS_MINIMIZE) != 0),
//                     (LPARAM)hWnd);
//
//          if (WndPrev && Clear)
//             WndPrev->state &= ~(WNDS_ACTIVEFRAME|WNDS_HASCAPTION);
//       }
//       else
//       {
//          ERR("Application is keeping itself Active to prevent the change!\n");
//          Ret = FALSE;
//       }
//
//       UserDerefObjectCo(WndPrev);
//    }
//    return Ret;
// }
//
// // Win: xxxFocusSetInputContext
// VOID IntFocusSetInputContext(PWND pWnd, BOOL bActivate, BOOL bCallback)
// {
//     PTHREADINFO pti;
//     PWND pImeWnd;
//     USER_REFERENCE_ENTRY Ref;
//     HWND hImeWnd;
//     WPARAM wParam;
//     LPARAM lParam;
//
//     if (!pWnd || !pWnd->pcls || IS_WND_IMELIKE(pWnd))
//         return;
//
//     pti = pWnd->head.pti;
//     if (!pti || (pti->TIF_flags & TIF_INCLEANUP))
//         return;
//
//     pImeWnd = pti->spwndDefaultIme;
//     if (!pImeWnd)
//         return;
//
//     UserRefObjectCo(pImeWnd, &Ref);
//
//     hImeWnd = UserHMGetHandle(pImeWnd);
//     wParam = (bActivate ? IMS_IMEACTIVATE : IMS_IMEDEACTIVATE);
//     lParam = (LPARAM)UserHMGetHandle(pWnd);
//
//     if (bCallback)
//         co_IntSendMessageWithCallBack(hImeWnd, WM_IME_SYSTEM, wParam, lParam, NULL, 0, NULL);
//     else
//         co_IntSendMessage(hImeWnd, WM_IME_SYSTEM, wParam, lParam);
//
//     UserDerefObjectCo(pImeWnd);
// }
//
// //
// // Deactivating the foreground message queue.
// //
// // Release Active, Capture and Focus Windows associated with this message queue.
// //
// // Win: xxxDeactivate
// BOOL FASTCALL
// IntDeactivateWindow(PTHREADINFO pti, HANDLE tid)
// {
//    USER_REFERENCE_ENTRY Ref;
//    PTHREADINFO ptiPrev;
//    PWND pwndPrev;
//    BOOL InAAPM = FALSE;
//    PTHREADINFO ptiCurrent = PsGetCurrentThreadWin32Thread();
//
//    if ( !pti->MessageQueue->spwndActive )
//    {
//       TRACE("IDAW E : Nothing to do, Active is NULL! pti 0x%p tid 0x%p\n",pti,tid);
//       return TRUE;
//    }
//
//    TRACE("IDAW : pti 0x%p tid 0x%p\n",pti,tid);
//
//    if (ptiCurrent != pti)
//    {
//       IntReferenceThreadInfo(pti);
//       IntReferenceThreadInfo(ptiCurrent);
//    }
//
//    if (!(pti->TIF_flags & TIF_INACTIVATEAPPMSG) )
//    {
//       pti->TIF_flags |= TIF_INACTIVATEAPPMSG;
//       InAAPM = TRUE;
//    }
//
//    //
//    // Check for Capture and release it.
//    //
//    if ( pti->MessageQueue->spwndCapture )
//    {
//       MSG msg;
//       PWND pwndCapture = pti->MessageQueue->spwndCapture;
//
//       UserRefObjectCo(pwndCapture, &Ref);
//       co_IntSendMessage(UserHMGetHandle(pwndCapture), WM_CANCELMODE, 0, 0);
//       UserDerefObjectCo(pwndCapture);
//
//       /* Generate mouse move message */
//       msg.message = WM_MOUSEMOVE;
//       msg.wParam = UserGetMouseButtonsState();
//       msg.lParam = MAKELPARAM(gpsi->ptCursor.x, gpsi->ptCursor.y);
//       msg.pt = gpsi->ptCursor;
//       co_MsqInsertMouseMessage(&msg, 0, 0, TRUE);
//    }
//
//    //
//    // Check for Active and release it.
//    //
//    if ( pti->MessageQueue->spwndActive )
//    {
//       pwndPrev = pti->MessageQueue->spwndActive;
//       ptiPrev = pwndPrev->head.pti;
//
//       if (!co_IntSendDeactivateMessages(UserHMGetHandle(pwndPrev), 0, TRUE))
//       {
//          if (InAAPM) pti->TIF_flags &= ~TIF_INACTIVATEAPPMSG;
//          if (ptiCurrent != pti)
//          {
//             IntDereferenceThreadInfo(pti);
//             IntDereferenceThreadInfo(ptiCurrent);
//          }
//          return FALSE;
//       }
//
//       if ( pti->MessageQueue->spwndActive == pwndPrev )
//       {
//           pti->MessageQueue->spwndActivePrev = pwndPrev;
//           pti->MessageQueue->spwndActive     = NULL;
//       }
//
//       if (ptiPrev->TIF_flags & TIF_INCLEANUP) ptiPrev = NULL;
//    }
//    else
//    {
//       ptiPrev = pti;
//       pwndPrev = (PWND)-1; // Avoid zero Active window.
//    }
//
//    if ( ptiPrev )
//    {
//       HANDLE OldTID = PsGetThreadId(ptiPrev->pEThread);
//       PWND cWindow;
//       HWND *List, *phWnd;
//
//       List = IntWinListChildren(UserGetDesktopWindow());
//       if ( List )
//       {
//          if ( OldTID )
//          {
//             for (phWnd = List; *phWnd; ++phWnd)
//             {
//                cWindow = ValidateHwndNoErr(*phWnd);
//                if ( cWindow && cWindow->head.pti == ptiPrev )
//                { // FALSE if the window is being deactivated,
//                  // ThreadId that owns the window being activated.
//                  //ERR("IDW : WM_ACTIVATEAPP(0) hwnd %p tid Old %p New %p\n",UserHMGetHandle(cWindow),OldTID,tid);
//                  UserRefObjectCo(cWindow, &Ref);
//                  co_IntSendMessage(*phWnd, WM_ACTIVATEAPP, FALSE, (LPARAM)tid);
//                  UserDerefObjectCo(cWindow);
//                }
//             }
//          }
//          ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
//       }
//    }
//
//    //
//    // Now check for a change (Bounce), if Active same as previous window, release it too.
//    //
//    if ( pti->MessageQueue->spwndActive == pwndPrev )
//    {
//       if (!co_IntSendDeactivateMessages(UserHMGetHandle(pwndPrev), 0, FALSE))
//       {
//          if (InAAPM) pti->TIF_flags &= ~TIF_INACTIVATEAPPMSG;
//          if (ptiCurrent != pti)
//          {
//             IntDereferenceThreadInfo(pti);
//             IntDereferenceThreadInfo(ptiCurrent);
//          }
//          return FALSE;
//       }
//
//       if ( pti->MessageQueue->spwndActive == pwndPrev )
//       {
//           pti->MessageQueue->spwndActivePrev = pwndPrev;
//           pti->MessageQueue->spwndActive     = NULL;
//       }
//    }
//
//    //
//    // Check for Focus and release it.
//    //
//    if ( pti->MessageQueue->spwndFocus )
//    {
//       PWND pwndFocus = pti->MessageQueue->spwndFocus;
//
//       //
//       // Fix win.c:test_SetForegroundWindow:SetActiveWindow(0)!
//       //
//       pti->MessageQueue->spwndFocus = NULL; // Null out Focus.
//
//       UserRefObjectCo(pwndFocus, &Ref);
//       co_IntSendMessage(UserHMGetHandle(pwndFocus), WM_KILLFOCUS, 0, 0);
//       if (IS_IMM_MODE())
//       {
//          IntFocusSetInputContext(pwndFocus, FALSE, FALSE);
//       }
//       UserDerefObjectCo(pwndFocus);
//    }
//
//    /* Check for keyboard modifiers and release them (CORE-14768) */
//    MsqReleaseModifierKeys(pti->MessageQueue);
//
//    if (InAAPM) pti->TIF_flags &= ~TIF_INACTIVATEAPPMSG;
//    if (ptiCurrent != pti)
//    {
//       IntDereferenceThreadInfo(pti);
//       IntDereferenceThreadInfo(ptiCurrent);
//    }
//    return TRUE;
// }
//
// //
// // Activating another threads foreground window after a switch.
// //
// VOID FASTCALL
// IntActivateWindow(PWND Wnd, PTHREADINFO pti, HANDLE tid, DWORD Type)
// {
//    USER_REFERENCE_ENTRY Ref;
//    PUSER_MESSAGE_QUEUE pmq = pti->MessageQueue;
//
//    if (Wnd)
//    {
//       Wnd = VerifyWnd(Wnd);
//
//       if (!Wnd) return;
//
//       UserRefObjectCo(Wnd, &Ref);
//
//       if (!gpqForeground)
//       {
//           // No foreground queue set.
//           co_IntSetForegroundMessageQueue( Wnd, pti, (BOOL)Type, 0);
//       }
//       else
//       {
//           // Same Active and Wnd.
//           if ( pmq->spwndActive == Wnd )
//           {
//               WPARAM wParam = (Wnd->head.pti->MessageQueue == gpqForeground);
//
//               co_IntSendMessage( UserHMGetHandle(Wnd), WM_NCACTIVATE, wParam, 0);
//
//               if (wParam)
//               {
//                   UpdateShellHook(Wnd);
//
//                   co_WinPosSetWindowPos(Wnd, HWND_TOP, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);
//               }
//           }
//           else // Not the same, set the active Wnd.
//           {
//               co_IntSetActiveWindow(Wnd,(BOOL)Type,TRUE,TRUE);
//           }
//       }
//       UserDerefObjectCo(Wnd);
//    }
//    else // Handle no Wnd!
//    {
//       if ( tid &&                  // TID,
//            pmq->spwndActive &&     // Active WND not zero,
//            gpqForeground == pmq )  // Same message queues.
//       {
//            Wnd = pmq->spwndActive; // Use active window from current queue.
//
//            UserRefObjectCo(Wnd, &Ref);
//
//            co_IntSendMessage( UserHMGetHandle(Wnd), WM_NCACTIVATE, TRUE, 0);
//
//            UpdateShellHook(Wnd);
//
//            co_WinPosSetWindowPos(Wnd, HWND_TOP, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);
//
//            UserDerefObjectCo(Wnd);
//       }
//       else if (gpqForeground != pmq)
//       {
//            // Not the same message queue so clear flags for foreground switching.
//            pti->TIF_flags &= ~TIF_ALLOWFOREGROUNDACTIVATE;
//            pti->ppi->W32PF_flags &= ~W32PF_ALLOWFOREGROUNDACTIVATE;
//       }
//    }
// }
//
// BOOL FASTCALL
// co_IntMakeWindowActive(PWND Window)
// {
//    PWND spwndOwner;
//    if (VerifyWnd(Window))
//    {  // Set last active for window and it's owner.
//       spwndOwner = Window;
//       while (spwndOwner->spwndOwner)
//       {
//          spwndOwner = spwndOwner->spwndOwner;
//       }
//       WndSetLastActive(spwndOwner, Window);
//       return TRUE;
//    }
//    ERR("MakeWindowActive Failed!\n");
//    return FALSE;
// }
//
// BOOL FASTCALL
// co_IntSendActivateMessages(PWND WindowPrev, PWND Window, BOOL MouseActivate, BOOL Async)
// {
//    USER_REFERENCE_ENTRY Ref, RefPrev, RefCall;
//    HANDLE OldTID, NewTID;
//    PTHREADINFO pti, ptiOld, ptiNew;
//    BOOL InAAPM = FALSE;
//
//    //ERR("SendActivateMessages\n");
//
//    pti = PsGetCurrentThreadWin32Thread();
//
//    if (Window)
//    {
//       UserRefObjectCo(Window, &Ref);
//
//       if (WindowPrev) UserRefObjectCo(WindowPrev, &RefPrev);
//
//       pti->MessageQueue->QF_flags &= ~QF_EVENTDEACTIVATEREMOVED;
//
//       /* Send palette messages */
//       if (gpsi->PUSIFlags & PUSIF_PALETTEDISPLAY &&
//           //co_IntPostOrSendMessage(UserHMGetHandle(Window), WM_QUERYNEWPALETTE, 0, 0))
//           co_IntSendMessage(UserHMGetHandle(Window), WM_QUERYNEWPALETTE, 0, 0))
//       {
//          UserSendNotifyMessage( HWND_BROADCAST,
//                                 WM_PALETTEISCHANGING,
//                                (WPARAM)UserHMGetHandle(Window),
//                                 0);
//       }
//       //// Fixes CORE-6434.
//       if (!(Window->style & WS_CHILD))
//       {
//          PWND pwndTemp = co_GetDesktopWindow(Window)->spwndChild;
//
//          while (pwndTemp && !(pwndTemp->style & WS_VISIBLE)) pwndTemp = pwndTemp->spwndNext;
//
//          if (Window != pwndTemp || (WindowPrev && !IntIsWindowVisible(WindowPrev)))
//          {
//             if (!Async || pti->MessageQueue == gpqForeground)
//             {
//                UINT flags = SWP_NOSIZE | SWP_NOMOVE;
//                if (Window == pwndTemp) flags |= SWP_NOACTIVATE;
//                //ERR("co_IntSendActivateMessages SetWindowPos! Async %d pti Q == FGQ %d\n",Async,pti->MessageQueue == gpqForeground);
//                co_WinPosSetWindowPos(Window, HWND_TOP, 0, 0, 0, 0, flags);
//             }
//          }
//       }
//       ////
//       //// CORE-1161 and CORE-6651
//       if (Window->spwndPrev)
//       {
//          HWND *phwndTopLevel, *phwndCurrent;
//          PWND pwndCurrent, pwndDesktop;
//
//          pwndDesktop = co_GetDesktopWindow(Window);//UserGetDesktopWindow();
//          if (Window->spwndParent == pwndDesktop )
//          {
//             phwndTopLevel = IntWinListChildren(pwndDesktop);
//             phwndCurrent = phwndTopLevel;
//             while(*phwndCurrent)
//             {
//                 pwndCurrent = UserGetWindowObject(*phwndCurrent);
//
//                 if (pwndCurrent && pwndCurrent->spwndOwner == Window )
//                 {
//                     co_WinPosSetWindowPos(pwndCurrent, HWND_TOP, 0, 0, 0, 0, SWP_NOSIZE|SWP_NOMOVE|SWP_NOACTIVATE);
//                 }
//                 phwndCurrent++;
//             }
//             ExFreePoolWithTag(phwndTopLevel, USERTAG_WINDOWLIST);
//           }
//       }
//       ////
//    }
//
//    OldTID = WindowPrev ? IntGetWndThreadId(WindowPrev) : NULL;
//    NewTID = Window ? IntGetWndThreadId(Window) : NULL;
//    ptiOld = WindowPrev ? WindowPrev->head.pti : NULL;
//    ptiNew = Window ? Window->head.pti : NULL;
//
//    //ERR("SendActivateMessage Old -> %x, New -> %x\n", OldTID, NewTID);
//
//    if (!(pti->TIF_flags & TIF_INACTIVATEAPPMSG) &&
//         (OldTID != NewTID) )
//    {
//       PWND cWindow;
//       HWND *List, *phWnd;
//
//       List = IntWinListChildren(UserGetDesktopWindow());
//       if ( List )
//       {
//          if ( OldTID )
//          {
//             ptiOld->TIF_flags |= TIF_INACTIVATEAPPMSG;
//             // Note: Do not set pci flags, this does crash!
//             for (phWnd = List; *phWnd; ++phWnd)
//             {
//                cWindow = ValidateHwndNoErr(*phWnd);
//                if (cWindow && cWindow->head.pti == ptiOld)
//                { // FALSE if the window is being deactivated,
//                  // ThreadId that owns the window being activated.
//                  //ERR("SAM : WM_ACTIVATEAPP(0) tid Old %p New %p\n",OldTID,NewTID);
//                  UserRefObjectCo(cWindow, &RefCall);
//                  co_IntSendMessage(*phWnd, WM_ACTIVATEAPP, FALSE, (LPARAM)NewTID);
//                  UserDerefObjectCo(cWindow);
//                }
//             }
//             ptiOld->TIF_flags &= ~TIF_INACTIVATEAPPMSG;
//          }
//          if ( NewTID )
//          {  //// Prevents a resource crash due to reentrance!
//             InAAPM = TRUE;
//             pti->TIF_flags |= TIF_INACTIVATEAPPMSG;
//             ////
//             for (phWnd = List; *phWnd; ++phWnd)
//             {
//                cWindow = ValidateHwndNoErr(*phWnd);
//                if (cWindow && cWindow->head.pti == ptiNew)
//                { // TRUE if the window is being activated,
//                  // ThreadId that owns the window being deactivated.
//                  //ERR("SAM : WM_ACTIVATEAPP(1) hwnd %p tid New %p Old %p\n",UserHMGetHandle(cWindow),NewTID,OldTID);
//                  UserRefObjectCo(cWindow, &RefCall);
//                  co_IntSendMessage(*phWnd, WM_ACTIVATEAPP, TRUE, (LPARAM)OldTID);
//                  UserDerefObjectCo(cWindow);
//                }
//             }
//          }
//          ExFreePoolWithTag(List, USERTAG_WINDOWLIST);
//       }
//    }
//
//    if (Window)
//    {
//       if (WindowPrev)
//          UserDerefObjectCo(WindowPrev); // Now allow the previous window to die.
//
//       if (Window->state & WNDS_ACTIVEFRAME)
//       {  // If already active frame do not allow NCPaint.
//          //ERR("SendActivateMessage Is Active Frame!\n");
//          Window->state |= WNDS_NONCPAINT;
//       }
//
//       if (Window->style & WS_MINIMIZE)
//       {
//          TRACE("Widow was minimized\n");
//       }
//
//       co_IntMakeWindowActive(Window);
//
//       co_IntSendMessage( UserHMGetHandle(Window),
//                          WM_NCACTIVATE,
//                         (WPARAM)(Window == (gpqForeground ? gpqForeground->spwndActive : NULL)),
//                          0);
//
//       co_IntSendMessage( UserHMGetHandle(Window),
//                          WM_ACTIVATE,
//                          MAKEWPARAM(MouseActivate ? WA_CLICKACTIVE : WA_ACTIVE, (Window->style & WS_MINIMIZE) != 0),
//                         (LPARAM)(WindowPrev ? UserHMGetHandle(WindowPrev) : 0));
//
//       if (Window->style & WS_VISIBLE)
//          UpdateShellHook(Window);
//
//       Window->state &= ~WNDS_NONCPAINT;
//
//       UserDerefObjectCo(Window);
//    }
//    return InAAPM;
// }
//
// // Win: xxxSendFocusMessages
// VOID FASTCALL
// IntSendFocusMessages( PTHREADINFO pti, PWND pWnd)
// {
//    PWND pWndPrev;
//    PUSER_MESSAGE_QUEUE ThreadQueue = pti->MessageQueue; // Queue can change...
//    HWND hwndPrev;
//    USER_REFERENCE_ENTRY Ref;
//
//    ThreadQueue->QF_flags &= ~QF_FOCUSNULLSINCEACTIVE;
//    if (!pWnd && ThreadQueue->spwndActive)
//    {
//       ThreadQueue->QF_flags |= QF_FOCUSNULLSINCEACTIVE;
//    }
//
//    pWndPrev = ThreadQueue->spwndFocus;
//    if (pWndPrev)
//       UserRefObjectCo(pWndPrev, &Ref);
//
//    /* check if the specified window can be set in the input data of a given queue */
//    if (!pWnd || ThreadQueue == pWnd->head.pti->MessageQueue)
//       /* set the current thread focus window */
//       ThreadQueue->spwndFocus = pWnd;
//
//    if (pWnd)
//    {
//       if (pWndPrev)
//       {
//          co_IntSendMessage(UserHMGetHandle(pWndPrev), WM_KILLFOCUS, (WPARAM)UserHMGetHandle(pWnd), 0);
//          if (IS_IMM_MODE())
//          {
//             IntFocusSetInputContext(pWndPrev, FALSE, FALSE);
//          }
//       }
//       if (ThreadQueue->spwndFocus == pWnd)
//       {
//          if (IS_IMM_MODE())
//          {
//             IntFocusSetInputContext(pWnd, TRUE, FALSE);
//          }
//
//          IntNotifyWinEvent(EVENT_OBJECT_FOCUS, pWnd, OBJID_CLIENT, CHILDID_SELF, 0);
//
//          hwndPrev = (pWndPrev ? UserHMGetHandle(pWndPrev) : NULL);
//          co_IntSendMessage(UserHMGetHandle(pWnd), WM_SETFOCUS, (WPARAM)hwndPrev, 0);
//       }
//    }
//    else
//    {
//       if (pWndPrev)
//       {
//          IntNotifyWinEvent(EVENT_OBJECT_FOCUS, NULL, OBJID_CLIENT, CHILDID_SELF, 0);
//
//          co_IntSendMessage(UserHMGetHandle(pWndPrev), WM_KILLFOCUS, 0, 0);
//          if (IS_IMM_MODE())
//          {
//             IntFocusSetInputContext(pWndPrev, FALSE, FALSE);
//          }
//       }
//    }
//
//    if (pWndPrev)
//       UserDerefObjectCo(pWndPrev);
// }
//
// BOOL FASTCALL
// FindRemoveEventMsg(PTHREADINFO pti, DWORD Event, DWORD EventLast)
// {
//    PUSER_MESSAGE Message;
//    PLIST_ENTRY Entry;
//    BOOL Ret = FALSE;
//
//    Entry = pti->PostedMessagesListHead.Flink;
//    while (Entry != &pti->PostedMessagesListHead)
//    {
//       // Scan posted queue messages to see if we received async messages.
//       Message = CONTAINING_RECORD(Entry, USER_MESSAGE, ListEntry);
//       Entry = Entry->Flink;
//
//       if (Message->dwQEvent == EventLast)
//       {
//          //ERR("Event D/SAW: Last Activate/Deactivate %d\n", EventLast);
//          return Ret;
//       }
//
//       if (Message->dwQEvent == Event)
//       {
//          //ERR("Event D/SAW: Found one in the Post Msg Queue! Activate/Deactivate %d\n", Event);
//          ClearMsgBitsMask(pti, Message->QS_Flags);
//          MsqDestroyMessage(Message);
//          Ret = TRUE;
//       }
//    }
//    return Ret;
// }
//
// BOOL FASTCALL
// ToggleFGActivate(PTHREADINFO pti)
// {
//    BOOL Ret;
//    PPROCESSINFO ppi = pti->ppi;
//
//    Ret = !!(pti->TIF_flags & TIF_ALLOWFOREGROUNDACTIVATE);
//    if (Ret)
//    {
//       pti->TIF_flags &= ~TIF_ALLOWFOREGROUNDACTIVATE;
//    }
//    else
//       Ret = !!(ppi->W32PF_flags & W32PF_ALLOWFOREGROUNDACTIVATE);
//
//    if (Ret)
//       ppi->W32PF_flags &= ~W32PF_ALLOWFOREGROUNDACTIVATE;
//    //ERR("ToggleFGActivate is %d\n",Ret);
//    return Ret;
// }
//
// BOOL FASTCALL
// IsAllowedFGActive(PTHREADINFO pti, PWND Wnd)
// {
//    // Not allowed if one or more,,
//    if (!ToggleFGActivate(pti) ||              // bits not set,
//         pti->rpdesk != gpdeskInputDesktop ||  // not current Desktop,
//         pti->MessageQueue == gpqForeground || // if already the queue foreground,
//         IsFGLocked() ||                       // foreground is locked,
//         (Wnd->ExStyle & WS_EX_NOACTIVATE))    // or, does not become the foreground window when the user clicks it.
//    {
//       return FALSE;
//    }
//    //ERR("IsAllowedFGActive is TRUE\n");
//    return TRUE;
// }
//
// /*
//    Can the system force foreground from one or more conditions.
//  */
// BOOL FASTCALL
// CanForceFG(PPROCESSINFO ppi)
// {
//    if (!ptiLastInput ||
//         ptiLastInput->ppi == ppi ||
//        !gptiForeground ||
//         gptiForeground->ppi == ppi ||
//         ppi->W32PF_flags & (W32PF_ALLOWFOREGROUNDACTIVATE | W32PF_SETFOREGROUNDALLOWED) ||
//         gppiInputProvider == ppi ||
//        !gpqForeground
//       ) return TRUE;
//    //ERR("CanForceFG is FALSE\n");
//    return FALSE;
// }
//
// //
// //  Switching out foreground message queues.
// //
// BOOL FASTCALL
// co_IntSetForegroundMessageQueue(
//      PWND Wnd,
//      PTHREADINFO pti,
//      BOOL MouseActivate,
//      DWORD Type )
// {
//    PTHREADINFO ptiChg, ptiPrev;
//    PUSER_MESSAGE_QUEUE pumq, pumqChg, pumqPrev;
//    BOOL Removed, Ret = TRUE;
//
//    if (Wnd && !VerifyWnd(Wnd))
//    {
//       return FALSE;
//    }
//
//    if ( !gptiForeground || gptiForeground->TIF_flags & TIF_INCLEANUP )
//       ptiPrev = NULL;
//    else
//       ptiPrev = gptiForeground;
//
//    if (Wnd)
//    {
//       ptiChg = Wnd->head.pti;
//       IntSetFocusMessageQueue(Wnd->head.pti->MessageQueue);
//       gptiForeground = Wnd->head.pti;
//       //ERR("Set Foreground pti 0x%p Q 0x%p hWnd 0x%p\n", Wnd->head.pti, Wnd->head.pti->MessageQueue, UserHMGetHandle(Wnd));
//    }
//    else
//    {
//       ptiChg = NULL;
//       IntSetFocusMessageQueue(NULL);
//       gptiForeground = NULL;
//       //ERR("Set Foreground pti 0x0 Q 0x0 hWnd 0x0\n");
//    }
//
//    //
//    // Process the changing out of the message queues.
//    //
//    if (gpqForegroundPrev != gpqForeground)
//    {
//       pumqPrev = NULL;
//       if ( ptiPrev && !(ptiPrev->TIF_flags & TIF_INCLEANUP) )
//       {
//          pumqPrev = ptiPrev->MessageQueue;
//       }
//
//       pumq = pti ? pti->MessageQueue : NULL;
//
//       // Deactivate the previous message queue.
//       if (pumqPrev)
//       {
//           if ( pumq != pumqPrev )
//           {
//               MSG Msg;
//               HWND hWndPrev = pumqPrev->spwndActive ? UserHMGetHandle(pumqPrev->spwndActive) : NULL;
//               HANDLE tid = gptiForeground ? PsGetThreadId(gptiForeground->pEThread) : NULL; // TID from changing Window PTI.
//
//               Msg.message = WM_ASYNC_SETACTIVEWINDOW;
//               Msg.hwnd = hWndPrev;
//               Msg.wParam = (WPARAM)pumqPrev->spwndActive;
//               Msg.lParam = 0;
//               Msg.time = 0;
//               //ERR("SFWAMQ : DAW P pti 0x%p tid 0x%p hWndPrev 0x%p\n",ptiPrev,tid,hWndPrev);
//               MsqPostMessage(ptiPrev, &Msg, FALSE, QS_EVENT, POSTEVENT_DAW, (LONG_PTR)tid);
//           }
//       }
//
//       pumqChg = NULL;
//       if ( ptiChg && !(ptiChg->TIF_flags & TIF_INCLEANUP) )
//       {
//          pumqChg = ptiChg->MessageQueue;
//       }
//
//       pumq = pti ? pti->MessageQueue : NULL;
//
//       // Activate changing message queue.
//       if (pumqChg)
//       {
//          /*
//             Henri Verbeet,
//             What happens is that we get the WM_WINE_SETACTIVEWINDOW message sent by the
//             other thread after we already changed the foreground window back to our own
//             window.
//           */
//           //ERR("SFWAMQ : 1\n");
//           Removed = FindRemoveEventMsg(ptiChg, POSTEVENT_DAW, POSTEVENT_NONE);
//
//           if (pumqChg != pumq)
//           {
//               MSG Msg;
//               HWND hWnd = Wnd ? UserHMGetHandle(Wnd) : NULL;
//               HANDLE tid = ptiPrev ? PsGetThreadId(ptiPrev->pEThread) : NULL;
//
//               if (Removed) pumqChg->QF_flags |= QF_EVENTDEACTIVATEREMOVED;
//
//               Msg.message = WM_ASYNC_SETACTIVEWINDOW;
//               Msg.hwnd = hWnd;
//               Msg.wParam = (WPARAM)Wnd;
//               Msg.lParam = (LPARAM)tid; //// Fixme! Type flags?
//               Msg.time = 0;
//               //ERR("SFWAMQ : SAW P pti 0x%p tid 0x%p hWnd 0x%p\n",ptiChg,tid,hWnd);
//               MsqPostMessage(ptiChg, &Msg, FALSE, QS_EVENT, POSTEVENT_SAW, (LONG_PTR)Type|MouseActivate);
//           }
//           else // Current message queue same as changed message queue.
//           {
//               if (pumq->spwndActive == Wnd)
//               {
//                  co_IntSendMessage( UserHMGetHandle(Wnd), WM_NCACTIVATE, TRUE, (LPARAM)UserHMGetHandle(Wnd));
//
//                  UpdateShellHook(Wnd);
//
//                  co_WinPosSetWindowPos(Wnd, HWND_TOP, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE);
//               }
//               else
//               {
//                  //ERR("SFWAMQ : SAW I pti 0x%p hWnd 0x%p\n", ptiChg, Wnd ? UserHMGetHandle(Wnd) : NULL);
//                  Ret = co_IntSetActiveWindow(Wnd, MouseActivate, TRUE/*Type*/, FALSE);
//                  //if (!Ret) ERR("SFWAMQ : ISAW : return error\n");
//                  return Ret;
//               }
//           }
//       }
//
//       // Handle same message queue after switch out.
//       pumqPrev = NULL;
//       if ( ptiPrev && !(ptiPrev->TIF_flags & TIF_INCLEANUP) )
//       {
//          pumqPrev = ptiPrev->MessageQueue;
//       }
//       pumq = pti ? pti->MessageQueue : NULL;
//
//       if ( pumqPrev && pumq == pumqPrev )
//       {
//           HANDLE tid = Wnd ? PsGetThreadId(Wnd->head.pti->pEThread) : NULL;
//           //ERR("SFWAMQ : DAW I pti 0x%p tid 0x%p hWnd 0x%p\n", ptiPrev, tid, Wnd ? UserHMGetHandle(Wnd) : NULL);
//           IntDeactivateWindow(pti, tid);
//       }
//    }
//    return Ret;
// }
//
// /*
//    MSDN:
//    The system restricts which processes can set the foreground window. A process
//    can set the foreground window only if one of the following conditions is true:
//
//     * The process is the foreground process.
//     * The process was started by the foreground process.
//     * The process received the last input event.
//     * There is no foreground process.
//     * The foreground process is being debugged.
//     * The foreground is not locked (see LockSetForegroundWindow).
//     * The foreground lock time-out has expired (see SPI_GETFOREGROUNDLOCKTIMEOUT in SystemParametersInfo).
//     * No menus are active.
// */
// static
// BOOL FASTCALL
// co_IntSetForegroundAndFocusWindow(
//      PWND Wnd,
//      BOOL MouseActivate,
//      BOOL bFlash )
// {
//    HWND hWnd = Wnd ? UserHMGetHandle(Wnd) : NULL;
//    PUSER_MESSAGE_QUEUE PrevForegroundQueue;
//    PTHREADINFO pti;
//    BOOL Ret = FALSE;
//
//    if (Wnd) ASSERT_REFS_CO(Wnd);
//
//    TRACE("co_IntSetForegroundAndFocusWindow hwnd=%p mouse=%d\n", hWnd, MouseActivate);
//
//    PrevForegroundQueue = IntGetFocusMessageQueue(); // Use this active desktop.
//    pti = PsGetCurrentThreadWin32Thread();
//
//    if (Wnd && PrevForegroundQueue)
//    {  // Same Window Q as foreground just do active.
//       if (Wnd->head.pti->MessageQueue == PrevForegroundQueue)
//       {
//          //ERR("Same Window Q as foreground just do active.\n");
//          if (pti->MessageQueue == PrevForegroundQueue)
//          { // Same WQ and TQ go active.
//             //ERR("Same WQ and TQ go active.\n");
//             Ret = IntUserSetActiveWindow(Wnd, MouseActivate, TRUE, FALSE);
//          }
//          else if (Wnd->head.pti->MessageQueue->spwndActive == Wnd)
//          { // Same WQ and it is active.
//             //ERR("Same WQ and it is active.\n");
//             Ret = TRUE;
//          }
//          else
//          { // Same WQ as FG but not the same TQ send active.
//             //ERR("Same WQ as FG but not the same TQ send active.\n");
//             MSG Msg;
//             PTHREADINFO ptiNew = Wnd->head.pti;
//
//             Msg.message = WM_ASYNC_SETACTIVEWINDOW;
//             Msg.hwnd = hWnd;
//             Msg.wParam = (WPARAM)Wnd;
//             Msg.lParam = 0;
//             Msg.time = 0;
//             //ERR("SFAFW 1 : SAW P pti 0x%p hWnd 0x%p\n",ptiNew,hWnd);
//             MsqPostMessage(ptiNew, &Msg, FALSE, QS_EVENT, POSTEVENT_SAW, (LONG_PTR)MouseActivate);
//
//             Ret = TRUE;
//          }
//          return Ret;
//       }
//    }
//
//    if ( (( !IsFGLocked() || pti->ppi == gppiInputProvider ) &&
//          ( CanForceFG(pti->ppi) || pti->TIF_flags & (TIF_SYSTEMTHREAD|TIF_CSRSSTHREAD|TIF_ALLOWFOREGROUNDACTIVATE) )) ||
//         pti->ppi == ppiScrnSaver
//       )
//    {
//
//       ToggleFGActivate(pti);
//
//       return co_IntSetForegroundMessageQueue( Wnd, pti, MouseActivate, 0 );
//    }
//
//    if (!Wnd) return FALSE; // No window, always return FALSE.
//
//    //// if (bFlash) FIXME : handle flash!!!
//
//    if (pti->MessageQueue == Wnd->head.pti->MessageQueue)
//    {
//        //ERR("Same PQ and WQ go active.\n");
//        Ret = IntUserSetActiveWindow(Wnd, MouseActivate, TRUE, FALSE);
//        //if (!Ret) ERR("ISFAFW : IUSAW : return error\n");
//    }
//    else if (Wnd->head.pti->MessageQueue->spwndActive == Wnd)
//    {
//        TRACE("Same Active and Wnd.\n"); // Leave this for now.
//    }
//    else
//    {
//        //ERR("Activate Not same PQ and WQ and Wnd.\n");
//        //// CORE-10785 fix hang, ROSTESTS-208 allows test to run.
//        MSG Msg;
//        PTHREADINFO ptiNew = Wnd->head.pti;
//
//        Msg.message = WM_ASYNC_SETACTIVEWINDOW;
//        Msg.hwnd = hWnd;
//        Msg.wParam = (WPARAM)Wnd;
//        Msg.lParam = 0;
//        Msg.time = 0;
//        //ERR("SFAFW 2 : SAW P pti 0x%p hWnd 0x%p\n",ptiNew,hWnd);
//        MsqPostMessage(ptiNew, &Msg, FALSE, QS_EVENT, POSTEVENT_SAW, (LONG_PTR)MouseActivate);
//    }
//    // Always return FALSE.
//    return FALSE;
// }
// /* --- pretranslation split: the remaining function groups live in focus.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
// /* focus.split2.c — #include fragment of focus.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original focus.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */
//
//
// //
// // Set the Active Window.
// //
// BOOL FASTCALL
// co_IntSetActiveWindow(
//     PWND Wnd,
//     BOOL bMouse,
//     BOOL bFocus,
//     BOOL Async )
// {
//    PTHREADINFO pti;
//    PUSER_MESSAGE_QUEUE ThreadQueue;
//    PWND pWndChg, WndPrev; // State changes.
//    HWND hWndPrev;
//    HWND hWnd = 0;
//    BOOL InAAPM;
//    CBTACTIVATESTRUCT cbt;
//
//    //ERR("co_IntSetActiveWindow 1\n");
//
//    pti = PsGetCurrentThreadWin32Thread();
//    ThreadQueue = pti->MessageQueue;
//    ASSERT(ThreadQueue != 0);
//
//    pWndChg = ThreadQueue->spwndActive; // Keep to notify of a preemptive switch.
//    hWndPrev = (pWndChg ? UserHMGetHandle(pWndChg) : NULL);
//
//    if ( !Wnd || Wnd == UserGetDesktopWindow() )
//    {
//       //ERR("ISAW : NULL %p\n",Wnd);
//       return FALSE;
//    }
//
//    ASSERT_REFS_CO(Wnd);
//    hWnd = UserHMGetHandle(Wnd);
//    //ERR("co_IntSetActiveWindow 2 hWnd 0x%p\n",hWnd);
//
//    if (Wnd->ExStyle & WS_EX_NOACTIVATE)
//        return TRUE;
//
//    /* check if the specified window can be set in the input data of a given queue */
//    if ( ThreadQueue != Wnd->head.pti->MessageQueue )
//    {
//       //ERR("ISAW : Must have the same Message Queue\n");
//       return FALSE;
//    }
//
//    if (!VerifyWnd(Wnd))
//    {
//       //ERR("ISAW : Window is in Destroy!\n");
//       return FALSE;
//    }
//
//    if ( Wnd == pWndChg )
//    {
//       //ERR("ISAW : Nothing to do\n");
//       return TRUE; // Fix CORE-8780 and CORE-11979. See CORE-11324 for breakage.
//    }
//
//    if ( Wnd->state & WNDS_BEINGACTIVATED ) return TRUE;
//
//    /* Call CBT hook chain */
//    cbt.fMouse     = bMouse;
//    cbt.hWndActive = hWndPrev;
//    if (co_HOOK_CallHooks( WH_CBT, HCBT_ACTIVATE, (WPARAM)hWnd, (LPARAM)&cbt))
//    {
//       ERR("SetActiveWindow: WH_CBT Call Hook return!\n");
//       return FALSE;
//    }
//
//    ThreadQueue->QF_flags &= ~QF_EVENTDEACTIVATEREMOVED;
//
//    if ( ThreadQueue->spwndActive && ThreadQueue->spwndActive->state & WNDS_DESTROYED )
//       ThreadQueue->spwndActive = NULL;
//    else
//       ThreadQueue->spwndActivePrev = ThreadQueue->spwndActive;
//
//    WndPrev = ThreadQueue->spwndActive; // Keep to save changing active.
//
//    if (WndPrev)
//    {
//       if (ThreadQueue == gpqForeground) gpqForegroundPrev = ThreadQueue;
//       if (!co_IntSendDeactivateMessages(UserHMGetHandle(WndPrev), hWnd, TRUE)) return FALSE;
//    }
//
//    WndPrev = ThreadQueue->spwndActive; // Again keep to save changing active.
//
//    // While in calling message proc or hook:
//    // Fail if a preemptive switch was made, current active not made previous,
//    // focus window is dead or no longer the same thread queue.
//    if ( ThreadQueue->spwndActivePrev != ThreadQueue->spwndActive ||
//         pWndChg != WndPrev ||
//         (Wnd && !VerifyWnd(Wnd)) ||
//         ThreadQueue != pti->MessageQueue )
//    {
//       ERR("SetActiveWindow: Summary ERROR, active state changed!\n");
//       return FALSE;
//    }
//
//    if (!WndPrev) ThreadQueue->QF_flags &= ~QF_FOCUSNULLSINCEACTIVE;
//
//    /* set the current thread active window */
//    ThreadQueue->spwndActive = Wnd;
//
//    // Set state flag to prevent recursions.
//    Wnd->state |= WNDS_BEINGACTIVATED;
//
//    IntNotifyWinEvent(EVENT_SYSTEM_FOREGROUND, Wnd, OBJID_WINDOW, CHILDID_SELF, WEF_SETBYWNDPTI);
//
//    // Clear out activate EVENT messages.
//    FindRemoveEventMsg(pti, POSTEVENT_SAW, POSTEVENT_DAW);
//
//    WndPrev = VerifyWnd(ThreadQueue->spwndActivePrev); // Now should be set but verify it again.
//
//    InAAPM = co_IntSendActivateMessages(WndPrev, Wnd, bMouse, Async);
//
//    /* now change focus if necessary */
//    //// Fixes CORE-6452 allows setting focus on window.
//    if (bFocus && !(ThreadQueue->QF_flags & QF_FOCUSNULLSINCEACTIVE))
//    {
//       /* Do not change focus if the window is no longer active */
//       if (pti->MessageQueue->spwndActive != IntGetNonChildAncestor(pti->MessageQueue->spwndFocus))
//       {
//          PWND pWndSend = pti->MessageQueue->spwndActive;
//          // Clear focus if the active window is minimized.
//          if (pWndSend && pti->MessageQueue->spwndActive->style & WS_MINIMIZE) pWndSend = NULL;
//          // Send focus messages and if so, set the focus.
//          IntSendFocusMessages( pti, pWndSend);
//       }
//    }
//    ////
//    if (InAAPM)
//    {
//       pti->TIF_flags &= ~TIF_INACTIVATEAPPMSG;
//    }
//
//    // Checked in MENU_TrackMenu
//    ThreadQueue->QF_flags |= QF_ACTIVATIONCHANGE;
//
//    //ERR("co_IntSetActiveWindow Exit\n");
//    Wnd->state &= ~WNDS_BEINGACTIVATED;
//    return (ThreadQueue->spwndActive == Wnd);
// }
//
// //
// // Set the Active Window.
// //
// // Window is not optional!
// //
// BOOL FASTCALL
// IntUserSetActiveWindow(
//     PWND Wnd,
//     BOOL bMouse,
//     BOOL bFocus,
//     BOOL Async)
// {
//    PTHREADINFO pti;
//    PUSER_MESSAGE_QUEUE ThreadQueue;
//
//    //ERR("IntUserSetActiveWindow 1\n");
//    ASSERT_REFS_CO(Wnd);
//    if ((Wnd->style & (WS_POPUP|WS_CHILD)) == WS_CHILD) return FALSE;
//    //ERR("IntUserSetActiveWindow 1a hWnd 0x%p\n",UserHMGetHandle(Wnd));
//
//    //ERR("IntUserSetActiveWindow 2\n");
//    pti = PsGetCurrentThreadWin32Thread();
//    ThreadQueue = pti->MessageQueue;
//    ASSERT(ThreadQueue != 0);
//
//    while (Wnd)
//    {
//       BOOL Ret, DoFG, AllowFG;
//
//       if (ThreadQueue == Wnd->head.pti->MessageQueue)
//       {
//          if (IsAllowedFGActive(pti, Wnd))
//          {
//              DoFG = TRUE;
//          }
//          else
//          {
//              //ERR("IntUserSetActiveWindow 3 Go Out!\n");
//              break;
//          }
//          AllowFG = !pti->cVisWindows; // Nothing is visable.
//          //ERR("IntUserSetActiveWindow 3a DoFG = %d AllowFG = %d\n",DoFG,AllowFG);
//       }
//       else //if (ThreadQueue != Wnd->head.pti->MessageQueue)
//       {
//          //PUSER_MESSAGE_QUEUE ForegroundQueue = IntGetFocusMessageQueue();
//          // Rule 1 & 4, We are foreground so set this FG window or NULL foreground....
//          if (!gpqForeground || gpqForeground == ThreadQueue)
//          {
//             DoFG = TRUE;
//          }
//          else
//             DoFG = FALSE;
//          if (DoFG)
//          {
//             if (pti->TIF_flags & TIF_ALLOWFOREGROUNDACTIVATE || pti->cVisWindows)
//                AllowFG = TRUE;
//             else
//                AllowFG = FALSE;
//          }
//          else
//             AllowFG = FALSE;
//          //ERR("IntUserSetActiveWindow 3b DoFG = %d AllowFG = %d\n",DoFG,AllowFG);
//       }
//       Ret = FALSE;
//       if (DoFG)
//       {
//          pti->TIF_flags |= TIF_ALLOWFOREGROUNDACTIVATE;
//          //ERR("IntUserSetActiveWindow 3c FG set\n");
//          Ret = co_IntSetForegroundAndFocusWindow(Wnd, bMouse, TRUE);
//          if (AllowFG)
//          {
//             pti->TIF_flags |= TIF_ALLOWFOREGROUNDACTIVATE;
//          }
//          else
//          {
//             pti->TIF_flags &= ~TIF_ALLOWFOREGROUNDACTIVATE;
//          }
//       }
//       return Ret;
//    }
//
//    return co_IntSetActiveWindow(Wnd, bMouse, bFocus, Async);
// }
//
// BOOL FASTCALL
// co_IntMouseActivateWindow(PWND Wnd)
// {
//    TRACE("Mouse Active\n");
//    if (Wnd && (Wnd->ExStyle & WS_EX_NOACTIVATE))
//       return TRUE;
//    return co_IntSetForegroundAndFocusWindow(Wnd, TRUE, TRUE);
// }
//
// /* Win: PWND xxxSetActiveWindow(Wnd) */
// BOOL FASTCALL
// UserSetActiveWindow(  PWND Wnd )
// {
//   PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//
//   if (Wnd)
//   {
//      if ((Wnd->style & (WS_POPUP|WS_CHILD)) == WS_CHILD) return FALSE;
//
//      return IntUserSetActiveWindow(Wnd, FALSE, TRUE, FALSE);
//   }
//   /*
//      Yes your eye are not deceiving you~!
//
//      First part of wines Win.c test_SetActiveWindow:
//
//      flush_events( TRUE );
//      ShowWindow(hwnd, SW_HIDE);
//      SetFocus(0);
//      SetActiveWindow(0);
//      check_wnd_state(0, 0, 0, 0); <-- This should pass if ShowWindow does it's job!!! As of 10/28/2012 it does!
//
//      Now Handle wines Msg.c test_SetActiveWindow( 0 )...
//   */
//   TRACE("USAW: Previous active window\n");
//   if (  gpqForegroundPrev &&
//         gpqForegroundPrev->spwndActivePrev &&
//        (gpqForegroundPrev->spwndActivePrev->style & (WS_VISIBLE|WS_DISABLED)) == WS_VISIBLE  &&
//       !(gpqForegroundPrev->spwndActivePrev->state2 & WNDS2_BOTTOMMOST) &&
//        (Wnd = VerifyWnd(gpqForegroundPrev->spwndActivePrev)) != NULL )
//   {
//      TRACE("USAW:PAW hwnd %p\n", UserHMGetHandle(Wnd));
//      return IntUserSetActiveWindow(Wnd, FALSE, TRUE, FALSE);
//   }
//
//   // Activate anyone but the active window.
//   if ( pti->MessageQueue->spwndActive &&
//       (Wnd = VerifyWnd(pti->MessageQueue->spwndActive)) != NULL )
//   {
//       //ERR("USAW:AOWM hwnd %p\n", UserHMGetHandle(Wnd));
//       if (!ActivateOtherWindowMin(Wnd))
//       {
//          // Okay, now go find someone else to play with!
//          //ERR("USAW: Going to WPAOW\n");
//          co_WinPosActivateOtherWindow(Wnd);
//       }
//       return TRUE;
//   }
//
//   TRACE("USAW: Nothing\n");
//   return FALSE;
// }
//
// // Win: PWND xxxSetFocus(Window)
// HWND FASTCALL
// co_UserSetFocus(PWND Window)
// {
//    HWND hWndPrev = 0;
//    PWND pwndTop;
//    PTHREADINFO pti;
//    PUSER_MESSAGE_QUEUE ThreadQueue;
//
//    if (Window)
//       ASSERT_REFS_CO(Window);
//
//    pti = PsGetCurrentThreadWin32Thread();
//    ThreadQueue = pti->MessageQueue;
//    ASSERT(ThreadQueue != 0);
//
//    TRACE("Enter SetFocus hWnd 0x%p pti 0x%p\n",Window ? UserHMGetHandle(Window) : 0, pti );
//
//    hWndPrev = ThreadQueue->spwndFocus ? UserHMGetHandle(ThreadQueue->spwndFocus) : 0;
//
//    if (Window != 0)
//    {
//       if (hWndPrev == UserHMGetHandle(Window))
//       {
//          return hWndPrev ? (IntIsWindow(hWndPrev) ? hWndPrev : 0) : 0; /* Nothing to do */
//       }
//
//       if (Window->head.pti->MessageQueue != ThreadQueue)
//       {
//          ERR("SetFocus Must have the same Q!\n");
//          return 0;
//       }
//
//       /* Check if we can set the focus to this window */
//       //// Fixes wine win test_SetParent both "todo" line 3710 and 3720...
//       for (pwndTop = Window; pwndTop; pwndTop = pwndTop->spwndParent)
//       {
//          if (pwndTop->style & (WS_MINIMIZED|WS_DISABLED)) return 0;
//          if ((pwndTop->style & (WS_POPUP|WS_CHILD)) != WS_CHILD) break;
//          if (pwndTop->spwndParent == NULL) break;
//       }
//       ////
//       if (co_HOOK_CallHooks( WH_CBT, HCBT_SETFOCUS, (WPARAM)UserHMGetHandle(Window), (LPARAM)hWndPrev))
//       {
//          ERR("SetFocus 1 WH_CBT Call Hook return!\n");
//          return 0;
//       }
//
//       /* Activate pwndTop if needed. */
//       if (pwndTop != ThreadQueue->spwndActive)
//       {
//          PUSER_MESSAGE_QUEUE ForegroundQueue = IntGetFocusMessageQueue(); // Keep it based on desktop.
//          if (ThreadQueue != ForegroundQueue && IsAllowedFGActive(pti, pwndTop)) // Rule 2 & 3.
//          {
//             //ERR("SetFocus: Set Foreground!\n");
//             if (!(pwndTop->style & WS_VISIBLE))
//             {
//                 pti->ppi->W32PF_flags |= W32PF_ALLOWFOREGROUNDACTIVATE;
//             }
//             if (!co_IntSetForegroundAndFocusWindow(pwndTop, FALSE, TRUE))
//             {
//                ERR("SetFocus: Set Foreground and Focus Failed!\n");
//                return 0;
//             }
//          }
//
//          /* Set Active when it is needed. */
//          if (pwndTop != ThreadQueue->spwndActive)
//          {
//             //ERR("SetFocus: Set Active! %p\n",pwndTop?UserHMGetHandle(pwndTop):0);
//             if (!co_IntSetActiveWindow(pwndTop, FALSE, FALSE, FALSE))
//             {
//                ERR("SetFocus: Set Active Failed!\n");
//                return 0;
//             }
//          }
//
//          /* Abort if window destroyed */
//          if (Window->state2 & WNDS2_INDESTROY) return 0;
//          /* Do not change focus if the window is no longer active */
//          if (pwndTop != ThreadQueue->spwndActive)
//          {
//             ERR("SetFocus: Top window did not go active!\n");
//             return 0;
//          }
//       }
//
//       // Check again! SetActiveWindow could have set the focus via WM_ACTIVATE.
//       hWndPrev = ThreadQueue->spwndFocus ? UserHMGetHandle(ThreadQueue->spwndFocus) : 0;
//
//       IntSendFocusMessages( pti, Window);
//
//       TRACE("Focus: %p -> %p\n", hWndPrev, UserHMGetHandle(Window));
//    }
//    else /* NULL hwnd passed in */
//    {
//       if (co_HOOK_CallHooks( WH_CBT, HCBT_SETFOCUS, (WPARAM)0, (LPARAM)hWndPrev))
//       {
//          ERR("SetFocus: 2 WH_CBT Call Hook return!\n");
//          return 0;
//       }
//       //ERR("SetFocus: Set Focus NULL\n");
//       /* set the current thread focus window null */
//       IntSendFocusMessages( pti, NULL);
//    }
//    return hWndPrev ? (IntIsWindow(hWndPrev) ? hWndPrev : 0) : 0;
// }
//
// HWND FASTCALL
// UserGetForegroundWindow(VOID)
// {
//     PUSER_MESSAGE_QUEUE ForegroundQueue = IntGetFocusMessageQueue();
//     if (!ForegroundQueue)
//         return NULL;
//     return (ForegroundQueue->spwndActive ? UserHMGetHandle(ForegroundQueue->spwndActive) : NULL);
// }
//
// HWND FASTCALL UserGetActiveWindow(VOID)
// {
//     PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//     PUSER_MESSAGE_QUEUE ThreadQueue = pti->MessageQueue;
//     if (!ThreadQueue)
//         return NULL;
//     return (ThreadQueue->spwndActive ? UserHMGetHandle(ThreadQueue->spwndActive) : NULL);
// }
//
// HWND __stdcall
// IntGetCapture(VOID)
// {
//    PTHREADINFO pti;
//    PUSER_MESSAGE_QUEUE ThreadQueue;
//    HWND Ret;
//
//    TRACE("Enter IntGetCapture\n");
//
//    pti = PsGetCurrentThreadWin32Thread();
//    ThreadQueue = pti->MessageQueue;
//    Ret = ((ThreadQueue && ThreadQueue->spwndCapture) ? UserHMGetHandle(ThreadQueue->spwndCapture) : NULL);
//
//    TRACE("Leave IntGetCapture, ret=%p\n", Ret);
//    return Ret;
// }
//
// HWND FASTCALL
// co_UserSetCapture(HWND hWnd)
// {
//    PTHREADINFO pti;
//    PUSER_MESSAGE_QUEUE ThreadQueue;
//    PWND pWnd, Window = NULL;
//    HWND hWndPrev;
//
//    pti = PsGetCurrentThreadWin32Thread();
//    ThreadQueue = pti->MessageQueue;
//
//    if (ThreadQueue->QF_flags & QF_CAPTURELOCKED)
//       return NULL;
//
//    if (hWnd && (Window = UserGetWindowObject(hWnd)))
//    {
//       if (Window->head.pti->MessageQueue != ThreadQueue)
//       {
//          ERR("Window Thread does not match Current!\n");
//          return NULL;
//       }
//    }
//
//    hWndPrev = MsqSetStateWindow(pti, MSQ_STATE_CAPTURE, hWnd);
//
//    if (hWndPrev)
//    {
//       pWnd = UserGetWindowObject(hWndPrev);
//       if (pWnd)
//          IntNotifyWinEvent(EVENT_SYSTEM_CAPTUREEND, pWnd, OBJID_WINDOW, CHILDID_SELF, WEF_SETBYWNDPTI);
//    }
//
//    if (Window)
//       IntNotifyWinEvent(EVENT_SYSTEM_CAPTURESTART, Window, OBJID_WINDOW, CHILDID_SELF, WEF_SETBYWNDPTI);
//
//    //
//    // Only send the message if we have a previous Window!
//    // Fix msg_menu tracking popup menu and win test_capture_4!!!!
//    //
//    if (hWndPrev)
//    {
//       if (ThreadQueue->MenuOwner && Window) ThreadQueue->QF_flags |= QF_CAPTURELOCKED;
//
//       co_IntSendMessage(hWndPrev, WM_CAPTURECHANGED, 0, (LPARAM)hWnd);
//
//       ThreadQueue->QF_flags &= ~QF_CAPTURELOCKED;
//    }
//
//    if (hWnd == NULL) // Release mode.
//    {
//       MOUSEINPUT mi;
//    /// These are HACKS!
//       /* Also remove other windows if not capturing anymore */
//       MsqSetStateWindow(pti, MSQ_STATE_MENUOWNER, NULL);
//       MsqSetStateWindow(pti, MSQ_STATE_MOVESIZE, NULL);
//    ///
//       /* Somebody may have missed some mouse movements */
//       mi.dx = 0;
//       mi.dy = 0;
//       mi.mouseData = 0;
//       mi.dwFlags = MOUSEEVENTF_MOVE;
//       mi.time = 0;
//       mi.dwExtraInfo = 0;
//       UserSendMouseInput(&mi, FALSE);
//    }
//    return hWndPrev;
// }
//
// /*
//   API Call
// */
// BOOL
// FASTCALL
// IntReleaseCapture(VOID)
// {
//    PTHREADINFO pti;
//    PUSER_MESSAGE_QUEUE ThreadQueue;
//
//    pti = PsGetCurrentThreadWin32Thread();
//    ThreadQueue = pti->MessageQueue;
//
//    // Can not release inside WM_CAPTURECHANGED!!
//    if (ThreadQueue->QF_flags & QF_CAPTURELOCKED) return FALSE;
//
//    co_UserSetCapture(NULL);
//
//    return TRUE;
// }
//
// /*
//   API Call
// */
// BOOL FASTCALL
// co_IntSetForegroundWindow(PWND Window)
// {
//    if (Window) ASSERT_REFS_CO(Window);
//
//    return co_IntSetForegroundAndFocusWindow(Window, FALSE, TRUE);
// }
//
// /*
//   API Call
// */
// BOOL FASTCALL
// co_IntSetForegroundWindowMouse(PWND Window)
// {
//    if (Window) ASSERT_REFS_CO(Window);
//
//    return co_IntSetForegroundAndFocusWindow(Window, TRUE, FALSE);
// }
//
// /*
//   API Call
// */
// BOOL FASTCALL
// IntLockSetForegroundWindow(UINT uLockCode)
// {
//    ULONG Err = ERROR_ACCESS_DENIED;
//    PPROCESSINFO ppi = PsGetCurrentProcessWin32Process();
//    switch (uLockCode)
//    {
//       case LSFW_LOCK:
//          if ( CanForceFG(ppi) && !gppiLockSFW )
//          {
//             gppiLockSFW = ppi;
//             return TRUE;
//          }
//          break;
//       case LSFW_UNLOCK:
//          if ( gppiLockSFW == ppi)
//          {
//             gppiLockSFW = NULL;
//             return TRUE;
//          }
//          break;
//       default:
//          Err = ERROR_INVALID_PARAMETER;
//    }
//    EngSetLastError(Err);
//    return FALSE;
// }
//
// /*
//   API Call
// */
// BOOL FASTCALL
// IntAllowSetForegroundWindow(DWORD dwProcessId)
// {
//    PPROCESSINFO ppi, ppiCur;
//    PEPROCESS Process = NULL;
//
//    ppi = NULL;
//    if (dwProcessId != ASFW_ANY)
//    {
//       if (!NT_SUCCESS(PsLookupProcessByProcessId(UlongToHandle(dwProcessId), &Process)))
//       {
//          EngSetLastError(ERROR_INVALID_PARAMETER);
//          return FALSE;
//       }
//       ppi = PsGetProcessWin32Process(Process);
//       if (!ppi)
//       {
//          ObDereferenceObject(Process);
//          return FALSE;
//       }
//    }
//    ppiCur = PsGetCurrentProcessWin32Process();
//    if (!CanForceFG(ppiCur))
//    {
//       if (Process) ObDereferenceObject(Process);
//       EngSetLastError(ERROR_ACCESS_DENIED);
//       return FALSE;
//    }
//    if (dwProcessId == ASFW_ANY)
//    {  // All processes will be enabled to set the foreground window.
//       //ERR("ptiLastInput is CLEARED!!\n");
//       ptiLastInput = NULL;
//    }
//    else
//    {  // Rule #3, last input event in force.
//       ERR("ptiLastInput is SET!!\n");
//       //ptiLastInput = ppi->ptiList; // See CORE-6384 & CORE-7030.
//       ObDereferenceObject(Process);
//    }
//    return TRUE;
// }
//
// /*
//  * @implemented
//  */
// HWND __stdcall
// NtUserGetForegroundWindow(VOID)
// {
//    HWND Ret;
//
//    TRACE("Enter NtUserGetForegroundWindow\n");
//    UserEnterExclusive();
//
//    Ret = UserGetForegroundWindow();
//
//    TRACE("Leave NtUserGetForegroundWindow, ret=%p\n", Ret);
//    UserLeave();
//    return Ret;
// }
//
// HWND __stdcall
// NtUserSetActiveWindow(HWND hWnd)
// {
//    USER_REFERENCE_ENTRY Ref;
//    HWND hWndPrev;
//    PWND Window, pwndPrev;
//    HWND Ret = NULL;
//
//    TRACE("Enter NtUserSetActiveWindow(%p)\n", hWnd);
//    UserEnterExclusive();
//
//    Window = NULL;
//    if (hWnd)
//    {
//       if (!(Window = UserGetWindowObject(hWnd)))
//       {
//          ERR("NtUserSetActiveWindow: Invalid handle 0x%p!\n",hWnd);
//          goto Exit; // Return NULL
//       }
//    }
//
//    if (!Window ||
//         Window->head.pti->MessageQueue == gptiCurrent->MessageQueue)
//    {
//       pwndPrev = gptiCurrent->MessageQueue->spwndActive;
//       hWndPrev = (pwndPrev ? UserHMGetHandle(pwndPrev) : NULL);
//       if (Window) UserRefObjectCo(Window, &Ref);
//       UserSetActiveWindow(Window);
//       if (Window) UserDerefObjectCo(Window);
//       Ret = ((hWndPrev && IntIsWindow(hWndPrev)) ? hWndPrev : NULL);
//    }
//
// Exit:
//    TRACE("Leave NtUserSetActiveWindow, ret=%p\n", Ret);
//    UserLeave();
//    return Ret;
// }
//
// /*
//  * @implemented
//  */
// HWND __stdcall
// NtUserSetCapture(HWND hWnd)
// {
//    HWND Ret;
//
//    TRACE("Enter NtUserSetCapture(%p)\n", hWnd);
//    UserEnterExclusive();
//
//    Ret = co_UserSetCapture(hWnd);
//
//    TRACE("Leave NtUserSetCapture, ret=%p\n", Ret);
//    UserLeave();
//    return Ret;
// }
//
// /*
//  * @implemented
//  */
// HWND __stdcall
// NtUserSetFocus(HWND hWnd)
// {
//    PWND Window;
//    USER_REFERENCE_ENTRY Ref;
//    HWND ret = NULL;
//
//    TRACE("Enter NtUserSetFocus(%p)\n", hWnd);
//    UserEnterExclusive();
//
//    if (hWnd)
//    {
//       if (!(Window = UserGetWindowObject(hWnd)))
//       {
//          ERR("NtUserSetFocus: Invalid handle 0x%p!\n",hWnd);
//          goto Exit; // Return NULL
//       }
//
//       UserRefObjectCo(Window, &Ref);
//       ret = co_UserSetFocus(Window);
//       UserDerefObjectCo(Window);
//    }
//    else
//    {
//       ret = co_UserSetFocus(NULL);
//    }
//
// Exit:
//    TRACE("Leave NtUserSetFocus, ret=%p\n", ret);
//    UserLeave();
//    return ret;
// }
//
// /* EOF */
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
