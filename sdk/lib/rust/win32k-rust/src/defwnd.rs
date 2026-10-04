use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{PsGetCurrentThreadWin32Thread, RtlUnicodeToMultiByteSize};
use ntkrnl_sys::base::NT_SUCCESS;

// min/max (windef.h macros).
#[inline(always)]
fn min<T: PartialOrd>(a: T, b: T) -> T { if a < b { a } else { b } }
#[inline(always)]
fn max<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } }

// LOWORD/HIWORD/MAKELONG (minwindef.h macros).
#[inline(always)]
fn LOWORD(l: DWORD) -> DWORD { l & 0xffff }
#[inline(always)]
fn HIWORD(l: DWORD) -> DWORD { (l >> 16) & 0xffff }
#[inline(always)]
fn MAKELONG(low: u16, high: u16) -> LPARAM { (((low as u32) | ((high as u32) << 16)) as LONG) as LPARAM }

// DC_NC — nonclient redraw flag (winuser.h: DC_NC = DC_CAPTION|DC_FRAME = 0x900C).
const DC_NC: DWORD = DC_CAPTION | DC_FRAME;

// WM_CTLCOLOR (windowsx.h, 16-bit compat) — NOT in the bindgen constants. Without
// this const the `WM_CTLCOLOR =>` match arm in IntDefWindowProc silently became an
// irrefutable catch-all binding that swallowed every message after it in the match
// (WM_NCCALCSIZE/WM_NCPAINT/WM_NCACTIVATE/WM_ERASEBKGND/...), which is why no window
// ever painted non-client chrome. Keep this defined.
const WM_CTLCOLOR: UINT = 0x0019;

// HOOKID_TO_FLAG(HookId) = 1 << (HookId + 1) (hook.h).
#[inline(always)]
fn HOOKID_TO_FLAG(hook_id: INT) -> u32 { 1u32 << (hook_id + 1) }

// ISITHOOKED(HookId) — current thread fsHooks & HOOKID_TO_FLAG(HookId).
#[inline(always)]
unsafe fn ISITHOOKED(hook_id: INT) -> BOOL {
    let pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    if pti.is_null() { return FALSE as BOOL; }
    (((*pti).fsHooks & HOOKID_TO_FLAG(hook_id)) != 0) as BOOL
}

// SYSTEMCUR(func) = gasyscur[ROCR_##func].handle (cursoricon.h). The bare cursor
// names map to ROCR_* indices (win32k-sys consts).
const ARROW: u32 = ROCR_ARROW;
const SIZEWE: u32 = ROCR_SIZEWE;
const SIZENS: u32 = ROCR_SIZENS;
const SIZENWSE: u32 = ROCR_SIZENWSE;
const SIZENESW: u32 = ROCR_SIZENESW;
#[inline(always)]
unsafe fn SYSTEMCUR(rocr: u32) -> PCURICON_OBJECT {
    let base = core::ptr::addr_of!(gasyscur) as *const SYSTEMCURICO;
    (*base.add(rocr as usize)).handle
}

// head_pti(obj) — THRDESKHEAD.pti is the flattened (bindgen) field at offset
// 2*sizeof(usize). Matches ime.rs/input.rs.
#[inline(always)]
unsafe fn head_pti<T>(obj: *const T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}

// gpsi PERUSERSERVERINFO tail — ptCursor/hbrGray/dtCaretBlink live in the anon
// _PERUSERSERVERINFO bindgen-flattened to __bindgen_padding_0. Matches sysparams.rs.
#[inline(always)]
unsafe fn pusi(p: PSERVERINFO) -> *mut _PERUSERSERVERINFO {
    (p as *mut u8).add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// ValidateHwndNoErr (window.c FASTCALL), and same-file-untranslated defwnd.c
// helpers (DrawTextExWorker / DefWndDoSizeMove / DefWndGetHotKey) + menu trackers.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
}
#[cfg(not(target_arch = "x86"))]
// ValidateHwndNoErr — real FASTCALL export (window.rs). i386 fastcall, cdecl else.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
}
// DrawTextExWorker — text.c WINAPI (stdcall `_DrawTextExWorker@24`).
extern "system" {
    fn DrawTextExWorker(hdc: HDC, str: LPWSTR, count: INT, rect: LPRECT, flags: UINT, dtp: *mut DRAWTEXTPARAMS) -> INT;
}
// win32k-internal FASTCALL helpers (i386 `@Name@N`, cdecl elsewhere).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn DefWndDoSizeMove(pwnd: PWND, wParam: u16);
    fn DefWndGetHotKey(Wnd: PWND) -> UINT; // hotkey.rs, as the C: UINT
    fn co_IntSendMessage(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn co_UserRedrawWindow(Wnd: PWND, lprcUpdate: LPRECT, hrgnUpdate: HRGN, flags: u32) -> u8;
    fn co_WinPosShowWindow(Wnd: PWND, Cmd: INT) -> BOOL;
    fn co_GetDesktopWindow(pWnd: PWND) -> PWND;
    fn GetNCHitEx(pWnd: PWND, pt: POINT) -> LONG;
    fn MENU_InitSysMenuPopup(Menu: PMENU, Style: ULONG, ClsStyle: ULONG, HitTest: LONG);
    fn co_IntSnapWindow(pWnd: PWND, Edge: UINT);
    fn co_IntCalculateSnapPosition(pWnd: PWND, Edge: UINT, Rect: *mut RECTL);
    fn co_UserSetFocus(Window: PWND) -> HWND;
    fn co_UserGetUpdateRgn(Window: PWND, hRgn: HRGN, bErase: BOOL) -> INT;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn DefWndDoSizeMove(pwnd: PWND, wParam: u16);
    fn DefWndGetHotKey(Wnd: PWND) -> UINT; // hotkey.rs, as the C: UINT
    fn co_IntSendMessage(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn co_UserRedrawWindow(Wnd: PWND, lprcUpdate: LPRECT, hrgnUpdate: HRGN, flags: u32) -> u8;
    fn co_WinPosShowWindow(Wnd: PWND, Cmd: INT) -> BOOL;
    fn co_GetDesktopWindow(pWnd: PWND) -> PWND;
    fn GetNCHitEx(pWnd: PWND, pt: POINT) -> LONG;
    fn MENU_InitSysMenuPopup(Menu: PMENU, Style: ULONG, ClsStyle: ULONG, HitTest: LONG);
    fn co_IntSnapWindow(pWnd: PWND, Edge: UINT);
    fn co_IntCalculateSnapPosition(pWnd: PWND, Edge: UINT, Rect: *mut RECTL);
    fn co_UserSetFocus(Window: PWND) -> HWND;
    fn co_UserGetUpdateRgn(Window: PWND, hRgn: HRGN, bErase: BOOL) -> INT;
}
// cdecl (`_Name`) win32k-internal helpers.
extern "C" {
    fn MENU_TrackMouseMenuBar(pWnd: PWND, ht: ULONG, pt: POINT);
    fn MENU_TrackKbdMenuBar(pwnd: PWND, wParam: UINT, wChar: WCHAR);
    fn NC_HandleNCLButtonDown(Wnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn NC_HandleNCRButtonDown(Wnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn NC_HandleNCLButtonDblClk(Wnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn co_IntShellHookNotify(Message: WPARAM, wParam: WPARAM, lParam: LPARAM);
    fn MENU_IsMenuActive() -> PWND;
    fn MENU_EndMenu(Wnd: PWND);
    fn co_UserDestroyWindow(Window: PWND) -> BOOL;
    fn NC_HandleNCCalcSize(Wnd: PWND, wparam: WPARAM, Rect: *mut RECTL, Suppress: BOOL) -> LRESULT;
    fn NC_HandleNCActivate(Wnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn NC_DoNCPaint(pWnd: PWND, hDC: HDC, Flags: INT) -> u32;
}
// DefSetText — APIENTRY (stdcall `_DefSetText@8`).
extern "system" {
    fn DefSetText(Wnd: PWND, Text: PCWSTR) -> BOOL;
}
// UserHMGetHandle (object.h FORCEINLINE) / co_IntUnsnapWindow (winpos.h FORCEINLINE).
#[inline(always)]
unsafe fn UserHMGetHandle(pWnd: PWND) -> HWND {
    *(pWnd as *const HWND)
}
#[inline(always)]
unsafe fn co_IntUnsnapWindow(pWnd: PWND) {
    co_IntSnapWindow(pWnd, HTNOWHERE as UINT);
}
// IntIsSnapAllowedForWindow (winpos.h FORCEINLINE; mirrors nonclient.rs).
#[inline(always)]
unsafe fn IntIsSnapAllowedForWindow(Wnd: PWND) -> BOOLEAN {
    let style: UINT = (*Wnd).style;
    let tbws: UINT = WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN;
    let tbes: UINT = WS_EX_TOOLWINDOW;
    let istb: bool =
        (style & tbws) == tbws && ((*Wnd).ExStyle & (tbes | WS_EX_APPWINDOW)) == tbes;
    let thickframe: bool = (style & WS_THICKFRAME) != 0
        && (style & (WS_DLGFRAME | WS_BORDER)) != WS_DLGFRAME;
    (thickframe && (style & WS_CHILD) == 0 && !istb) as BOOLEAN
}
extern "system" {
    // SafeProbeForRead — safe_user_io.h NTAPI (stdcall `_SafeProbeForRead@12`).
    fn SafeProbeForRead(Address: *mut core::ffi::c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    // co_HOOK_CallHooks — APIENTRY (stdcall `_co_HOOK_CallHooks@16`).
    fn co_HOOK_CallHooks(HookId: INT, Code: INT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
}
// UserReferenceObject / UserDereferenceObject for UserRefObjectCo (fastcall i386).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(obj: PVOID) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(obj: PVOID) -> BOOL;
}
// RECTL_bIsEmptyRect (rect.h inline).
#[inline(always)]
unsafe fn RECTL_bIsEmptyRect(prcl: *const RECTL) -> BOOL {
    ((*prcl).left >= (*prcl).right || (*prcl).top >= (*prcl).bottom) as BOOL
}
// UserRefObjectCo / UserDerefObjectCo (object.h static __inline; mirrors focus.rs).
#[inline(always)]
unsafe fn UserRefObjectCo(obj: PWND, Ref: *mut USER_REFERENCE_ENTRY) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*Ref).obj = obj as PVOID;
    UserReferenceObject(obj as PVOID);
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let entry: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*Ref).Entry);
    (*entry).Next = (*list_head).Next;
    (*list_head).Next = entry;
}
#[inline(always)]
unsafe fn UserDerefObjectCo(obj: PWND) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let reference_entry: *mut SINGLE_LIST_ENTRY = (*list_head).Next;
    if !reference_entry.is_null() {
        (*list_head).Next = (*reference_entry).Next;
    }
    UserDereferenceObject(obj as PVOID);
}

// ---- file-scope preamble (translated) ----
pub const SC_ABOUT: WPARAM = 0xF180;
pub const SC_PUREMENU: WPARAM = 0xF093;

#[no_mangle]
pub unsafe extern "system" fn DrawTextW(
    hdc: HDC,
    str: LPCWSTR,
    count: INT,
    rect: LPRECT,
    mut flags: UINT,
) -> INT {
    let mut dtp: DRAWTEXTPARAMS = core::mem::zeroed();

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(dtp) as *mut u8,
        0,
        core::mem::size_of::<DRAWTEXTPARAMS>(),
    );
    dtp.cbSize = core::mem::size_of::<DRAWTEXTPARAMS>() as UINT;
    if (flags & DT_TABSTOP) != 0 {
        dtp.iTabLength = ((flags >> 8) & 0xff) as INT;
        flags &= 0xffff00ff;
    }
    DrawTextExWorker(
        hdc,
        str as LPWSTR,
        count,
        rect,
        flags,
        core::ptr::addr_of_mut!(dtp),
    )
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn DefWndControlColor(hDC: HDC, ctlType: UINT) -> HBRUSH {
    DefWndControlColor_impl(hDC, ctlType)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn DefWndControlColor(hDC: HDC, ctlType: UINT) -> HBRUSH {
    DefWndControlColor_impl(hDC, ctlType)
}

unsafe fn DefWndControlColor_impl(hDC: HDC, ctlType: UINT) -> HBRUSH {
    if ctlType == CTLCOLOR_SCROLLBAR {
        let hb = IntGetSysColorBrush(COLOR_SCROLLBAR as i32) as HBRUSH;
        let bk = IntGetSysColor(COLOR_3DHILIGHT as i32);
        IntGdiSetTextColor(hDC, IntGetSysColor(COLOR_3DFACE as i32));
        IntGdiSetBkColor(hDC, bk);

        if bk == IntGetSysColor(COLOR_WINDOW as i32) {
            return (*pusi(gpsi)).hbrGray;
        }

        NtGdiUnrealizeObject(hb as HANDLE);
        return hb;
    }

    IntGdiSetTextColor(hDC, IntGetSysColor(COLOR_WINDOWTEXT as i32));

    if ctlType == CTLCOLOR_EDIT || ctlType == CTLCOLOR_LISTBOX {
        IntGdiSetBkColor(hDC, IntGetSysColor(COLOR_WINDOW as i32));
    } else {
        IntGdiSetBkColor(hDC, IntGetSysColor(COLOR_3DFACE as i32));
        return IntGetSysColorBrush(COLOR_3DFACE as i32) as HBRUSH;
    }

    IntGetSysColorBrush(COLOR_WINDOW as i32) as HBRUSH
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn DefWndHandleWindowPosChanging(pWnd: PWND, WinPos: PWINDOWPOS) -> LRESULT {
    DefWndHandleWindowPosChanging_impl(pWnd, WinPos)
}

#[cfg(target_pointer_width = "32")]
unsafe extern "fastcall" fn DefWndHandleWindowPosChanging(pWnd: PWND, WinPos: PWINDOWPOS) -> LRESULT {
    DefWndHandleWindowPosChanging_impl(pWnd, WinPos)
}

unsafe fn DefWndHandleWindowPosChanging_impl(pWnd: PWND, WinPos: PWINDOWPOS) -> LRESULT {
    let mut maxTrack: POINT = core::mem::zeroed();
    let mut minTrack: POINT = core::mem::zeroed();
    let style = (*pWnd).style;

    if ((*WinPos).flags & SWP_NOSIZE as u32) != 0 {
        return 0;
    }

    if (style & WS_THICKFRAME as u32) != 0 || (style & (WS_POPUP | WS_CHILD) as u32) == 0 {
        co_WinPosGetMinMaxInfo(
            pWnd,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(minTrack).cast(),
            core::ptr::addr_of_mut!(maxTrack).cast(),
        );
        (*WinPos).cx = min((*WinPos).cx, maxTrack.x);
        (*WinPos).cy = min((*WinPos).cy, maxTrack.y);
        if style & WS_MINIMIZE as u32 == 0 {
            if (*WinPos).cx < minTrack.x {
                (*WinPos).cx = minTrack.x;
            }
            if (*WinPos).cy < minTrack.y {
                (*WinPos).cy = minTrack.y;
            }
        }
    } else {
        (*WinPos).cx = max((*WinPos).cx, 0);
        (*WinPos).cy = max((*WinPos).cy, 0);
    }
    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DefWndHandleWindowPosChanged(pWnd: PWND, WinPos: PWINDOWPOS) -> LRESULT {
    DefWndHandleWindowPosChanged_impl(pWnd, WinPos)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DefWndHandleWindowPosChanged(pWnd: PWND, WinPos: PWINDOWPOS) -> LRESULT {
    DefWndHandleWindowPosChanged_impl(pWnd, WinPos)
}

unsafe fn DefWndHandleWindowPosChanged_impl(pWnd: PWND, WinPos: PWINDOWPOS) -> LRESULT {

    let mut Rect: RECT = core::mem::zeroed();
    let style = (*pWnd).style;

    IntGetClientRect(pWnd, (&mut Rect as *mut tagRECT).cast());
    IntMapWindowPoints(
        pWnd,
        if (style & WS_CHILD as u32) != 0 { IntGetParent(pWnd) } else { core::ptr::null_mut() },
        (&mut Rect as *mut RECT).cast(),
        2,
    );

    if ((*WinPos).flags & SWP_NOCLIENTMOVE as u32) == 0 {
        co_IntSendMessage(
            UserHMGetHandle(pWnd),
            WM_MOVE,
            0,
            MAKELONG(Rect.left as u16, Rect.top as u16),
        );
    }

    if ((*WinPos).flags & SWP_NOCLIENTSIZE as u32) == 0 || ((*WinPos).flags & SWP_STATECHANGED as u32) != 0 {
        if (style & WS_MINIMIZE as u32) != 0 {
            co_IntSendMessage(UserHMGetHandle(pWnd), WM_SIZE, SIZE_MINIMIZED as WPARAM, 0);
        } else {
            let wp: WPARAM = if (style & WS_MAXIMIZE as u32) != 0 {
                SIZE_MAXIMIZED as WPARAM
            } else {
                SIZE_RESTORED as WPARAM
            };
            co_IntSendMessage(
                UserHMGetHandle(pWnd),
                WM_SIZE,
                wp,
                MAKELONG((Rect.right - Rect.left) as u16, (Rect.bottom - Rect.top) as u16),
            );
        }
    }
    0
}

fastcall_fn! {
unsafe fn DefWndHandleSysCommand(mut pWnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut lResult: LRESULT = 0;
    let mut Hook: u8 = 0u8;

    if ISITHOOKED(WH_CBT as INT) != 0 || ((*(*(*pWnd).head.rpdesk).pDeskInfo).fsHooks & HOOKID_TO_FLAG(WH_CBT as INT)) != 0 {
        Hook = 1u8;
        lResult = co_HOOK_CallHooks(WH_CBT as INT, HCBT_SYSCOMMAND as _, wParam, lParam) as LRESULT;

        if lResult != 0 {
            return lResult;
        }
    }

    match (wParam & 0xfff0) as u32 {
        SC_MOVE | SC_SIZE => {
            DefWndDoSizeMove(pWnd, (wParam) as _);
        }

        SC_MINIMIZE => {
            if UserHMGetHandle(pWnd) == UserGetActiveWindow() {
                IntShowOwnedPopups(pWnd, 0u8 as _);
            }
            co_WinPosShowWindow(pWnd, SW_MINIMIZE as _);
        }

        SC_MAXIMIZE => {
            if ((*pWnd).style & WS_MINIMIZE as u32) != 0 && UserHMGetHandle(pWnd) == UserGetActiveWindow() {
                IntShowOwnedPopups(pWnd, 1u8 as _);
            }
            co_WinPosShowWindow(pWnd, SW_MAXIMIZE as _);
        }

        SC_RESTORE => {
            if ((*pWnd).style & WS_MINIMIZE as u32) != 0 && UserHMGetHandle(pWnd) == UserGetActiveWindow() {
                IntShowOwnedPopups(pWnd, 1u8 as _);
            }
            co_WinPosShowWindow(pWnd, SW_RESTORE as _);
        }

        SC_CLOSE => {
            return co_IntSendMessage(UserHMGetHandle(pWnd), WM_CLOSE, 0, 0);
        }

        SC_SCREENSAVE => {
            ERR!("Screensaver Called!\n");
            UserPostMessage(hwndSAS, WM_LOGONNOTIFY, LN_START_SCREENSAVE as _, 0);
        }

        SC_HOTKEY => {
            let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

            pWnd = ValidateHwndNoErr(lParam as HWND);
            if !pWnd.is_null() {
                if !(*pWnd).spwndLastActive.is_null() {
                    pWnd = (*pWnd).spwndLastActive;
                }
                UserRefObjectCo(pWnd, &mut Ref);
                co_IntSetForegroundWindow(pWnd);
                UserDerefObjectCo(pWnd);
                if (*pWnd).style & WS_MINIMIZE as u32 != 0 {
                    UserPostMessage(UserHMGetHandle(pWnd), WM_SYSCOMMAND, SC_RESTORE as _, 0);
                }
            }
        }

        SC_MOUSEMENU => {
            let mut Pt: POINT = core::mem::zeroed();
            Pt.x = LOWORD(lParam as u32) as i16 as _;
            Pt.y = HIWORD(lParam as u32) as i16 as _;
            MENU_TrackMouseMenuBar(pWnd, (wParam & 0x000f) as _, Pt);
        }

        SC_KEYMENU => {
            MENU_TrackKbdMenuBar(pWnd, wParam as UINT, lParam as u16);
        }

        _ => {
            return 0;
        }
    }

    if Hook != 0 { 1 } else { 0 }
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntFindChildWindowToOwner(RootWnd: PWND, Owner: PWND) -> PWND {
    co_IntFindChildWindowToOwner_impl(RootWnd, Owner)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntFindChildWindowToOwner(RootWnd: PWND, Owner: PWND) -> PWND {
    co_IntFindChildWindowToOwner_impl(RootWnd, Owner)
}

unsafe fn co_IntFindChildWindowToOwner_impl(RootWnd: PWND, Owner: PWND) -> PWND {
    let mut Child: PWND = (*RootWnd).spwndChild;
    while !Child.is_null() {
        let OwnerWnd: PWND = (*Child).spwndOwner;
        if OwnerWnd.is_null() {
            Child = (*Child).spwndNext;
            continue;
        }

        if ((*Child).style & WS_POPUP) == 0
            || ((*Child).style & WS_VISIBLE) == 0
            || (*head_pti(Owner)).MessageQueue != (*head_pti(Child)).MessageQueue
        {
            Child = (*Child).spwndNext;
            continue;
        }

        if OwnerWnd == Owner {
            return Child;
        }

        Child = (*Child).spwndNext;
    }
    core::ptr::null_mut()
}

unsafe extern "system" fn DefWndHandleSetCursor(pWnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut pwndPopUP: PWND = core::ptr::null_mut();
    let Msg: WORD = HIWORD(lParam as u32) as WORD;

    /* Not for child windows. */
    if UserHMGetHandle(pWnd) != wParam as HWND {
        return 0;
    }

    'outer: {
        match (LOWORD(lParam as u32) as i16) as i32 {
            x if x == HTERROR as i32 => {
                let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

                if Msg as u32 == WM_LBUTTONDOWN {
                    pwndPopUP = co_IntFindChildWindowToOwner(UserGetDesktopWindow(), pWnd);
                    if !pwndPopUP.is_null() {
                        if pwndPopUP != (*UserGetDesktopWindow()).spwndChild {
                            let pwndOrigActive: PWND = (*gpqForeground).spwndActive;

                            co_WinPosSetWindowPos(pWnd, core::ptr::null_mut(), 0, 0, 0, 0,
                                (SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) as UINT);

                            UserRefObjectCo(pwndPopUP, &mut Ref);
                            co_IntSetForegroundWindow(pwndPopUP);
                            UserDerefObjectCo(pwndPopUP);

                            if pwndOrigActive != (*gpqForeground).spwndActive {
                                break 'outer;
                            }
                        }
                    }
                }

                if Msg as u32 == WM_LBUTTONDOWN || Msg as u32 == WM_MBUTTONDOWN ||
                   Msg as u32 == WM_RBUTTONDOWN || Msg as u32 == WM_XBUTTONDOWN {
                    if !pwndPopUP.is_null() {
                        let mut fwi: FLASHWINFO = core::mem::zeroed();
                        fwi.cbSize = core::mem::size_of::<FLASHWINFO>() as UINT;
                        fwi.hwnd = UserHMGetHandle(pwndPopUP);
                        fwi.dwFlags = FLASHW_ALL;
                        fwi.uCount = gspv.dwForegroundFlashCount;
                        fwi.dwTimeout = ((*pusi(gpsi)).dtCaretBlink >> 3) as UINT;

                        IntFlashWindowEx(pwndPopUP, &mut fwi);
                    }
                    UserPostMessage(hwndSAS, WM_LOGONNOTIFY, LN_MESSAGE_BEEP as WPARAM, 0);
                }
            }

            x if x == HTCLIENT as i32 => {
                if !(*(*pWnd).pcls).spcur.is_null() {
                    IntSystemSetCursor((*(*pWnd).pcls).spcur);
                }
                return 0;
            }

            x if x == HTLEFT as i32 || x == HTRIGHT as i32 => {
                if (*pWnd).style & WS_MAXIMIZE != 0 {
                    break 'outer;
                }
                IntSystemSetCursor(SYSTEMCUR(SIZEWE));
                return 1;
            }

            x if x == HTTOP as i32 || x == HTBOTTOM as i32 => {
                if (*pWnd).style & WS_MAXIMIZE != 0 {
                    break 'outer;
                }
                IntSystemSetCursor(SYSTEMCUR(SIZENS));
                return 1;
            }

            x if x == HTTOPLEFT as i32 || x == HTBOTTOMRIGHT as i32 => {
                if (*pWnd).style & WS_MAXIMIZE != 0 {
                    break 'outer;
                }
                IntSystemSetCursor(SYSTEMCUR(SIZENWSE));
                return 1;
            }

            x if x == HTBOTTOMLEFT as i32 || x == HTTOPRIGHT as i32 => {
                if (*pWnd).style & WS_MAXIMIZE != 0 {
                    break 'outer;
                }
                IntSystemSetCursor(SYSTEMCUR(SIZENESW));
                return 1;
            }

            _ => {}
        }
    }

    IntSystemSetCursor(SYSTEMCUR(ARROW));
    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DefWndPrint(pwnd: PWND, hdc: HDC, uFlags: ULONG) {
    DefWndPrint_impl(pwnd, hdc, uFlags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DefWndPrint(pwnd: PWND, hdc: HDC, uFlags: ULONG) {
    DefWndPrint_impl(pwnd, hdc, uFlags)
}

unsafe fn DefWndPrint_impl(pwnd: PWND, hdc: HDC, uFlags: ULONG) {

    if (uFlags & PRF_CHECKVISIBLE) != 0 && IntIsWindowVisible(pwnd) == 0 {
        return;
    }

    if (uFlags & PRF_CHILDREN) != 0
        || (uFlags & PRF_OWNED) != 0
        || (uFlags & PRF_NONCLIENT) != 0
    {
        FIXME!(b"WM_PRINT message with unsupported flags\n\0".as_ptr());
    }

    if (uFlags & PRF_ERASEBKGND) != 0 {
        co_IntSendMessage(UserHMGetHandle(pwnd), WM_ERASEBKGND, hdc as WPARAM, 0);
    }

    if (uFlags & PRF_CLIENT) != 0 {
        co_IntSendMessage(UserHMGetHandle(pwnd), WM_PRINTCLIENT, hdc as WPARAM, uFlags as LPARAM);
    }
}

#[no_mangle]
unsafe extern "C" fn UserPaintCaption(pWnd: PWND, mut Flags: INT) -> BOOL {
    let mut Ret: BOOL = 0;

    if ((*pWnd).style & WS_VISIBLE) != 0 && ((*pWnd).style & WS_CAPTION) == WS_CAPTION {
        if (*pWnd).state & WNDS_HASCAPTION != 0
            && (*head_pti(pWnd)).MessageQueue == gpqForeground
        {
            Flags |= DC_ACTIVE as INT;
        }

        if (*gpsi).dwSRVIFlags & SRVINFO_APIHOOK != 0 {
            co_IntSendMessage(
                UserHMGetHandle(pWnd),
                WM_NCUAHDRAWCAPTION,
                Flags as WPARAM,
                0,
            );
        } else {
            let hDC = UserGetDCEx(pWnd, core::ptr::null_mut(), DCX_WINDOW | DCX_USESTYLE);
            UserDrawCaptionBar(pWnd, hDC, Flags | DC_FRAME as INT);
            UserReleaseDC(pWnd, hDC, 0);
        }
        Ret = 1;
    }
    Ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DefWndSetIcon(pWnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    DefWndSetIcon_impl(pWnd, wParam, lParam)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DefWndSetIcon(pWnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    DefWndSetIcon_impl(pWnd, wParam, lParam)
}

unsafe fn DefWndSetIcon_impl(pWnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut hIcon: HICON;
    let mut hIconSmall: HICON;
    let hIconOld: HICON;

    if wParam > ICON_SMALL2 as WPARAM {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    hIconSmall = UserGetProp(pWnd, (*gpsi).atomIconSmProp, 1u8) as HICON;
    hIcon = UserGetProp(pWnd, (*gpsi).atomIconProp, 1u8) as HICON;

    hIconOld = if wParam == ICON_BIG as WPARAM { hIcon } else { hIconSmall };

    match wParam as u32 {
        ICON_BIG => {
            hIcon = lParam as HICON;
        }
        ICON_SMALL => {
            hIconSmall = lParam as HICON;
        }
        ICON_SMALL2 => {
            ERR!("FIXME: Set ICON_SMALL2 support!\n");
        }
        _ => {}
    }

    UserSetProp(pWnd, (*gpsi).atomIconProp, hIcon as _, 1u8);
    UserSetProp(pWnd, (*gpsi).atomIconSmProp, hIconSmall as _, 1u8);

    if ((*pWnd).style & WS_CAPTION as u32) == WS_CAPTION as u32 {
        UserPaintCaption(pWnd, DC_ICON as INT);
    }

    hIconOld as LRESULT
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DefWndGetIcon(pWnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    DefWndGetIcon_impl(pWnd, wParam, lParam)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DefWndGetIcon(pWnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    DefWndGetIcon_impl(pWnd, wParam, lParam)
}

unsafe fn DefWndGetIcon_impl(pWnd: PWND, wParam: WPARAM, _lParam: LPARAM) -> LRESULT {
    let hIconRet: HICON;
    if wParam > ICON_SMALL2 as WPARAM {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if wParam == ICON_BIG as WPARAM {
        hIconRet = UserGetProp(pWnd, (*gpsi).atomIconProp, 1u8) as HICON;
    } else {
        hIconRet = UserGetProp(pWnd, (*gpsi).atomIconSmProp, 1u8) as HICON;
    }
    hIconRet as LRESULT
}

// DWP_GetEnabledPopup — FASTCALL (window.h `@DWP_GetEnabledPopup@4`).
#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn DWP_GetEnabledPopup(pWnd: PWND) -> PWND {
    DWP_GetEnabledPopup_impl(pWnd)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn DWP_GetEnabledPopup(pWnd: PWND) -> PWND {
    DWP_GetEnabledPopup_impl(pWnd)
}
unsafe fn DWP_GetEnabledPopup_impl(pWnd: PWND) -> PWND {
    let mut pwndNode1: PWND;
    let pti: PTHREADINFO = head_pti(pWnd);
    let mut ptiNode: PTHREADINFO;
    let mut bFoundNullNode: u8 = 0u8;

    pwndNode1 = (*pWnd).spwndNext;
    while pwndNode1 != pWnd {
        if pwndNode1.is_null() {
            if bFoundNullNode != 0 {
                return core::ptr::null_mut();
            }
            bFoundNullNode = 1u8;
            pwndNode1 = (*(*pWnd).spwndParent).spwndChild;
            continue;
        }

        ptiNode = head_pti(pwndNode1);
        if ((((*pti).TIF_flags & TIF_16BIT) == 0) && (*ptiNode).MessageQueue == (*pti).MessageQueue)
            || ((((*pti).TIF_flags & TIF_16BIT) != 0) && ptiNode == pti)
        {
            let style: DWORD = (*pwndNode1).style;
            if (style & WS_VISIBLE) != 0 && (style & WS_DISABLED) == 0 {
                let mut pwndNode2: PWND = (*pwndNode1).spwndOwner;
                while !pwndNode2.is_null() {
                    if pwndNode2 == pWnd {
                        return pwndNode1;
                    }
                    pwndNode2 = (*pwndNode2).spwndOwner;
                }
            }
        }

        pwndNode1 = (*pwndNode1).spwndNext;
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DefWndScreenshot(pWnd: PWND) {
    DefWndScreenshot_impl(pWnd)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DefWndScreenshot(pWnd: PWND) {
    DefWndScreenshot_impl(pWnd)
}

unsafe fn DefWndScreenshot_impl(pWnd: PWND) {
    let mut rect: RECT = core::mem::zeroed();
    let hdc: HDC;
    let w: INT;
    let h: INT;
    let hbitmap: HBITMAP;
    let hdc2: HDC;
    let mut scd: SETCLIPBDATA = SETCLIPBDATA {
        fGlobalHandle: 0,
        fIncSerialNumber: 0,
    };

    UserOpenClipboard(UserHMGetHandle(pWnd));
    UserEmptyClipboard();

    hdc = UserGetWindowDC(pWnd);
    IntGetWindowRect(pWnd, (&mut rect as *mut tagRECT).cast());
    w = rect.right - rect.left;
    h = rect.bottom - rect.top;

    hbitmap = NtGdiCreateCompatibleBitmap(hdc, w, h);
    hdc2 = NtGdiCreateCompatibleDC(hdc);
    NtGdiSelectBitmap(hdc2, hbitmap);

    NtGdiBitBlt(hdc2, 0, 0, w, h, hdc, 0, 0, SRCCOPY as u32, CLR_INVALID, 0);

    UserSetClipboardData(CF_BITMAP as u32, hbitmap as *mut core::ffi::c_void, &mut scd);

    UserReleaseDC(pWnd, hdc, 0);
    UserReleaseDC(pWnd, hdc2, 0);

    UserCloseClipboard();
}

unsafe fn co_UserTrackSystemMenu(pWnd: PWND, nClickPos: LONG, puCmdType: PUINT) -> BOOL {
    let mut MenuRef: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut WndRef: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let pMenu: PMENU;
    let uDefaultCmd: UINT;

    FIXME!(b"co_UserTrackSystemMenu() called\n\0".as_ptr() as *const u8);

    UserRefObjectCo(pWnd as _, &mut WndRef);

    if ((*pWnd).style & WS_DISABLED as u32) != 0
        || ((*pWnd).state2 & WNDS2_INDESTROY) != 0
        || ((*head_pti(pWnd)).MessageQueue != gpqForeground
            && co_IntSetForegroundWindow(pWnd) == 0)
    {
        UserDerefObjectCo(pWnd as _);
        return 0;
    }

    pMenu = IntGetSystemMenu(pWnd, 0);
    if pMenu.is_null() {
        UserDerefObjectCo(pWnd as _);
        return 0;
    }
    UserRefObjectCo(pMenu as _, &mut MenuRef);

    if !puCmdType.is_null() {
        uDefaultCmd = *puCmdType;
    } else if ((*pWnd).style & (WS_MINIMIZE | WS_MAXIMIZE) as u32) != 0 {
        uDefaultCmd = SC_RESTORE;
    } else {
        uDefaultCmd = SC_MAXIMIZE;
    }
    UserSetMenuDefaultItem(pMenu, uDefaultCmd, 0);

    if nClickPos == -1 {
        FIXME!(b"Use WM_KLUDGEMINRECT and TPM_VERTICAL\n\0".as_ptr() as *const u8);
    }

    IntTrackPopupMenuEx(
        pMenu,
        (TPM_LEFTBUTTON | TPM_RIGHTBUTTON | TPM_SYSTEM_MENU) as u32,
        LOWORD(nClickPos as u32) as i32,
        HIWORD(nClickPos as u32) as i32,
        pWnd,
        core::ptr::null_mut(),
    );

    UserDerefObjectCo(pMenu as _);
    UserDerefObjectCo(pWnd as _);
    1
}

unsafe fn IntDefWindowProc_WM_DEVICECHANGE() -> LRESULT {
    1 as LRESULT
}

unsafe fn IntDefWindowProc_WM_CLIENTSHUTDOWN(Wnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    IntClientShutdown(Wnd, wParam, lParam)
}

unsafe fn IntDefWindowProc_WM_NCLBUTTONDOWN(Wnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    NC_HandleNCLButtonDown(Wnd, wParam, lParam)
}

unsafe fn IntDefWindowProc_WM_NCRBUTTONDOWN(Wnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    NC_HandleNCRButtonDown(Wnd, wParam, lParam)
}

unsafe fn IntDefWindowProc_WM_LBUTTONDBLCLK(Wnd: PWND, lParam: LPARAM) -> LRESULT {
    NC_HandleNCLButtonDblClk(Wnd, HTCLIENT as WPARAM, lParam)
}

unsafe fn IntDefWindowProc_WM_NCLBUTTONDBLCLK(Wnd: PWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    NC_HandleNCLButtonDblClk(Wnd, wParam, lParam)
}

unsafe fn IntDefWindowProc_WM_CTLCOLORMSGBOX(Msg: UINT, wParam: WPARAM) -> LRESULT {
    DefWndControlColor(wParam as HDC, Msg - WM_CTLCOLORMSGBOX) as LRESULT
}

unsafe fn IntDefWindowProc_WM_CTLCOLOR(wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    DefWndControlColor(wParam as HDC, ((lParam >> 16) & 0xffff) as UINT) as LRESULT
}

unsafe fn IntDefWindowProc_WM_GETHOTKEY(Wnd: PWND) -> LRESULT {
    DefWndGetHotKey(Wnd) as LRESULT
}

unsafe fn IntDefWindowProc_WM_SETHOTKEY(Wnd: PWND, wParam: WPARAM) -> LRESULT {
    DefWndSetHotKey(Wnd, wParam) as LRESULT
}

unsafe fn IntDefWindowProc_WM_SETREDRAW(Wnd: PWND, wParam: WPARAM) -> LRESULT {
    if wParam != 0 {
        if (*Wnd).style & WS_VISIBLE == 0 {
            IntSetStyle(Wnd, WS_VISIBLE, 0);
            (*Wnd).state |= WNDS_SENDNCPAINT;
        }
    } else {
        if (*Wnd).style & WS_VISIBLE != 0 {
            co_UserRedrawWindow(Wnd, core::ptr::null_mut(), core::ptr::null_mut(), RDW_ALLCHILDREN | RDW_VALIDATE);
            IntSetStyle(Wnd, 0, WS_VISIBLE);
        }
    }
    0
}

fastcall_fn! {
#[no_mangle]
unsafe fn IntDefWindowProc(
    Wnd: PWND,
    Msg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
    Ansi: BOOL,
) -> LRESULT {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let mut lResult: LRESULT = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let Status: NTSTATUS;

    if Msg > WM_USER {
        return 0;
    }

    match Msg {
        WM_DEVICECHANGE => {
            return IntDefWindowProc_WM_DEVICECHANGE();
        }

        WM_GETTEXTLENGTH => {
            let buf: PWSTR;
            let mut len: ULONG = 0;

            if !Wnd.is_null() && (*Wnd).strName.Length != 0 {
                buf = (*Wnd).strName.Buffer;
                if !buf.is_null()
                    && NT_SUCCESS(RtlUnicodeToMultiByteSize(
                        &mut len,
                        buf as *mut _,
                        (*Wnd).strName.Length as ULONG,
                    ))
                {
                    lResult = ((*Wnd).strName.Length as usize
                        / core::mem::size_of::<u16>()) as LRESULT;
                }
            }
        }

        WM_GETTEXT => {
            let mut buf: PWSTR = core::ptr::null_mut();
            let outbuf: PWSTR = lParam as PWSTR;

            if !Wnd.is_null() && wParam != 0 {
                if !(*Wnd).strName.Buffer.is_null() {
                    buf = (*Wnd).strName.Buffer;
                } else {
                    *outbuf.offset(0) = 0;
                }

                if !buf.is_null() {
                    if (*Wnd).strName.Length != 0 {
                        let a = (*Wnd).strName.Length as usize / core::mem::size_of::<u16>();
                        let b = (wParam - 1) as usize;
                        lResult = (if a < b { a } else { b }) as LRESULT;
                        core::ptr::copy_nonoverlapping(
                            buf,
                            outbuf,
                            lResult as usize,
                        );
                        *outbuf.offset(lResult as isize) = 0;
                    } else {
                        *outbuf.offset(0) = 0;
                    }
                }
            }
        }

        WM_SETTEXT => {
            DefSetText(Wnd, lParam as PCWSTR);

            if ((*Wnd).style & WS_CAPTION as ULONG) == WS_CAPTION as ULONG {
                UserPaintCaption(Wnd, DC_TEXT as INT);
            }
            IntNotifyWinEvent(
                EVENT_OBJECT_NAMECHANGE,
                Wnd,
                OBJID_WINDOW as LONG,
                CHILDID_SELF as LONG,
                0,
            );
            lResult = 1;
        }

        WM_SYSCOMMAND => {
            lResult = DefWndHandleSysCommand(Wnd, wParam, lParam);
        }

        WM_SHOWWINDOW => {
            'block: {
                if ((*Wnd).style & WS_VISIBLE as ULONG) != 0 && wParam != 0 {
                    break 'block;
                }
                if ((*Wnd).style & WS_VISIBLE as ULONG) == 0 && wParam == 0 {
                    break 'block;
                }
                if (*Wnd).spwndOwner.is_null() {
                    break 'block;
                }
                if (lParam & 0xFFFF) != 0 {
                    co_WinPosShowWindow(
                        Wnd,
                        if wParam != 0 { SW_SHOWNOACTIVATE } else { SW_HIDE } as INT,
                    );
                }
            }
        }

        WM_CLIENTSHUTDOWN => {
            return IntDefWindowProc_WM_CLIENTSHUTDOWN(Wnd, wParam, lParam);
        }

        WM_APPCOMMAND => {
            'block: {
                if ((*Wnd).style & (WS_POPUP | WS_CHILD) as ULONG) != WS_CHILD as ULONG
                    && Wnd != co_GetDesktopWindow(Wnd)
                {
                    if co_HOOK_CallHooks(WH_SHELL as INT, HSHELL_APPCOMMAND as INT, wParam, lParam) == 0 {
                        co_IntShellHookNotify(HSHELL_APPCOMMAND as WPARAM, wParam, lParam);
                    }
                    break 'block;
                }
                let Parent = (*Wnd).spwndParent;
                UserRefObjectCo(Parent, &mut Ref);
                lResult = co_IntSendMessage(UserHMGetHandle(Parent), WM_APPCOMMAND, wParam, lParam);
                UserDerefObjectCo(Parent);
            }
        }

        WM_POPUPSYSTEMMENU => {
            co_UserTrackSystemMenu(Wnd, lParam as LONG, core::ptr::null_mut());
        }

        WM_KEYF1 => {
            let mut hi: HELPINFO = core::mem::zeroed();
            let hMenu: HMENU = (*Wnd).IDMenu as HMENU;
            let pwndActive = MENU_IsMenuActive();
            hi.cbSize = core::mem::size_of::<HELPINFO>() as UINT;
            hi.MousePos = (*pusi(gpsi)).ptCursor;
            hi.iContextType = HELPINFO_MENUITEM as INT;
            hi.hItemHandle = if !pwndActive.is_null() {
                UserHMGetHandle(pwndActive) as *mut core::ffi::c_void
            } else {
                UserHMGetHandle(Wnd) as *mut core::ffi::c_void
            };
            hi.iCtrlId = if ((*Wnd).style & (WS_POPUP | WS_CHILD) as ULONG) == WS_CHILD as ULONG {
                IntMenuItemFromPoint(Wnd, hMenu, hi.MousePos)
            } else {
                0
            };
            hi.dwContextId = IntGetWindowContextHelpId(Wnd) as ULONG_PTR;

            co_IntSendMessage(UserHMGetHandle(Wnd), WM_HELP, 0, &mut hi as *mut HELPINFO as LPARAM);
        }

        WM_SETICON => {
            return DefWndSetIcon(Wnd, wParam, lParam);
        }

        WM_GETICON => {
            return DefWndGetIcon(Wnd, wParam, lParam);
        }

        WM_HELP => {
            let Parent = IntGetParent(Wnd);
            co_IntSendMessage(UserHMGetHandle(Parent), Msg, wParam, lParam);
        }

        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
            (*(*pti).MessageQueue).QF_flags &=
                !(QF_FMENUSTATUS | QF_FMENUSTATUSBREAK) as ULONG;
        }

        WM_NCLBUTTONDOWN => {
            return IntDefWindowProc_WM_NCLBUTTONDOWN(Wnd, wParam, lParam);
        }

        WM_NCRBUTTONDOWN => {
            return IntDefWindowProc_WM_NCRBUTTONDOWN(Wnd, wParam, lParam);
        }

        WM_LBUTTONDBLCLK => {
            return IntDefWindowProc_WM_LBUTTONDBLCLK(Wnd, lParam);
        }

        WM_NCLBUTTONDBLCLK => {
            return IntDefWindowProc_WM_NCLBUTTONDBLCLK(Wnd, wParam, lParam);
        }

        WM_RBUTTONUP => {
            let mut Pt: POINT = core::mem::zeroed();
            Pt.x = (lParam as i16) as LONG;
            Pt.y = ((lParam >> 16) as i16) as LONG;
            IntClientToScreen(Wnd, (&mut Pt as *mut tagPOINT).cast());
            lParam = ((Pt.x as u16 as u32) | ((Pt.y as u16 as u32) << 16)) as LPARAM;
            co_IntSendMessage(
                UserHMGetHandle(Wnd),
                WM_CONTEXTMENU,
                UserHMGetHandle(Wnd) as WPARAM,
                lParam,
            );
        }

        WM_NCRBUTTONUP => {}

        WM_XBUTTONUP | WM_NCXBUTTONUP => {
            let hi = (wParam >> 16) as u16 as u32;
            if hi == XBUTTON1 || hi == XBUTTON2 {
                let lo = (wParam & 0xFFFF) as u16 as u32;
                co_IntSendMessage(
                    UserHMGetHandle(Wnd),
                    WM_APPCOMMAND,
                    UserHMGetHandle(Wnd) as WPARAM,
                    ((lo & 0xFFFF) | (((FAPPCOMMAND_MOUSE | hi) & 0xFFFF) << 16)) as LPARAM,
                );
            }
        }

        WM_CONTEXTMENU => {
            if ((*Wnd).style & WS_CHILD as ULONG) != 0 {
                co_IntSendMessage(
                    UserHMGetHandle(IntGetParent(Wnd)),
                    Msg,
                    UserHMGetHandle(Wnd) as WPARAM,
                    lParam,
                );
            } else {
                let mut Pt: POINT = core::mem::zeroed();
                let Style: LONG_PTR;
                let HitCode: LONG;

                Style = (*Wnd).style as LONG_PTR;

                Pt.x = (lParam as i16) as LONG;
                Pt.y = ((lParam >> 16) as i16) as LONG;
                if (Style & WS_CHILD as LONG_PTR) != 0 {
                    IntScreenToClient(IntGetParent(Wnd), (&mut Pt as *mut tagPOINT).cast());
                }

                HitCode = GetNCHitEx(Wnd, Pt);

                if HitCode == HTCAPTION as LONG || HitCode == HTSYSMENU as LONG {
                    let SystemMenu = IntGetSystemMenu(Wnd, 0);
                    if !SystemMenu.is_null() {
                        MENU_InitSysMenuPopup(
                            SystemMenu,
                            (*Wnd).style,
                            (*(*Wnd).pcls).style,
                            HitCode,
                        );

                        let Flags = if HitCode == HTCAPTION as LONG {
                            TPM_LEFTBUTTON | TPM_RIGHTBUTTON
                        } else {
                            TPM_LEFTBUTTON
                        };

                        IntTrackPopupMenuEx(
                            SystemMenu,
                            Flags | TPM_SYSTEM_MENU,
                            Pt.x,
                            Pt.y,
                            Wnd,
                            core::ptr::null_mut(),
                        );
                    }
                }
            }
        }

        WM_KEYDOWN => {
            if wParam == VK_F10 as WPARAM {
                (*(*pti).MessageQueue).QF_flags |= QF_FF10STATUS as ULONG;

                if (UserGetKeyState(VK_SHIFT as DWORD) as i32 & 0x8000) != 0 {
                    co_IntSendMessage(
                        UserHMGetHandle(Wnd),
                        WM_CONTEXTMENU,
                        UserHMGetHandle(Wnd) as WPARAM,
                        (((-1i32 as u16 as u32) | ((-1i32 as u16 as u32) << 16)) as i32) as LPARAM,
                    );
                }
            }
            // input.h IS_KEY_DOWN: async key state is 2 bits per key —
            // byte (vk*2)/8, KS_DOWN_BIT mask 1 << ((vk%4)*2).
            let lwin_down = (*(*(&raw const gafAsyncKeyState)).as_ptr().offset((VK_LWIN as usize * 2 / 8) as isize)
                & (1 << ((VK_LWIN as usize % 4) * 2))) != 0;
            let rwin_down = (*(*(&raw const gafAsyncKeyState)).as_ptr().offset((VK_RWIN as usize * 2 / 8) as isize)
                & (1 << ((VK_RWIN as usize % 4) * 2))) != 0;
            if g_bWindowSnapEnabled != 0 && (lwin_down || rwin_down) {
                let hwndTop = UserGetForegroundWindow();
                let topWnd = UserGetWindowObject(hwndTop);
                let mut allowSnap: BOOL;

                if topWnd.is_null() {
                    return 0;
                }

                allowSnap = IntIsSnapAllowedForWindow(topWnd) as BOOL;
                if allowSnap == 0
                    && ((*topWnd).style & (WS_MINIMIZEBOX | WS_THICKFRAME) as ULONG)
                        == WS_MINIMIZEBOX as ULONG
                {
                    allowSnap = (wParam == VK_DOWN as WPARAM) as BOOL;
                }

                if allowSnap != 0 {
                    let mut snapped: UINT = IntGetWindowSnapEdge(topWnd);

                    if wParam == VK_DOWN as WPARAM {
                        if ((*topWnd).style & WS_MAXIMIZE as ULONG) != 0 {
                            co_IntSendMessage(hwndTop, WM_SYSCOMMAND, SC_RESTORE as WPARAM, 0x10000);
                        } else if snapped != 0 {
                            co_IntUnsnapWindow(topWnd);
                        } else {
                            co_IntSendMessage(hwndTop, WM_SYSCOMMAND, SC_MINIMIZE as WPARAM, 0x10000);
                        }
                    } else if wParam == VK_UP as WPARAM {
                        if ((*topWnd).style & WS_MINIMIZE as ULONG) != 0 {
                            co_IntSendMessage(hwndTop, WM_SYSCOMMAND, SC_RESTORE as WPARAM, 0x10000);
                        } else {
                            co_IntSendMessage(hwndTop, WM_SYSCOMMAND, SC_MAXIMIZE as WPARAM, 0x10000);
                        }
                    } else if wParam == VK_LEFT as WPARAM || wParam == VK_RIGHT as WPARAM {
                        let edge: UINT = if wParam == VK_LEFT as WPARAM { HTLEFT } else { HTRIGHT };
                        let otherEdge: UINT = if edge == HTLEFT { HTRIGHT } else { HTLEFT };

                        if ((*topWnd).style & WS_MAXIMIZE as ULONG) != 0 {
                            let normalRect: RECTL = (*topWnd).InternalPos.NormalRect;
                            co_IntCalculateSnapPosition(
                                topWnd,
                                edge,
                                core::ptr::addr_of_mut!((*topWnd).InternalPos.NormalRect),
                            );
                            IntSetSnapEdge(topWnd, edge);
                            co_IntSendMessage(hwndTop, WM_SYSCOMMAND, SC_RESTORE as WPARAM, 0x10000);
                            IntSetSnapInfo(topWnd, edge, &normalRect as *const RECTL);
                            snapped = HTNOWHERE;
                        }

                        if snapped == otherEdge {
                            co_IntUnsnapWindow(topWnd);
                        } else {
                            co_IntSnapWindow(topWnd, edge);
                        }
                    }
                }
            }
        }

        WM_SYSKEYDOWN => {
            if ((lParam >> 16) as u32 & KF_ALTDOWN as u32) != 0 {
                if ((lParam >> 16) as u32 & KF_REPEAT as u32) == 0 {
                    if (wParam == VK_MENU as WPARAM
                        || wParam == VK_LMENU as WPARAM
                        || wParam == VK_RMENU as WPARAM)
                        && ((*(*pti).MessageQueue).QF_flags & QF_FMENUSTATUS as ULONG) == 0
                    {
                        (*(*pti).MessageQueue).QF_flags |= QF_FMENUSTATUS as ULONG;
                    } else {
                        (*(*pti).MessageQueue).QF_flags &= !(QF_FMENUSTATUS as ULONG);
                    }
                }

                (*(*pti).MessageQueue).QF_flags &= !(QF_FF10STATUS as ULONG);

                if wParam == VK_F4 as WPARAM {
                    let top = UserGetAncestor(Wnd, GA_ROOT);
                    if ((*(*top).pcls).style & CS_NOCLOSE) == 0 {
                        UserPostMessage(UserHMGetHandle(top), WM_SYSCOMMAND, SC_CLOSE as WPARAM, 0);
                    }
                } else if wParam == VK_SNAPSHOT as WPARAM {
                    let mut pwnd = Wnd;
                    while !IntGetParent(pwnd).is_null() {
                        pwnd = IntGetParent(pwnd);
                    }
                    DefWndScreenshot(pwnd);
                } else if wParam == VK_ESCAPE as WPARAM || wParam == VK_TAB as WPARAM {
                    let Active = UserGetActiveWindow();
                    if Active.is_null() {
                    } else {
                        let wParamTmp: WPARAM =
                            if (UserGetKeyState(VK_SHIFT as DWORD) as i32 & 0x8000) != 0 {
                                SC_PREVWINDOW as WPARAM
                            } else {
                                SC_NEXTWINDOW as WPARAM
                            };
                        co_IntSendMessage(Active, WM_SYSCOMMAND, wParamTmp, wParam as LPARAM);
                    }
                }
            } else if wParam == VK_F10 as WPARAM {
                if (UserGetKeyState(VK_SHIFT as DWORD) as i32 & 0x8000) != 0 {
                    co_IntSendMessage(
                        UserHMGetHandle(Wnd),
                        WM_CONTEXTMENU,
                        UserHMGetHandle(Wnd) as WPARAM,
                        (((-1i32 as u16 as u32) | ((-1i32 as u16 as u32) << 16)) as i32) as LPARAM,
                    );
                }
                (*(*pti).MessageQueue).QF_flags |= QF_FF10STATUS as ULONG;
            } else if wParam == VK_ESCAPE as WPARAM
                && (UserGetKeyState(VK_SHIFT as DWORD) as i32 & 0x8000) != 0
            {
                co_IntSendMessage(
                    UserHMGetHandle(Wnd),
                    WM_SYSCOMMAND,
                    SC_KEYMENU as WPARAM,
                    b' ' as LPARAM,
                );
            }
        }

        WM_KEYUP | WM_SYSKEYUP => {
            if ((wParam == VK_MENU as WPARAM
                || wParam == VK_LMENU as WPARAM
                || wParam == VK_RMENU as WPARAM)
                && ((*(*pti).MessageQueue).QF_flags
                    & (QF_FMENUSTATUS | QF_FMENUSTATUSBREAK) as ULONG)
                    == QF_FMENUSTATUS as ULONG)
                || (wParam == VK_F10 as WPARAM
                    && ((*(*pti).MessageQueue).QF_flags & QF_FF10STATUS as ULONG) != 0)
            {
                co_IntSendMessage(
                    UserHMGetHandle(UserGetAncestor(Wnd, GA_ROOT)),
                    WM_SYSCOMMAND,
                    SC_KEYMENU as WPARAM,
                    0,
                );
            }
            (*(*pti).MessageQueue).QF_flags &=
                !(QF_FMENUSTATUS | QF_FMENUSTATUSBREAK | QF_FF10STATUS) as ULONG;
        }

        WM_SYSCHAR => {
            'block: {
                (*(*pti).MessageQueue).QF_flags &=
                    !(QF_FMENUSTATUS | QF_FMENUSTATUSBREAK) as ULONG;
                if wParam == VK_RETURN as WPARAM && ((*Wnd).style & WS_MINIMIZE as ULONG) != 0 {
                    UserPostMessage(UserHMGetHandle(Wnd), WM_SYSCOMMAND, SC_RESTORE as WPARAM, 0);
                    break 'block;
                }
                if ((lParam >> 16) as u32 & KF_ALTDOWN as u32) != 0 && wParam != 0 {
                    if wParam == VK_TAB as WPARAM || wParam == VK_ESCAPE as WPARAM {
                        break 'block;
                    }
                    if wParam == VK_SPACE as WPARAM && ((*Wnd).style & WS_CHILD as ULONG) != 0 {
                        co_IntSendMessage(
                            UserHMGetHandle(IntGetParent(Wnd)),
                            Msg,
                            wParam,
                            lParam,
                        );
                    } else {
                        co_IntSendMessage(
                            UserHMGetHandle(Wnd),
                            WM_SYSCOMMAND,
                            SC_KEYMENU as WPARAM,
                            wParam as LPARAM,
                        );
                    }
                } else if wParam != VK_ESCAPE as WPARAM {
                    UserPostMessage(hwndSAS, WM_LOGONNOTIFY, LN_MESSAGE_BEEP as WPARAM, 0);
                }
            }
        }

        WM_CANCELMODE => {
            (*(*pti).MessageQueue).QF_flags &=
                !(QF_FMENUSTATUS | QF_FMENUSTATUSBREAK) as ULONG;

            MENU_EndMenu(Wnd);
            if IntGetCaptureWindow() == UserHMGetHandle(Wnd) {
                IntReleaseCapture();
            }
        }

        WM_CLOSE => {
            co_UserDestroyWindow(Wnd);
        }

        WM_CTLCOLORMSGBOX
        | WM_CTLCOLOREDIT
        | WM_CTLCOLORLISTBOX
        | WM_CTLCOLORBTN
        | WM_CTLCOLORDLG
        | WM_CTLCOLORSTATIC
        | WM_CTLCOLORSCROLLBAR => {
            return IntDefWindowProc_WM_CTLCOLORMSGBOX(Msg, wParam);
        }

        WM_CTLCOLOR => {
            return IntDefWindowProc_WM_CTLCOLOR(wParam, lParam);
        }

        WM_SETCURSOR => {
            if ((*Wnd).style & WS_CHILD as ULONG) != 0 {
                let lo = (lParam & 0xFFFF) as u16 as u32;
                if lo < HTLEFT || lo > HTBOTTOMRIGHT {
                    let parent = (*Wnd).spwndParent;
                    if parent != UserGetDesktopWindow()
                        && co_IntSendMessage(UserHMGetHandle(parent), WM_SETCURSOR, wParam, lParam) != 0
                    {
                        return 1;
                    }
                }
            }
            return DefWndHandleSetCursor(Wnd, wParam, lParam);
        }

        WM_MOUSEACTIVATE => {
            let mut done = false;
            if ((*Wnd).style & WS_CHILD as ULONG) != 0 {
                let pwndParent = IntGetParent(Wnd);
                let hwndParent = if !pwndParent.is_null() {
                    UserHMGetHandle(pwndParent)
                } else {
                    core::ptr::null_mut()
                };
                if !hwndParent.is_null() {
                    lResult = co_IntSendMessage(hwndParent, WM_MOUSEACTIVATE, wParam, lParam);
                    if lResult != 0 {
                        done = true;
                    }
                }
            }
            if !done {
                return if (lParam >> 16) as u16 as u32 == WM_LBUTTONDOWN
                    && (lParam & 0xFFFF) as u16 as u32 == HTCAPTION
                {
                    MA_NOACTIVATE as LRESULT
                } else {
                    MA_ACTIVATE as LRESULT
                };
            }
        }

        WM_ACTIVATE => {
            if (wParam & 0xFFFF) as u16 as u32 != WA_INACTIVE
                && ((*Wnd).style & WS_MINIMIZE as ULONG) == 0
            {
                co_UserSetFocus(Wnd);
            }
        }

        WM_MOUSEWHEEL => {
            if ((*Wnd).style & WS_CHILD as ULONG) != 0 {
                let pwndParent = IntGetParent(Wnd);
                let hwndParent = if !pwndParent.is_null() {
                    UserHMGetHandle(pwndParent)
                } else {
                    core::ptr::null_mut()
                };
                return co_IntSendMessage(hwndParent, WM_MOUSEWHEEL, wParam, lParam);
            }
        }

        WM_ERASEBKGND | WM_ICONERASEBKGND => {
            let mut Rect: RECT = core::mem::zeroed();
            let mut hBrush: HBRUSH = (*(*Wnd).pcls).hbrBackground;
            if hBrush.is_null() {
                return 0;
            }
            if (hBrush as usize) <= COLOR_MENUBAR as usize {
                hBrush = IntGetSysColorBrush(hBrush as usize as INT) as HBRUSH;
            }
            if ((*(*Wnd).pcls).style & CS_PARENTDC) != 0 {
                IntGetClientRect(Wnd, (&mut Rect as *mut tagRECT).cast());
                GreDPtoLP(wParam as HDC, (&mut Rect as *mut RECT).cast(), 2);
            } else {
                GdiGetClipBox(wParam as HDC, (&mut Rect as *mut tagRECT).cast());
            }
            FillRect(wParam as HDC, &Rect, hBrush);
            return 1;
        }

        WM_GETHOTKEY => {
            return IntDefWindowProc_WM_GETHOTKEY(Wnd);
        }
        WM_SETHOTKEY => {
            return IntDefWindowProc_WM_SETHOTKEY(Wnd, wParam);
        }

        WM_NCHITTEST => {
            let mut Point: POINT = core::mem::zeroed();
            Point.x = (lParam as i16) as LONG;
            Point.y = ((lParam >> 16) as i16) as LONG;
            return GetNCHitEx(Wnd, Point) as LRESULT;
        }

        WM_PRINT => {
            DefWndPrint(Wnd, wParam as HDC, lParam as ULONG);
            return 0;
        }

        WM_SYSCOLORCHANGE => {
            UserPaintCaption(Wnd, DC_NC as INT);
            co_UserRedrawWindow(
                Wnd,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                (RDW_ALLCHILDREN | RDW_INVALIDATE | RDW_ERASE) as ULONG,
            );
            return 0;
        }

        WM_PAINTICON | WM_PAINT => {
            let mut Ps: PAINTSTRUCT = core::mem::zeroed();
            let hDC: HDC;

            if ((*Wnd).state2 & WNDS2_INDESTROY) != 0 {}

            if ((*Wnd).state2 & WNDS2_STARTPAINT) != 0
                && RECTL_bIsEmptyRect(&(*Wnd).rcClient) == 0
            {
                return 0;
            }

            hDC = IntBeginPaint(Wnd, &mut Ps);
            if !hDC.is_null() {
                if ((*Wnd).style & WS_MINIMIZE as ULONG) != 0 && !(*(*Wnd).pcls).spicn.is_null() {
                    let mut ClientRect: RECT = core::mem::zeroed();
                    let x: INT;
                    let y: INT;

                    IntGetClientRect(Wnd, (&mut ClientRect as *mut tagRECT).cast());
                    x = (ClientRect.right - ClientRect.left - UserGetSystemMetrics(SM_CXICON)) / 2;
                    y = (ClientRect.bottom - ClientRect.top - UserGetSystemMetrics(SM_CYICON)) / 2;
                    UserReferenceObject((*(*Wnd).pcls).spicn as *mut core::ffi::c_void);
                    UserDrawIconEx(
                        hDC,
                        x,
                        y,
                        (*(*Wnd).pcls).spicn,
                        0,
                        0,
                        0,
                        core::ptr::null_mut(),
                        (DI_NORMAL | DI_COMPAT | DI_DEFAULTSIZE) as UINT,
                    );
                    UserDereferenceObject((*(*Wnd).pcls).spicn as *mut core::ffi::c_void);
                }

                IntEndPaint(Wnd, &mut Ps);
            }
            return 0;
        }

        WM_SYNCPAINT => {
            let hRgn: HRGN;
            (*Wnd).state &= !WNDS_SYNCPAINTPENDING;
            hRgn = NtGdiCreateRectRgn(0, 0, 0, 0);
            if !hRgn.is_null() {
                if co_UserGetUpdateRgn(Wnd, hRgn, 0) != NULLREGION as INT {
                    let pRgn = REGION_LockRgn(hRgn);
                    if !pRgn.is_null() {
                        REGION_UnlockRgn(pRgn);
                    }
                    if wParam == 0 {
                        wParam = (RDW_ERASENOW | RDW_ERASE | RDW_FRAME | RDW_ALLCHILDREN) as WPARAM;
                    }
                    co_UserRedrawWindow(Wnd, core::ptr::null_mut(), pRgn as HRGN, wParam as ULONG);
                }
                GreDeleteObject(hRgn as HANDLE);
            }
            return 0;
        }

        WM_SETREDRAW => {
            return IntDefWindowProc_WM_SETREDRAW(Wnd, wParam);
        }

        WM_WINDOWPOSCHANGING => {
            return DefWndHandleWindowPosChanging(Wnd, lParam as PWINDOWPOS);
        }

        WM_WINDOWPOSCHANGED => {
            return DefWndHandleWindowPosChanged(Wnd, lParam as PWINDOWPOS);
        }

        WM_NCCALCSIZE => {
            return NC_HandleNCCalcSize(Wnd, wParam, lParam as *mut RECTL, 0);
        }

        WM_NCACTIVATE => {
            return NC_HandleNCActivate(Wnd, wParam, lParam);
        }

        WM_NCPAINT => {
            let hDC = UserGetDCEx(
                Wnd,
                wParam as HANDLE,
                (DCX_WINDOW | DCX_INTERSECTRGN | DCX_USESTYLE | DCX_KEEPCLIPRGN) as ULONG,
            );
            (*Wnd).state |= WNDS_FORCEMENUDRAW;
            NC_DoNCPaint(Wnd, hDC, -1);
            (*Wnd).state &= !WNDS_FORCEMENUDRAW;
            UserReleaseDC(Wnd, hDC, 0);
            return 0;
        }

        WM_NCUAHDRAWCAPTION => {
            let hDC = UserGetDCEx(Wnd, core::ptr::null_mut(), (DCX_WINDOW | DCX_USESTYLE) as ULONG);
            UserDrawCaptionBar(Wnd, hDC, (wParam | DC_FRAME as WPARAM) as INT);
            UserReleaseDC(Wnd, hDC, 0);
            return 0;
        }

        WM_NCUAHDRAWFRAME => {
            NC_DoNCPaint(Wnd, wParam as HDC, (lParam | DC_NC as LPARAM) as INT);
            return 0;
        }

        WM_CBT => {
            match wParam as u32 {
                HCBT_MOVESIZE => {
                    let mut rt: RECTL = core::mem::zeroed();

                    if lParam != 0 {
                        Status = SafeProbeForRead(
                            lParam as *mut core::ffi::c_void,
                            core::mem::size_of::<RECT>() as _,
                            1,
                        );
                        // Validate-only probe; the capture is the access, same exit.
                        if !NT_SUCCESS(Status)
                            || !crate::copy::capture_from_user(
                                &mut rt as *mut RECTL as *mut core::ffi::c_void,
                                lParam as *const core::ffi::c_void,
                                core::mem::size_of::<RECT>(),
                            )
                        {
                            lResult = 1;
                        }
                    }
                    if lResult == 0 {
                        lResult = co_HOOK_CallHooks(
                            WH_CBT as INT,
                            HCBT_MOVESIZE as INT,
                            UserHMGetHandle(Wnd) as WPARAM,
                            if lParam != 0 {
                                &mut rt as *mut RECTL as LPARAM
                            } else {
                                0
                            },
                        ) as LRESULT;
                    }
                }
                _ => {}
            }
        }

        _ => {}
    }
    lResult
}
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
extern "system" {
    fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> INT;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn DefWndSetHotKey(Wnd: PWND, wParam: WPARAM) -> INT; // hotkey.rs, as the C: INT
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn DefWndSetHotKey(Wnd: PWND, wParam: WPARAM) -> INT; // hotkey.rs, as the C: INT
}