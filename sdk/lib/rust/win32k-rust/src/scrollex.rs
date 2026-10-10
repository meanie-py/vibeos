#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::PsGetCurrentThreadWin32Thread;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS};

// File-local, matching copy.rs / safe_user_io.rs in this crate.
const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005_u32 as NTSTATUS;
#[allow(unused_imports)]
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// DBG_DEFAULT_CHANNEL(UserPainting) — function-like macro, no translatable decl.

// Debug-channel macros — no kernel symbol, no-op.
#[allow(unused_macros)]

// --- FASTCALL (i386) / cdecl (else) win32k exports not covered by win32k-sys ---
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
    fn SetLastNtError(Status: NTSTATUS);
}

// SafeProbeForRead/Write: NTAPI (extern "system" = stdcall i386 / cdecl else).
extern "system" {
    fn SafeProbeForRead(Address: *const c_void, Length: usize, Alignment: u32) -> NTSTATUS;
    fn SafeProbeForWrite(Address: *mut c_void, Length: usize, Alignment: u32) -> NTSTATUS;
}

// --- FORCEINLINE / macro helpers (no exported kernel symbol) ---


// RECTL_vOffsetRect (rect.h FORCEINLINE).
#[inline(always)]
unsafe fn RECTL_vOffsetRect(prcl: *mut RECTL, cx: INT, cy: INT) {
    (*prcl).left += cx;
    (*prcl).right += cx;
    (*prcl).top += cy;
    (*prcl).bottom += cy;
}

// IntSysCreateRectpRgnIndirect(prc) (region.h macro) -> IntSysCreateRectpRgn(...).
#[inline(always)]
unsafe fn IntSysCreateRectpRgnIndirect(prc: *const RECTL) -> PREGION {
    IntSysCreateRectpRgn((*prc).left, (*prc).top, (*prc).right, (*prc).bottom)
}

// DC_LockDc / DC_UnlockDc (dc.h FORCEINLINE).
#[inline(always)]
unsafe fn DC_LockDc(hdc: HDC) -> PDC {
    GDIOBJ_LockObject(hdc as HGDIOBJ, GDIObjType_GDIObjType_DC_TYPE as UCHAR) as PDC
}
#[inline(always)]
unsafe fn DC_UnlockDc(pdc: PDC) {
    GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*pdc).BaseObject));
}

// UserHMGetHandle(obj) = obj->head.h (HANDLE at offset 0 of HEAD).
#[inline(always)]
unsafe fn UserHMGetHandle(obj: PWND) -> HWND {
    *(obj as *const HWND)
}

// UserIsDesktopWindow(pWnd) (desktop.h macro): pWnd && pWnd->fnid == FNID_DESKTOP.
#[inline(always)]
unsafe fn UserIsDesktopWindow(pWnd: PWND) -> bool {
    !pWnd.is_null() && (*pWnd).fnid == FNID_DESKTOP
}

// MAKELONG(a,b) (minwindef.h macro).
#[inline(always)]
fn MAKELONG(a: INT, b: INT) -> LPARAM {
    (((a as u16) as u32) | (((b as u16) as u32) << 16)) as LONG as LPARAM
}

// UserRefObjectCo / UserDerefObjectCo are `static __inline` in object.h —
// reimplemented here (PushEntryList / PopEntryList inlined).
#[inline(always)]
unsafe fn UserRefObjectCo<T>(obj: *mut T, entry: PUSER_REFERENCE_ENTRY) {
    let W32Thread = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*entry).obj = obj as PVOID;
    UserReferenceObject(obj as PVOID);
    let head = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    (*entry).Entry.Next = (*head).Next;
    (*head).Next = core::ptr::addr_of_mut!((*entry).Entry);
}
#[inline(always)]
unsafe fn UserDerefObjectCo<T>(obj: *mut T) {
    let W32Thread = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let head = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let first = (*head).Next;
    if !first.is_null() {
        (*head).Next = (*first).Next;
    }
    UserDereferenceObject(obj as PVOID);
}

// UserReferenceObject / UserDereferenceObject / co_UserHideCaret are in the
// win32k-sys glob (`use win32k_sys::win32k::*`) — not re-declared here.

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntFixCaret(Window: PWND, lprc: *mut RECTL, flags: UINT) -> HWND {
    co_IntFixCaret_impl(Window, lprc, flags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntFixCaret(Window: PWND, lprc: *mut RECTL, flags: UINT) -> HWND {
    co_IntFixCaret_impl(Window, lprc, flags)
}

unsafe fn co_IntFixCaret_impl(Window: PWND, lprc: *mut RECTL, flags: UINT) -> HWND {
    let CaretInfo: PTHRDCARETINFO;
    let pti: PTHREADINFO;
    let ThreadQueue: PUSER_MESSAGE_QUEUE;
    let hWndCaret: HWND;
    let WndCaret: PWND;

    ASSERT_REFS_CO!(Window);

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ThreadQueue = (*pti).MessageQueue;
    CaretInfo = core::ptr::addr_of_mut!((*ThreadQueue).CaretInfo);
    hWndCaret = (*CaretInfo).hWnd;

    WndCaret = ValidateHwndNoErr(hWndCaret);

    if WndCaret == Window
        || ((flags & SW_SCROLLCHILDREN) != 0 && IntIsChildWindow(Window, WndCaret) != 0)
    {
        let mut pt: POINT = core::mem::zeroed();
        let mut FromOffset: POINT = core::mem::zeroed();
        let mut ToOffset: POINT = core::mem::zeroed();
        let mut rcCaret: RECTL = core::mem::zeroed();

        pt.x = (*CaretInfo).Pos.x;
        pt.y = (*CaretInfo).Pos.y;
        IntGetClientOrigin(WndCaret, &mut FromOffset as *mut _ as PPOINTL);
        IntGetClientOrigin(Window, &mut ToOffset as *mut _ as PPOINTL);
        rcCaret.left = pt.x;
        rcCaret.top = pt.y;
        rcCaret.right = pt.x + (*CaretInfo).Size.cx;
        rcCaret.bottom = pt.y + (*CaretInfo).Size.cy;
        if RECTL_bIntersectRect(lprc, lprc, &mut rcCaret) != 0 {
            co_UserHideCaret(core::ptr::null_mut());
            (*lprc).left = pt.x;
            (*lprc).top = pt.y;
            return hWndCaret;
        }
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntGetUpdateRgn(Window: PWND, Rgn: PREGION, bErase: BOOL) -> INT {
    co_IntGetUpdateRgn_impl(Window, Rgn, bErase)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntGetUpdateRgn(Window: PWND, Rgn: PREGION, bErase: BOOL) -> INT {
    co_IntGetUpdateRgn_impl(Window, Rgn, bErase)
}

unsafe fn co_IntGetUpdateRgn_impl(Window: PWND, Rgn: PREGION, bErase: BOOL) -> INT {
    let RegionType: INT;
    let mut Rect: RECTL = core::mem::zeroed();
    let UpdateRgn: PREGION;

    if bErase != 0 {
        co_IntPaintWindows(Window, RDW_NOCHILDREN, FALSE as BOOL);
    }

    (*Window).state &= !WNDS_UPDATEDIRTY;

    if (*Window).hrgnUpdate.is_null() {
        REGION_SetRectRgn(Rgn, 0, 0, 0, 0);
        return NULLREGION as INT;
    }

    UpdateRgn = REGION_LockRgn((*Window).hrgnUpdate);
    if UpdateRgn.is_null() {
        return ERROR as INT;
    }

    Rect = (*Window).rcClient;
    IntIntersectWithParents(Window, &mut Rect);
    REGION_SetRectRgn(Rgn, Rect.left, Rect.top, Rect.right, Rect.bottom);
    RegionType = IntGdiCombineRgn(Rgn, Rgn, UpdateRgn, RGN_AND as INT);
    REGION_bOffsetRgn(Rgn, -(*Window).rcClient.left, -(*Window).rcClient.top);
    REGION_UnlockRgn(UpdateRgn);

    RegionType
}

unsafe fn UserScrollDC(
    hDC: HDC,
    dx: INT,
    dy: INT,
    prcScroll: *const RECTL,
    prcClip: *const RECTL,
    hrgnUpdate: HRGN,
    mut RgnUpdate: PREGION,
    prcUpdate: *mut RECTL,
) -> INT {
    let pDC: PDC;
    let mut rcScroll: RECTL = core::mem::zeroed();
    let mut rcClip: RECTL = core::mem::zeroed();
    let mut rcSrc: RECTL = core::mem::zeroed();
    let mut rcDst: RECTL = core::mem::zeroed();
    let Result: INT;

    if GdiGetClipBox(hDC, &mut rcClip) == ERROR as i32 {
        return ERROR as i32;
    }

    rcScroll = rcClip;
    if !prcClip.is_null() {
        RECTL_bIntersectRect(&mut rcClip, &mut rcClip, prcClip);
    }

    if !prcScroll.is_null() {
        rcScroll = *prcScroll;
        RECTL_bIntersectRect(&mut rcSrc, &mut rcClip, prcScroll);
    } else {
        rcSrc = rcClip;
    }

    rcDst = rcSrc;
    RECTL_vOffsetRect(&mut rcDst, dx, dy);
    RECTL_bIntersectRect(&mut rcDst, &mut rcDst, &mut rcClip);

    if NtGdiBitBlt(
        hDC,
        rcDst.left,
        rcDst.top,
        rcDst.right - rcDst.left,
        rcDst.bottom - rcDst.top,
        hDC,
        rcDst.left - dx,
        rcDst.top - dy,
        SRCCOPY as u32,
        CLR_INVALID,
        0,
    ) == 0
    {
        return ERROR as i32;
    }

    if !RgnUpdate.is_null() || !hrgnUpdate.is_null() || !prcUpdate.is_null() {
        let RgnOwn: PREGION;
        let RgnTmp: PREGION;

        pDC = DC_LockDc(hDC);
        if pDC.is_null() {
            return ERROR as i32;
        }

        if !hrgnUpdate.is_null() {
            RgnUpdate = REGION_LockRgn(hrgnUpdate);
            if RgnUpdate.is_null() {
                DC_UnlockDc(pDC);
                return ERROR as i32;
            }
        }

        rcDst = rcScroll;
        RECTL_vOffsetRect(&mut rcDst, dx, dy);
        RECTL_bIntersectRect(&mut rcDst, &mut rcDst, &mut rcClip);
        if !RgnUpdate.is_null() {
            RgnOwn = RgnUpdate;
            REGION_SetRectRgn(RgnOwn, rcDst.left, rcDst.top, rcDst.right, rcDst.bottom);
        } else {
            RgnOwn = IntSysCreateRectpRgnIndirect(&mut rcDst);
        }

        RgnTmp = IntSysCreateRectpRgnIndirect(&mut rcSrc);
        IntGdiCombineRgn(RgnOwn, RgnOwn, RgnTmp, RGN_OR as i32);

        IntGdiCombineRgn(RgnTmp, RgnTmp, (*pDC).prgnVis, RGN_AND as i32);
        REGION_bOffsetRgn(RgnTmp, dx, dy);
        Result = IntGdiCombineRgn(RgnOwn, RgnOwn, RgnTmp, RGN_DIFF as i32);

        DC_UnlockDc(pDC);

        REGION_Delete(RgnTmp);

        if !prcUpdate.is_null() {
            REGION_GetRgnBox(RgnOwn, prcUpdate);
        }

        if !hrgnUpdate.is_null() {
            REGION_UnlockRgn(RgnUpdate);
        } else if RgnUpdate.is_null() {
            REGION_Delete(RgnOwn);
        }
    } else {
        Result = NULLREGION as i32;
    }

    Result
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntScrollWindowEx(
    Window: PWND,
    dx: INT,
    dy: INT,
    prcScroll: *const RECT,
    prcClip: *const RECT,
    hrgnUpdate: HRGN,
    prcUpdate: LPRECT,
    flags: UINT,
) -> DWORD {
    IntScrollWindowEx_impl(Window, dx, dy, prcScroll, prcClip, hrgnUpdate, prcUpdate, flags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntScrollWindowEx(
    Window: PWND,
    dx: INT,
    dy: INT,
    prcScroll: *const RECT,
    prcClip: *const RECT,
    hrgnUpdate: HRGN,
    prcUpdate: LPRECT,
    flags: UINT,
) -> DWORD {
    IntScrollWindowEx_impl(Window, dx, dy, prcScroll, prcClip, hrgnUpdate, prcUpdate, flags)
}

unsafe fn IntScrollWindowEx_impl(
    Window: PWND,
    dx: INT,
    dy: INT,
    prcScroll: *const RECT,
    prcClip: *const RECT,
    hrgnUpdate: HRGN,
    prcUpdate: LPRECT,
    flags: UINT,
) -> DWORD {
    let mut Result: INT = 0;
    let mut rcScroll: RECTL = core::mem::zeroed();
    let mut rcClip: RECTL = core::mem::zeroed();
    let mut rcCaret: RECTL = core::mem::zeroed();
    let CaretWnd: PWND;
    let hDC: HDC;
    let mut RgnUpdate: PREGION = core::ptr::null_mut();
    let mut RgnTemp: PREGION;
    let mut RgnWinupd: PREGION = core::ptr::null_mut();
    let hwndCaret: HWND;
    let mut dcxflags: DWORD = 0;
    let rdw_flags: i32;
    let mut CaretRef: USER_REFERENCE_ENTRY = core::mem::zeroed();

    if Window.is_null() || IntIsWindowDrawable(Window) == 0 {
        return ERROR;
    }

    IntGetClientRect(Window, &mut rcClip);

    if !prcScroll.is_null() {
        RECTL_bIntersectRect(&mut rcScroll, &mut rcClip, prcScroll as *mut RECTL);
    } else {
        rcScroll = rcClip;
    }

    if !prcClip.is_null() {
        RECTL_bIntersectRect(&mut rcClip, &mut rcClip as *mut RECTL, prcClip as *mut RECTL);
    }

    if rcClip.right <= rcClip.left
        || rcClip.bottom <= rcClip.top
        || (dx == 0 && dy == 0)
    {
        return NULLREGION as DWORD;
    }

    RgnUpdate = IntSysCreateRectpRgn(0, 0, 0, 0);
    if RgnUpdate.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return ERROR;
    }

    'cleanup: {
        if !hrgnUpdate.is_null() {
            RgnTemp = REGION_LockRgn(hrgnUpdate);
            if RgnTemp.is_null() {
                EngSetLastError(ERROR_INVALID_HANDLE);
                Result = ERROR as INT;
                break 'cleanup;
            }
            IntGdiCombineRgn(RgnUpdate, RgnTemp, core::ptr::null_mut(), RGN_COPY as INT);
            REGION_UnlockRgn(RgnTemp);
        }

        if flags & SW_SCROLLWNDDCE != 0 {
            dcxflags = DCX_USESTYLE;

            if (*(*Window).pcls).style & (CS_OWNDC | CS_CLASSDC) == 0 {
                dcxflags |= DCX_CACHE;
            }

            if flags & SW_SCROLLCHILDREN != 0 && (*Window).style & WS_CLIPCHILDREN != 0 {
                dcxflags |= DCX_CACHE | DCX_NOCLIPCHILDREN;
            }
        } else {
            dcxflags = DCX_CACHE | DCX_USESTYLE;
            if flags & SW_SCROLLCHILDREN != 0 {
                dcxflags |= DCX_NOCLIPCHILDREN;
            }
        }

        hDC = UserGetDCEx(Window, core::ptr::null_mut(), dcxflags);
        if hDC.is_null() {
            Result = ERROR as INT;
            break 'cleanup;
        }

        rdw_flags = if (flags & SW_ERASE != 0) && (flags & SW_INVALIDATE != 0) {
            (RDW_INVALIDATE | RDW_ERASE) as i32
        } else {
            RDW_INVALIDATE as i32
        };

        rcCaret = rcScroll;
        hwndCaret = co_IntFixCaret(Window, &mut rcCaret, flags);

        Result = UserScrollDC(
            hDC,
            dx,
            dy,
            &rcScroll,
            &rcClip,
            core::ptr::null_mut(),
            RgnUpdate,
            prcUpdate as *mut RECTL,
        );

        UserReleaseDC(Window, hDC, 0u8 as BOOL);

        RgnTemp = IntSysCreateRectpRgn(0, 0, 0, 0);
        if RgnTemp.is_null() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            Result = ERROR as INT;
            break 'cleanup;
        }

        if co_IntGetUpdateRgn(Window, RgnTemp, 0u8 as BOOL) != NULLREGION as INT {
            let RgnClip: PREGION = IntSysCreateRectpRgnIndirect(&rcClip);
            if !RgnClip.is_null() {
                if !hrgnUpdate.is_null() {
                    RgnWinupd = IntSysCreateRectpRgn(0, 0, 0, 0);
                    IntGdiCombineRgn(RgnWinupd, RgnTemp, core::ptr::null_mut(), RGN_COPY as INT);
                }

                REGION_bOffsetRgn(RgnTemp, dx, dy);

                IntGdiCombineRgn(RgnTemp, RgnTemp, RgnClip, RGN_AND as INT);

                if !hrgnUpdate.is_null() {
                    IntGdiCombineRgn(RgnWinupd, RgnWinupd, RgnTemp, RGN_OR as INT);
                }

                co_UserRedrawWindow(Window, core::ptr::null_mut(), RgnTemp, rdw_flags as ULONG);

                REGION_Delete(RgnClip);
            }
        }
        REGION_Delete(RgnTemp);

        if flags & SW_SCROLLCHILDREN != 0 {
            let mut Child: PWND;
            let mut rcChild: RECTL;
            let mut ClientOrigin: POINT = core::mem::zeroed();
            let mut WndRef: USER_REFERENCE_ENTRY = core::mem::zeroed();
            let mut rcDummy: RECTL = core::mem::zeroed();
            let mut lParam: LPARAM;

            IntGetClientOrigin(Window, &mut ClientOrigin as *mut _ as PPOINTL);

            Child = (*Window).spwndChild;
            while !Child.is_null() {
                rcChild = (*Child).rcWindow;
                RECTL_vOffsetRect(&mut rcChild, -ClientOrigin.x, -ClientOrigin.y);

                RECTL_vOffsetRect(&mut (*Child).rcWindow, dx, dy);
                RECTL_vOffsetRect(&mut (*Child).rcClient, dx, dy);

                if prcScroll.is_null()
                    || RECTL_bIntersectRect(&mut rcDummy, &mut rcChild, &mut rcScroll) != 0
                {
                    UserRefObjectCo(Child, &mut WndRef);

                    if UserIsDesktopWindow((*Window).spwndParent) {
                        lParam = MAKELONG((*Child).rcClient.left, (*Child).rcClient.top);
                    } else {
                        lParam = MAKELONG(rcChild.left + dx, rcChild.top + dy);
                    }

                    co_IntSendMessage(UserHMGetHandle(Child), WM_MOVE, 0, lParam);

                    UserDerefObjectCo(Child as PVOID);
                }

                Child = (*Child).spwndNext;
            }
        }

        if flags & (SW_INVALIDATE | SW_ERASE) != 0 {
            co_UserRedrawWindow(
                Window,
                core::ptr::null_mut(),
                RgnUpdate,
                (rdw_flags
                    | (if flags & SW_SCROLLCHILDREN != 0 {
                        RDW_ALLCHILDREN as i32
                    } else {
                        RDW_NOCHILDREN as i32
                    })) as ULONG,
            );
        }

        if !hwndCaret.is_null() && {
            CaretWnd = UserGetWindowObject(hwndCaret);
            !CaretWnd.is_null()
        } {
            UserRefObjectCo(CaretWnd, &mut CaretRef);

            co_IntSetCaretPos(rcCaret.left + dx, rcCaret.top + dy);
            co_UserShowCaret(CaretWnd);

            UserDerefObjectCo(CaretWnd as PVOID);
        }

        if !hrgnUpdate.is_null() && Result != ERROR as INT {
            RgnTemp = REGION_LockRgn(hrgnUpdate);
            if !RgnWinupd.is_null() {
                IntGdiCombineRgn(RgnTemp, RgnUpdate, RgnWinupd, RGN_OR as INT);
            } else {
                IntGdiCombineRgn(RgnTemp, RgnUpdate, core::ptr::null_mut(), RGN_COPY as INT);
            }
            REGION_UnlockRgn(RgnTemp);
        }
    }

    if !RgnWinupd.is_null() {
        REGION_Delete(RgnWinupd);
    }

    if !RgnUpdate.is_null() {
        REGION_Delete(RgnUpdate);
    }

    Result as DWORD
}

// IntScrollWindow is FASTCALL-exported (@IntScrollWindow@20); called by menu.c.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntScrollWindow(
    pWnd: PWND,
    dx: INT,
    dy: INT,
    lpRect: *const RECT,
    prcClip: *const RECT,
) -> BOOL {
    (IntScrollWindowEx(
        pWnd,
        dx,
        dy,
        lpRect,
        prcClip,
        0 as HRGN,
        core::ptr::null_mut(),
        (if !lpRect.is_null() { 0 } else { SW_SCROLLCHILDREN })
            | (SW_ERASE | SW_INVALIDATE | SW_SCROLLWNDDCE),
    ) != ERROR as DWORD) as BOOL
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntScrollWindow(
    pWnd: PWND,
    dx: INT,
    dy: INT,
    lpRect: *const RECT,
    prcClip: *const RECT,
) -> BOOL {
    (IntScrollWindowEx(
        pWnd,
        dx,
        dy,
        lpRect,
        prcClip,
        0 as HRGN,
        core::ptr::null_mut(),
        (if !lpRect.is_null() { 0 } else { SW_SCROLLCHILDREN })
            | (SW_ERASE | SW_INVALIDATE | SW_SCROLLWNDDCE),
    ) != ERROR as DWORD) as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn NtUserScrollDC(
    hDC: HDC,
    dx: INT,
    dy: INT,
    prcUnsafeScroll: *const RECT,
    prcUnsafeClip: *const RECT,
    hrgnUpdate: HRGN,
    prcUnsafeUpdate: LPRECT,
) -> BOOL {
    let mut Ret: BOOL = FALSE as BOOL;
    let Result: DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut rcScroll: RECTL = core::mem::zeroed();
    let mut rcClip: RECTL = core::mem::zeroed();
    let mut rcUpdate: RECTL = core::mem::zeroed();

    UserEnterExclusive();

    Status = SafeProbeForRead(
        prcUnsafeScroll as *const core::ffi::c_void,
        core::mem::size_of::<RECT>(),
        1,
    );
    if NT_SUCCESS(Status) {
        Status = SafeProbeForRead(
            prcUnsafeClip as *const core::ffi::c_void,
            core::mem::size_of::<RECT>(),
            1,
        );
    }
    if NT_SUCCESS(Status) {
        Status = SafeProbeForWrite(
            prcUnsafeUpdate as *mut core::ffi::c_void,
            core::mem::size_of::<RECT>(),
            1,
        );
    }
    if NT_SUCCESS(Status) {
        // The probe above validated the RANGE, not the MAPPING -- it cannot
        // touch-test under DUMMY_PSEH. The C captures these inside the same
        // _SEH2_TRY as the probe; loading them raw here is what made
        // `user32_apitest ScrollDC` BUGCHECK 0x1E at NtUserScrollDC+0x180.
        if !prcUnsafeScroll.is_null()
            && !crate::copy::capture_from_user(
                core::ptr::addr_of_mut!(rcScroll) as *mut core::ffi::c_void,
                prcUnsafeScroll as *const core::ffi::c_void,
                core::mem::size_of::<RECTL>(),
            )
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
        if NT_SUCCESS(Status)
            && !prcUnsafeClip.is_null()
            && !crate::copy::capture_from_user(
                core::ptr::addr_of_mut!(rcClip) as *mut core::ffi::c_void,
                prcUnsafeClip as *const core::ffi::c_void,
                core::mem::size_of::<RECTL>(),
            )
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
    }

    'exit: {
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'exit;
        }

        Result = UserScrollDC(
            hDC,
            dx,
            dy,
            if !prcUnsafeScroll.is_null() {
                &rcScroll
            } else {
                core::ptr::null()
            },
            if !prcUnsafeClip.is_null() {
                &rcClip
            } else {
                core::ptr::null()
            },
            hrgnUpdate,
            core::ptr::null_mut(),
            if !prcUnsafeUpdate.is_null() {
                &mut rcUpdate
            } else {
                core::ptr::null_mut()
            },
        ) as DWORD;

        if Result == ERROR as DWORD {
            break 'exit;
        }

        if !prcUnsafeUpdate.is_null() {
            // Fault-tolerant write-back. This is NtUserScrollDC's own store;
            // the identical fix applied earlier landed in the SIBLING
            // (NtUserScrollWindowEx, below), which is why ScrollDC kept
            // bugchecking at the same PC after it was "fixed" twice.
            if !crate::copy::publish_to_user(
                prcUnsafeUpdate as *mut core::ffi::c_void,
                core::ptr::addr_of!(rcUpdate) as *const core::ffi::c_void,
                core::mem::size_of::<RECTL>(),
            ) {
                Status = STATUS_ACCESS_VIOLATION;
                SetLastNtError(Status);
            }

            if !NT_SUCCESS(Status) {
                break 'exit;
            }
        }

        Ret = TRUE as BOOL;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserScrollWindowEx(
    hWnd: HWND,
    dx: INT,
    dy: INT,
    prcUnsafeScroll: *const RECT,
    prcUnsafeClip: *const RECT,
    hrgnUpdate: HRGN,
    prcUnsafeUpdate: LPRECT,
    flags: UINT,
) -> DWORD {
    let mut Result: DWORD = ERROR as DWORD;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Window: PWND = core::ptr::null_mut();
    let mut rcScroll: RECTL = core::mem::zeroed();
    let mut rcClip: RECTL = core::mem::zeroed();
    let mut rcUpdate: RECTL = core::mem::zeroed();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterExclusive();

    'cleanup: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() || IntIsWindowDrawable(Window) == 0 {
            Window = core::ptr::null_mut();
            break 'cleanup;
        }
        UserRefObjectCo(Window, &mut Ref);

        Status = SafeProbeForRead(
            prcUnsafeScroll as *const core::ffi::c_void,
            core::mem::size_of::<RECT>(),
            1,
        );
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(
                prcUnsafeClip as *const core::ffi::c_void,
                core::mem::size_of::<RECT>(),
                1,
            );
        }
        // Same capture as NtUserScrollDC above. The sibling having the bug
        // after the first one is fixed is the asymmetry worth acting on: both
        // were translated from the same C shape, so a fix to one is a bug
        // report against the other.
        if NT_SUCCESS(Status)
            && !prcUnsafeScroll.is_null()
            && !crate::copy::capture_from_user(
                core::ptr::addr_of_mut!(rcScroll) as *mut core::ffi::c_void,
                prcUnsafeScroll as *const core::ffi::c_void,
                core::mem::size_of::<RECTL>(),
            )
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
        if NT_SUCCESS(Status)
            && !prcUnsafeClip.is_null()
            && !crate::copy::capture_from_user(
                core::ptr::addr_of_mut!(rcClip) as *mut core::ffi::c_void,
                prcUnsafeClip as *const core::ffi::c_void,
                core::mem::size_of::<RECTL>(),
            )
        {
            Status = STATUS_ACCESS_VIOLATION;
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'cleanup;
        }

        Result = IntScrollWindowEx(
            Window,
            dx,
            dy,
            if !prcUnsafeScroll.is_null() {
                &mut rcScroll as *mut RECTL as *const RECT
            } else {
                core::ptr::null()
            },
            if !prcUnsafeClip.is_null() {
                &mut rcClip as *mut RECTL as *const RECT
            } else {
                core::ptr::null()
            },
            hrgnUpdate,
            if !prcUnsafeUpdate.is_null() {
                &mut rcUpdate as *mut RECTL as LPRECT
            } else {
                core::ptr::null_mut()
            },
            flags,
        ) as DWORD;

        if !prcUnsafeUpdate.is_null() {
            Status = SafeProbeForWrite(
                prcUnsafeUpdate as *mut core::ffi::c_void,
                core::mem::size_of::<RECT>(),
                1,
            );
            if NT_SUCCESS(Status)
                && !crate::copy::publish_to_user(
                    prcUnsafeUpdate as *mut core::ffi::c_void,
                    core::ptr::addr_of!(rcUpdate) as *const core::ffi::c_void,
                    core::mem::size_of::<RECTL>(),
                )
            {
                Status = STATUS_ACCESS_VIOLATION;
            }

            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                Result = ERROR as DWORD;
            }
        }
    }

    if !Window.is_null() {
        UserDerefObjectCo(Window as PVOID);
    }

    UserLeave();
    Result
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  COPYRIGHT:        See COPYING in the top level directory
//  *  PROJECT:          ReactOS Win32k subsystem
//  *  PURPOSE:          Window scrolling function
//  *  FILE:             win32ss/user/ntuser/scrollex.c
//  *  PROGRAMER:        Filip Navara (xnavara@volny.cz)
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// DBG_DEFAULT_CHANNEL(UserPainting);
//
// static
// HWND FASTCALL
// co_IntFixCaret(PWND Window, RECTL *lprc, UINT flags)
// {
//    PTHRDCARETINFO CaretInfo;
//    PTHREADINFO pti;
//    PUSER_MESSAGE_QUEUE ThreadQueue;
//    HWND hWndCaret;
//    PWND WndCaret;
//
//    ASSERT_REFS_CO(Window);
//
//    pti = PsGetCurrentThreadWin32Thread();
//    ThreadQueue = pti->MessageQueue;
//    CaretInfo = &ThreadQueue->CaretInfo;
//    hWndCaret = CaretInfo->hWnd;
//
//    WndCaret = ValidateHwndNoErr(hWndCaret);
//
//    // FIXME: Check for WndCaret can be NULL
//    if (WndCaret == Window ||
//          ((flags & SW_SCROLLCHILDREN) && IntIsChildWindow(Window, WndCaret)))
//    {
//       POINT pt, FromOffset, ToOffset;
//       RECTL rcCaret;
//
//       pt.x = CaretInfo->Pos.x;
//       pt.y = CaretInfo->Pos.y;
//       IntGetClientOrigin(WndCaret, &FromOffset);
//       IntGetClientOrigin(Window, &ToOffset);
//       rcCaret.left = pt.x;
//       rcCaret.top = pt.y;
//       rcCaret.right = pt.x + CaretInfo->Size.cx;
//       rcCaret.bottom = pt.y + CaretInfo->Size.cy;
//       if (RECTL_bIntersectRect(lprc, lprc, &rcCaret))
//       {
//          co_UserHideCaret(0);
//          lprc->left = pt.x;
//          lprc->top = pt.y;
//          return hWndCaret;
//       }
//    }
//
//    return 0;
// }
//
// /*
//     Old GetUpdateRgn, for scrolls, see above note.
//  */
// INT FASTCALL
// co_IntGetUpdateRgn(PWND Window, PREGION Rgn, BOOL bErase)
// {
//     int RegionType;
//     RECTL Rect;
//     PREGION UpdateRgn;
//
//     ASSERT_REFS_CO(Window);
//
//     if (bErase)
//     {
//        co_IntPaintWindows(Window, RDW_NOCHILDREN, FALSE);
//     }
//
//     Window->state &= ~WNDS_UPDATEDIRTY;
//
//     if (Window->hrgnUpdate == NULL)
//     {
//         REGION_SetRectRgn(Rgn, 0, 0, 0, 0);
//         return NULLREGION;
//     }
//
//     UpdateRgn = REGION_LockRgn(Window->hrgnUpdate);
//     if (!UpdateRgn)
//        return ERROR;
//
//     Rect = Window->rcClient;
//     IntIntersectWithParents(Window, &Rect);
//     REGION_SetRectRgn(Rgn, Rect.left, Rect.top, Rect.right, Rect.bottom);
//     RegionType = IntGdiCombineRgn(Rgn, Rgn, UpdateRgn, RGN_AND);
//     REGION_bOffsetRgn(Rgn, -Window->rcClient.left, -Window->rcClient.top);
//     REGION_UnlockRgn(UpdateRgn);
//
//     return RegionType;
// }
//
// static
// INT FASTCALL
// UserScrollDC(
//    HDC hDC,
//    INT dx,
//    INT dy,
//    const RECTL *prcScroll,
//    const RECTL *prcClip,
//    HRGN hrgnUpdate,
//    PREGION RgnUpdate,
//    RECTL *prcUpdate)
// {
//    PDC pDC;
//    RECTL rcScroll, rcClip, rcSrc, rcDst;
//    INT Result;
//
//    if (GdiGetClipBox(hDC, &rcClip) == ERROR)
//    {
//        ERR("GdiGetClipBox failed for HDC %p\n", hDC);
//        return ERROR;
//    }
//
//    rcScroll = rcClip;
//    if (prcClip)
//    {
//       RECTL_bIntersectRect(&rcClip, &rcClip, prcClip);
//    }
//
//    if (prcScroll)
//    {
//       rcScroll = *prcScroll;
//       RECTL_bIntersectRect(&rcSrc, &rcClip, prcScroll);
//    }
//    else
//    {
//       rcSrc = rcClip;
//    }
//
//    rcDst = rcSrc;
//    RECTL_vOffsetRect(&rcDst, dx, dy);
//    RECTL_bIntersectRect(&rcDst, &rcDst, &rcClip);
//
//    if (!NtGdiBitBlt( hDC,
//                      rcDst.left,
//                      rcDst.top,
//                      rcDst.right - rcDst.left,
//                      rcDst.bottom - rcDst.top,
//                      hDC,
//                      rcDst.left - dx,
//                      rcDst.top - dy,
//                      SRCCOPY,
//                      CLR_INVALID,
//                      0))
//    {
//       return ERROR;
//    }
//
//    /* Calculate the region that was invalidated by moving or
//       could not be copied, because it was not visible */
//    if (RgnUpdate || hrgnUpdate || prcUpdate)
//    {
//       PREGION RgnOwn, RgnTmp;
//
//       pDC = DC_LockDc(hDC);
//       if (!pDC)
//       {
//          return ERROR;
//       }
//
//        if (hrgnUpdate)
//        {
//            NT_ASSERT(RgnUpdate == NULL);
//            RgnUpdate = REGION_LockRgn(hrgnUpdate);
//            if (!RgnUpdate)
//            {
//                DC_UnlockDc(pDC);
//                return ERROR;
//            }
//        }
//
//       /* Begin with the shifted and then clipped scroll rect */
//       rcDst = rcScroll;
//       RECTL_vOffsetRect(&rcDst, dx, dy);
//       RECTL_bIntersectRect(&rcDst, &rcDst, &rcClip);
//       if (RgnUpdate)
//       {
//          RgnOwn = RgnUpdate;
//          REGION_SetRectRgn(RgnOwn, rcDst.left, rcDst.top, rcDst.right, rcDst.bottom);
//       }
//       else
//       {
//          RgnOwn = IntSysCreateRectpRgnIndirect(&rcDst);
//       }
//
//       /* Add the source rect */
//       RgnTmp = IntSysCreateRectpRgnIndirect(&rcSrc);
//       IntGdiCombineRgn(RgnOwn, RgnOwn, RgnTmp, RGN_OR);
//
//       /* Substract the part of the dest that was visible in source */
//       IntGdiCombineRgn(RgnTmp, RgnTmp, pDC->prgnVis, RGN_AND);
//       REGION_bOffsetRgn(RgnTmp, dx, dy);
//       Result = IntGdiCombineRgn(RgnOwn, RgnOwn, RgnTmp, RGN_DIFF);
//
//       /* DO NOT Unlock DC while messing with prgnVis! */
//       DC_UnlockDc(pDC);
//
//       REGION_Delete(RgnTmp);
//
//       if (prcUpdate)
//       {
//          REGION_GetRgnBox(RgnOwn, prcUpdate);
//       }
//
//       if (hrgnUpdate)
//       {
//          REGION_UnlockRgn(RgnUpdate);
//       }
//       else if (!RgnUpdate)
//       {
//          REGION_Delete(RgnOwn);
//       }
//    }
//    else
//       Result = NULLREGION;
//
//    return Result;
// }
//
// DWORD
// FASTCALL
// IntScrollWindowEx(
//    PWND Window,
//    INT dx,
//    INT dy,
//    const RECT *prcScroll,
//    const RECT *prcClip,
//    HRGN hrgnUpdate,
//    LPRECT prcUpdate,
//    UINT flags)
// {
//    INT Result;
//    RECTL rcScroll, rcClip, rcCaret;
//    PWND CaretWnd;
//    HDC hDC;
//    PREGION RgnUpdate = NULL, RgnTemp, RgnWinupd = NULL;
//    HWND hwndCaret;
//    DWORD dcxflags = 0;
//    int rdw_flags;
//    USER_REFERENCE_ENTRY CaretRef;
//
//    if (!Window || !IntIsWindowDrawable(Window))
//    {
//       return ERROR;
//    }
//
//    IntGetClientRect(Window, &rcClip);
//
//    if (prcScroll)
//       RECTL_bIntersectRect(&rcScroll, &rcClip, prcScroll);
//    else
//       rcScroll = rcClip;
//
//    if (prcClip)
//       RECTL_bIntersectRect(&rcClip, &rcClip, prcClip);
//
//    if (rcClip.right <= rcClip.left || rcClip.bottom <= rcClip.top ||
//          (dx == 0 && dy == 0))
//    {
//       return NULLREGION;
//    }
//
//    /* We must use a copy of the region, as we can't hold an exclusive lock
//     * on it while doing callouts to user-mode */
//    RgnUpdate = IntSysCreateRectpRgn(0, 0, 0, 0);
//    if(!RgnUpdate)
//    {
//        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//        return ERROR;
//    }
//
//    if (hrgnUpdate)
//    {
//        RgnTemp = REGION_LockRgn(hrgnUpdate);
//        if (!RgnTemp)
//        {
//            EngSetLastError(ERROR_INVALID_HANDLE);
//            Result = ERROR;
//            goto Cleanup;
//        }
//        IntGdiCombineRgn(RgnUpdate, RgnTemp, NULL, RGN_COPY);
//        REGION_UnlockRgn(RgnTemp);
//    }
//
//    /* ScrollWindow uses the window DC, ScrollWindowEx doesn't */
//    if (flags & SW_SCROLLWNDDCE)
//    {
//       dcxflags = DCX_USESTYLE;
//
//       if (!(Window->pcls->style & (CS_OWNDC|CS_CLASSDC)))
//          dcxflags |= DCX_CACHE; // AH??? wine~ If not Powned or with Class go Cheap!
//
//       if (flags & SW_SCROLLCHILDREN && Window->style & WS_CLIPCHILDREN)
//          dcxflags |= DCX_CACHE|DCX_NOCLIPCHILDREN;
//    }
//    else
//    {
//        /* So in this case ScrollWindowEx uses Cache DC. */
//        dcxflags = DCX_CACHE|DCX_USESTYLE;
//        if (flags & SW_SCROLLCHILDREN) dcxflags |= DCX_NOCLIPCHILDREN;
//    }
//
//    hDC = UserGetDCEx(Window, 0, dcxflags);
//    if (!hDC)
//    {
//       /* FIXME: SetLastError? */
//       Result = ERROR;
//       goto Cleanup;
//    }
//
//    rdw_flags = (flags & SW_ERASE) && (flags & SW_INVALIDATE) ? RDW_INVALIDATE | RDW_ERASE : RDW_INVALIDATE ;
//
//    rcCaret = rcScroll;
//    hwndCaret = co_IntFixCaret(Window, &rcCaret, flags);
//
//    Result = UserScrollDC( hDC,
//                           dx,
//                           dy,
//                           &rcScroll,
//                           &rcClip,
//                           NULL,
//                           RgnUpdate,
//                           prcUpdate);
//
//    UserReleaseDC(Window, hDC, FALSE);
//
//    /*
//     * Take into account the fact that some damage may have occurred during
//     * the scroll. Keep a copy in hrgnWinupd to be added to hrngUpdate at the end.
//     */
//
//    RgnTemp = IntSysCreateRectpRgn(0, 0, 0, 0);
//    if (!RgnTemp)
//    {
//        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//        Result = ERROR;
//        goto Cleanup;
//    }
//
//    if (co_IntGetUpdateRgn(Window, RgnTemp, FALSE) != NULLREGION)
//    {
//       PREGION RgnClip = IntSysCreateRectpRgnIndirect(&rcClip);
//       if (RgnClip)
//       {
//           if (hrgnUpdate)
//           {
//              RgnWinupd = IntSysCreateRectpRgn(0, 0, 0, 0);
//              // FIXME: What to do if RgnWinupd == NULL??
//              IntGdiCombineRgn( RgnWinupd, RgnTemp, 0, RGN_COPY);
//           }
//
//           REGION_bOffsetRgn(RgnTemp, dx, dy);
//
//           IntGdiCombineRgn(RgnTemp, RgnTemp, RgnClip, RGN_AND);
//
//           if (hrgnUpdate)
//               IntGdiCombineRgn( RgnWinupd, RgnWinupd, RgnTemp, RGN_OR );
//
//           co_UserRedrawWindow(Window, NULL, RgnTemp, rdw_flags );
//
//           REGION_Delete(RgnClip);
//       }
//    }
//    REGION_Delete(RgnTemp);
//
//    if (flags & SW_SCROLLCHILDREN)
//    {
//       PWND Child;
//       RECTL rcChild;
//       POINT ClientOrigin;
//       USER_REFERENCE_ENTRY WndRef;
//       RECTL rcDummy;
//       LPARAM lParam;
//
//       IntGetClientOrigin(Window, &ClientOrigin);
//
//       for (Child = Window->spwndChild; Child; Child = Child->spwndNext)
//       {
//          rcChild = Child->rcWindow;
//          RECTL_vOffsetRect(&rcChild, -ClientOrigin.x, -ClientOrigin.y);
//
//          /* Adjust window positions */
//          RECTL_vOffsetRect(&Child->rcWindow, dx, dy);
//          RECTL_vOffsetRect(&Child->rcClient, dx, dy);
//
//          if (!prcScroll || RECTL_bIntersectRect(&rcDummy, &rcChild, &rcScroll))
//          {
//             UserRefObjectCo(Child, &WndRef);
//
//             if (UserIsDesktopWindow(Window->spwndParent))
//                lParam = MAKELONG(Child->rcClient.left, Child->rcClient.top);
//             else
//                lParam = MAKELONG(rcChild.left + dx, rcChild.top + dy);
//
//             /* wine sends WM_POSCHANGING, WM_POSCHANGED messages */
//             /* windows sometimes a WM_MOVE */
//             co_IntSendMessage(UserHMGetHandle(Child), WM_MOVE, 0, lParam);
//
//             UserDerefObjectCo(Child);
//          }
//       }
//    }
//
//    if (flags & (SW_INVALIDATE | SW_ERASE))
//    {
//       co_UserRedrawWindow( Window,
//                            NULL,
//                            RgnUpdate,
//                            rdw_flags |                                    /*    HACK    */
//                           ((flags & SW_SCROLLCHILDREN) ? RDW_ALLCHILDREN : RDW_NOCHILDREN) );
//    }
//
//    if (hwndCaret && (CaretWnd = UserGetWindowObject(hwndCaret)))
//    {
//       UserRefObjectCo(CaretWnd, &CaretRef);
//
//       co_IntSetCaretPos(rcCaret.left + dx, rcCaret.top + dy);
//       co_UserShowCaret(CaretWnd);
//
//       UserDerefObjectCo(CaretWnd);
//    }
//
//    if (hrgnUpdate && (Result != ERROR))
//    {
//        /* Give everything back to the caller */
//        RgnTemp = REGION_LockRgn(hrgnUpdate);
//        /* The handle should still be valid */
//        ASSERT(RgnTemp);
//        if (RgnWinupd)
//            IntGdiCombineRgn(RgnTemp, RgnUpdate, RgnWinupd, RGN_OR);
//        else
//            IntGdiCombineRgn(RgnTemp, RgnUpdate, NULL, RGN_COPY);
//        REGION_UnlockRgn(RgnTemp);
//    }
//
// Cleanup:
//    if (RgnWinupd)
//    {
//        REGION_Delete(RgnWinupd);
//    }
//
//    if (RgnUpdate)
//    {
//       REGION_Delete(RgnUpdate);
//    }
//
//    return Result;
// }
//
//
// BOOL FASTCALL
// IntScrollWindow(PWND pWnd,
//                 int dx,
//                 int dy,
//                 CONST RECT *lpRect,
//                 CONST RECT *prcClip)
// {
//    return IntScrollWindowEx( pWnd, dx, dy, lpRect, prcClip, 0, NULL,
//                             (lpRect ? 0 : SW_SCROLLCHILDREN) | (SW_ERASE|SW_INVALIDATE|SW_SCROLLWNDDCE)) != ERROR;
// }
//
// /*
//  * NtUserScrollDC
//  *
//  * Status
//  *    @implemented
//  */
// BOOL __stdcall
// NtUserScrollDC(
//    HDC hDC,
//    INT dx,
//    INT dy,
//    const RECT *prcUnsafeScroll,
//    const RECT *prcUnsafeClip,
//    HRGN hrgnUpdate,
//    LPRECT prcUnsafeUpdate)
// {
//    BOOL Ret = FALSE;
//    DWORD Result;
//    NTSTATUS Status = STATUS_SUCCESS;
//    RECTL rcScroll, rcClip, rcUpdate;
//
//    TRACE("Enter NtUserScrollDC\n");
//    UserEnterExclusive();
//
//    Status = SafeProbeForRead(prcUnsafeScroll, sizeof(*prcUnsafeScroll), 1);
//    if (NT_SUCCESS(Status))
//        Status = SafeProbeForRead(prcUnsafeClip, sizeof(*prcUnsafeClip), 1);
//    if (NT_SUCCESS(Status))
//        Status = SafeProbeForWrite(prcUnsafeUpdate, sizeof(*prcUnsafeUpdate), 1);
//    if (NT_SUCCESS(Status))
//    {
//        if (prcUnsafeScroll)
//        {
//
//           rcScroll = *prcUnsafeScroll;
//        }
//        if (prcUnsafeClip)
//        {
//
//           rcClip = *prcUnsafeClip;
//        }
//        if (prcUnsafeUpdate)
//        {
//
//        }
//    }
//
//    if (!NT_SUCCESS(Status))
//    {
//       SetLastNtError(Status);
//       goto Exit; // Return FALSE
//    }
//
//    Result = UserScrollDC( hDC,
//                           dx,
//                           dy,
//                           prcUnsafeScroll ? &rcScroll : NULL,
//                           prcUnsafeClip   ? &rcClip   : NULL,
//                           hrgnUpdate,
//                           NULL,
//                           prcUnsafeUpdate ? &rcUpdate : NULL);
//    if(Result == ERROR)
//    {
//       /* FIXME: Only if hRgnUpdate is invalid we should SetLastError(ERROR_INVALID_HANDLE) */
//       goto Exit; // Return FALSE
//    }
//
//    if (prcUnsafeUpdate)
//    {
//       _SEH2_TRY
//       {
//          *prcUnsafeUpdate = rcUpdate;
//       }
//       _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//       {
//          Status = _SEH2_GetExceptionCode();
//       }
//       _SEH2_END;
//
//       if (!NT_SUCCESS(Status))
//       {
//          /* FIXME: SetLastError? */
//          /* FIXME: correct? We have already scrolled! */
//          goto Exit; // Return FALSE
//       }
//    }
//
//    Ret = TRUE;
//
// Exit:
//    TRACE("Leave NtUserScrollDC, ret=%i\n", Ret);
//    UserLeave();
//    return Ret;
// }
//
// /*
//  * NtUserScrollWindowEx
//  *
//  * Status
//  *    @implemented
//  */
//
// DWORD __stdcall
// NtUserScrollWindowEx(
//    HWND hWnd,
//    INT dx,
//    INT dy,
//    const RECT *prcUnsafeScroll,
//    const RECT *prcUnsafeClip,
//    HRGN hrgnUpdate,
//    LPRECT prcUnsafeUpdate,
//    UINT flags)
// {
//    DWORD Result = ERROR;
//    NTSTATUS Status = STATUS_SUCCESS;
//    PWND Window = NULL;
//    RECTL rcScroll, rcClip, rcUpdate;
//    USER_REFERENCE_ENTRY Ref;
//
//    TRACE("Enter NtUserScrollWindowEx\n");
//    UserEnterExclusive();
//
//    Window = UserGetWindowObject(hWnd);
//    if (!Window || !IntIsWindowDrawable(Window))
//    {
//       Window = NULL; /* prevent deref at cleanup */
//       goto Cleanup; // Return ERROR
//    }
//    UserRefObjectCo(Window, &Ref);
//
//    Status = SafeProbeForRead(prcUnsafeScroll, sizeof(*prcUnsafeScroll), 1);
//    if (NT_SUCCESS(Status))
//        Status = SafeProbeForRead(prcUnsafeClip, sizeof(*prcUnsafeClip), 1);
//    if (NT_SUCCESS(Status))
//    {
//        if (prcUnsafeScroll)
//        {
//
//           rcScroll = *prcUnsafeScroll;
//        }
//
//        if (prcUnsafeClip)
//        {
//
//           rcClip = *prcUnsafeClip;
//        }
//    }
//
//    if (!NT_SUCCESS(Status))
//    {
//       SetLastNtError(Status);
//       goto Cleanup; // Return ERROR
//    }
//
//    Result = IntScrollWindowEx(Window,
//                               dx, dy,
//                               prcUnsafeScroll ? &rcScroll : NULL,
//                               prcUnsafeClip   ? &rcClip   : NULL,
//                               hrgnUpdate,
//                               prcUnsafeUpdate ? &rcUpdate : NULL,
//                               flags);
//
//    if (prcUnsafeUpdate)
//    {
//       Status = SafeProbeForWrite(prcUnsafeUpdate, sizeof(*prcUnsafeUpdate), 1);
//       if (NT_SUCCESS(Status))
//       {
//           /* Probe here, to not fail on invalid pointer before scrolling */
//
//           *prcUnsafeUpdate = rcUpdate;
//       }
//
//       if (!NT_SUCCESS(Status))
//       {
//          SetLastNtError(Status);
//          Result = ERROR;
//       }
//    }
//
// Cleanup:
//    if (Window)
//       UserDerefObjectCo(Window);
//
//    TRACE("Leave NtUserScrollWindowEx, ret=%lu\n", Result);
//    UserLeave();
//    return Result;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
