use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{PsGetCurrentThreadWin32Thread, RtlNtStatusToDosError};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS};
use ntkrnl_sys::ddk::{ExFreePoolWithTag};
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};
use ntkrnl_sys::base::IsListEmpty;

// Not in ntkrnl_sys::base; declared per-file, as the rest of the crate does.
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// ---- file-scope preamble (translated) ----
pub const FW_COMPLETED: u32 = 0;

// HRGN_WINDOW / PRGN_WINDOW sentinel handles (paint.h: (HRGN)1 / (PREGION)1).
const HRGN_WINDOW: HRGN = 1 as HRGN;
const PRGN_WINDOW: PREGION = 1 as PREGION;

// DC_NC (winuser.h: DC_CAPTION|DC_FRAME = 0x900C).
const DC_NC: u32 = DC_CAPTION | DC_FRAME;

// USERTAG pool tag ('lwsU').
const USERTAG_WINDOWLIST: u32 = u32::from_le_bytes(*b"Uswl");
const HWND_BROADCAST: HWND = 0xffff as HWND;

// UserHMGetHandle(obj) = obj->head.h (handle at object offset 0).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HWND {
    *(obj as *const HWND)
}

// ASSERT_REFS_CO — debug assertion, no-op in non-DBG build.
macro_rules! ASSERT_REFS_CO { ($($t:tt)*) => {{}}; }

// minwindef.h word macros.
#[inline(always)]
fn LOWORD(l: DWORD) -> DWORD { l & 0xffff }
#[inline(always)]
fn HIWORD(l: DWORD) -> DWORD { (l >> 16) & 0xffff }
#[inline(always)]
fn MAKELONG<A: Into<u32>, B: Into<u32>>(low: A, high: B) -> LONG {
    ((low.into() & 0xffff) | ((high.into() & 0xffff) << 16)) as LONG
}
#[inline(always)]
fn HandleToUlong<T>(h: *mut T) -> ULONG { h as usize as ULONG }

// ---- bindgen-flattened field accessors ----
// THRDESKHEAD.pti is at object offset 2 pointer-widths (bindgen-flattened head).
#[inline(always)]
unsafe fn head_pti<T>(obj: *const T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}
// gpsi->dtCaretBlink: in the anonymous PERUSERSERVERINFO member that bindgen
// flattened into tagSERVERINFO's __bindgen_padding_0 tail.
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

extern "C" {
    fn GetW32ThreadInfo() -> PTHREADINFO;
    // _MmCopyToCaller — mmcopy.h macro forwards here (copy.rs, cdecl).
    fn _MmCopyToCaller(Destination: *mut core::ffi::c_void, Source: *const core::ffi::c_void, Length: UINT) -> NTSTATUS;
}
extern "system" {
    // SafeProbeForRead / ProbeForRead — NTAPI (stdcall `_..@12`).
    fn SafeProbeForRead(Address: *mut core::ffi::c_void, Length: usize, Alignment: u32) -> i32;
}
// ValidateHwndNoErr — real FASTCALL export (window.rs). i386 fastcall, cdecl else.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
}

// ── header-inline / macro helpers (no exported symbol; reimplemented locally) ──
// MmCopyToCaller (mmcopy.h macro).
#[inline(always)]
unsafe fn MmCopyToCaller(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize) -> NTSTATUS {
    _MmCopyToCaller(dst, src, len as UINT)
}
// region.h macros.
#[inline(always)]
unsafe fn IntSysCreateRectpRgnIndirect(prc: *const RECTL) -> PREGION {
    IntSysCreateRectpRgn((*prc).left, (*prc).top, (*prc).right, (*prc).bottom)
}
#[inline(always)]
unsafe fn GreCreateRectRgnIndirect(prc: *const RECTL) -> HRGN {
    NtGdiCreateRectRgn((*prc).left, (*prc).top, (*prc).right, (*prc).bottom)
}
#[inline(always)]
unsafe fn GreSetRectRgnIndirect(hRgn: HRGN, prc: *const RECTL) -> BOOL {
    NtGdiSetRectRgn(hRgn, (*prc).left, (*prc).top, (*prc).right, (*prc).bottom)
}
// RECTL_bIsEmptyRect (rect.h inline).
#[inline(always)]
unsafe fn RECTL_bIsEmptyRect(prcl: *const RECTL) -> u32 {
    ((*prcl).left >= (*prcl).right || (*prcl).top >= (*prcl).bottom) as u32
}
// UserIsDesktopWindow / UserIsMessageWindow (desktop.h inlines).
#[inline(always)]
unsafe fn UserIsDesktopWindow(Wnd: PWND) -> BOOL {
    (!Wnd.is_null() && (*Wnd).fnid == FNID_DESKTOP) as BOOL
}
#[inline(always)]
unsafe fn UserIsMessageWindow(Wnd: PWND) -> BOOL {
    (!Wnd.is_null() && (*Wnd).fnid == FNID_MESSAGEWND) as BOOL
}

// UserRefObjectCo / UserDerefObjectCo (object.h static __inline; mirrors focus.rs).
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
#[inline(always)]
unsafe fn UserRefObjectCo(obj: PWND, pRefEntry: *mut USER_REFERENCE_ENTRY) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*pRefEntry).obj = obj as PVOID;
    UserReferenceObject(obj as PVOID);
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let entry: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*pRefEntry).Entry);
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

// FASTCALL (`@Name@N`) on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn MsqDecPaintCountQueue(pti: PTHREADINFO);
    fn MsqIncPaintCountQueue(pti: PTHREADINFO);
    fn MsqIsHung(pti: PTHREADINFO, TimeOut: DWORD) -> BOOL;
    fn IsThreadSuspended(pti: PTHREADINFO) -> BOOL;
    fn NC_IconForWindow(pWnd: PWND) -> PCURICON_OBJECT;
    fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn MsqDecPaintCountQueue(pti: PTHREADINFO);
    fn MsqIncPaintCountQueue(pti: PTHREADINFO);
    fn MsqIsHung(pti: PTHREADINFO, TimeOut: DWORD) -> BOOL;
    fn IsThreadSuspended(pti: PTHREADINFO) -> BOOL;
    fn NC_IconForWindow(pWnd: PWND) -> PCURICON_OBJECT;
    fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
}
// NC_DoNCPaint — cdecl (nonclient.h `LRESULT NC_DoNCPaint(PWND,HDC,INT)`).
extern "C" {
    fn NC_DoNCPaint(pWnd: PWND, hDC: HDC, Flags: INT) -> u32;
}
// IntWndBelongsToThread(WndObj, W32Thread) = WndObj->head.pti && == W32Thread (window.h macro).
#[inline(always)]
unsafe fn IntWndBelongsToThread(Wnd: PWND, pti: PTHREADINFO) -> BOOL {
    let wpti = head_pti(Wnd);
    (!wpti.is_null() && wpti == pti) as BOOL
}
// IntEqualRect (winpos.h FORCEINLINE).
#[inline(always)]
unsafe fn IntEqualRect(lprc1: *const RECTL, lprc2: *const RECTL) -> BOOL {
    if lprc1.is_null() || lprc2.is_null() {
        return FALSE as BOOL;
    }
    ((*lprc1).left == (*lprc2).left && (*lprc1).top == (*lprc2).top
        && (*lprc1).right == (*lprc2).right && (*lprc1).bottom == (*lprc2).bottom) as BOOL
}

// STDCALL (APIENTRY) win32k/GDI helpers.
extern "system" {
    fn SystemTimerProc(hwnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD);
    fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    fn DrawTextW(hDC: HDC, lpchText: LPCWSTR, cchText: INT, lprc: LPRECT, format: UINT) -> INT;
}
// UserExtTextOutW (rtl_text.rs) — extern "system" (`_UserExtTextOutW@28` on
// i386). Decl must match the def or LLVM UB-folds the caller (§9.30).
extern "system" {
    fn UserExtTextOutW(hdc: HDC, x: INT, y: INT, flOpts: UINT, prcl: *const RECTL, lpString: LPCWSTR, Count: INT) -> BOOL;
}

// RECTL_vOffsetRect (rect inline).
#[inline(always)]
unsafe fn RECTL_vOffsetRect(prcl: LPRECT, cx: i32, cy: i32) {
    (*prcl).left += cx;
    (*prcl).right += cx;
    (*prcl).top += cy;
    (*prcl).bottom += cy;
}


#[cfg(target_arch = "x86")]
#[allow(non_snake_case)]
#[no_mangle]
unsafe extern "fastcall" fn IntIntersectWithParents(Child: PWND, WindowRect: *mut RECTL) -> u8 {
    IntIntersectWithParents_impl(Child, WindowRect)
}

#[cfg(not(target_arch = "x86"))]
#[allow(non_snake_case)]
#[no_mangle]
unsafe extern "C" fn IntIntersectWithParents(Child: PWND, WindowRect: *mut RECTL) -> u8 {
    IntIntersectWithParents_impl(Child, WindowRect)
}

#[allow(non_snake_case)]
unsafe fn IntIntersectWithParents_impl(Child: PWND, WindowRect: *mut RECTL) -> u8 {
    let mut ParentWnd: PWND;

    if (*Child).ExStyle & WS_EX_REDIRECTED != 0 {
        return 1u8;
    }

    ParentWnd = (*Child).spwndParent;
    while !ParentWnd.is_null() {
        if (*ParentWnd).style & WS_VISIBLE == 0
            || (*ParentWnd).style & WS_MINIMIZE != 0
            || RECTL_bIntersectRect(WindowRect, WindowRect, core::ptr::addr_of_mut!((*ParentWnd).rcClient)) == 0
        {
            return 0u8;
        }

        if (*ParentWnd).ExStyle & WS_EX_REDIRECTED != 0 {
            return 1u8;
        }

        ParentWnd = (*ParentWnd).spwndParent;
    }

    1u8
}

fastcall_fn! {
#[allow(non_snake_case)]
unsafe fn IntValidateParents(Child: PWND, mut Recurse: i32) -> i32 {
    let mut ParentRect: RECTL = core::mem::zeroed();
    let mut Rect: RECTL = core::mem::zeroed();
    let mut Start: i32;
    let mut Ret: i32 = 1;
    let mut ParentWnd: PWND = Child;
    let mut Rgn: PREGION = core::ptr::null_mut();

    if (*ParentWnd).style & WS_CHILD != 0 {
        loop {
            ParentWnd = (*ParentWnd).spwndParent;
            if (*ParentWnd).style & WS_CHILD == 0 {
                break;
            }
        }
    }

    if (*ParentWnd).state & WNDS_SYNCPAINTPENDING == 0 {
        Recurse = 0;
    }

    Start = 1;
    ParentWnd = (*Child).spwndParent;
    while !ParentWnd.is_null() {
        if (*ParentWnd).style & WS_CLIPCHILDREN != 0 {
            break;
        }

        if (*ParentWnd).hrgnUpdate != 0 as _ {
            if Recurse != 0 {
                Ret = 0;
                break;
            }

            if Start != 0 {
                Start = 0;

                Rect = (*Child).rcWindow;

                if IntIntersectWithParents(Child, &mut Rect) == 0 {
                    break;
                }

                Rgn = IntSysCreateRectpRgnIndirect(&mut Rect);

                if (*Child).hrgnClip != 0 as _ {
                    let RgnClip: PREGION = REGION_LockRgn((*Child).hrgnClip);
                    IntGdiCombineRgn(Rgn, Rgn, RgnClip, RGN_AND as i32);
                    REGION_UnlockRgn(RgnClip);
                }
            }

            ParentRect = (*ParentWnd).rcWindow;

            if IntIntersectWithParents(ParentWnd, &mut ParentRect) == 0 {
                break;
            }

            IntInvalidateWindows(
                ParentWnd,
                Rgn,
                (RDW_VALIDATE | RDW_NOCHILDREN | RDW_NOUPDATEDIRTY) as ULONG,
            );
        }
        ParentWnd = (*ParentWnd).spwndParent;
    }

    if !Rgn.is_null() {
        REGION_Delete(Rgn);
    }

    Ret
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSendSyncPaint(Wnd: PWND, Flags: ULONG) {
    IntSendSyncPaint_impl(Wnd, Flags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSendSyncPaint(Wnd: PWND, Flags: ULONG) {
    IntSendSyncPaint_impl(Wnd, Flags)
}

unsafe fn IntSendSyncPaint_impl(Wnd: PWND, Flags: ULONG) {
    let ptiCur: PTHREADINFO;
    let ptiWnd: PTHREADINFO;
    let mut Message: PUSER_SENT_MESSAGE;
    let mut Entry: PLIST_ENTRY;
    let mut bSend: u8 = 1u8;

    ptiWnd = head_pti(Wnd);
    ptiCur = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if head_pti(Wnd) != ptiCur
        && (*Wnd).state & WNDS_SENDNCPAINT != 0
        && (*Wnd).state & WNDS_SENDERASEBACKGROUND != 0
        && (*Wnd).style & WS_VISIBLE != 0
    {
        TRACE!(b"SendSyncPaint Wnd in State!\n\0".as_ptr() as *const i8);
        if !IsListEmpty(core::ptr::addr_of_mut!((*ptiWnd).SentMessagesListHead) as *const _) {
            Entry = (*ptiWnd).SentMessagesListHead.Flink;
            Message = (Entry as *mut u8)
                .offset(-(core::mem::offset_of!(_USER_SENT_MESSAGE, ListEntry) as isize))
                as PUSER_SENT_MESSAGE;
            loop {
                TRACE!(b"LOOP it\n\0".as_ptr() as *const i8);
                if (*Message).Msg.message == WM_SYNCPAINT
                    && (*Message).Msg.hwnd == UserHMGetHandle(Wnd)
                {
                    ERR!("SendSyncPaint Found one in the Sent Msg Queue!\n");
                    bSend = 0u8;
                    break;
                }
                Entry = (*Message).ListEntry.Flink;
                Message = (Entry as *mut u8)
                    .offset(-(core::mem::offset_of!(_USER_SENT_MESSAGE, ListEntry) as isize))
                    as PUSER_SENT_MESSAGE;
                if Entry == core::ptr::addr_of_mut!((*ptiWnd).SentMessagesListHead) {
                    break;
                }
            }
        }
        if bSend != 0 {
            TRACE!(b"Sending WM_SYNCPAINT\n\0".as_ptr() as *const i8);
            co_IntSendMessageNoWait(
                UserHMGetHandle(Wnd),
                WM_SYNCPAINT,
                Flags as WPARAM,
                0,
            );
            (*Wnd).state |= WNDS_SYNCPAINTPENDING;
        }
    }

    if UserIsDesktopWindow(Wnd) != 0 {
        if Flags & RDW_ALLCHILDREN != 0
            || (Flags & RDW_NOCHILDREN == 0 && (*Wnd).style & WS_CLIPCHILDREN != 0)
        {
            let mut spwndChild: PWND = (*Wnd).spwndChild;
            while !spwndChild.is_null() {
                if (*spwndChild).style & WS_CHILD != 0 && head_pti(spwndChild) != ptiCur {
                    spwndChild = (*spwndChild).spwndNext;
                    continue;
                }
                IntSendSyncPaint(spwndChild, Flags);
                spwndChild = (*spwndChild).spwndNext;
            }
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntCalcWindowRgn(Wnd: PWND, Client: i32) -> HRGN {
    IntCalcWindowRgn_impl(Wnd, Client)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntCalcWindowRgn(Wnd: PWND, Client: i32) -> HRGN {
    IntCalcWindowRgn_impl(Wnd, Client)
}

unsafe fn IntCalcWindowRgn_impl(Wnd: PWND, Client: i32) -> HRGN {
    let hRgnWindow: HRGN;

    if Client != 0 {
        hRgnWindow = NtGdiCreateRectRgn(
            (*Wnd).rcClient.left,
            (*Wnd).rcClient.top,
            (*Wnd).rcClient.right,
            (*Wnd).rcClient.bottom,
        );
    } else {
        hRgnWindow = NtGdiCreateRectRgn(
            (*Wnd).rcWindow.left,
            (*Wnd).rcWindow.top,
            (*Wnd).rcWindow.right,
            (*Wnd).rcWindow.bottom,
        );
    }

    if !(*Wnd).hrgnClip.is_null() && ((*Wnd).style & WS_MINIMIZE as u32) == 0 {
        NtGdiOffsetRgn(hRgnWindow, -(*Wnd).rcWindow.left, -(*Wnd).rcWindow.top);
        NtGdiCombineRgn(hRgnWindow, hRgnWindow, (*Wnd).hrgnClip, RGN_AND as i32);
        NtGdiOffsetRgn(hRgnWindow, (*Wnd).rcWindow.left, (*Wnd).rcWindow.top);
    }

    hRgnWindow
}

fastcall_fn! {
unsafe fn IntGetNCUpdateRgn(Window: PWND, Validate: i32) -> HRGN {
    let hRgnNonClient: HRGN;
    let hRgnWindow: HRGN;
    let RgnType: u32;
    let NcType: u32;
    let mut update: RECT = core::mem::zeroed();

    if (*Window).hrgnUpdate != core::ptr::null_mut()
        && (*Window).hrgnUpdate != HRGN_WINDOW
    {
        hRgnNonClient = IntCalcWindowRgn(Window, 0);

        if hRgnNonClient == core::ptr::null_mut() {
            return HRGN_WINDOW;
        }

        hRgnWindow = IntCalcWindowRgn(Window, 1);
        if hRgnWindow == core::ptr::null_mut() {
            GreDeleteObject(hRgnNonClient as _ );
            return HRGN_WINDOW;
        }

        NcType = IntGdiGetRgnBox(hRgnNonClient, &mut update as *mut _ as _ ) as u32;

        RgnType = NtGdiCombineRgn(hRgnNonClient, hRgnNonClient, hRgnWindow, RGN_DIFF as i32) as u32;

        if RgnType == ERROR {
            GreDeleteObject(hRgnWindow as _ );
            GreDeleteObject(hRgnNonClient as _ );
            return HRGN_WINDOW;
        } else if RgnType == NULLREGION {
            GreDeleteObject(hRgnWindow as _ );
            GreDeleteObject(hRgnNonClient as _ );
            (*Window).state &= !WNDS_UPDATEDIRTY;
            return core::ptr::null_mut();
        }

        if Validate != 0 {
            if NtGdiCombineRgn((*Window).hrgnUpdate, (*Window).hrgnUpdate, hRgnWindow, RGN_AND as i32) as u32 == NULLREGION {
                IntGdiSetRegionOwner((*Window).hrgnUpdate, GDI_OBJ_HMGR_POWNED);
                GreDeleteObject((*Window).hrgnUpdate as _ );
                (*Window).state &= !WNDS_UPDATEDIRTY;
                (*Window).hrgnUpdate = core::ptr::null_mut();
                if (*Window).state & WNDS_INTERNALPAINT == 0 {
                    MsqDecPaintCountQueue(head_pti(Window));
                }
            }
        }

        if NcType == SIMPLEREGION {
            let mut window: RECT = core::mem::zeroed();
            IntGetWindowRect(Window, &mut window as *mut _ as _ );

            if IntEqualRect(&mut window as *mut _ as _, &mut update as *mut _ as _) != 0 {
                GreDeleteObject(hRgnNonClient as _ );
                GreDeleteObject(hRgnWindow as _ );
                return HRGN_WINDOW;
            }
        }

        GreDeleteObject(hRgnWindow as _ );

        return hRgnNonClient;
    } else {
        return (*Window).hrgnUpdate;
    }
}
}

#[cfg(target_pointer_width = "32")]
#[no_mangle]
unsafe extern "fastcall" fn IntSendNCPaint(pWnd: PWND, hRgn: HRGN) {
    IntSendNCPaintImpl(pWnd, hRgn)
}

#[cfg(not(target_pointer_width = "32"))]
#[no_mangle]
unsafe extern "C" fn IntSendNCPaint(pWnd: PWND, hRgn: HRGN) {
    IntSendNCPaintImpl(pWnd, hRgn)
}

unsafe fn IntSendNCPaintImpl(pWnd: PWND, mut hRgn: HRGN) {
    (*pWnd).state &= !WNDS_SENDNCPAINT;

    if pWnd == (*(*GetW32ThreadInfo()).MessageQueue).spwndActive
        && ((*pWnd).state & WNDS_ACTIVEFRAME) == 0
    {
        (*pWnd).state |= WNDS_ACTIVEFRAME;
        (*pWnd).state &= !WNDS_NONCPAINT;
        hRgn = HRGN_WINDOW;
    }

    if (*pWnd).state2 & WNDS2_FORCEFULLNCPAINTCLIPRGN != 0 {
        (*pWnd).state2 &= !WNDS2_FORCEFULLNCPAINTCLIPRGN;
        hRgn = HRGN_WINDOW;
    }

    if !hRgn.is_null() {
        co_IntSendMessage(UserHMGetHandle(pWnd), WM_NCPAINT, hRgn as WPARAM, 0);
    }
}

fastcall_fn! {
unsafe fn IntSendChildNCPaint(mut pWnd: PWND) {
    pWnd = (*pWnd).spwndChild;
    while !pWnd.is_null() {
        if (*pWnd).hrgnUpdate.is_null() && ((*pWnd).state & WNDS_SENDNCPAINT) != 0 {
            let Next: PWND;
            let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

            UserRefObjectCo(pWnd, &mut Ref);
            IntSendNCPaint(pWnd, HRGN_WINDOW);

            Next = (*pWnd).spwndNext;
            UserDerefObjectCo(pWnd);
            pWnd = Next;
        } else {
            pWnd = (*pWnd).spwndNext;
        }
    }
}
}

fastcall_fn! {
#[no_mangle]
unsafe fn co_IntPaintWindows(mut Wnd: PWND, Flags: ULONG, Recurse: i32) {
    let hDC: HDC;
    let hWnd: HWND = UserHMGetHandle(Wnd as _) as HWND;
    let mut TempRegion: HRGN = core::ptr::null_mut();

    (*Wnd).state &= !WNDS_PAINTNOTPROCESSED;

    if ((*Wnd).state & WNDS_SENDNCPAINT) != 0 || ((*Wnd).state & WNDS_SENDERASEBACKGROUND) != 0 {
        if ((*Wnd).style & WS_VISIBLE) == 0 {
            (*Wnd).state &= !(WNDS_SENDNCPAINT | WNDS_SENDERASEBACKGROUND | WNDS_ERASEBACKGROUND);
            return;
        } else {
            if (*Wnd).hrgnUpdate.is_null() {
                (*Wnd).state &= !(WNDS_SENDERASEBACKGROUND | WNDS_ERASEBACKGROUND);
            }

            if head_pti(Wnd) == PsGetCurrentThreadWin32Thread() as _ {
                if ((*Wnd).state & WNDS_SENDNCPAINT) != 0 {
                    TempRegion = IntGetNCUpdateRgn(Wnd, 1);
                    IntSendNCPaint(Wnd, TempRegion);

                    if TempRegion > HRGN_WINDOW && GreIsHandleValid(TempRegion as _ ) != 0 {
                        GreDeleteObject(TempRegion as _ );
                    }
                }

                if ((*Wnd).state & WNDS_SENDERASEBACKGROUND) != 0 {
                    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
                    if !(*Wnd).hrgnUpdate.is_null() {
                        hDC = UserGetDCEx(
                            Wnd,
                            (*Wnd).hrgnUpdate as _ ,
                            DCX_CACHE | DCX_USESTYLE | DCX_INTERSECTRGN | DCX_KEEPCLIPRGN,
                        );

                        if (*head_pti(Wnd)).ppi != (*pti).ppi {
                            ERR!("Sending DC to another Process!!!\n");
                        }

                        (*Wnd).state &= !(WNDS_SENDERASEBACKGROUND | WNDS_ERASEBACKGROUND);
                        if co_IntSendMessage(hWnd, WM_ERASEBKGND, hDC as WPARAM, 0) == 0 {
                            (*Wnd).state |= WNDS_SENDERASEBACKGROUND | WNDS_ERASEBACKGROUND;
                        }
                        UserReleaseDC(Wnd, hDC, 0);
                    }
                }
            }
        }
    }

    if IntIsWindow(hWnd) == 0 {
        return;
    }

    if (Flags & RDW_NOCHILDREN) == 0
        && ((*Wnd).style & WS_MINIMIZE) == 0
        && ((Flags & RDW_ALLCHILDREN) != 0
            || ((Flags & RDW_CLIPCHILDREN) != 0 && ((*Wnd).style & WS_CLIPCHILDREN) != 0))
    {
        let List: *mut HWND;
        let mut phWnd: *mut HWND;
        let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

        List = IntWinListChildren(Wnd);
        if !List.is_null() {
            phWnd = List;
            while !(*phWnd).is_null() {
                Wnd = UserGetWindowObject(*phWnd);
                if Wnd.is_null() {
                    phWnd = phWnd.add(1);
                    continue;
                }

                if head_pti(Wnd) != pti as _ && ((*Wnd).style & WS_CHILD) != 0 {
                    phWnd = phWnd.add(1);
                    continue;
                }

                if ((*Wnd).style & WS_VISIBLE) != 0 {
                    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
                    UserRefObjectCo(Wnd, &mut Ref);
                    co_IntPaintWindows(Wnd, Flags, 1);
                    UserDerefObjectCo(Wnd);
                }
                phWnd = phWnd.add(1);
            }
            ExFreePoolWithTag(List as _, USERTAG_WINDOWLIST);
        }
    }
}
}

fastcall_fn! {
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn co_IntUpdateWindows(Wnd: PWND, mut Flags: ULONG, Recurse: i32) {
    let hWnd = UserHMGetHandle(Wnd);
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    if (*Wnd).hrgnUpdate != core::ptr::null_mut() || ((*Wnd).state & WNDS_INTERNALPAINT) != 0 {
        if (*Wnd).hrgnUpdate != core::ptr::null_mut() {
            if IntValidateParents(Wnd, Recurse) == 0 {
                return;
            }
        }

        if (*Wnd).state & WNDS_INTERNALPAINT != 0 {
            (*Wnd).state &= !WNDS_INTERNALPAINT;

            if (*Wnd).hrgnUpdate == core::ptr::null_mut() {
                MsqDecPaintCountQueue(head_pti(Wnd));
            }
        }

        (*Wnd).state |= WNDS_PAINTNOTPROCESSED;
        (*Wnd).state &= !WNDS_UPDATEDIRTY;

        (*Wnd).state2 |= WNDS2_WMPAINTSENT;

        UserRefObjectCo(Wnd, &mut Ref);
        co_IntSendMessage(hWnd, WM_PAINT, 0, 0);

        if (*Wnd).state & WNDS_PAINTNOTPROCESSED != 0 {
            co_IntPaintWindows(Wnd, RDW_NOCHILDREN, 0);
        }
        UserDerefObjectCo(Wnd);
    }

    Flags = if Flags & RDW_NOCHILDREN != 0 { RDW_NOCHILDREN } else { RDW_ALLCHILDREN };

    if (Flags & RDW_NOCHILDREN) == 0
        && (Flags & RDW_ALLCHILDREN) != 0
        && UserIsDesktopWindow(Wnd) == 0
    {
        let mut Child: PWND = (*Wnd).spwndChild;
        while Child != core::ptr::null_mut() {
            if (*Child).ExStyle & WS_EX_TRANSPARENT != 0
                && ((*Child).hrgnUpdate != core::ptr::null_mut()
                    || (*Child).state & WNDS_INTERNALPAINT != 0)
            {
                let mut Next: PWND = (*Child).spwndNext;
                while Next != core::ptr::null_mut() {
                    if (*Next).hrgnUpdate != core::ptr::null_mut()
                        || (*Next).state & WNDS_INTERNALPAINT != 0
                    {
                        break;
                    }
                    Next = (*Next).spwndNext;
                }

                if Next != core::ptr::null_mut() {
                    Child = (*Child).spwndNext;
                    continue;
                }
            }

            if (*Child).style & WS_VISIBLE != 0 {
                let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
                UserRefObjectCo(Child, &mut Ref);
                co_IntUpdateWindows(Child, Flags, 1);
                UserDerefObjectCo(Child);
            }

            Child = (*Child).spwndNext;
        }
    }
}
}

#[cfg(target_arch = "x86")]
#[allow(non_snake_case)]
unsafe extern "fastcall" fn UserUpdateWindows(pWnd: PWND, Flags: ULONG) {
    if (*pWnd).ExStyle & WS_EX_TRANSPARENT != 0 {
        let mut Next: PWND = (*pWnd).spwndNext;

        while !Next.is_null() {
            if head_pti(Next) == head_pti(pWnd)
                && (!(*Next).hrgnUpdate.is_null() || (*Next).state & WNDS_INTERNALPAINT != 0)
            {
                return;
            }

            Next = (*Next).spwndNext;
        }
    }
    co_IntUpdateWindows(pWnd, Flags, 0);
}

#[cfg(not(target_arch = "x86"))]
#[allow(non_snake_case)]
unsafe extern "C" fn UserUpdateWindows(pWnd: PWND, Flags: ULONG) {
    if (*pWnd).ExStyle & WS_EX_TRANSPARENT != 0 {
        let mut Next: PWND = (*pWnd).spwndNext;

        while !Next.is_null() {
            if head_pti(Next) == head_pti(pWnd)
                && (!(*Next).hrgnUpdate.is_null() || (*Next).state & WNDS_INTERNALPAINT != 0)
            {
                return;
            }

            Next = (*Next).spwndNext;
        }
    }
    co_IntUpdateWindows(pWnd, Flags, 0);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserSyncAndPaintWindows(pWnd: PWND, Flags: ULONG) {
    let mut Parent: PWND = pWnd;
    loop {
        Parent = (*Parent).spwndParent;
        if Parent.is_null() {
            break;
        }
        if (*Parent).style & WS_CLIPCHILDREN != 0 {
            break;
        }
        if !(*Parent).hrgnUpdate.is_null() || (*Parent).state & WNDS_INTERNALPAINT != 0 {
            return;
        }
    }

    IntSendSyncPaint(pWnd, Flags);
    co_IntPaintWindows(pWnd, Flags, 0);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserSyncAndPaintWindows(pWnd: PWND, Flags: ULONG) {
    let mut Parent: PWND = pWnd;
    loop {
        Parent = (*Parent).spwndParent;
        if Parent.is_null() {
            break;
        }
        if (*Parent).style & WS_CLIPCHILDREN != 0 {
            break;
        }
        if !(*Parent).hrgnUpdate.is_null() || (*Parent).state & WNDS_INTERNALPAINT != 0 {
            return;
        }
    }

    IntSendSyncPaint(pWnd, Flags);
    co_IntPaintWindows(pWnd, Flags, 0);
}

fastcall_fn! {
#[no_mangle]
unsafe fn IntInvalidateWindows(Wnd: PWND, Rgn: PREGION, mut Flags: ULONG) {
    let mut RgnType: INT = NULLREGION as INT;
    let HadPaintMessage: BOOL;

    if Rgn as usize > PRGN_WINDOW as usize {
        if (Flags & RDW_INVALIDATE) != 0 && (Flags & RDW_FRAME) == 0 {
            let RgnClient = IntSysCreateRectpRgnIndirect(core::ptr::addr_of_mut!((*Wnd).rcClient));
            if !RgnClient.is_null() {
                RgnType = IntGdiCombineRgn(Rgn, Rgn, RgnClient, RGN_AND as INT);
                REGION_Delete(RgnClient);
            }
        }

        if (*Wnd).hrgnClip.is_null() || ((*Wnd).style & WS_MINIMIZE) != 0 {
            let RgnWindow = IntSysCreateRectpRgnIndirect(core::ptr::addr_of_mut!((*Wnd).rcWindow));
            if !RgnWindow.is_null() {
                RgnType = IntGdiCombineRgn(Rgn, Rgn, RgnWindow, RGN_AND as INT);
                REGION_Delete(RgnWindow);
            }
        } else {
            let RgnClip = REGION_LockRgn((*Wnd).hrgnClip);
            if !RgnClip.is_null() {
                REGION_bOffsetRgn(Rgn, -(*Wnd).rcWindow.left, -(*Wnd).rcWindow.top);
                RgnType = IntGdiCombineRgn(Rgn, Rgn, RgnClip, RGN_AND as INT);
                REGION_bOffsetRgn(Rgn, (*Wnd).rcWindow.left, (*Wnd).rcWindow.top);
                REGION_UnlockRgn(RgnClip);
            }
        }
    } else {
        RgnType = NULLREGION as INT;
    }

    if (RgnType == NULLREGION as INT && (Flags & RDW_INVALIDATE) != 0) || RgnType == ERROR as INT {
        return;
    }

    HadPaintMessage = IntIsWindowDirty(Wnd);

    if Flags & (RDW_INVALIDATE | RDW_INTERNALPAINT | RDW_ERASE | RDW_FRAME) != 0 {
        if Flags & RDW_INTERNALPAINT != 0 {
            (*Wnd).state |= WNDS_INTERNALPAINT;
        }

        if Flags & RDW_INVALIDATE != 0 {
            (*Wnd).state &= !WNDS_NONCPAINT;

            if head_pti(Wnd) != PsGetCurrentThreadWin32Thread() as _ {
                (*Wnd).state |= WNDS_UPDATEDIRTY;
                if (*Wnd).state2 & WNDS2_WMPAINTSENT != 0 {
                    (*Wnd).state2 |= WNDS2_ENDPAINTINVALIDATE;
                }
            }

            if Flags & RDW_FRAME != 0 {
                (*Wnd).state |= WNDS_SENDNCPAINT;
            }

            if Flags & RDW_ERASE != 0 {
                (*Wnd).state |= WNDS_SENDERASEBACKGROUND;
            }

            if RgnType != NULLREGION as INT && Rgn as usize > PRGN_WINDOW as usize {
                if (*Wnd).hrgnUpdate.is_null() {
                    (*Wnd).hrgnUpdate = NtGdiCreateRectRgn(0, 0, 0, 0);
                    IntGdiSetRegionOwner((*Wnd).hrgnUpdate, GDI_OBJ_HMGR_PUBLIC);
                }

                if (*Wnd).hrgnUpdate != HRGN_WINDOW {
                    let RgnUpdate = REGION_LockRgn((*Wnd).hrgnUpdate);
                    if !RgnUpdate.is_null() {
                        RgnType = IntGdiCombineRgn(RgnUpdate, RgnUpdate, Rgn, RGN_OR as INT);
                        REGION_UnlockRgn(RgnUpdate);
                        if RgnType == NULLREGION as INT {
                            IntGdiSetRegionOwner((*Wnd).hrgnUpdate, GDI_OBJ_HMGR_POWNED);
                            GreDeleteObject((*Wnd).hrgnUpdate as _ );
                            (*Wnd).hrgnUpdate = core::ptr::null_mut();
                        }
                    }
                }
            }

            Flags |= RDW_ERASE | RDW_FRAME;
        }

        if HadPaintMessage == 0 && IntIsWindowDirty(Wnd) != 0 {
            MsqIncPaintCountQueue(head_pti(Wnd));
        }
    } else if Flags & (RDW_VALIDATE | RDW_NOINTERNALPAINT | RDW_NOERASE | RDW_NOFRAME) != 0 {
        if (*Wnd).state & WNDS_UPDATEDIRTY != 0 && (Flags & RDW_NOUPDATEDIRTY) == 0 {
            return;
        }

        if Flags & RDW_NOINTERNALPAINT != 0 {
            (*Wnd).state &= !WNDS_INTERNALPAINT;
        }

        if Flags & RDW_VALIDATE != 0 {
            if Flags & RDW_NOFRAME != 0 {
                (*Wnd).state &= !WNDS_SENDNCPAINT;
            }

            if Flags & RDW_NOERASE != 0 {
                (*Wnd).state &= !(WNDS_SENDERASEBACKGROUND | WNDS_ERASEBACKGROUND);
            }

            if (*Wnd).hrgnUpdate as usize > HRGN_WINDOW as usize
                && RgnType != NULLREGION as INT
                && Rgn as usize > PRGN_WINDOW as usize
            {
                let RgnUpdate = REGION_LockRgn((*Wnd).hrgnUpdate);
                if !RgnUpdate.is_null() {
                    RgnType = IntGdiCombineRgn(RgnUpdate, RgnUpdate, Rgn, RGN_DIFF as INT);
                    REGION_UnlockRgn(RgnUpdate);

                    if RgnType == NULLREGION as INT {
                        IntGdiSetRegionOwner((*Wnd).hrgnUpdate, GDI_OBJ_HMGR_POWNED);
                        GreDeleteObject((*Wnd).hrgnUpdate as _ );
                        (*Wnd).hrgnUpdate = core::ptr::null_mut();
                    }
                }
            }

            if (*Wnd).hrgnUpdate.is_null() {
                (*Wnd).state &= !(WNDS_SENDERASEBACKGROUND | WNDS_ERASEBACKGROUND);
            }
        }

        if HadPaintMessage != 0 && IntIsWindowDirty(Wnd) == 0 {
            MsqDecPaintCountQueue(head_pti(Wnd));
        }
    }

    if (Flags & RDW_NOCHILDREN) == 0
        && ((*Wnd).style & WS_MINIMIZE) == 0
        && ((Flags & RDW_ALLCHILDREN) != 0 || ((*Wnd).style & WS_CLIPCHILDREN) == 0)
    {
        let mut Child = (*Wnd).spwndChild;
        while !Child.is_null() {
            if (*Child).style & WS_VISIBLE != 0 {
                let RgnTemp = IntSysCreateRectpRgn(0, 0, 0, 0);
                if !RgnTemp.is_null() {
                    if Rgn as usize > PRGN_WINDOW as usize {
                        IntGdiCombineRgn(RgnTemp, Rgn, core::ptr::null_mut(), RGN_COPY as INT);
                    }
                    IntInvalidateWindows(
                        Child,
                        if Rgn as usize > PRGN_WINDOW as usize { RgnTemp } else { Rgn },
                        Flags,
                    );
                    REGION_Delete(RgnTemp);
                }
            }
            Child = (*Child).spwndNext;
        }
    }
}
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntIsWindowDrawable(Wnd: PWND) -> u32 {
    IntIsWindowDrawable_impl(Wnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntIsWindowDrawable(Wnd: PWND) -> u32 {
    IntIsWindowDrawable_impl(Wnd)
}

unsafe fn IntIsWindowDrawable_impl(Wnd: PWND) -> u32 {
    let mut WndObject = Wnd;

    while !WndObject.is_null() {
        if (*WndObject).state2 & WNDS2_INDESTROY != 0
            || (*WndObject).state & WNDS_DESTROYED != 0
            || WndObject.is_null()
            || ((*WndObject).style & WS_VISIBLE) == 0
            || (((*WndObject).style & WS_MINIMIZE) != 0 && WndObject != Wnd)
        {
            return 0;
        }

        WndObject = (*WndObject).spwndParent;
    }

    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_UserRedrawWindow(
    mut Window: PWND,
    UpdateRect: *const RECTL,
    UpdateRgn: PREGION,
    mut Flags: ULONG,
) -> u32 {

    const PRGN_WINDOW: PREGION = 1 as PREGION;

    let mut TmpRgn: PREGION = core::ptr::null_mut();

    if IntIsWindowDrawable(Window) == 0 {
        return 1;
    }

    if Window.is_null() {
        Window = UserGetDesktopWindow();
    }

    if Flags & (RDW_INVALIDATE | RDW_VALIDATE) != 0 {
        if !UpdateRgn.is_null() {
            TmpRgn = IntSysCreateRectpRgn(0, 0, 0, 0);

            if UpdateRgn > PRGN_WINDOW {
                IntGdiCombineRgn(TmpRgn, UpdateRgn, core::ptr::null_mut(), RGN_COPY as i32);
            }

            if Window != UserGetDesktopWindow() {
                REGION_bOffsetRgn(TmpRgn, (*Window).rcClient.left, (*Window).rcClient.top);
            }
        } else {
            if !UpdateRect.is_null() {
                if Window == UserGetDesktopWindow() {
                    TmpRgn = IntSysCreateRectpRgnIndirect(UpdateRect);
                } else {
                    TmpRgn = IntSysCreateRectpRgn(
                        (*Window).rcClient.left + (*UpdateRect).left,
                        (*Window).rcClient.top + (*UpdateRect).top,
                        (*Window).rcClient.left + (*UpdateRect).right,
                        (*Window).rcClient.top + (*UpdateRect).bottom,
                    );
                }
            } else {
                if (Flags & (RDW_INVALIDATE | RDW_FRAME)) == (RDW_INVALIDATE | RDW_FRAME)
                    || (Flags & (RDW_VALIDATE | RDW_NOFRAME)) == (RDW_VALIDATE | RDW_NOFRAME)
                {
                    if RECTL_bIsEmptyRect(core::ptr::addr_of!((*Window).rcWindow)) == 0 {
                        TmpRgn = IntSysCreateRectpRgnIndirect(core::ptr::addr_of!((*Window).rcWindow));
                    }
                } else {
                    if RECTL_bIsEmptyRect(core::ptr::addr_of!((*Window).rcClient)) == 0 {
                        TmpRgn = IntSysCreateRectpRgnIndirect(core::ptr::addr_of!((*Window).rcClient));
                    }
                }
            }
        }
    }

    if TmpRgn.is_null() {
        TmpRgn = PRGN_WINDOW;
    }

    if Flags & (RDW_INVALIDATE | RDW_VALIDATE | RDW_INTERNALPAINT | RDW_NOINTERNALPAINT) != 0
        && !TmpRgn.is_null()
    {
        IntInvalidateWindows(Window, TmpRgn, Flags);
    }

    if Flags & RDW_UPDATENOW != 0 {
        UserUpdateWindows(Window, Flags);
    } else if Flags & RDW_ERASENOW != 0 {
        if (Flags & (RDW_NOCHILDREN | RDW_ALLCHILDREN)) == 0 {
            Flags |= RDW_CLIPCHILDREN;
        }

        UserSyncAndPaintWindows(Window, Flags);
    }

    if TmpRgn > PRGN_WINDOW {
        REGION_Delete(TmpRgn);
    }

    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_UserRedrawWindow(
    mut Window: PWND,
    UpdateRect: *const RECTL,
    UpdateRgn: PREGION,
    mut Flags: ULONG,
) -> u32 {

    const PRGN_WINDOW: PREGION = 1 as PREGION;

    let mut TmpRgn: PREGION = core::ptr::null_mut();

    if IntIsWindowDrawable(Window) == 0 {
        return 1;
    }

    if Window.is_null() {
        Window = UserGetDesktopWindow();
    }

    if Flags & (RDW_INVALIDATE | RDW_VALIDATE) != 0 {
        if !UpdateRgn.is_null() {
            TmpRgn = IntSysCreateRectpRgn(0, 0, 0, 0);

            if UpdateRgn > PRGN_WINDOW {
                IntGdiCombineRgn(TmpRgn, UpdateRgn, core::ptr::null_mut(), RGN_COPY as i32);
            }

            if Window != UserGetDesktopWindow() {
                REGION_bOffsetRgn(TmpRgn, (*Window).rcClient.left, (*Window).rcClient.top);
            }
        } else {
            if !UpdateRect.is_null() {
                if Window == UserGetDesktopWindow() {
                    TmpRgn = IntSysCreateRectpRgnIndirect(UpdateRect);
                } else {
                    TmpRgn = IntSysCreateRectpRgn(
                        (*Window).rcClient.left + (*UpdateRect).left,
                        (*Window).rcClient.top + (*UpdateRect).top,
                        (*Window).rcClient.left + (*UpdateRect).right,
                        (*Window).rcClient.top + (*UpdateRect).bottom,
                    );
                }
            } else {
                if (Flags & (RDW_INVALIDATE | RDW_FRAME)) == (RDW_INVALIDATE | RDW_FRAME)
                    || (Flags & (RDW_VALIDATE | RDW_NOFRAME)) == (RDW_VALIDATE | RDW_NOFRAME)
                {
                    if RECTL_bIsEmptyRect(core::ptr::addr_of!((*Window).rcWindow)) == 0 {
                        TmpRgn = IntSysCreateRectpRgnIndirect(core::ptr::addr_of!((*Window).rcWindow));
                    }
                } else {
                    if RECTL_bIsEmptyRect(core::ptr::addr_of!((*Window).rcClient)) == 0 {
                        TmpRgn = IntSysCreateRectpRgnIndirect(core::ptr::addr_of!((*Window).rcClient));
                    }
                }
            }
        }
    }

    if TmpRgn.is_null() {
        TmpRgn = PRGN_WINDOW;
    }

    if Flags & (RDW_INVALIDATE | RDW_VALIDATE | RDW_INTERNALPAINT | RDW_NOINTERNALPAINT) != 0
        && !TmpRgn.is_null()
    {
        IntInvalidateWindows(Window, TmpRgn, Flags);
    }

    if Flags & RDW_UPDATENOW != 0 {
        UserUpdateWindows(Window, Flags);
    } else if Flags & RDW_ERASENOW != 0 {
        if (Flags & (RDW_NOCHILDREN | RDW_ALLCHILDREN)) == 0 {
            Flags |= RDW_CLIPCHILDREN;
        }

        UserSyncAndPaintWindows(Window, Flags);
    }

    if TmpRgn > PRGN_WINDOW {
        REGION_Delete(TmpRgn);
    }

    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn PaintSuspendedWindow(pwnd: PWND, hrgnOrig: HRGN) {
    PaintSuspendedWindow_impl(pwnd, hrgnOrig)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn PaintSuspendedWindow(pwnd: PWND, hrgnOrig: HRGN) {
    PaintSuspendedWindow_impl(pwnd, hrgnOrig)
}

unsafe fn PaintSuspendedWindow_impl(pwnd: PWND, hrgnOrig: HRGN) {
    if !(*pwnd).hrgnUpdate.is_null() {
        let hDC: HDC;
        let Flags: INT = (DC_NC | DC_NOSENDMSG) as INT;
        let mut hrgnTemp: HRGN;
        let mut Rect: RECT;
        let mut _type: INT;
        let prgn: PREGION;

        if (*pwnd).hrgnUpdate as usize > HRGN_WINDOW as usize {
            hrgnTemp = NtGdiCreateRectRgn(0, 0, 0, 0);
            _type = NtGdiCombineRgn(hrgnTemp, (*pwnd).hrgnUpdate, core::ptr::null_mut(), RGN_COPY as i32);
            if _type == ERROR as INT {
                GreDeleteObject(hrgnTemp as _);
                hrgnTemp = HRGN_WINDOW;
            }
        } else {
            hrgnTemp = GreCreateRectRgnIndirect(core::ptr::addr_of_mut!((*pwnd).rcWindow));
        }

        if !hrgnOrig.is_null()
            && hrgnTemp as usize > HRGN_WINDOW as usize
            && NtGdiCombineRgn(hrgnTemp, hrgnTemp, hrgnOrig, RGN_AND as i32) == NULLREGION as i32
        {
            GreDeleteObject(hrgnTemp as _);
            return;
        }

        hDC = UserGetDCEx(pwnd, hrgnTemp as _ , DCX_WINDOW | DCX_INTERSECTRGN | DCX_USESTYLE | DCX_KEEPCLIPRGN);

        Rect = core::mem::transmute((*pwnd).rcWindow);
        RECTL_vOffsetRect(core::ptr::addr_of_mut!(Rect), -(*pwnd).rcWindow.left, -(*pwnd).rcWindow.top);

        FillRect(hDC, core::ptr::addr_of_mut!(Rect), IntGetSysColorBrush(COLOR_WINDOW as i32) as _ );

        NC_DoNCPaint(pwnd, hDC, Flags);

        UserReleaseDC(pwnd, hDC, 0u8 as _);

        prgn = REGION_LockRgn(hrgnTemp);
        IntInvalidateWindows(pwnd, prgn, (RDW_INVALIDATE | RDW_FRAME | RDW_ERASE | RDW_ALLCHILDREN) as ULONG);
        REGION_UnlockRgn(prgn);

        (*pwnd).state |= (WNDS_SENDNCPAINT | WNDS_SENDERASEBACKGROUND | WNDS_UPDATEDIRTY) as u32;

        if hrgnTemp as usize > HRGN_WINDOW as usize && GreIsHandleValid(hrgnTemp as _) != 0 {
            GreDeleteObject(hrgnTemp as _);
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn UpdateTheadChildren(mut pWnd: PWND, hRgn: HRGN) {
    PaintSuspendedWindow(pWnd, hRgn);

    if ((*pWnd).style & WS_CLIPCHILDREN) == 0 {
        return;
    }

    pWnd = (*pWnd).spwndChild;
    while !pWnd.is_null() {
        UpdateTheadChildren(pWnd, hRgn);
        pWnd = (*pWnd).spwndNext;
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UpdateTheadChildren(mut pWnd: PWND, hRgn: HRGN) {
    PaintSuspendedWindow(pWnd, hRgn);

    if ((*pWnd).style & WS_CLIPCHILDREN) == 0 {
        return;
    }

    pWnd = (*pWnd).spwndChild;
    while !pWnd.is_null() {
        UpdateTheadChildren(pWnd, hRgn);
        pWnd = (*pWnd).spwndNext;
    }
}

fastcall_fn! {
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn UpdateThreadWindows(pWnd: PWND, pti: PTHREADINFO, hRgn: HRGN) {
    let mut pwndTemp: PWND = pWnd;

    while !pwndTemp.is_null() {
        if head_pti(pwndTemp) == pti {
            UserUpdateWindows(pwndTemp, RDW_ALLCHILDREN);
        } else {
            if IsThreadSuspended(head_pti(pwndTemp)) != 0
                || MsqIsHung(head_pti(pwndTemp), MSQ_HUNG) != 0
            {
                UpdateTheadChildren(pwndTemp, hRgn);
            } else {
                UserUpdateWindows(pwndTemp, RDW_ALLCHILDREN);
            }
        }
        pwndTemp = (*pwndTemp).spwndNext;
    }
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntIsWindowDirty(Wnd: PWND) -> BOOL {
    ((((*Wnd).style & WS_VISIBLE) != 0)
        && ((*Wnd).hrgnUpdate != core::ptr::null_mut()
            || ((*Wnd).state & WNDS_INTERNALPAINT) != 0)) as BOOL
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntIsWindowDirty(Wnd: PWND) -> BOOL {
    ((((*Wnd).style & WS_VISIBLE) != 0)
        && ((*Wnd).hrgnUpdate != core::ptr::null_mut()
            || ((*Wnd).state & WNDS_INTERNALPAINT) != 0)) as BOOL
}

fastcall_fn! {
unsafe fn IntFindWindowToRepaint(mut Window: PWND, Thread: PTHREADINFO) -> PWND {
    let mut hChild: PWND;
    let mut TempWindow: PWND;

    while Window != core::ptr::null_mut() {
        if IntWndBelongsToThread(Window, Thread) != 0 {
            if IntIsWindowDirty(Window) != 0 {
                /* Make sure all non-transparent siblings are already drawn. */
                if (*Window).ExStyle & WS_EX_TRANSPARENT != 0 {
                    TempWindow = (*Window).spwndNext;
                    while TempWindow != core::ptr::null_mut() {
                        if (*TempWindow).ExStyle & WS_EX_TRANSPARENT == 0
                            && IntWndBelongsToThread(TempWindow, Thread) != 0
                            && IntIsWindowDirty(TempWindow) != 0
                        {
                            return TempWindow;
                        }
                        TempWindow = (*TempWindow).spwndNext;
                    }
                }
                return Window;
            }
        }
        /* find a child of the specified window that needs repainting */
        if (*Window).spwndChild != core::ptr::null_mut() {
            hChild = IntFindWindowToRepaint((*Window).spwndChild, Thread);
            if hChild != core::ptr::null_mut() {
                return hChild;
            }
        }

        Window = (*Window).spwndNext;
    }
    Window
}
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntPaintWindow(Window: PWND) {
    co_IntPaintWindows(Window, RDW_NOCHILDREN, 0);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntPaintWindow(Window: PWND) {
    co_IntPaintWindows(Window, RDW_NOCHILDREN, 0);
}

fastcall_fn! {
#[no_mangle]
unsafe fn IntGetPaintMessage(
    mut Window: PWND,
    MsgFilterMin: UINT,
    MsgFilterMax: UINT,
    Thread: PTHREADINFO,
    Message: *mut MSG,
    Remove: BOOL,
) -> BOOL {
    let StartWnd: PWND;
    let PaintWnd: PWND;

    if (MsgFilterMin != 0 || MsgFilterMax != 0)
        && (MsgFilterMin > WM_PAINT || MsgFilterMax < WM_PAINT)
    {
        return 0;
    }

    if (*Thread).TIF_flags & TIF_SYSTEMTHREAD != 0 {
        ERR!("WM_PAINT is in a System Thread!\n");
    }

    StartWnd = UserGetDesktopWindow();
    PaintWnd = IntFindWindowToRepaint(StartWnd, Thread);

    (*Message).hwnd = if !PaintWnd.is_null() {
        UserHMGetHandle(PaintWnd)
    } else {
        core::ptr::null_mut()
    };

    if (*Message).hwnd.is_null() && (*Thread).cPaintsReady != 0 {
        // ERR! is a no-op macro in this port — print for real: this firing
        // means the dirty-window bookkeeping leaked (cPaintsReady counted a
        // window IntFindWindowToRepaint can no longer find), which is the
        // root the stuck-QS_PAINT spin below papers over. Keep it visible.
        extern "C" {
            fn DbgPrint(Format: *const u8, ...) -> u32;
        }
        DbgPrint(
            b"[WIN32K] IntGetPaintMessage: cPaintsReady=%u but no dirty window (leak) - resetting paint state\n\0".as_ptr(),
            (*Thread).cPaintsReady,
        );
        (*Thread).cPaintsReady = 0;
        // Upstream zeroes the count but leaves the QS_PAINT wake/change bits
        // set — MsgWaitForMultipleObjects then returns "input available"
        // forever while PeekMessage retrieves nothing, and the caller spins at
        // 100% CPU (hit on arm64: explorer's main thread, 2026-07-06). Clear
        // the bits along with the count so the bandaid actually resets the
        // paint bookkeeping it claims to.
        (*(*Thread).pcti).fsWakeBits &= !(QS_PAINT as u16);
        (*(*Thread).pcti).fsChangeBits &= !(QS_PAINT as u16);
        return 0;
    }

    if (*Message).hwnd.is_null() {
        return 0;
    }

    if !(Window.is_null()
        || PaintWnd == Window
        || IntIsChildWindow(Window, PaintWnd) != 0)
    {
        return 0;
    }

    if (*PaintWnd).state & WNDS_INTERNALPAINT != 0 {
        (*PaintWnd).state &= !WNDS_INTERNALPAINT;
        if (*PaintWnd).hrgnUpdate.is_null() {
            MsqDecPaintCountQueue(Thread);
        }
    }
    (*PaintWnd).state2 &= !WNDS2_STARTPAINT;
    (*PaintWnd).state &= !WNDS_UPDATEDIRTY;

    Window = PaintWnd;
    while !Window.is_null() && UserIsDesktopWindow(Window) == 0 {
        if !(*Window).spwndParent.is_null()
            && (*(*Window).spwndParent).style & WS_CLIPCHILDREN == 0
        {
            (*PaintWnd).state2 |= WNDS2_WMPAINTSENT;
        }
        Window = (*Window).spwndParent;
    }

    (*Message).lParam = 0;
    (*Message).wParam = 0;
    (*Message).message = WM_PAINT;
    1
}
}

#[cfg(target_arch = "x86")]
#[allow(non_snake_case)]
unsafe extern "fastcall" fn IntPrintWindow(
    pwnd: PWND,
    hdcBlt: HDC,
    nFlags: UINT,
) -> BOOL {
    IntPrintWindow_impl(pwnd, hdcBlt, nFlags)
}

#[cfg(not(target_arch = "x86"))]
#[allow(non_snake_case)]
unsafe extern "C" fn IntPrintWindow(
    pwnd: PWND,
    hdcBlt: HDC,
    nFlags: UINT,
) -> BOOL {
    IntPrintWindow_impl(pwnd, hdcBlt, nFlags)
}

#[allow(non_snake_case)]
unsafe fn IntPrintWindow_impl(
    pwnd: PWND,
    hdcBlt: HDC,
    nFlags: UINT,
) -> BOOL {
    let cx: INT;
    let cy: INT;
    let xSrc: INT;
    let ySrc: INT;

    if (nFlags & PW_CLIENTONLY) != 0 {
        cx = (*pwnd).rcClient.right - (*pwnd).rcClient.left;
        cy = (*pwnd).rcClient.bottom - (*pwnd).rcClient.top;
        xSrc = (*pwnd).rcClient.left - (*pwnd).rcWindow.left;
        ySrc = (*pwnd).rcClient.top - (*pwnd).rcWindow.top;
    } else {
        cx = (*pwnd).rcWindow.right - (*pwnd).rcWindow.left;
        cy = (*pwnd).rcWindow.bottom - (*pwnd).rcWindow.top;
        xSrc = 0;
        ySrc = 0;
    }

    let _ = (cx, cy, xSrc, ySrc);

    // TODO: Setup Redirection for Print.
    return FALSE as BOOL;

    #[allow(unreachable_code)]
    {
        co_IntUpdateWindows(pwnd, RDW_ALLCHILDREN, FALSE as i32);

        let hdcSrc: HDC = UserGetDCEx(pwnd, core::ptr::null_mut(), DCX_CACHE | DCX_WINDOW);
        NtGdiBitBlt(
            hdcBlt,
            0,
            0,
            cx,
            cy,
            hdcSrc,
            xSrc,
            ySrc,
            SRCCOPY as DWORD,
            CLR_INVALID,
            0,
        );

        UserReleaseDC(pwnd, hdcSrc, FALSE as BOOL);

        // TODO: Release Redirection from Print.

        TRUE as BOOL
    }
}

fastcall_fn! {
#[no_mangle]
unsafe fn IntFlashWindowEx(pWnd: PWND, pfwi: PFLASHWINFO) -> BOOL {
    let mut FlashState: DWORD_PTR;
    let mut uCount: UINT = (*pfwi).uCount;
    let mut Activate: BOOL = 0;
    let mut Ret: BOOL = 0;

    FlashState = UserGetProp(pWnd, AtomFlashWndState as _, 1u8) as DWORD_PTR;

    if FlashState == FLASHW_FINISHED as DWORD_PTR {
        FlashState |= FLASHW_KILLSYSTIMER as DWORD_PTR;
        (*pfwi).dwFlags = FLASHW_STOP;
    } else {
        if FlashState != 0 {
            if (*pfwi).dwFlags == FLASHW_SYSTIMER {
                (*pfwi).dwFlags = LOWORD(FlashState as u32) as u32;
                uCount = HIWORD(FlashState as u32) as UINT;
                FlashState = MAKELONG(LOWORD(FlashState as u32), 0u32) as DWORD_PTR;
            } else {
                FlashState = 0;
            }
        }

        if FlashState == 0 {
            if ((*pWnd).state & WNDS_ACTIVEFRAME) != 0
                || ((*pfwi).dwFlags & FLASHW_CAPTION) != 0
                    && ((*pWnd).style & (WS_BORDER | WS_DLGFRAME)) != 0
            {
                FlashState = (FLASHW_STARTED | FLASHW_ACTIVE) as DWORD_PTR;
            }
        }

        Ret = ((FlashState & FLASHW_ACTIVE as DWORD_PTR) != 0) as BOOL;

        if ((*pfwi).dwFlags & FLASHW_TIMERNOFG) == FLASHW_TIMERNOFG
            && gpqForeground == (*head_pti(pWnd)).MessageQueue
        {
            (*pfwi).dwFlags = FLASHW_STOP;
        }
    }

    if (*pfwi).dwFlags == FLASHW_STOP {
        if !gpqForeground.is_null() && (*gpqForeground).spwndActive == pWnd {
            Activate = 1;
        } else {
            Activate = 0;
        }
    } else {
        Activate = ((FlashState & FLASHW_ACTIVE as DWORD_PTR) == 0) as BOOL;
    }

    if (*pfwi).dwFlags == FLASHW_STOP || ((*pfwi).dwFlags & FLASHW_CAPTION) != 0 {
        co_IntSendMessage(UserHMGetHandle(pWnd), WM_NCACTIVATE, Activate as WPARAM, 0);
    }

    if ((*pfwi).dwFlags & FLASHW_TRAY) != 0 {
        TRACE!(b"FIXME: Flash window no Tray support!\n\0".as_ptr() as *const i8);
    }

    if (*pfwi).dwFlags == FLASHW_STOP {
        if (FlashState & FLASHW_KILLSYSTIMER as DWORD_PTR) != 0 {
            IntKillTimer(pWnd, ID_EVENT_SYSTIMER_FLASHWIN as _, 1i32);
        }

        UserRemoveProp(pWnd, AtomFlashWndState as _, 1u8);
    } else {
        if uCount != 0 {
            FlashState |= FLASHW_COUNT as DWORD_PTR;

            if (Activate ^ ((FlashState & FLASHW_STARTED as DWORD_PTR) != 0) as BOOL) == 0 {
                uCount -= 1;
            }

            if (FlashState & FLASHW_KILLSYSTIMER as DWORD_PTR) == 0 {
                (*pfwi).dwFlags |= FLASHW_TIMER;
            }
        }

        if ((*pfwi).dwFlags & FLASHW_TIMER) != 0 {
            FlashState |= FLASHW_KILLSYSTIMER as DWORD_PTR;

            IntSetTimer(
                pWnd,
                ID_EVENT_SYSTIMER_FLASHWIN as _,
                if (*pfwi).dwTimeout != 0 {
                    (*pfwi).dwTimeout
                } else {
                    (*gpsi_per_user()).dtCaretBlink
                },
                core::mem::transmute::<_, TIMERPROC>(SystemTimerProc as *const ()),
                TMRF_SYSTEM as _,
            );
        }

        if (FlashState & FLASHW_COUNT as DWORD_PTR) != 0 && uCount == 0 {
            FlashState = FLASHW_FINISHED as DWORD_PTR;
        } else {
            FlashState ^= (FlashState ^ (-((Activate != 0) as i64) as DWORD_PTR))
                & FLASHW_ACTIVE as DWORD_PTR;
            FlashState ^= (FlashState ^ (*pfwi).dwFlags as DWORD_PTR)
                & ((FLASHW_MASK & !FLASHW_TIMER) as DWORD_PTR);
        }
        FlashState = MAKELONG(LOWORD(FlashState as u32), uCount as u16) as DWORD_PTR;
        UserSetProp(pWnd, AtomFlashWndState as _, FlashState as _, 1u8);
    }
    Ret
}
}

fastcall_fn! {
#[no_mangle]
unsafe fn IntBeginPaint(Window: PWND, Ps: PPAINTSTRUCT) -> HDC {
    let mut Rect: RECT = core::mem::zeroed();
    let mut type_: INT;
    let mut Erase: BOOL = 0;

    co_UserHideCaret(Window);

    (*Window).state2 |= WNDS2_STARTPAINT;
    (*Window).state &= !WNDS_PAINTNOTPROCESSED;

    if (*Window).state & WNDS_SENDNCPAINT != 0 {
        let mut hRgn: HRGN;
        // Application can keep update dirty.
        loop {
            (*Window).state &= !WNDS_UPDATEDIRTY;
            hRgn = IntGetNCUpdateRgn(Window, 0);
            IntSendNCPaint(Window, hRgn);
            if hRgn as usize > HRGN_WINDOW as usize && GreIsHandleValid(hRgn as _ ) != 0 {
                /* NOTE: The region can already be deleted! */
                GreDeleteObject(hRgn as _ );
            }
            if (*Window).state & WNDS_UPDATEDIRTY == 0 {
                break;
            }
        }
    } else {
        (*Window).state &= !WNDS_UPDATEDIRTY;
    }

    RtlZeroMemory(Ps as *mut _, core::mem::size_of::<PAINTSTRUCT>() as _);

    if (*Window).state2 & WNDS2_ENDPAINTINVALIDATE != 0 {
        ERR!("BP: Another thread invalidated this window\n");
    }

    (*Ps).hdc = UserGetDCEx(
        Window,
        (*Window).hrgnUpdate as _ ,
        DCX_INTERSECTRGN | DCX_USESTYLE,
    );
    if (*Ps).hdc.is_null() {
        return core::ptr::null_mut();
    }

    // If set, always clear flags out due to the conditions later on for sending the message.
    if (*Window).state & WNDS_SENDERASEBACKGROUND != 0 {
        (*Window).state &= !(WNDS_SENDERASEBACKGROUND | WNDS_ERASEBACKGROUND);
        Erase = 1;
    }

    if !(*Window).hrgnUpdate.is_null() {
        MsqDecPaintCountQueue(head_pti(Window));
        IntGdiSetRegionOwner((*Window).hrgnUpdate, GDI_OBJ_HMGR_POWNED as _);
        /* The region is part of the dc now and belongs to the process! */
        (*Window).hrgnUpdate = core::ptr::null_mut();
    } else {
        if (*Window).state & WNDS_INTERNALPAINT != 0 {
            MsqDecPaintCountQueue(head_pti(Window));
        }
    }

    type_ = GdiGetClipBox((*Ps).hdc, core::ptr::addr_of_mut!((*Ps).rcPaint) as *mut _);

    IntGetClientRect(Window, core::ptr::addr_of_mut!(Rect) as *mut _);

    // Diagnostic (arm64 black-desktop hunt): the desktop window's erase
    // decision inputs — Erase flag (was WNDS_SENDERASEBACKGROUND set?) and
    // the clip-box type (NULLREGION = empty update region kills the erase).
    if (*Window).fnid == FNID_DESKTOP as u16 as _ {
        extern "C" {
            fn DbgPrint(Format: *const u8, ...) -> u32;
        }
        static BP_LOGS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
        let n = BP_LOGS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        if n < 16 {
            DbgPrint(
                b"[DESKBP] #%u erase=%d cliptype=%d rcPaint=(%d,%d,%d,%d)\n\0".as_ptr(),
                n,
                Erase,
                type_,
                (*Ps).rcPaint.left,
                (*Ps).rcPaint.top,
                (*Ps).rcPaint.right,
                (*Ps).rcPaint.bottom,
            );
        }
    }

    (*Window).state &= !WNDS_INTERNALPAINT;

    if Erase != 0
        && type_ != NULLREGION as i32
        && ((*(*Window).pcls).style & CS_PARENTDC == 0
            || RECTL_bIntersectRect(
                core::ptr::addr_of_mut!(Rect) as *mut _,
                core::ptr::addr_of_mut!(Rect) as *mut _,
                core::ptr::addr_of_mut!((*Ps).rcPaint) as *mut _,
            ) != 0)
    {
        (*Ps).fErase = (co_IntSendMessage(
            UserHMGetHandle(Window) as HWND,
            WM_ERASEBKGND,
            (*Ps).hdc as WPARAM,
            0,
        ) == 0) as u8 as i32;
        if (*Ps).fErase != 0 {
            (*Window).state |= WNDS_SENDERASEBACKGROUND | WNDS_ERASEBACKGROUND;
        }
    } else {
        (*Ps).fErase = 0;
    }

    IntSendChildNCPaint(Window);

    (*Ps).hdc
}
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntEndPaint(Wnd: PWND, Ps: PPAINTSTRUCT) -> BOOL {
    IntEndPaint_impl(Wnd, Ps)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntEndPaint(Wnd: PWND, Ps: PPAINTSTRUCT) -> BOOL {
    IntEndPaint_impl(Wnd, Ps)
}

unsafe fn IntEndPaint_impl(Wnd: PWND, Ps: PPAINTSTRUCT) -> BOOL {
    let hdc: HDC = (*Ps).hdc;

    UserReleaseDC(Wnd, hdc, 1u8 as BOOL);

    if (*Wnd).state2 & WNDS2_ENDPAINTINVALIDATE != 0 {
        ERR!("EP: Another thread invalidated this window\n");
        (*Wnd).state2 &= !WNDS2_ENDPAINTINVALIDATE;
    }

    (*Wnd).state2 &= !(WNDS2_WMPAINTSENT | WNDS2_STARTPAINT);

    co_UserShowCaret(Wnd);

    1u8 as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntFillWindow(
    mut pWndParent: PWND,
    pWnd: PWND,
    hDC: HDC,
    mut hBrush: HBRUSH,
) -> BOOL {
    IntFillWindow_impl(pWndParent, pWnd, hDC, hBrush)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntFillWindow(
    pWndParent: PWND,
    pWnd: PWND,
    hDC: HDC,
    hBrush: HBRUSH,
) -> BOOL {
    IntFillWindow_impl(pWndParent, pWnd, hDC, hBrush)
}

unsafe fn IntFillWindow_impl(
    mut pWndParent: PWND,
    pWnd: PWND,
    hDC: HDC,
    mut hBrush: HBRUSH,
) -> BOOL {
    let mut Rect: RECT = core::mem::zeroed();
    let mut Rect1: RECT = core::mem::zeroed();
    let type_: INT;

    if pWndParent.is_null() {
        pWndParent = pWnd;
    }

    type_ = GdiGetClipBox(hDC, &mut Rect as *mut _ as _ );

    IntGetClientRect(pWnd, &mut Rect1 as *mut _ as _ );

    if type_ != NULLREGION as INT
        && ((*(*pWnd).pcls).style & CS_PARENTDC as ULONG == 0
            || RECTL_bIntersectRect(&mut Rect as *mut _ as _, &mut Rect as *mut _ as _, &mut Rect1 as *mut _ as _) != 0)
    {
        let mut ppt: POINT = core::mem::zeroed();
        let mut x: INT = 0;
        let mut y: INT = 0;

        if UserIsDesktopWindow(pWndParent) == 0 {
            x = (*pWndParent).rcClient.left - (*pWnd).rcClient.left;
            y = (*pWndParent).rcClient.top - (*pWnd).rcClient.top;
        }

        GreSetBrushOrg(hDC, x, y, &mut ppt as *mut _ as _ );

        if (hBrush as usize) < CTLCOLOR_MAX as usize {
            hBrush = GetControlColor(
                pWndParent,
                pWnd,
                hDC,
                HandleToUlong(hBrush as *mut core::ffi::c_void) + WM_CTLCOLORMSGBOX,
            );
        }

        FillRect(hDC, &mut Rect, hBrush);

        GreSetBrushOrg(hDC, ppt.x, ppt.y, core::ptr::null_mut());

        1
    } else {
        0
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserBeginPaint(hWnd: HWND, UnsafePs: *mut PAINTSTRUCT) -> HDC {
    let mut Window: PWND;
    let mut Ps: PAINTSTRUCT = core::mem::zeroed();
    let Status: NTSTATUS;
    let hDC: HDC;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Ret: HDC = core::ptr::null_mut();

    UserEnterExclusive();

    'cleanup: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'cleanup;
        }

        UserRefObjectCo(Window, &mut Ref);

        hDC = IntBeginPaint(Window, &mut Ps);

        Status = MmCopyToCaller(
            UnsafePs as *mut core::ffi::c_void,
            &mut Ps as *mut PAINTSTRUCT as *mut core::ffi::c_void,
            core::mem::size_of::<PAINTSTRUCT>() as usize,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'cleanup;
        }

        Ret = hDC;
    }

    if !Window.is_null() {
        UserDerefObjectCo(Window);
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserEndPaint(hWnd: HWND, pUnsafePs: *const PAINTSTRUCT) -> BOOL {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let Window: PWND;
    let mut Ps: PAINTSTRUCT = core::mem::zeroed();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'cleanup: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'cleanup;
        }

        UserRefObjectCo(Window, &mut Ref);

        Status = SafeProbeForRead(
            pUnsafePs as *mut core::ffi::c_void,
            core::mem::size_of::<PAINTSTRUCT>(),
            1,
        );
        // The probe VALIDATES only; this capture is the access, and a fault lands in
        // Status, which is checked immediately below.
        if NT_SUCCESS(Status)
            && !crate::copy::capture_from_user(
                &mut Ps as *mut PAINTSTRUCT as *mut core::ffi::c_void,
                pUnsafePs as *const core::ffi::c_void,
                core::mem::size_of::<PAINTSTRUCT>(),
            )
        {
            Status = STATUS_ACCESS_VIOLATION;
        }
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        Ret = IntEndPaint(Window, &mut Ps);
    }

    if !Window.is_null() {
        UserDerefObjectCo(Window);
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserFillWindow(
    hWndParent: HWND,
    hWnd: HWND,
    hDC: HDC,
    hBrush: HBRUSH,
) -> BOOL {
    let mut ret: BOOL = 0;
    let pWnd: PWND;
    let mut pWndParent: PWND = core::ptr::null_mut();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterExclusive();

    'exit: {
        if hDC.is_null() {
            break 'exit;
        }

        pWnd = UserGetWindowObject(hWnd);
        if pWnd.is_null() {
            break 'exit;
        }

        if !hWndParent.is_null() {
            pWndParent = UserGetWindowObject(hWndParent);
            if pWndParent.is_null() {
                break 'exit;
            }
        }

        UserRefObjectCo(pWnd, &mut Ref);
        ret = IntFillWindow(pWndParent, pWnd, hDC, hBrush);
        UserDerefObjectCo(pWnd);
    }

    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserFlashWindowEx(pfwi: PFLASHWINFO) -> BOOL {
    let pWnd: PWND;
    let mut finfo: FLASHWINFO = core::mem::zeroed();
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'exit: {
        {
            let __SehStatus = SafeProbeForRead(
                pfwi as *mut _,
                core::mem::size_of::<FLASHWINFO>(),
                1,
            );
            if !NT_SUCCESS(__SehStatus) {
                SetLastNtError(__SehStatus);
                break 'exit;
            }
        }
        // The probe VALIDATES only; this capture is the access, and takes the same
        // exit. Not theoretical: the wine user32:win subtest passes an unmapped
        // pfwi here, and the plain RtlCopyMemory this replaced BUGCHECKed 0x1E.
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(finfo) as *mut core::ffi::c_void,
            pfwi as *const core::ffi::c_void,
            core::mem::size_of::<FLASHWINFO>(),
        ) {
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            break 'exit;
        }

        pWnd = ValidateHwndNoErr(finfo.hwnd);
        if pWnd.is_null()
            || finfo.cbSize != core::mem::size_of::<FLASHWINFO>() as u32
            || (finfo.dwFlags & !(FLASHW_ALL | FLASHW_TIMER | FLASHW_TIMERNOFG)) != 0
        {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'exit;
        }

        Ret = IntFlashWindowEx(pWnd, core::ptr::addr_of_mut!(finfo));
    }

    UserLeave();
    Ret
}

fastcall_fn! {
#[no_mangle]
unsafe fn co_UserGetUpdateRgn(Window: PWND, hRgn: HRGN, bErase: i32) -> i32 {
    let RegionType: i32;
    let Type: u8;
    let mut Rect: RECTL = core::mem::zeroed();

    ASSERT_REFS_CO!(Window);

    if bErase != 0 {
        let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
        UserRefObjectCo(Window, &mut Ref);
        co_IntPaintWindows(Window, RDW_NOCHILDREN, 0);
        UserDerefObjectCo(Window);
    }

    (*Window).state &= !WNDS_UPDATEDIRTY;

    if (*Window).hrgnUpdate == core::ptr::null_mut() {
        NtGdiSetRectRgn(hRgn, 0, 0, 0, 0);
        return NULLREGION as i32;
    }

    Rect = (*Window).rcClient;
    Type = IntIntersectWithParents(Window, &mut Rect);

    if (*Window).hrgnUpdate == HRGN_WINDOW {
        ERR!("GURn: Caller is passing Window Region 1\n");
        if Type == 0 {
            NtGdiSetRectRgn(hRgn, 0, 0, 0, 0);
            return NULLREGION as i32;
        }

        RegionType = SIMPLEREGION as i32;

        if UserIsDesktopWindow(Window) == 0 {
            RECTL_vOffsetRect(
                &mut Rect as *mut _ as _ ,
                -(*Window).rcClient.left,
                -(*Window).rcClient.top,
            );
        }
        GreSetRectRgnIndirect(hRgn, &mut Rect);
    } else {
        let hrgnTemp: HRGN = GreCreateRectRgnIndirect(&mut Rect);

        RegionType = NtGdiCombineRgn(hRgn, hrgnTemp, (*Window).hrgnUpdate, RGN_AND as i32);

        if RegionType == ERROR as i32 || RegionType == NULLREGION as i32 {
            if !hrgnTemp.is_null() {
                GreDeleteObject(hrgnTemp as _ );
            }
            NtGdiSetRectRgn(hRgn, 0, 0, 0, 0);
            return RegionType;
        }

        if UserIsDesktopWindow(Window) == 0 {
            NtGdiOffsetRgn(hRgn, -(*Window).rcClient.left, -(*Window).rcClient.top);
        }
        if !hrgnTemp.is_null() {
            GreDeleteObject(hrgnTemp as _ );
        }
    }
    RegionType
}
}

fastcall_fn! {
#[allow(non_snake_case)]
#[no_mangle]
unsafe fn co_UserGetUpdateRect(Window: PWND, pRect: PRECT, bErase: i32) -> i32 {
    let RegionType: INT;
    let mut Ret: i32 = 1;

    if bErase != 0 {
        let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
        UserRefObjectCo(Window, &mut Ref);
        co_IntPaintWindows(Window, RDW_NOCHILDREN, 0);
        UserDerefObjectCo(Window);
    }

    (*Window).state &= !WNDS_UPDATEDIRTY;

    if (*Window).hrgnUpdate.is_null() {
        (*pRect).left = 0;
        (*pRect).top = 0;
        (*pRect).right = 0;
        (*pRect).bottom = 0;
        Ret = 0;
    } else {
        if (*Window).hrgnUpdate == HRGN_WINDOW {
            *pRect = core::mem::transmute((*Window).rcClient);
            ERR!("GURt: Caller is retrieving Window Region 1\n");
        } else {
            RegionType = IntGdiGetRgnBox((*Window).hrgnUpdate, pRect as _ );

            if RegionType != ERROR as INT && RegionType != NULLREGION as INT {
                RECTL_bIntersectRect(pRect as *mut _ as _, pRect as *mut _ as _, core::ptr::addr_of_mut!((*Window).rcClient));
            }
        }

        if IntIntersectWithParents(Window, pRect as *mut RECTL) != 0 {
            if UserIsDesktopWindow(Window) == 0 {
                RECTL_vOffsetRect(pRect, -(*Window).rcClient.left, -(*Window).rcClient.top);
            }
            if (*(*Window).pcls).style & CS_OWNDC as u32 != 0 {
                let hdc: HDC;
                hdc = UserGetDCEx(Window, core::ptr::null_mut(), DCX_USESTYLE);
                GreDPtoLP(hdc, pRect as LPPOINT as _ , 2);
                UserReleaseDC(Window, hdc, 0);
            }
        } else {
            (*pRect).left = 0;
            (*pRect).top = 0;
            (*pRect).right = 0;
            (*pRect).bottom = 0;
        }
    }
    Ret
}
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetUpdateRgn(hWnd: HWND, hRgn: HRGN, bErase: i32) -> i32 {
    let Window: PWND;
    let mut ret: i32 = ERROR as i32;

    UserEnterExclusive();

    Window = UserGetWindowObject(hWnd);
    if !Window.is_null() {
        ret = co_UserGetUpdateRgn(Window, hRgn, bErase);
    }

    UserLeave();
    ret
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn NtUserGetUpdateRect(hWnd: HWND, UnsafeRect: LPRECT, bErase: i32) -> i32 {
    let Window: PWND;
    let mut Rect: RECTL = core::mem::zeroed();
    let Status: NTSTATUS;
    let mut Ret: i32 = 0;

    UserEnterExclusive();

    'exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'exit;
        }

        Ret = co_UserGetUpdateRect(Window, core::ptr::addr_of_mut!(Rect) as *mut _, bErase);

        if !UnsafeRect.is_null() {
            Status = MmCopyToCaller(
                UnsafeRect as *mut core::ffi::c_void,
                core::ptr::addr_of!(Rect) as *const core::ffi::c_void,
                core::mem::size_of::<RECTL>(),
            );
            if !NT_SUCCESS(Status) {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                Ret = 0;
            }
        }
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRedrawWindow(
    hWnd: HWND,
    lprcUpdate: *const RECT,
    hrgnUpdate: HRGN,
    flags: UINT,
) -> BOOL {
    let mut SafeUpdateRect: RECTL = core::mem::zeroed();
    let Wnd: PWND;
    let mut Ret: BOOL = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let mut RgnUpdate: PREGION = core::ptr::null_mut();

    UserEnterExclusive();

    'exit: {
        Wnd = UserGetWindowObject(if !hWnd.is_null() { hWnd } else { IntGetDesktopWindow() });
        if Wnd.is_null() {
            break 'exit;
        }

        if !lprcUpdate.is_null() {
            Status = SafeProbeForRead(
                lprcUpdate as *mut core::ffi::c_void,
                core::mem::size_of::<RECTL>(),
                1,
            );
            // The probe VALIDATES only; this capture is the access, and a fault lands
            // in Status, which is checked immediately below.
            if NT_SUCCESS(Status)
                && !crate::copy::capture_from_user(
                    &mut SafeUpdateRect as *mut RECTL as *mut core::ffi::c_void,
                    lprcUpdate as *const core::ffi::c_void,
                    core::mem::size_of::<RECTL>(),
                )
            {
                Status = STATUS_ACCESS_VIOLATION;
            }
            if !NT_SUCCESS(Status) {
                EngSetLastError(RtlNtStatusToDosError(Status));
                break 'exit;
            }
        }

        if flags
            & !(RDW_ERASE
                | RDW_FRAME
                | RDW_INTERNALPAINT
                | RDW_INVALIDATE
                | RDW_NOERASE
                | RDW_NOFRAME
                | RDW_NOINTERNALPAINT
                | RDW_VALIDATE
                | RDW_ERASENOW
                | RDW_UPDATENOW
                | RDW_ALLCHILDREN
                | RDW_NOCHILDREN)
            != 0
        {
            EngSetLastError(ERROR_INVALID_FLAGS);
            break 'exit;
        }

        if (hrgnUpdate as usize) > (HRGN_WINDOW as usize) {
            RgnUpdate = REGION_LockRgn(hrgnUpdate);
            if RgnUpdate.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'exit;
            }
            REGION_UnlockRgn(RgnUpdate);
        } else if hrgnUpdate == HRGN_WINDOW {
            ERR!("NTRW: Caller is passing Window Region 1\n");
        }

        UserRefObjectCo(Wnd, &mut Ref);

        Ret = co_UserRedrawWindow(
            Wnd,
            if !lprcUpdate.is_null() {
                &SafeUpdateRect
            } else {
                core::ptr::null()
            },
            RgnUpdate,
            flags,
        ) as BOOL;

        UserDerefObjectCo(Wnd);
    }

    UserLeave();
    Ret
}

unsafe extern "system" fn UserDrawCaptionText(
    pWnd: PWND,
    hDc: HDC,
    Text: PUNICODE_STRING,
    lpRc: *const RECTL,
    uFlags: UINT,
    mut hFont: HFONT,
) -> BOOL {
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let OldTextColor: COLORREF;
    let mut nclm: NONCLIENTMETRICSW = core::mem::zeroed();
    let Status: NTSTATUS;
    let mut bDeleteFont: BOOLEAN = 0u8;
    let mut Size: SIZE = core::mem::zeroed();
    let mut Ret: BOOL = 1;
    let mut fit: ULONG = 0;
    let Length: ULONG;
    let mut r: RECTL = *lpRc;

    nclm.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;
    if UserSystemParametersInfo(
        SPI_GETNONCLIENTMETRICS,
        nclm.cbSize,
        core::ptr::addr_of_mut!(nclm) as *mut core::ffi::c_void,
        0,
    ) == 0
    {
        return 0;
    }

    if hFont.is_null() {
        if uFlags & DC_SMALLCAP != 0 {
            Status = TextIntCreateFontIndirect(
                core::ptr::addr_of_mut!(nclm.lfSmCaptionFont),
                core::ptr::addr_of_mut!(hFont),
            );
        } else {
            Status = TextIntCreateFontIndirect(
                core::ptr::addr_of_mut!(nclm.lfCaptionFont),
                core::ptr::addr_of_mut!(hFont),
            );
        }

        if !NT_SUCCESS(Status) {
            return 0;
        }

        bDeleteFont = 1u8;
    }

    IntGdiSetBkMode(hDc, TRANSPARENT as i32);

    hOldFont = NtGdiSelectFont(hDc, hFont);

    if uFlags & DC_INBUTTON != 0 {
        OldTextColor = IntGdiSetTextColor(hDc, IntGetSysColor(COLOR_BTNTEXT as i32));
    } else {
        OldTextColor = IntGdiSetTextColor(
            hDc,
            IntGetSysColor(if uFlags & DC_ACTIVE != 0 {
                COLOR_CAPTIONTEXT as i32
            } else {
                COLOR_INACTIVECAPTIONTEXT as i32
            }),
        );
    }

    if !pWnd.is_null() && (*pWnd).style & WS_SYSMENU != 0 {
        r.right -= UserGetSystemMetrics((SM_CYCAPTION as i32).try_into().unwrap()) - 1;
        if (*pWnd).style & (WS_MAXIMIZEBOX | WS_MINIMIZEBOX) != 0
            && (*pWnd).ExStyle & WS_EX_TOOLWINDOW == 0
        {
            r.right -= UserGetSystemMetrics((SM_CXSIZE as i32).try_into().unwrap()) + 1;
            r.right -= UserGetSystemMetrics((SM_CXSIZE as i32).try_into().unwrap()) + 1;
        }
    }

    GreGetTextExtentExW(
        hDc,
        (*Text).Buffer,
        ((*Text).Length / core::mem::size_of::<u16>() as u16) as ULONG,
        // C-exact: a negative width (narrow caption after the sysmenu
        // adjustments above) wraps to a huge ULONG MaxExtent — "everything
        // fits" — exactly like the C implicit INT->ULONG conversion.
        // try_into().unwrap() here panicked the kernel on narrow captions.
        (r.right - r.left) as ULONG,
        core::ptr::addr_of_mut!(fit),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Size),
        0,
    );

    Length = if (*Text).Length as ULONG / core::mem::size_of::<u16>() as ULONG == fit {
        fit
    } else {
        fit + 1
    };

    if (*Text).Length as ULONG / core::mem::size_of::<u16>() as ULONG > Length {
        Ret = 0;
    }

    if Ret != 0 {
        UserExtTextOutW(
            hDc,
            (*lpRc).left,
            (*lpRc).top + ((*lpRc).bottom - (*lpRc).top - Size.cy) / 2,
            ETO_CLIPPED,
            lpRc as *mut RECTL,
            (*Text).Buffer,
            // C-exact: implicit ULONG->INT truncation, never a panic.
            Length as INT,
        );
    } else {
        DrawTextW(
            hDc,
            (*Text).Buffer,
            ((*Text).Length / core::mem::size_of::<u16>() as u16) as i32,
            core::ptr::addr_of_mut!(r) as _ ,
            DT_END_ELLIPSIS | DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_LEFT,
        );
    }

    IntGdiSetTextColor(hDc, OldTextColor);

    if !hOldFont.is_null() {
        NtGdiSelectFont(hDc, hOldFont);
    }

    if bDeleteFont != 0 {
        GreDeleteObject(hFont as _ );
    }

    Ret
}

#[no_mangle]
unsafe extern "C" fn UserDrawCaption(
    pWnd: PWND,
    hDc: HDC,
    lpRc: *mut RECTL,
    hFont: HFONT,
    hIcon: HICON,
    Str: PUNICODE_STRING,
    uFlags: UINT,
) -> BOOL {
    let mut Ret: BOOL = 0;
    let hBgBrush: HBRUSH;
    let mut hOldBrush: HBRUSH = core::ptr::null_mut();
    let mut Rect: RECTL = *lpRc;
    let mut HasIcon: BOOL;

    RECTL_vMakeWellOrdered(lpRc);

    if hIcon.is_null() && !pWnd.is_null() {
        HasIcon = ((uFlags & DC_ICON) != 0
            && (uFlags & DC_SMALLCAP) == 0
            && ((*pWnd).style & WS_SYSMENU as i32 as u32) != 0
            && ((*pWnd).ExStyle & WS_EX_TOOLWINDOW as i32 as u32) == 0) as BOOL;
    } else {
        HasIcon = (!hIcon.is_null()) as BOOL;
    }

    'cleanup: {
        if (uFlags & DC_GRADIENT) != 0 && (uFlags & DC_INBUTTON) == 0 {
            static mut gcap: GRADIENT_RECT = GRADIENT_RECT {
                UpperLeft: 0,
                LowerRight: 1,
            };
            let mut Vertices: [TRIVERTEX; 2] = core::mem::zeroed();
            let mut Colors: [COLORREF; 2] = [0; 2];

            Colors[0] = IntGetSysColor(if (uFlags & DC_ACTIVE) != 0 {
                COLOR_ACTIVECAPTION.try_into().unwrap()
            } else {
                COLOR_INACTIVECAPTION.try_into().unwrap()
            });

            Colors[1] = IntGetSysColor(if (uFlags & DC_ACTIVE) != 0 {
                COLOR_GRADIENTACTIVECAPTION.try_into().unwrap()
            } else {
                COLOR_GRADIENTINACTIVECAPTION.try_into().unwrap()
            });

            Vertices[0].x = Rect.left;
            Vertices[0].y = Rect.top;
            Vertices[0].Red = ((Colors[0] as u16) << 8) as u16;
            Vertices[0].Green = ((Colors[0] as u16) & 0xFF00) as u16;
            Vertices[0].Blue = (((Colors[0] >> 8) as u16) & 0xFF00) as u16;
            Vertices[0].Alpha = 0;

            Vertices[1].x = Rect.right;
            Vertices[1].y = Rect.bottom;
            Vertices[1].Red = ((Colors[1] as u16) << 8) as u16;
            Vertices[1].Green = ((Colors[1] as u16) & 0xFF00) as u16;
            Vertices[1].Blue = (((Colors[1] >> 8) as u16) & 0xFF00) as u16;
            Vertices[1].Alpha = 0;

            if GreGradientFill(
                hDc,
                Vertices.as_mut_ptr(),
                2,
                core::ptr::addr_of_mut!(gcap) as *mut _,
                1,
                GRADIENT_FILL_RECT_H,
            ) == 0
            {
                break 'cleanup;
            }
        } else {
            if (uFlags & DC_INBUTTON) != 0 {
                hBgBrush = IntGetSysColorBrush(COLOR_3DFACE.try_into().unwrap()) as _ ;
            } else if (uFlags & DC_ACTIVE) != 0 {
                hBgBrush = IntGetSysColorBrush(COLOR_ACTIVECAPTION.try_into().unwrap()) as _ ;
            } else {
                hBgBrush = IntGetSysColorBrush(COLOR_INACTIVECAPTION.try_into().unwrap()) as _ ;
            }

            hOldBrush = NtGdiSelectBrush(hDc, hBgBrush);

            if hOldBrush.is_null() {
                break 'cleanup;
            }

            if NtGdiPatBlt(
                hDc,
                Rect.left,
                Rect.top,
                Rect.right - Rect.left,
                Rect.bottom - Rect.top,
                (PATCOPY as i32).try_into().unwrap(),
            ) == 0
            {
                break 'cleanup;
            }
        }

        if HasIcon != 0 {
            let mut pIcon: PCURICON_OBJECT = core::ptr::null_mut();

            if !hIcon.is_null() {
                pIcon = UserGetCurIconObject(hIcon);
            } else if !pWnd.is_null() {
                pIcon = NC_IconForWindow(pWnd);
                if !pIcon.is_null() {
                    UserReferenceObject(pIcon as *mut _);
                }
            }

            if !pIcon.is_null() {
                let cx: LONG = UserGetSystemMetrics(SM_CXSMICON);
                let cy: LONG = UserGetSystemMetrics(SM_CYSMICON);
                let x: LONG = Rect.left - cx / 2 + 1 + (Rect.bottom - Rect.top) / 2;
                let y: LONG = (Rect.top + Rect.bottom - cy) / 2;
                UserDrawIconEx(
                    hDc,
                    x,
                    y,
                    pIcon,
                    cx,
                    cy,
                    0,
                    core::ptr::null_mut(),
                    DI_NORMAL,
                );
                UserDereferenceObject(pIcon as *mut _);
            } else {
                HasIcon = 0;
            }
        }

        if HasIcon != 0 {
            Rect.left += Rect.bottom - Rect.top;
        }

        if (uFlags & DC_TEXT) != 0 {
            let mut Set: BOOL = 0;
            Rect.left += 2;

            if !Str.is_null() {
                Set = UserDrawCaptionText(pWnd, hDc, Str, &Rect, uFlags, hFont);
            } else if !pWnd.is_null() {
                let mut ustr: _UNICODE_STRING = core::mem::zeroed();
                ustr.Buffer = (*pWnd).strName.Buffer;
                ustr.Length = core::cmp::min((*pWnd).strName.Length as u32, MAXUSHORT as u32) as USHORT;
                ustr.MaximumLength =
                    core::cmp::min((*pWnd).strName.MaximumLength() as u32, MAXUSHORT as u32) as USHORT;
                Set = UserDrawCaptionText(pWnd, hDc, &mut ustr, &Rect, uFlags, hFont);
            }
            if !pWnd.is_null() {
                if Set != 0 {
                    (*pWnd).state2 &= !WNDS2_CAPTIONTEXTTRUNCATED;
                } else {
                    (*pWnd).state2 |= WNDS2_CAPTIONTEXTTRUNCATED;
                }
            }
        }

        Ret = 1;
    }

    if !hOldBrush.is_null() {
        NtGdiSelectBrush(hDc, hOldBrush);
    }

    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn UserRealizePalette(hdc: HDC) -> INT {
    UserRealizePaletteImpl(hdc)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn UserRealizePalette(hdc: HDC) -> INT {
    UserRealizePaletteImpl(hdc)
}

unsafe fn UserRealizePaletteImpl(mut hdc: HDC) -> INT {
    let hWnd: HWND;
    let hWndDesktop: HWND;
    let Ret: DWORD;

    Ret = IntGdiRealizePalette(hdc) as DWORD;
    if Ret != 0 {
        hWnd = IntWindowFromDC(hdc);
        if !hWnd.is_null() {
            hWndDesktop = IntGetDesktopWindow();
            if hWndDesktop != hWnd {
                let pWnd: PWND = UserGetWindowObject(hWndDesktop);
                hdc = UserGetWindowDC(pWnd);
                IntPaintDesktop(hdc);
                UserReleaseDC(pWnd, hdc, 0u8 as _);
            }
            UserSendNotifyMessage(
                HWND_BROADCAST as HWND,
                WM_PALETTECHANGED,
                hWnd as WPARAM,
                0,
            );
        }
    }
    Ret as INT
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserDrawCaptionTemp(
    hWnd: HWND,
    hDC: HDC,
    lpRc: LPCRECT,
    hFont: HFONT,
    hIcon: HICON,
    str: PUNICODE_STRING,
    uFlags: UINT,
) -> BOOL {
    let mut pWnd: PWND = core::ptr::null_mut();
    let mut SafeStr: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let mut SafeRect: RECTL = core::mem::zeroed();
    let mut Ret: BOOL;
    let mut caption_guard: Option<ntkrnl_sys::probe::PoolAllocGuard> = None;

    UserEnterExclusive();

    if !hWnd.is_null() {
        pWnd = UserGetWindowObject(hWnd);
        if pWnd.is_null() {
            UserLeave();
            return 0;
        }
    }

    // C: _SEH2_TRY {
    //        ProbeForRead(lpRc, sizeof(RECTL), sizeof(ULONG));
    //        RtlCopyMemory(&SafeRect, lpRc, sizeof(RECTL));
    //        if (str != NULL) {
    //            SafeStr = ProbeForReadUnicodeString(str);
    //            if (SafeStr.Length != 0)
    //                ProbeForRead(SafeStr.Buffer, SafeStr.Length, sizeof(WCHAR));
    //        }
    //    } _SEH2_EXCEPT { Status = _SEH2_GetExceptionCode(); }
    // (The earlier translation probed SafeStr.Buffer BEFORE capturing SafeStr,
    // and never range-checked the header read at all.)
    Status = STATUS_SUCCESS as NTSTATUS;
    'seh: {
        if let Err(st) = crate::safe_user_io::copy_from_user(
            core::ptr::addr_of_mut!(SafeRect) as *mut core::ffi::c_void,
            lpRc as *const core::ffi::c_void,
            core::mem::size_of::<RECTL>(),
            core::mem::size_of::<ULONG>() as u32,
        ) {
            Status = st;
            break 'seh;
        }
        if !str.is_null() {
            SafeStr = match crate::safe_user_io::capture_user(str as *const _UNICODE_STRING) {
                Ok(v) => v,
                Err(st) => {
                    Status = st;
                    break 'seh;
                }
            };
            // The C probes the text here and UserDrawCaption then draws it IN
            // PLACE, outside the _SEH2_TRY. Draw a pool copy instead.
            match crate::safe_user_io::capture_string_buffer(
                &mut SafeStr,
                u32::from_le_bytes(*b"Usdc"),
            ) {
                Ok(guard) => caption_guard = guard,
                Err(st) => {
                    Status = st;
                    break 'seh;
                }
            }
        }
    }

    if Status != STATUS_SUCCESS as NTSTATUS {
        SetLastNtError(Status);
        UserLeave();
        return 0;
    }

    'block: {
        if !str.is_null() {
            Ret = UserDrawCaption(
                pWnd,
                hDC,
                core::ptr::addr_of_mut!(SafeRect),
                hFont,
                hIcon,
                core::ptr::addr_of_mut!(SafeStr),
                uFlags,
            );
        } else {
            if RECTL_bIsEmptyRect(core::ptr::addr_of_mut!(SafeRect)) != 0
                && hFont.is_null()
                && hIcon.is_null()
            {
                Ret = 1;
                if uFlags & DC_DRAWCAPTIONMD != 0 {
                    ERR!("NC Caption Mode\n");
                    // C-exact UINT->INT: uFlags is user-controlled (syscall
                    // param) — try_into().unwrap() was a user-triggerable
                    // kernel panic on uFlags >= 0x80000000.
                    UserDrawCaptionBar(pWnd, hDC, uFlags as _);
                    break 'block;
                } else if uFlags & DC_DRAWFRAMEMD != 0 {
                    ERR!("NC Paint Mode\n");
                    NC_DoNCPaint(pWnd, hDC, uFlags as _); // same: user-controlled uFlags
                    break 'block;
                }
            }
            Ret = UserDrawCaption(
                pWnd,
                hDC,
                core::ptr::addr_of_mut!(SafeRect),
                hFont,
                hIcon,
                core::ptr::null_mut(),
                uFlags,
            );
        }
    }

    UserLeave();
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserDrawCaptionTemp(
    hWnd: HWND,
    hDC: HDC,
    lpRc: LPCRECT,
    hFont: HFONT,
    hIcon: HICON,
    str: PUNICODE_STRING,
    uFlags: UINT,
) -> BOOL {
    let mut pWnd: PWND = core::ptr::null_mut();
    let mut SafeStr: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let mut SafeRect: RECTL = core::mem::zeroed();
    let mut Ret: BOOL;
    let mut caption_guard: Option<ntkrnl_sys::probe::PoolAllocGuard> = None;

    UserEnterExclusive();

    if !hWnd.is_null() {
        pWnd = UserGetWindowObject(hWnd);
        if pWnd.is_null() {
            UserLeave();
            return 0;
        }
    }

    // C: _SEH2_TRY {
    //        ProbeForRead(lpRc, sizeof(RECTL), sizeof(ULONG));
    //        RtlCopyMemory(&SafeRect, lpRc, sizeof(RECTL));
    //        if (str != NULL) {
    //            SafeStr = ProbeForReadUnicodeString(str);
    //            if (SafeStr.Length != 0)
    //                ProbeForRead(SafeStr.Buffer, SafeStr.Length, sizeof(WCHAR));
    //        }
    //    } _SEH2_EXCEPT { Status = _SEH2_GetExceptionCode(); }
    // (The earlier translation probed SafeStr.Buffer BEFORE capturing SafeStr,
    // and never range-checked the header read at all.)
    Status = STATUS_SUCCESS as NTSTATUS;
    'seh: {
        if let Err(st) = crate::safe_user_io::copy_from_user(
            core::ptr::addr_of_mut!(SafeRect) as *mut core::ffi::c_void,
            lpRc as *const core::ffi::c_void,
            core::mem::size_of::<RECTL>(),
            core::mem::size_of::<ULONG>() as u32,
        ) {
            Status = st;
            break 'seh;
        }
        if !str.is_null() {
            SafeStr = match crate::safe_user_io::capture_user(str as *const _UNICODE_STRING) {
                Ok(v) => v,
                Err(st) => {
                    Status = st;
                    break 'seh;
                }
            };
            // The C probes the text here and UserDrawCaption then draws it IN
            // PLACE, outside the _SEH2_TRY. Draw a pool copy instead.
            match crate::safe_user_io::capture_string_buffer(
                &mut SafeStr,
                u32::from_le_bytes(*b"Usdc"),
            ) {
                Ok(guard) => caption_guard = guard,
                Err(st) => {
                    Status = st;
                    break 'seh;
                }
            }
        }
    }

    if Status != STATUS_SUCCESS as NTSTATUS {
        SetLastNtError(Status);
        UserLeave();
        return 0;
    }

    'block: {
        if !str.is_null() {
            Ret = UserDrawCaption(
                pWnd,
                hDC,
                core::ptr::addr_of_mut!(SafeRect),
                hFont,
                hIcon,
                core::ptr::addr_of_mut!(SafeStr),
                uFlags,
            );
        } else {
            if RECTL_bIsEmptyRect(core::ptr::addr_of_mut!(SafeRect)) != 0
                && hFont.is_null()
                && hIcon.is_null()
            {
                Ret = 1;
                if uFlags & DC_DRAWCAPTIONMD != 0 {
                    ERR!("NC Caption Mode\n");
                    UserDrawCaptionBar(pWnd, hDC, uFlags as INT);
                    break 'block;
                } else if uFlags & DC_DRAWFRAMEMD != 0 {
                    ERR!("NC Paint Mode\n");
                    NC_DoNCPaint(pWnd, hDC, uFlags as INT);
                    break 'block;
                }
            }
            Ret = UserDrawCaption(
                pWnd,
                hDC,
                core::ptr::addr_of_mut!(SafeRect),
                hFont,
                hIcon,
                core::ptr::null_mut(),
                uFlags,
            );
        }
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDrawCaption(
    hWnd: HWND,
    hDC: HDC,
    lpRc: LPCRECT,
    uFlags: UINT,
) -> BOOL {
    NtUserDrawCaptionTemp(hWnd, hDC, lpRc, 0 as HFONT, 0 as HICON, core::ptr::null_mut(), uFlags)
}

fastcall_fn! {
unsafe fn co_UserExcludeUpdateRgn(hDC: HDC, Window: PWND) -> INT {
    let mut pt: POINT = core::mem::zeroed();
    let mut rc: RECT = core::mem::zeroed();

    if !(*Window).hrgnUpdate.is_null() {
        if (*Window).hrgnUpdate == HRGN_WINDOW {
            return NtGdiIntersectClipRect(hDC, 0, 0, 0, 0);
        } else {
            let mut ret: INT = ERROR.try_into().unwrap();
            let hrgn = NtGdiCreateRectRgn(0, 0, 0, 0);

            if !hrgn.is_null() && GreGetDCPoint(hDC, GdiGetDCOrg, &mut pt as *mut _ as _ ) != 0 {
                if NtGdiGetRandomRgn(hDC, hrgn, CLIPRGN as INT) == NULLREGION as INT {
                    NtGdiOffsetRgn(hrgn, pt.x, pt.y);
                } else {
                    let pm: PMONITOR = UserGetPrimaryMonitor();
                    let hrgnScreen = NtGdiCreateRectRgn(0, 0, 0, 0);
                    NtGdiCombineRgn(hrgnScreen, hrgnScreen, (*pm).hrgnMonitor, RGN_OR as INT);

                    NtGdiCombineRgn(hrgn, hrgnScreen, core::ptr::null_mut(), RGN_COPY as INT);

                    GreDeleteObject(hrgnScreen as _ );
                }

                NtGdiCombineRgn(hrgn, hrgn, (*Window).hrgnUpdate, RGN_DIFF as INT);

                NtGdiOffsetRgn(hrgn, -pt.x, -pt.y);

                ret = NtGdiExtSelectClipRgn(hDC, hrgn, RGN_COPY as INT);

                GreDeleteObject(hrgn as _ );
            }
            return ret;
        }
    } else {
        return GdiGetClipBox(hDC, &mut rc as *mut _ as _ );
    }
}
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserExcludeUpdateRgn(
    hDC: HDC,
    hWnd: HWND,
) -> INT {
    let mut ret: INT = ERROR.try_into().unwrap();
    let pWnd: PWND;

    UserEnterExclusive();

    pWnd = UserGetWindowObject(hWnd);

    if !hDC.is_null() && !pWnd.is_null() {
        ret = co_UserExcludeUpdateRgn(hDC, pWnd);
    }

    UserLeave();
    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserExcludeUpdateRgn(
    hDC: HDC,
    hWnd: HWND,
) -> INT {
    let mut ret: INT = ERROR as INT;
    let pWnd: PWND;

    UserEnterExclusive();

    pWnd = UserGetWindowObject(hWnd);

    if !hDC.is_null() && !pWnd.is_null() {
        ret = co_UserExcludeUpdateRgn(hDC, pWnd);
    }

    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserInvalidateRect(
    hWnd: HWND,
    lpUnsafeRect: *const RECT,
    bErase: BOOL,
) -> BOOL {
    let mut flags = RDW_INVALIDATE | (if bErase != 0 { RDW_ERASE } else { 0 });
    let mut lpUnsafeRect = lpUnsafeRect;
    if hWnd.is_null() {
        flags = RDW_ALLCHILDREN | RDW_INVALIDATE | RDW_FRAME | RDW_ERASE | RDW_ERASENOW;
        lpUnsafeRect = core::ptr::null_mut();
    }
    NtUserRedrawWindow(hWnd, lpUnsafeRect, core::ptr::null_mut(), flags)
}
#[no_mangle]
pub unsafe extern "system" fn NtUserInvalidateRgn(
    hWnd: HWND,
    hRgn: HRGN,
    bErase: BOOL,
) -> BOOL {
    if hWnd.is_null() {
        EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }
    NtUserRedrawWindow(
        hWnd,
        core::ptr::null_mut(),
        hRgn,
        RDW_INVALIDATE | (if bErase != 0 { RDW_ERASE } else { 0 }),
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtUserPrintWindow(
    hwnd: HWND,
    hdcBlt: HDC,
    nFlags: UINT,
) -> BOOL {
    let mut Window: PWND;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'exit: {
        if !hwnd.is_null() {
            Window = UserGetWindowObject(hwnd);
            if Window.is_null()
                || UserIsDesktopWindow(Window) != 0
                || UserIsMessageWindow(Window) != 0
            {
                break 'exit;
            }

            if !Window.is_null() {
                if (nFlags & PW_CLIENTONLY) == nFlags {
                    Ret = IntPrintWindow(Window, hdcBlt, nFlags);
                } else {
                    EngSetLastError(ERROR_INVALID_PARAMETER);
                }
            }
        }
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserValidateRect(
    hWnd: HWND,
    lpRect: *const RECT,
) -> BOOL {
    let mut flags: UINT = RDW_VALIDATE;
    let mut lpRect = lpRect;
    if hWnd.is_null() {
        flags = RDW_ALLCHILDREN | RDW_INVALIDATE | RDW_FRAME | RDW_ERASE | RDW_ERASENOW;
        lpRect = core::ptr::null_mut();
    }
    NtUserRedrawWindow(hWnd, lpRect, core::ptr::null_mut(), flags)
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_bIntersectRect(dst: *mut RECTL, src1: *mut RECTL, src2: *mut RECTL) -> u8;
    fn SetLastNtError(Status: i32);
    fn TextIntCreateFontIndirect(lf: LPLOGFONTW, NewFont: *mut HFONT) -> NTSTATUS;
    fn GetControlColor(pwndParent: PWND, pwnd: PWND, hdc: HDC, CtlMsg: UINT) -> HBRUSH;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_bIntersectRect(dst: *mut RECTL, src1: *mut RECTL, src2: *mut RECTL) -> u8;
    fn SetLastNtError(Status: i32);
    fn TextIntCreateFontIndirect(lf: LPLOGFONTW, NewFont: *mut HFONT) -> NTSTATUS;
    fn GetControlColor(pwndParent: PWND, pwnd: PWND, hdc: HDC, CtlMsg: UINT) -> HBRUSH;
}
