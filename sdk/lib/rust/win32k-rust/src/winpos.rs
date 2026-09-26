use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag, PsGetCurrentThreadWin32Thread};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS};
use core::ffi::c_void;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// LOWORD/HIWORD (minwindef.h).
#[inline(always)]
fn LOWORD(l: DWORD) -> DWORD { l & 0xffff }
#[inline(always)]
fn HIWORD(l: DWORD) -> DWORD { (l >> 16) & 0xffff }

// Pool tags (tags.h), little-endian multichar.
const USERTAG_SWP: u32 = u32::from_le_bytes(*b"Ussw");        // 'wssU'
const USERTAG_WINDOWLIST: u32 = u32::from_le_bytes(*b"Uswl"); // 'lwsU'

// HRGN_WINDOW = (HRGN)1 (ntuser.h).
const HRGN_WINDOW: HRGN = 1usize as HRGN;

// RtlCopyMemory — compiler memcpy intrinsic (no symbol).
#[inline(always)]
unsafe fn RtlCopyMemory(dst: PVOID, src: *const c_void, len: SIZE_T) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len as usize);
}

// RECTL_vSetEmptyRect (gdi inline) — mirrors cliprgn.rs.
#[inline(always)]
unsafe fn RECTL_vSetEmptyRect(prcl: *mut RECTL) {
    (*prcl).left = 0; (*prcl).top = 0; (*prcl).right = 0; (*prcl).bottom = 0;
}

// gpsi PERUSERSERVERINFO tail accessor (ptCursor etc). Mirrors sysparams.rs.
#[inline(always)]
unsafe fn pusi(p: PSERVERINFO) -> *mut _PERUSERSERVERINFO {
    (p as *mut u8).add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// THRDESKHEAD.pti flattened field (offset 2*usize). Mirrors ime.rs/input.rs.
#[inline(always)]
unsafe fn head_pti<T>(obj: *const T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}

// RECTL_bIntersectRect / SafeProbeForRead / VerifyWnd / UpdateShellHook /
// VIS_ComputeVisibleRegion — win32k internal exports (cdecl, fastcall on i386).
extern "system" {
    fn SafeProbeForRead(Address: *mut core::ffi::c_void, Length: SIZE_T, Alignment: u32) -> i32;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
    fn VerifyWnd(pWnd: PWND) -> PWND;
    fn VIS_ComputeVisibleRegion(Window: PWND, ClientArea: BOOLEAN, ClipChildren: BOOLEAN, ClipSiblings: BOOLEAN) -> PREGION;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
    fn VerifyWnd(pWnd: PWND) -> PWND;
    fn VIS_ComputeVisibleRegion(Window: PWND, ClientArea: BOOLEAN, ClipChildren: BOOLEAN, ClipSiblings: BOOLEAN) -> PREGION;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_vInflateRect(rect: *mut RECTL, dx: core::ffi::c_int, dy: core::ffi::c_int);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_vInflateRect(rect: *mut RECTL, dx: core::ffi::c_int, dy: core::ffi::c_int);
}
// RECTL_bPointInRect (rect.h inline).
#[inline(always)]
unsafe fn RECTL_bPointInRect(prcl: *const RECTL, x: INT, y: INT) -> BOOL {
    (x >= (*prcl).left && x < (*prcl).right && y >= (*prcl).top && y < (*prcl).bottom) as BOOL
}
// IntPtInWindow (winpos.h FORCEINLINE).
#[inline(always)]
unsafe fn IntPtInWindow(pwnd: PWND, x: INT, y: INT) -> BOOL {
    if RECTL_bPointInRect(core::ptr::addr_of!((*pwnd).rcWindow) as *const RECTL, x, y) == 0 {
        return FALSE as BOOL;
    }
    if (*pwnd).hrgnClip.is_null() || (*pwnd).style & WS_MINIMIZE != 0 {
        return TRUE as BOOL;
    }
    NtGdiPtInRegion((*pwnd).hrgnClip, x - (*pwnd).rcWindow.left, y - (*pwnd).rcWindow.top) as BOOL
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

// HWND_TERMINATOR = (HWND)UlongToHandle(1) (window.h). Mirrors ime.rs.
const HWND_TERMINATOR: HWND = 1usize as HWND;

// MAKELPARAM(low, high) = (LPARAM)(DWORD)MAKELONG(low, high).
#[inline(always)]
fn MAKELPARAM(low: i32, high: i32) -> LPARAM {
    (((low as u32) & 0xffff) | (((high as u32) & 0xffff) << 16)) as i32 as LPARAM
}

// RtlZeroMemory — compiler memset intrinsic.
#[inline(always)]
unsafe fn RtlZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}

// IS_IMM_MODE() = gpsi && (gpsi->dwSRVIFlags & SRVINFO_IMM32). Mirrors ime.rs.
#[inline(always)]
unsafe fn IS_IMM_MODE() -> BOOL {
    (!gpsi.is_null() && ((*gpsi).dwSRVIFlags & SRVINFO_IMM32) != 0) as BOOL
}

// IntIsDesktopWindow(WndObj) = WndObj->spwndParent == NULL (window.h macro).
#[inline(always)]
unsafe fn IntIsDesktopWindow(wnd: PWND) -> BOOL {
    (!wnd.is_null() && (*wnd).spwndParent.is_null()) as BOOL
}

// GreCreateRectRgnIndirect(prc) = NtGdiCreateRectRgn(left,top,right,bottom) (region.h).
#[inline(always)]
unsafe fn GreCreateRectRgnIndirect(prc: *const RECTL) -> HRGN {
    NtGdiCreateRectRgn((*prc).left, (*prc).top, (*prc).right, (*prc).bottom)
}

// ASSERT_REFS_CO (userfuncs.h macro) — debug-only cLockObj assert; no-op in release.
#[inline(always)]
unsafe fn ASSERT_REFS_CO(_obj: PWND) {}
// IntIsWindowSnapped (winpos.h FORCEINLINE; mirrors nonclient.rs).
#[inline(always)]
unsafe fn IntIsWindowSnapped(Wnd: PWND) -> BOOLEAN {
    (((*Wnd).ExStyle2
        & (WS_EX2_VERTICALLYMAXIMIZEDLEFT | WS_EX2_VERTICALLYMAXIMIZEDRIGHT) as ULONG)
        != 0) as BOOLEAN
}

extern "C" {
    // _MmCopyFromCaller / _MmCopyToCaller — mmcopy.h macros forward to these cdecl fns (copy.rs).
    fn _MmCopyFromCaller(Dest: *mut core::ffi::c_void, Src: *const core::ffi::c_void, Size: UINT) -> NTSTATUS;
    fn _MmCopyToCaller(Dest: *mut core::ffi::c_void, Src: *const core::ffi::c_void, Size: UINT) -> NTSTATUS;
}
// ValidateHwndNoErr — real FASTCALL export (window.rs). i386 fastcall, cdecl else.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ValidateHwndNoErr(hWnd: HWND) -> PWND;
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ValidateHwndNoErr(hWnd: HWND) -> PWND;
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
}
extern "system" {
    fn SafeProbeForWrite(Address: *mut core::ffi::c_void, Length: SIZE_T, Alignment: u32) -> i32;
}

// ── header-inline / macro helpers (no exported symbol; reimplemented locally) ──
#[inline(always)]
unsafe fn RECTL_vOffsetRect(rect: *mut RECTL, cx: core::ffi::c_long, cy: core::ffi::c_long) {
    (*rect).left += cx; (*rect).right += cx; (*rect).top += cy; (*rect).bottom += cy;
}
#[inline(always)]
unsafe fn RECTL_bIsEmptyRect(prcl: *const RECTL) -> u8 {
    ((*prcl).left >= (*prcl).right || (*prcl).top >= (*prcl).bottom) as u8
}
#[inline(always)]
unsafe fn RECTL_vSetRect(rect: *mut RECT, left: LONG, top: LONG, right: LONG, bottom: LONG) {
    (*rect).left = left; (*rect).top = top; (*rect).right = right; (*rect).bottom = bottom;
}
#[inline(always)]
fn HAS_DLGFRAME(style: u32, ex_style: u32) -> u8 {
    ((ex_style & WS_EX_DLGMODALFRAME) != 0
        || ((style & WS_DLGFRAME) != 0 && (style & WS_THICKFRAME) == 0)) as u8
}
#[inline(always)]
fn HAS_THICKFRAME(style: u32, _ex_style: u32) -> u8 {
    ((style & WS_THICKFRAME) != 0 && !((style & (WS_DLGFRAME | WS_BORDER)) == WS_DLGFRAME)) as u8
}
#[inline(always)]
fn HAS_THINFRAME(style: u32, _ex_style: u32) -> u8 {
    ((style & WS_BORDER) != 0 || (style & (WS_CHILD | WS_POPUP)) == 0) as u8
}
// UserIsDesktopWindow / UserIsMessageWindow (winsta.h inlines).
#[inline(always)]
unsafe fn UserIsDesktopWindow(Wnd: PWND) -> u8 {
    (!Wnd.is_null() && (*Wnd).fnid == FNID_DESKTOP) as u8
}
#[inline(always)]
unsafe fn UserIsMessageWindow(Wnd: PWND) -> BOOL {
    (!Wnd.is_null() && (*Wnd).fnid == FNID_MESSAGEWND) as BOOL
}
// UserHMGetHandle(obj) = obj->head.h (object.h FORCEINLINE).
#[inline(always)]
unsafe fn UserHMGetHandle(pwnd: PWND) -> HWND {
    *(pwnd as *const HWND)
}
// UserRefObjectCo / UserDerefObjectCo (object.h static __inline; mirrors focus.rs).
#[inline(always)]
unsafe fn UserRefObjectCo(obj: PWND, UserReferenceEntry: *mut USER_REFERENCE_ENTRY) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*UserReferenceEntry).obj = obj as PVOID;
    UserReferenceObject(obj as PVOID);
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let entry: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*UserReferenceEntry).Entry);
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
// MmCopyFromCaller / MmCopyToCaller (mmcopy.h macros).
#[inline(always)]
unsafe fn MmCopyFromCaller(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize) -> i32 {
    _MmCopyFromCaller(dst, src, len as UINT)
}
#[inline(always)]
unsafe fn MmCopyToCaller(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize) -> i32 {
    _MmCopyToCaller(dst, src, len as UINT)
}

// ---- file-scope preamble (translated) ----
pub const MINMAX_NOSWP: u32 = 0x00010000;

pub const SWP_EX_NOCOPY: u32 = 0x0001;

pub const SWP_EX_PAINTSELF: u32 = 0x0002;

pub const SWP_AGG_NOGEOMETRYCHANGE: u32 = SWP_NOSIZE | SWP_NOCLIENTSIZE | SWP_NOZORDER;

pub const SWP_AGG_NOPOSCHANGE: u32 = SWP_NOSIZE | SWP_NOMOVE | SWP_NOCLIENTSIZE | SWP_NOCLIENTMOVE | SWP_NOZORDER;

pub const SWP_AGG_STATUSFLAGS: u32 = SWP_AGG_NOPOSCHANGE | SWP_FRAMECHANGED | SWP_HIDEWINDOW | SWP_SHOWWINDOW;

pub const SWP_AGG_NOCLIENTCHANGE: u32 = SWP_NOCLIENTSIZE | SWP_NOCLIENTMOVE;

pub const PLACE_MIN: u32 = 0x0001;

pub const PLACE_MAX: u32 = 0x0002;

pub const PLACE_RECT: u32 = 0x0004;

// MAKELONG (minwindef.h macro).
#[inline(always)]
fn MAKELONG(a: u32, b: u32) -> LONG { ((a & 0xffff) | ((b & 0xffff) << 16)) as LONG }

unsafe fn dump_winpos_flags(flags: u32) {
    let dumped_flags: u32 = SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOREDRAW |
                            SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW |
                            SWP_HIDEWINDOW | SWP_NOCOPYBITS | SWP_NOOWNERZORDER |
                            SWP_NOSENDCHANGING | SWP_DEFERERASE | SWP_ASYNCWINDOWPOS |
                            SWP_NOCLIENTSIZE | SWP_NOCLIENTMOVE | SWP_STATECHANGED;
    let _ = flags;
    let _ = dumped_flags;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetClientOrigin(Window: PWND, Point: LPPOINT) -> i32 {
    IntGetClientOrigin_impl(Window, Point)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetClientOrigin(Window: PWND, Point: LPPOINT) -> i32 {
    IntGetClientOrigin_impl(Window, Point)
}

unsafe fn IntGetClientOrigin_impl(mut Window: PWND, Point: LPPOINT) -> i32 {
    Window = if !Window.is_null() { Window } else { UserGetDesktopWindow() };
    if Window.is_null() {
        (*Point).x = 0;
        (*Point).y = 0;
        return 0;
    }
    (*Point).x = (*Window).rcClient.left;
    (*Point).y = (*Window).rcClient.top;

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntGetClientRect(Wnd: PWND, Rect: *mut RECTL) {
    IntGetClientRectImpl(Wnd, Rect)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntGetClientRect(Wnd: PWND, Rect: *mut RECTL) {
    IntGetClientRectImpl(Wnd, Rect)
}

#[inline]
unsafe fn IntGetClientRectImpl(Wnd: PWND, Rect: *mut RECTL) {
    if (*Wnd).style & WS_MINIMIZED != 0 {
        (*Rect).left = 0;
        (*Rect).top = 0;
        (*Rect).right = UserGetSystemMetrics(SM_CXMINIMIZED);
        (*Rect).bottom = UserGetSystemMetrics(SM_CYMINIMIZED);
        return;
    }
    if UserIsDesktopWindow(Wnd) == 0 {
        *Rect = (*Wnd).rcClient;
        RECTL_vOffsetRect(Rect, -(*Wnd).rcClient.left, -(*Wnd).rcClient.top);
    } else {
        (*Rect).left = 0;
        (*Rect).top = 0;
        (*Rect).right = (*Wnd).rcClient.right;
        (*Rect).bottom = (*Wnd).rcClient.bottom;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntGetWindowRect(Wnd: PWND, Rect: *mut RECTL) -> BOOL {
    IntGetWindowRect_impl(Wnd, Rect)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntGetWindowRect(Wnd: PWND, Rect: *mut RECTL) -> BOOL {
    IntGetWindowRect_impl(Wnd, Rect)
}

unsafe fn IntGetWindowRect_impl(Wnd: PWND, Rect: *mut RECTL) -> BOOL {
    if Wnd.is_null() {
        return 0;
    }
    if UserIsDesktopWindow(Wnd) == 0 {
        *Rect = (*Wnd).rcWindow;
    } else {
        (*Rect).left = 0;
        (*Rect).top = 0;
        (*Rect).right = (*Wnd).rcWindow.right;
        (*Rect).bottom = (*Wnd).rcWindow.bottom;
    }
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntMapWindowPoints(FromWnd: PWND, ToWnd: PWND, lpPoints: LPPOINT, cPoints: UINT) -> INT {
    IntMapWindowPoints_impl(FromWnd, ToWnd, lpPoints, cPoints)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntMapWindowPoints(FromWnd: PWND, ToWnd: PWND, lpPoints: LPPOINT, cPoints: UINT) -> INT {
    IntMapWindowPoints_impl(FromWnd, ToWnd, lpPoints, cPoints)
}

unsafe fn IntMapWindowPoints_impl(FromWnd: PWND, ToWnd: PWND, lpPoints: LPPOINT, cPoints: UINT) -> INT {
    let mut mirror_from: BOOL = 0;
    let mut mirror_to: BOOL = 0;
    let mut Delta: POINT = core::mem::zeroed();
    let mut i: UINT;
    let mut Change: core::ffi::c_int = 1;

    Delta.x = 0;
    Delta.y = 0;

    if !FromWnd.is_null() && UserIsDesktopWindow(FromWnd) == 0 {
        if (*FromWnd).ExStyle & WS_EX_LAYOUTRTL != 0 {
            mirror_from = 1;
            Change = -Change;
            Delta.x = -(*FromWnd).rcClient.right;
        } else {
            Delta.x = (*FromWnd).rcClient.left;
        }
        Delta.y = (*FromWnd).rcClient.top;
    }

    if !ToWnd.is_null() && UserIsDesktopWindow(ToWnd) == 0 {
        if (*ToWnd).ExStyle & WS_EX_LAYOUTRTL != 0 {
            mirror_to = 1;
            Change = -Change;
            Delta.x += Change * (*ToWnd).rcClient.right;
        } else {
            Delta.x -= Change * (*ToWnd).rcClient.left;
        }
        Delta.y -= (*ToWnd).rcClient.top;
    }

    i = 0;
    while i != cPoints {
        (*lpPoints.add(i as usize)).x += Delta.x;
        (*lpPoints.add(i as usize)).x *= Change;
        (*lpPoints.add(i as usize)).y += Delta.y;
        i += 1;
    }

    if (mirror_from != 0 || mirror_to != 0) && cPoints == 2 {
        let p0x = (*lpPoints.add(0)).x;
        let p1x = (*lpPoints.add(1)).x;
        let tmp = if p0x < p1x { p0x } else { p1x };
        (*lpPoints.add(1)).x = if p0x > p1x { p0x } else { p1x };
        (*lpPoints.add(0)).x = tmp;
    }

    (((Delta.y as u32 & 0xFFFF) << 16) | (Delta.x as u32 & 0xFFFF)) as INT
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntClientToScreen(Wnd: PWND, lpPoint: LPPOINT) -> u8 {
    IntClientToScreen_impl(Wnd, lpPoint)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntClientToScreen(Wnd: PWND, lpPoint: LPPOINT) -> u8 {
    IntClientToScreen_impl(Wnd, lpPoint)
}

unsafe fn IntClientToScreen_impl(Wnd: PWND, lpPoint: LPPOINT) -> u8 {
    if !Wnd.is_null() && (*Wnd).fnid != FNID_DESKTOP {
        if (*Wnd).ExStyle & WS_EX_LAYOUTRTL != 0 {
            (*lpPoint).x = (*Wnd).rcClient.right - (*lpPoint).x;
        } else {
            (*lpPoint).x += (*Wnd).rcClient.left;
        }
        (*lpPoint).y += (*Wnd).rcClient.top;
    }
    1u8
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntScreenToClient(Wnd: PWND, lpPoint: LPPOINT) -> u8 {
    IntScreenToClientImpl(Wnd, lpPoint)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntScreenToClient(Wnd: PWND, lpPoint: LPPOINT) -> u8 {
    IntScreenToClientImpl(Wnd, lpPoint)
}

#[inline]
unsafe fn IntScreenToClientImpl(Wnd: PWND, lpPoint: LPPOINT) -> u8 {
    if !Wnd.is_null() && (*Wnd).fnid != FNID_DESKTOP {
        if (*Wnd).ExStyle & WS_EX_LAYOUTRTL != 0 {
            (*lpPoint).x = (*Wnd).rcClient.right - (*lpPoint).x;
        } else {
            (*lpPoint).x -= (*Wnd).rcClient.left;
        }
        (*lpPoint).y -= (*Wnd).rcClient.top;
    }
    1u8
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IsChildVisible(mut pWnd: PWND) -> BOOL {
    loop {
        if ((*pWnd).style & (WS_POPUP | WS_CHILD)) != WS_CHILD {
            return TRUE as BOOL;
        }
        pWnd = (*pWnd).spwndParent;
        if pWnd.is_null() {
            return FALSE as BOOL;
        }
        if (*pWnd).style & WS_VISIBLE == 0 {
            break;
        }
    }
    FALSE as BOOL
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IsChildVisible(mut pWnd: PWND) -> BOOL {
    loop {
        if ((*pWnd).style & (WS_POPUP | WS_CHILD)) != WS_CHILD {
            return TRUE as BOOL;
        }
        pWnd = (*pWnd).spwndParent;
        if pWnd.is_null() {
            return FALSE as BOOL;
        }
        if (*pWnd).style & WS_VISIBLE == 0 {
            break;
        }
    }
    FALSE as BOOL
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetLastTopMostWindow() -> PWND {
    IntGetLastTopMostWindow_impl()
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetLastTopMostWindow() -> PWND {
    IntGetLastTopMostWindow_impl()
}

unsafe fn IntGetLastTopMostWindow_impl() -> PWND {
    let mut pWnd: PWND;
    let rpdesk = (*gptiCurrent).rpdesk;

    if !rpdesk.is_null() {
        pWnd = (*(*(*rpdesk).pDeskInfo).spwnd).spwndChild;
        if !pWnd.is_null() && ((*pWnd).ExStyle & WS_EX_TOPMOST) != 0 {
            loop {
                if (*pWnd).spwndNext.is_null() {
                    break;
                }
                if ((*(*pWnd).spwndNext).ExStyle & WS_EX_TOPMOST) == 0 {
                    break;
                }
                pWnd = (*pWnd).spwndNext;
            }
            return pWnd;
        }
    }
    core::ptr::null_mut()
}

unsafe fn SelectWindowRgn(Window: PWND, hRgnClip: HRGN) {
    if !(*Window).hrgnClip.is_null() {
        IntGdiSetRegionOwner((*Window).hrgnClip, GDI_OBJ_HMGR_POWNED);
        GreDeleteObject((*Window).hrgnClip as HGDIOBJ);
        (*Window).hrgnClip = core::ptr::null_mut();
    }

    if (hRgnClip as usize) > 1 {
        IntGdiSetRegionOwner(hRgnClip, GDI_OBJ_HMGR_PUBLIC);

        (*Window).hrgnClip = hRgnClip;
    }
}

fastcall_fn! {
#[no_mangle]
unsafe fn ActivateOtherWindowMin(Wnd: PWND) -> BOOL {
    let mut ActivePrev: BOOL;
    let mut FindTopWnd: BOOL;
    let pWndTopMost: PWND;
    let mut pWndChild: PWND;
    let mut pWndSetActive: PWND;
    let mut pWndTemp: PWND;
    let mut pWndDesk: PWND;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let pti: PTHREADINFO = gptiCurrent;

    ActivePrev = ((*(*pti).MessageQueue).spwndActivePrev != core::ptr::null_mut()) as BOOL;
    FindTopWnd = 1;

    pWndTopMost = IntGetLastTopMostWindow();
    if !pWndTopMost.is_null() {
        pWndChild = (*pWndTopMost).spwndNext;
    } else {
        pWndChild = (*(*Wnd).spwndParent).spwndChild;
    }

    loop {
        if ActivePrev != 0 {
            pWndSetActive = (*(*pti).MessageQueue).spwndActivePrev;
        } else {
            pWndSetActive = pWndChild;
        }

        pWndTemp = core::ptr::null_mut();

        while !pWndSetActive.is_null() {
            if !VerifyWnd(pWndSetActive).is_null()
                && ((*pWndSetActive).ExStyle & WS_EX_NOACTIVATE) == 0
                && ((*pWndSetActive).style & (WS_VISIBLE | WS_DISABLED)) == WS_VISIBLE
                && ((*pWndSetActive).style & WS_ICONIC) == 0
            {
                if ((*pWndSetActive).ExStyle & WS_EX_TOOLWINDOW) == 0 {
                    UserRefObjectCo(pWndSetActive, &mut Ref);
                    co_IntSetForegroundWindow(pWndSetActive);
                    UserDerefObjectCo(pWndSetActive);
                    return 1;
                }
                if pWndTemp.is_null() {
                    pWndTemp = pWndSetActive;
                }
            }
            if ActivePrev != 0 {
                ActivePrev = 0;
                pWndSetActive = pWndChild;
            } else {
                pWndSetActive = (*pWndSetActive).spwndNext;
            }
        }

        if FindTopWnd == 0 {
            break;
        }
        FindTopWnd = 0;

        if !pWndChild.is_null() {
            pWndChild = (*(*pWndChild).spwndParent).spwndChild;
            continue;
        }

        pWndDesk = IntGetThreadDesktopWindow(pti);
        if pWndDesk.is_null() {
            pWndChild = core::ptr::null_mut();
            continue;
        }
        pWndChild = (*pWndDesk).spwndChild;
    }

    pWndSetActive = pWndTemp;
    if !pWndSetActive.is_null() {
        UserRefObjectCo(pWndSetActive, &mut Ref);
        co_IntSetForegroundWindow(pWndSetActive);
        UserDerefObjectCo(pWndSetActive);
        return 1;
    }
    0
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn can_activate_window(Wnd: PWND) -> u8 {
    can_activate_window_impl(Wnd)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn can_activate_window(Wnd: PWND) -> u8 {
    can_activate_window_impl(Wnd)
}

unsafe fn can_activate_window_impl(Wnd: PWND) -> u8 {
    if Wnd.is_null() {
        return 0u8;
    }

    let style = (*Wnd).style;
    if (style & WS_VISIBLE) == 0 {
        return 0u8;
    }
    if (style & WS_MINIMIZE) != 0 {
        return 0u8;
    }
    if (style & (WS_POPUP | WS_CHILD)) == WS_CHILD {
        return 0u8;
    }
    if ((*Wnd).ExStyle & WS_EX_NOACTIVATE) != 0 {
        return 0u8;
    }
    1u8
}

fastcall_fn! {
#[no_mangle]
unsafe fn co_WinPosActivateOtherWindow(Wnd: PWND) {
    let mut WndTo: PWND = core::ptr::null_mut();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    if IntIsDesktopWindow(Wnd) != 0 {
        IntSetFocusMessageQueue(core::ptr::null_mut());
        return;
    }

    'done: {
        /* If this is popup window, try to activate the owner first. */
        if ((*Wnd).style & WS_POPUP) != 0 {
            WndTo = (*Wnd).spwndOwner;
            if !WndTo.is_null() {
                TRACE!(b"WPAOW Popup with Owner\n\0".as_ptr() as *const i8);
                WndTo = UserGetAncestor(WndTo, GA_ROOT);
                if can_activate_window(WndTo) != 0 {
                    break 'done;
                }
            }
        }

        /* Pick a next top-level window. */
        WndTo = Wnd;
        loop {
            WndTo = (*WndTo).spwndNext;
            if WndTo.is_null() {
                break;
            }
            if can_activate_window(WndTo) != 0 {
                break 'done;
            }
        }

        if !Wnd.is_null() {
            WndTo = (*(*head_pti(Wnd)).MessageQueue).spwndActivePrev;
            if can_activate_window(WndTo) != 0 {
                break 'done;
            }
        }

        WndTo = UserGetDesktopWindow();
        if WndTo.is_null() || (*WndTo).spwndChild.is_null() {
            return;
        }
        WndTo = (*WndTo).spwndChild;
        loop {
            if WndTo == Wnd {
                WndTo = core::ptr::null_mut();
                break;
            }
            if can_activate_window(WndTo) != 0 {
                break 'done;
            }
            WndTo = (*WndTo).spwndNext;
            if WndTo.is_null() {
                break;
            }
        }
    }

    if !WndTo.is_null() {
        UserRefObjectCo(WndTo, &mut Ref);
    }

    if !gpqForeground.is_null()
        && ((*gpqForeground).spwndActive.is_null() || Wnd == (*gpqForeground).spwndActive)
    {
        if co_IntSetForegroundWindow(WndTo) != 0 {
            if !WndTo.is_null() {
                UserDerefObjectCo(WndTo);
            }
            return;
        }
    }

    if UserSetActiveWindow(WndTo) == 0 {
        UserSetActiveWindow(core::ptr::null_mut());
    }
    if !WndTo.is_null() {
        UserDerefObjectCo(WndTo);
    }
}
}

#[cfg(target_pointer_width = "32")]
unsafe extern "fastcall" fn WinPosInitInternalPos(Wnd: PWND, RestoreRect: *mut RECTL) {
    WinPosInitInternalPos_impl(Wnd, RestoreRect)
}

#[cfg(not(target_pointer_width = "32"))]
unsafe extern "C" fn WinPosInitInternalPos(Wnd: PWND, RestoreRect: *mut RECTL) {
    WinPosInitInternalPos_impl(Wnd, RestoreRect)
}

unsafe fn WinPosInitInternalPos_impl(Wnd: PWND, RestoreRect: *mut RECTL) {
    let mut Size: POINT = core::mem::zeroed();
    let mut Rect: RECTL = *RestoreRect;

    if !(*Wnd).spwndParent.is_null() && UserIsDesktopWindow((*Wnd).spwndParent) == 0 {
        RECTL_vOffsetRect(
            core::ptr::addr_of_mut!(Rect),
            -(*(*Wnd).spwndParent).rcClient.left,
            -(*(*Wnd).spwndParent).rcClient.top,
        );
    }

    Size.x = Rect.left;
    Size.y = Rect.top;

    if (*Wnd).InternalPosInitialized() == 0 {
        (*Wnd).InternalPos.flags = 0;
        (*Wnd).InternalPos.MaxPos.x = -1;
        (*Wnd).InternalPos.MaxPos.y = -1;
        (*Wnd).InternalPos.IconPos.x = -1;
        (*Wnd).InternalPos.IconPos.y = -1;
        (*Wnd).InternalPos.NormalRect = Rect;
        (*Wnd).set_InternalPosInitialized(1);
    }

    if (*Wnd).style & WS_MINIMIZE != 0 {
        (*Wnd).InternalPos.IconPos = core::mem::transmute_copy(&Size);
        (*Wnd).InternalPos.flags |= WPF_MININIT as u32;
    } else if (*Wnd).style & WS_MAXIMIZE != 0 {
        (*Wnd).InternalPos.flags |= WPF_MAXINIT as u32;

        if (*Wnd).spwndParent == (*(*(*Wnd).head.rpdesk).pDeskInfo).spwnd {
            if (*Wnd).state & WNDS_MAXIMIZESTOMONITOR != 0 {
                (*Wnd).InternalPos.flags &= !(WPF_MAXINIT as u32);
                (*Wnd).InternalPos.MaxPos.x = -1;
                (*Wnd).InternalPos.MaxPos.y = -1;
            } else {
                let pmonitor: PMONITOR = UserMonitorFromRect(
                    core::ptr::addr_of_mut!(Rect) as PRECTL,
                    MONITOR_DEFAULTTOPRIMARY,
                );
                let mut WorkArea: RECTL = (*pmonitor).rcMonitor;

                if (*Wnd).style & WS_MAXIMIZEBOX != 0 {
                    if ((*Wnd).style & WS_CAPTION) == WS_CAPTION
                        || ((*Wnd).style & (WS_CHILD | WS_POPUP)) == 0
                    {
                        WorkArea = (*pmonitor).rcWork;
                    }
                }

                (*Wnd).InternalPos.MaxPos.x = Rect.left - WorkArea.left;
                (*Wnd).InternalPos.MaxPos.y = Rect.top - WorkArea.top;
            }
        } else {
            (*Wnd).InternalPos.MaxPos = core::mem::transmute_copy(&Size);
        }
    } else {
        if IntIsWindowSnapped(Wnd) == 0
            || RECTL_bIsEmptyRect(core::ptr::addr_of!((*Wnd).InternalPos.NormalRect)) != 0
        {
            (*Wnd).InternalPos.NormalRect = Rect;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn IntGetWindowPlacement(Wnd: PWND, lpwndpl: *mut WINDOWPLACEMENT) -> u8 {
    if Wnd.is_null() {
        return 0;
    }

    if (*lpwndpl).length as usize != core::mem::size_of::<WINDOWPLACEMENT>() {
        return 0;
    }

    (*lpwndpl).flags = 0;

    WinPosInitInternalPos(Wnd, core::ptr::addr_of_mut!((*Wnd).rcWindow));

    (*lpwndpl).showCmd = SW_HIDE;

    if (*Wnd).style & WS_MINIMIZE != 0 {
        (*lpwndpl).showCmd = SW_SHOWMINIMIZED;
    } else {
        (*lpwndpl).showCmd = if (*Wnd).style & WS_MAXIMIZE != 0 {
            SW_SHOWMAXIMIZED
        } else {
            SW_SHOWNORMAL
        };
    }

    (*lpwndpl).rcNormalPosition = (*Wnd).InternalPos.NormalRect;

    if (*Wnd).InternalPos.flags & WPF_MININIT != 0 {
        (*lpwndpl).ptMinPosition.x = (*Wnd).InternalPos.IconPos.x;
        (*lpwndpl).ptMinPosition.y = (*Wnd).InternalPos.IconPos.y;
    } else {
        (*lpwndpl).ptMinPosition.x = -1;
        (*lpwndpl).ptMinPosition.y = -1;
    }

    if (*Wnd).InternalPos.flags & WPF_MAXINIT != 0
        && (*Wnd).state & WNDS_MAXIMIZESTOMONITOR == 0
    {
        (*lpwndpl).ptMaxPosition.x = (*Wnd).InternalPos.MaxPos.x;
        (*lpwndpl).ptMaxPosition.y = (*Wnd).InternalPos.MaxPos.y;
    } else {
        (*lpwndpl).ptMaxPosition.x = -1;
        (*lpwndpl).ptMaxPosition.y = -1;
    }

    if (*Wnd).spwndParent == (*(*(*Wnd).head.rpdesk).pDeskInfo).spwnd
        && (*Wnd).ExStyle & WS_EX_TOOLWINDOW == 0
    {
        let pmonitor: PMONITOR = UserMonitorFromRect(
            core::ptr::addr_of_mut!((*lpwndpl).rcNormalPosition) as PRECTL,
            MONITOR_DEFAULTTOPRIMARY,
        );

        if (*Wnd).InternalPos.flags & WPF_MININIT != 0 {
            (*lpwndpl).ptMinPosition.x -=
                (*pmonitor).rcWork.left - (*pmonitor).rcMonitor.left;
            (*lpwndpl).ptMinPosition.y -=
                (*pmonitor).rcWork.top - (*pmonitor).rcMonitor.top;
        }
        RECTL_vOffsetRect(
            core::ptr::addr_of_mut!((*lpwndpl).rcNormalPosition) as PRECTL,
            (*pmonitor).rcMonitor.left - (*pmonitor).rcWork.left,
            (*pmonitor).rcMonitor.top - (*pmonitor).rcWork.top,
        );
    }

    if (*Wnd).InternalPos.flags & WPF_RESTORETOMAXIMIZED != 0
        || (*Wnd).style & WS_MAXIMIZE != 0
    {
        (*lpwndpl).flags |= WPF_RESTORETOMAXIMIZED as u32;
    }

    if ((*Wnd).style & (WS_CHILD | WS_POPUP)) == WS_CHILD
        && (*Wnd).InternalPos.flags & WPF_SETMINPOSITION != 0
    {
        (*lpwndpl).flags |= WPF_SETMINPOSITION as u32;
    }

    1
}

unsafe fn make_rect_onscreen(rect: *mut RECT) {
    let pmonitor: PMONITOR = UserMonitorFromRect(rect as PRECTL, MONITOR_DEFAULTTONEAREST);

    if pmonitor.is_null() {
        return;
    }

    if (*rect).right <= (*pmonitor).rcWork.left {
        (*rect).right += (*pmonitor).rcWork.left - (*rect).left;
        (*rect).left = (*pmonitor).rcWork.left;
    } else if (*rect).left >= (*pmonitor).rcWork.right {
        (*rect).left += (*pmonitor).rcWork.right - (*rect).right;
        (*rect).right = (*pmonitor).rcWork.right;
    }
    if (*rect).bottom <= (*pmonitor).rcWork.top {
        (*rect).bottom += (*pmonitor).rcWork.top - (*rect).top;
        (*rect).top = (*pmonitor).rcWork.top;
    } else if (*rect).top >= (*pmonitor).rcWork.bottom {
        (*rect).top += (*pmonitor).rcWork.bottom - (*rect).bottom;
        (*rect).bottom = (*pmonitor).rcWork.bottom;
    }
}

unsafe fn make_point_onscreen(pt: *mut POINT) {
    let mut rect: RECT = core::mem::zeroed();

    RECTL_vSetRect(
        core::ptr::addr_of_mut!(rect),
        (*pt).x,
        (*pt).y,
        (*pt).x + 1,
        (*pt).y + 1,
    );
    make_rect_onscreen(core::ptr::addr_of_mut!(rect));
    (*pt).x = rect.left;
    (*pt).y = rect.top;
}


fastcall_fn! {
unsafe fn IntSetWindowPlacement(Wnd: PWND, wpl: *mut WINDOWPLACEMENT, Flags: UINT) -> BOOL {
    let sAsync: BOOL;
    let SWP_Flags: UINT;

    if (Flags & PLACE_MIN) != 0 { make_point_onscreen(core::ptr::addr_of_mut!((*wpl).ptMinPosition).cast()); }
    if (Flags & PLACE_MAX) != 0 { make_point_onscreen(core::ptr::addr_of_mut!((*wpl).ptMaxPosition).cast()); }
    if (Flags & PLACE_RECT) != 0 { make_rect_onscreen(core::ptr::addr_of_mut!((*wpl).rcNormalPosition).cast()); }

    if Wnd.is_null() || Wnd == (*(*(*Wnd).head.rpdesk).pDeskInfo).spwnd {
        return 0;
    }

    if (Flags & PLACE_MIN) != 0 { (*Wnd).InternalPos.IconPos = (*wpl).ptMinPosition; }
    if (Flags & PLACE_MAX) != 0 { (*Wnd).InternalPos.MaxPos = (*wpl).ptMaxPosition; }
    if (Flags & PLACE_RECT) != 0 { (*Wnd).InternalPos.NormalRect = (*wpl).rcNormalPosition; }

    SWP_Flags = SWP_NOZORDER | SWP_NOACTIVATE | (if ((*wpl).flags & WPF_ASYNCWINDOWPLACEMENT) != 0 { SWP_ASYNCWINDOWPOS } else { 0 });

    if ((*Wnd).style & WS_MINIMIZE) != 0 {
        if (Flags & PLACE_MIN) != 0 || ((*Wnd).InternalPos.flags & WPF_SETMINPOSITION) != 0 {
            co_WinPosSetWindowPos(Wnd, HWND_TOP,
                                  (*wpl).ptMinPosition.x, (*wpl).ptMinPosition.y, 0, 0,
                                  SWP_NOSIZE | SWP_Flags);
            (*Wnd).InternalPos.flags |= WPF_MININIT;
        }
    } else if ((*Wnd).style & WS_MAXIMIZE) != 0 {
        if (Flags & PLACE_MAX) != 0 {
            co_WinPosSetWindowPos(Wnd, HWND_TOP,
                                  (*wpl).ptMaxPosition.x, (*wpl).ptMaxPosition.y, 0, 0,
                                  SWP_NOSIZE | SWP_Flags);
            (*Wnd).InternalPos.flags |= WPF_MAXINIT;
        }
    } else if (Flags & PLACE_RECT) != 0 {
        co_WinPosSetWindowPos(Wnd, HWND_TOP,
                              (*wpl).rcNormalPosition.left, (*wpl).rcNormalPosition.top,
                              (*wpl).rcNormalPosition.right - (*wpl).rcNormalPosition.left,
                              (*wpl).rcNormalPosition.bottom - (*wpl).rcNormalPosition.top,
                              SWP_Flags);
    }

    sAsync = if (*head_pti(Wnd)).MessageQueue != (*gptiCurrent).MessageQueue && ((*wpl).flags & WPF_ASYNCWINDOWPLACEMENT) != 0 { 1 } else { 0 };

    if sAsync != 0 {
        co_IntSendMessageNoWait(UserHMGetHandle(Wnd), WM_ASYNC_SHOWWINDOW, (*wpl).showCmd as WPARAM, 0);
    } else {
        co_WinPosShowWindow(Wnd, (*wpl).showCmd as INT);
    }

    if ((*Wnd).style & WS_MINIMIZE) != 0 && sAsync == 0 {
        if ((*wpl).flags & WPF_SETMINPOSITION) != 0 {
            (*Wnd).InternalPos.flags |= WPF_SETMINPOSITION;
        }
        if ((*wpl).flags & WPF_RESTORETOMAXIMIZED) != 0 {
            (*Wnd).InternalPos.flags |= WPF_RESTORETOMAXIMIZED;
        }
    }
    1
}
}

fastcall_fn! {
#[no_mangle]
unsafe fn co_WinPosArrangeIconicWindows(parent: PWND) -> UINT {
    let mut rectParent: RECTL = core::mem::zeroed();
    let mut Child: PWND;
    let mut x: INT;
    let mut y: INT;
    let xspacing: INT;
    let yspacing: INT;
    let mut sx: INT;
    let mut sy: INT;

    IntGetClientRect(parent, &mut rectParent);

    x = rectParent.left;
    y = rectParent.bottom;

    xspacing = UserGetSystemMetrics(SM_CXMINIMIZED as ULONG) as INT;
    yspacing = UserGetSystemMetrics(SM_CYMINIMIZED as ULONG) as INT;

    Child = (*parent).spwndChild;
    while !Child.is_null() {
        if ((*Child).style & WS_MINIMIZE) != 0 {
            let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
            UserRefObjectCo(Child, &mut Ref);

            sx = x + UserGetSystemMetrics(SM_CXBORDER as ULONG) as INT;
            sy = y - yspacing - UserGetSystemMetrics(SM_CYBORDER as ULONG) as INT;

            (*Child).InternalPos.IconPos.x = sx;
            (*Child).InternalPos.IconPos.y = sy;
            (*Child).InternalPos.flags |= WPF_MININIT;

            co_WinPosSetWindowPos(
                Child,
                0 as HWND,
                sx,
                sy,
                xspacing,
                yspacing,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_ASYNCWINDOWPOS,
            );

            UserDerefObjectCo(Child);

            if x <= rectParent.right - xspacing {
                x += xspacing;
            } else {
                x = rectParent.left;
                y -= yspacing;
            }
        }
        Child = (*Child).spwndNext;
    }
    yspacing as UINT
}
}

fastcall_fn! {
unsafe fn WinPosFindIconPos(Window: PWND, pos2: *mut POINT) {
    let mut rectParent: RECT = core::mem::zeroed();
    let mut pwndChild: PWND;
    let pwndParent: PWND;
    let mut x: i32;
    let mut y: i32;
    let xspacing: i32;
    let yspacing: i32;

    pwndParent = (*Window).spwndParent;
    if UserIsDesktopWindow(pwndParent) != 0 {
        (*pos2).x = -32000;
        (*pos2).y = -32000;
        (*Window).InternalPos.flags |= WPF_MININIT;
        (*Window).InternalPos.IconPos.x = (*pos2).x;
        (*Window).InternalPos.IconPos.y = (*pos2).y;
        return;
    }

    IntGetClientRect(pwndParent, core::ptr::addr_of_mut!(rectParent) as *mut RECTL);

    x = rectParent.left;
    y = rectParent.bottom;

    xspacing = UserGetSystemMetrics(SM_CXMINIMIZED);
    yspacing = UserGetSystemMetrics(SM_CYMINIMIZED);

    (*pos2).x = x + UserGetSystemMetrics(SM_CXBORDER);
    (*pos2).y = y - yspacing - UserGetSystemMetrics(SM_CYBORDER);

    pwndChild = (*pwndParent).spwndChild;
    while !pwndChild.is_null() {
        if pwndChild == Window {
            pwndChild = (*pwndChild).spwndNext;
            continue;
        }

        if ((*pwndChild).style & (WS_VISIBLE | WS_MINIMIZE)) != (WS_VISIBLE | WS_MINIMIZE) {
            pwndChild = (*pwndChild).spwndNext;
            continue;
        }

        if (*pwndChild).InternalPos.IconPos.x != (*pos2).x
            && (*pwndChild).InternalPos.IconPos.y != (*pos2).y
        {
            break;
        }
        if x <= rectParent.right - xspacing {
            x += xspacing;
        } else {
            x = rectParent.left;
            y -= yspacing;
        }
        (*pos2).x = x + UserGetSystemMetrics(SM_CXBORDER);
        (*pos2).y = y - yspacing - UserGetSystemMetrics(SM_CYBORDER);

        pwndChild = (*pwndChild).spwndNext;
    }

    (*Window).InternalPos.IconPos.x = (*pos2).x;
    (*Window).InternalPos.IconPos.y = (*pos2).y;
    (*Window).InternalPos.flags |= WPF_MININIT;
}
}

#[no_mangle]
unsafe extern "C" fn UserHasWindowEdge(mut Style: DWORD, ExStyle: DWORD) -> BOOL {
    if Style & WS_MINIMIZE != 0 {
        return 1;
    }
    if ExStyle & WS_EX_DLGMODALFRAME != 0 {
        return 1;
    }
    if ExStyle & WS_EX_STATICEDGE != 0 {
        return 0;
    }
    if Style & WS_THICKFRAME != 0 {
        return 1;
    }
    Style &= WS_CAPTION;
    if Style == WS_DLGFRAME || Style == WS_CAPTION {
        return 1;
    }
    0
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn IntGetWindowBorderMeasures(Wnd: PWND, cx: *mut UINT, cy: *mut UINT) {
    IntGetWindowBorderMeasures_impl(Wnd, cx, cy)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe extern "fastcall" fn IntGetWindowBorderMeasures(Wnd: PWND, cx: *mut UINT, cy: *mut UINT) {
    IntGetWindowBorderMeasures_impl(Wnd, cx, cy)
}

unsafe fn IntGetWindowBorderMeasures_impl(Wnd: PWND, cx: *mut UINT, cy: *mut UINT) {

    if HAS_DLGFRAME((*Wnd).style, (*Wnd).ExStyle) != 0 && ((*Wnd).style & WS_MINIMIZE) == 0 {
        *cx = UserGetSystemMetrics(SM_CXDLGFRAME) as UINT;
        *cy = UserGetSystemMetrics(SM_CYDLGFRAME) as UINT;
    } else if HAS_THICKFRAME((*Wnd).style, (*Wnd).ExStyle) != 0 && ((*Wnd).style & WS_MINIMIZE) == 0 {
        *cx = UserGetSystemMetrics(SM_CXFRAME) as UINT;
        *cy = UserGetSystemMetrics(SM_CYFRAME) as UINT;
    } else if HAS_THINFRAME((*Wnd).style, (*Wnd).ExStyle) != 0 {
        *cx = UserGetSystemMetrics(SM_CXBORDER) as UINT;
        *cy = UserGetSystemMetrics(SM_CYBORDER) as UINT;
    } else {
        *cx = 0;
        *cy = 0;
    }
}

#[no_mangle]
unsafe extern "C" fn UserGetWindowBorders(Style: DWORD, ExStyle: DWORD, Size: *mut SIZE, WithClient: BOOL) {
    let mut Border: DWORD = 0;

    if UserHasWindowEdge(Style, ExStyle) != 0 {
        Border += 2;
    } else if (ExStyle & (WS_EX_STATICEDGE | WS_EX_DLGMODALFRAME)) == WS_EX_STATICEDGE {
        Border += 1;
    }
    if (ExStyle & WS_EX_CLIENTEDGE) != 0 && WithClient != 0 {
        Border += 2;
    }
    if (Style & WS_CAPTION) != 0 || (ExStyle & WS_EX_DLGMODALFRAME) != 0 {
        Border += 1;
    }
    (*Size).cx = Border as _;
    (*Size).cy = Border as _;
    if (Style & WS_THICKFRAME) != 0 && (Style & WS_MINIMIZE) == 0 {
        (*Size).cx += UserGetSystemMetrics(SM_CXFRAME) - UserGetSystemMetrics(SM_CXDLGFRAME);
        (*Size).cy += UserGetSystemMetrics(SM_CYFRAME) - UserGetSystemMetrics(SM_CYDLGFRAME);
    }
    (*Size).cx *= UserGetSystemMetrics(SM_CXBORDER);
    (*Size).cy *= UserGetSystemMetrics(SM_CYBORDER);
}

unsafe fn IntGetWindowBorders(Style: DWORD, ExStyle: DWORD) -> DWORD {
    let mut adjust: DWORD = 0;

    if ExStyle & WS_EX_WINDOWEDGE != 0 {
        adjust = 2;
    } else if ExStyle & WS_EX_STATICEDGE != 0 {
        adjust = 1;
    }

    if ExStyle & WS_EX_CLIENTEDGE != 0 {
        adjust += 2;
    }

    if Style & WS_CAPTION != 0 || ExStyle & WS_EX_DLGMODALFRAME != 0 {
        adjust += 1;
    }

    adjust
}

fastcall_fn! {
#[no_mangle]
unsafe fn co_WinPosGetMinMaxInfo(
    Window: PWND,
    MaxSize: *mut POINT,
    MaxPos: *mut POINT,
    MinTrack: *mut POINT,
    MaxTrack: *mut POINT,
) -> UINT {
    let mut MinMax: MINMAXINFO = core::mem::zeroed();
    let monitor: PMONITOR;
    let mut xinc: INT;
    let mut yinc: INT;
    let style: LONG = (*Window).style as LONG;
    let adjustedStyle: LONG;
    let exstyle: LONG = (*Window).ExStyle as LONG;
    let mut rc: RECT = core::mem::zeroed();
    let mut adjust: DWORD;

    rc = core::mem::transmute_copy(&(*Window).rcWindow);
    MinMax.ptReserved.x = rc.left;
    MinMax.ptReserved.y = rc.top;

    if (style & WS_CAPTION as LONG) == WS_CAPTION as LONG {
        adjustedStyle = style & !(WS_BORDER as LONG);
    } else {
        adjustedStyle = style;
    }

    if !(*Window).spwndParent.is_null() {
        IntGetClientRect((*Window).spwndParent, &mut rc as *mut RECT as *mut RECTL);
    }

    adjust = IntGetWindowBorders(adjustedStyle as DWORD, exstyle as DWORD);

    if (adjustedStyle & WS_THICKFRAME as LONG) != 0
        && (adjustedStyle & WS_CHILD as LONG) == 0
        && (adjustedStyle & WS_MINIMIZE as LONG) == 0
    {
        adjust += 1;
    }

    xinc = adjust as INT;
    yinc = adjust as INT;

    if (adjustedStyle & WS_THICKFRAME as LONG) != 0
        && (adjustedStyle & WS_CHILD as LONG) != 0
        && (adjustedStyle & WS_MINIMIZE as LONG) == 0
    {
        xinc += UserGetSystemMetrics(SM_CXFRAME as ULONG) - UserGetSystemMetrics(SM_CXDLGFRAME as ULONG);
        yinc += UserGetSystemMetrics(SM_CYFRAME as ULONG) - UserGetSystemMetrics(SM_CYDLGFRAME as ULONG);
    }

    RECTL_vInflateRect(
        (&mut rc as *mut RECT).cast(),
        xinc * UserGetSystemMetrics(SM_CXBORDER as ULONG),
        yinc * UserGetSystemMetrics(SM_CYBORDER as ULONG),
    );

    xinc = -rc.left;
    yinc = -rc.top;

    MinMax.ptMaxSize.x = rc.right - rc.left;
    MinMax.ptMaxSize.y = rc.bottom - rc.top;
    if (style & (WS_DLGFRAME | WS_BORDER) as LONG) != 0 {
        MinMax.ptMinTrackSize.x = UserGetSystemMetrics(SM_CXMINTRACK as ULONG);
        MinMax.ptMinTrackSize.y = UserGetSystemMetrics(SM_CYMINTRACK as ULONG);
    } else {
        MinMax.ptMinTrackSize.x = 2 * xinc;
        MinMax.ptMinTrackSize.y = 2 * yinc;
    }
    MinMax.ptMaxTrackSize.x = UserGetSystemMetrics(SM_CXMAXTRACK as ULONG);
    MinMax.ptMaxTrackSize.y = UserGetSystemMetrics(SM_CYMAXTRACK as ULONG);
    MinMax.ptMaxPosition.x = -xinc;
    MinMax.ptMaxPosition.y = -yinc;

    if !((*Window).InternalPos.MaxPos.x == -1 && (*Window).InternalPos.MaxPos.y == -1) {
        MinMax.ptMaxPosition = (*Window).InternalPos.MaxPos;
    }

    co_IntSendMessage(
        UserHMGetHandle(Window),
        WM_GETMINMAXINFO,
        0,
        &mut MinMax as *mut MINMAXINFO as LPARAM,
    );

    monitor = UserGetPrimaryMonitor();
    if !monitor.is_null() {
        let mut rc_work: RECT;

        rc_work = core::mem::transmute_copy(&(*monitor).rcMonitor);

        if (style & WS_MAXIMIZEBOX as LONG) != 0 {
            if (style & WS_CAPTION as LONG) == WS_CAPTION as LONG
                || (style & (WS_CHILD | WS_POPUP) as LONG) == 0
            {
                rc_work = core::mem::transmute_copy(&(*monitor).rcWork);
            }
        }

        if MinMax.ptMaxSize.x == UserGetSystemMetrics(SM_CXSCREEN as ULONG) + 2 * xinc
            && MinMax.ptMaxSize.y == UserGetSystemMetrics(SM_CYSCREEN as ULONG) + 2 * yinc
        {
            MinMax.ptMaxSize.x = (rc_work.right - rc_work.left) + 2 * xinc;
            MinMax.ptMaxSize.y = (rc_work.bottom - rc_work.top) + 2 * yinc;
        }
        if MinMax.ptMaxPosition.x == -xinc && MinMax.ptMaxPosition.y == -yinc {
            MinMax.ptMaxPosition.x = rc_work.left - xinc;
            MinMax.ptMaxPosition.y = rc_work.top - yinc;
        }
        if MinMax.ptMaxSize.x >= ((*monitor).rcMonitor.right - (*monitor).rcMonitor.left)
            && MinMax.ptMaxSize.y >= ((*monitor).rcMonitor.bottom - (*monitor).rcMonitor.top)
        {
            (*Window).state |= WNDS_MAXIMIZESTOMONITOR;
        } else {
            (*Window).state &= !WNDS_MAXIMIZESTOMONITOR;
        }
    }

    MinMax.ptMaxTrackSize.x = core::cmp::max(MinMax.ptMaxTrackSize.x, MinMax.ptMinTrackSize.x);
    MinMax.ptMaxTrackSize.y = core::cmp::max(MinMax.ptMaxTrackSize.y, MinMax.ptMinTrackSize.y);

    if !MaxSize.is_null() {
        *MaxSize = core::mem::transmute_copy(&MinMax.ptMaxSize);
    }
    if !MaxPos.is_null() {
        *MaxPos = core::mem::transmute_copy(&MinMax.ptMaxPosition);
    }
    if !MinTrack.is_null() {
        *MinTrack = core::mem::transmute_copy(&MinMax.ptMinTrackSize);
    }
    if !MaxTrack.is_null() {
        *MaxTrack = core::mem::transmute_copy(&MinMax.ptMaxTrackSize);
    }

    0
}
}

unsafe fn IntValidateParent(Child: PWND, ValidateRgn: PREGION) -> BOOL {
    let mut ParentWnd: PWND = (*Child).spwndParent;

    while !ParentWnd.is_null() {
        if (*ParentWnd).style & WS_CLIPCHILDREN != 0 {
            break;
        }

        if (*ParentWnd).hrgnUpdate != 0 as _ {
            IntInvalidateWindows(ParentWnd, ValidateRgn, (RDW_VALIDATE | RDW_NOCHILDREN) as u32);
        }

        ParentWnd = (*ParentWnd).spwndParent;
    }

    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn FixClientRect(ClientRect: PRECTL, WindowRect: PRECTL) {
    FixClientRect_impl(ClientRect, WindowRect)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn FixClientRect(ClientRect: PRECTL, WindowRect: PRECTL) {
    FixClientRect_impl(ClientRect, WindowRect)
}

unsafe fn FixClientRect_impl(ClientRect: PRECTL, WindowRect: PRECTL) {
    if (*ClientRect).left < (*WindowRect).left {
        (*ClientRect).left = (*WindowRect).left;
    } else if (*WindowRect).right < (*ClientRect).left {
        (*ClientRect).left = (*WindowRect).right;
    }
    if (*ClientRect).right < (*WindowRect).left {
        (*ClientRect).right = (*WindowRect).left;
    } else if (*WindowRect).right < (*ClientRect).right {
        (*ClientRect).right = (*WindowRect).right;
    }
    if (*ClientRect).top < (*WindowRect).top {
        (*ClientRect).top = (*WindowRect).top;
    } else if (*WindowRect).bottom < (*ClientRect).top {
        (*ClientRect).top = (*WindowRect).bottom;
    }
    if (*ClientRect).bottom < (*WindowRect).top {
        (*ClientRect).bottom = (*WindowRect).top;
    } else if (*WindowRect).bottom < (*ClientRect).bottom {
        (*ClientRect).bottom = (*WindowRect).bottom;
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn get_valid_rects(
    old_client: *mut RECTL,
    new_client: *mut RECTL,
    mut flags: UINT,
    valid: *mut RECTL,
)
{
    get_valid_rects_impl(old_client, new_client, flags, valid)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn get_valid_rects(
    old_client: *mut RECTL,
    new_client: *mut RECTL,
    flags: UINT,
    valid: *mut RECTL,
)
{
    get_valid_rects_impl(old_client, new_client, flags, valid)
}

unsafe fn get_valid_rects_impl(
    old_client: *mut RECTL,
    new_client: *mut RECTL,
    mut flags: UINT,
    valid: *mut RECTL,
)
{
    let cx: i32;
    let cy: i32;

    if flags & WVR_REDRAW != 0 {
        RECTL_vSetEmptyRect(core::ptr::addr_of_mut!(*valid.offset(0)));
        RECTL_vSetEmptyRect(core::ptr::addr_of_mut!(*valid.offset(1)));
        return;
    }

    if flags & WVR_VALIDRECTS != 0 {
        if RECTL_bIntersectRect(
            core::ptr::addr_of_mut!(*valid.offset(0)),
            core::ptr::addr_of_mut!(*valid.offset(0)),
            new_client,
        ) == 0
            || RECTL_bIntersectRect(
                core::ptr::addr_of_mut!(*valid.offset(1)),
                core::ptr::addr_of_mut!(*valid.offset(1)),
                old_client,
            ) == 0
        {
            RECTL_vSetEmptyRect(core::ptr::addr_of_mut!(*valid.offset(0)));
            RECTL_vSetEmptyRect(core::ptr::addr_of_mut!(*valid.offset(1)));
            return;
        }
        flags = WVR_ALIGNLEFT | WVR_ALIGNTOP;
    } else {
        *valid.offset(0) = *new_client;
        *valid.offset(1) = *old_client;
    }

    cx = core::cmp::min(
        (*valid.offset(0)).right - (*valid.offset(0)).left,
        (*valid.offset(1)).right - (*valid.offset(1)).left,
    );
    cy = core::cmp::min(
        (*valid.offset(0)).bottom - (*valid.offset(0)).top,
        (*valid.offset(1)).bottom - (*valid.offset(1)).top,
    );

    if flags & WVR_ALIGNBOTTOM != 0 {
        (*valid.offset(0)).top = (*valid.offset(0)).bottom - cy;
        (*valid.offset(1)).top = (*valid.offset(1)).bottom - cy;
    } else {
        (*valid.offset(0)).bottom = (*valid.offset(0)).top + cy;
        (*valid.offset(1)).bottom = (*valid.offset(1)).top + cy;
    }
    if flags & WVR_ALIGNRIGHT != 0 {
        (*valid.offset(0)).left = (*valid.offset(0)).right - cx;
        (*valid.offset(1)).left = (*valid.offset(1)).right - cx;
    } else {
        (*valid.offset(0)).right = (*valid.offset(0)).left + cx;
        (*valid.offset(1)).right = (*valid.offset(1)).left + cx;
    }
}

fastcall_fn! {
unsafe fn co_WinPosDoNCCALCSize(
    Window: PWND,
    WinPos: PWINDOWPOS,
    WindowRect: *mut RECTL,
    ClientRect: *mut RECTL,
    validRects: *mut RECTL,
) -> LONG {
    let Parent: PWND;
    let mut wvrFlags: UINT = 0;

    if ((*WinPos).flags & (SWP_FRAMECHANGED | SWP_NOSIZE)) != SWP_NOSIZE {
        let mut params: NCCALCSIZE_PARAMS = core::mem::zeroed();
        let mut winposCopy: WINDOWPOS = core::mem::zeroed();

        params.rgrc[0] = *WindowRect;
        params.rgrc[1] = (*Window).rcWindow;
        params.rgrc[2] = (*Window).rcClient;

        Parent = (*Window).spwndParent;
        if 0 != ((*Window).style & WS_CHILD) && !Parent.is_null() {
            RECTL_vOffsetRect(
                core::ptr::addr_of_mut!(params.rgrc[0]),
                -(*Parent).rcClient.left,
                -(*Parent).rcClient.top,
            );
            RECTL_vOffsetRect(
                core::ptr::addr_of_mut!(params.rgrc[1]),
                -(*Parent).rcClient.left,
                -(*Parent).rcClient.top,
            );
            RECTL_vOffsetRect(
                core::ptr::addr_of_mut!(params.rgrc[2]),
                -(*Parent).rcClient.left,
                -(*Parent).rcClient.top,
            );
        }

        params.lppos = core::ptr::addr_of_mut!(winposCopy);
        winposCopy = *WinPos;

        wvrFlags = co_IntSendMessage(
            UserHMGetHandle(Window),
            WM_NCCALCSIZE,
            1,
            core::ptr::addr_of_mut!(params) as LPARAM,
        ) as UINT;

        if params.rgrc[0].left <= params.rgrc[0].right
            && params.rgrc[0].top <= params.rgrc[0].bottom
        {
            *ClientRect = params.rgrc[0];
            if ((*Window).style & WS_CHILD) != 0 && !Parent.is_null() {
                RECTL_vOffsetRect(ClientRect, (*Parent).rcClient.left, (*Parent).rcClient.top);
            }
            FixClientRect(ClientRect, WindowRect);
        }

        if (*ClientRect).left != (*Window).rcClient.left
            || (*ClientRect).top != (*Window).rcClient.top
        {
            (*WinPos).flags &= !SWP_NOCLIENTMOVE;
        }

        if (*ClientRect).right - (*ClientRect).left
            != (*Window).rcClient.right - (*Window).rcClient.left
        {
            (*WinPos).flags &= !SWP_NOCLIENTSIZE;
        } else {
            wvrFlags &= !WVR_HREDRAW;
        }

        if (*ClientRect).bottom - (*ClientRect).top
            != (*Window).rcClient.bottom - (*Window).rcClient.top
        {
            (*WinPos).flags &= !SWP_NOCLIENTSIZE;
        } else {
            wvrFlags &= !WVR_VREDRAW;
        }

        *validRects.offset(0) = params.rgrc[1];
        *validRects.offset(1) = params.rgrc[2];
    } else {
        if ((*WinPos).flags & SWP_NOMOVE) == 0
            && ((*ClientRect).left != (*Window).rcClient.left
                || (*ClientRect).top != (*Window).rcClient.top)
        {
            (*WinPos).flags &= !SWP_NOCLIENTMOVE;
        }
    }

    if (*WinPos).flags & (SWP_NOCOPYBITS | SWP_NOREDRAW | SWP_SHOWWINDOW | SWP_HIDEWINDOW) != 0 {
        RECTL_vSetEmptyRect(validRects.offset(0));
        RECTL_vSetEmptyRect(validRects.offset(1));
    } else {
        get_valid_rects(
            core::ptr::addr_of_mut!((*Window).rcClient),
            ClientRect,
            wvrFlags,
            validRects,
        );
    }

    wvrFlags as LONG
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_WinPosDoWinPosChanging(
    Window: PWND,
    WinPos: PWINDOWPOS,
    WindowRect: PRECTL,
    ClientRect: PRECTL,
) -> u8 {
    co_WinPosDoWinPosChanging_impl(Window, WinPos, WindowRect, ClientRect)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_WinPosDoWinPosChanging(
    Window: PWND,
    WinPos: PWINDOWPOS,
    WindowRect: PRECTL,
    ClientRect: PRECTL,
) -> u8 {
    co_WinPosDoWinPosChanging_impl(Window, WinPos, WindowRect, ClientRect)
}

unsafe fn co_WinPosDoWinPosChanging_impl(
    Window: PWND,
    WinPos: PWINDOWPOS,
    WindowRect: PRECTL,
    ClientRect: PRECTL,
) -> u8 {
    if ((*WinPos).flags & SWP_NOSENDCHANGING as u32) == 0
        && !(((*WinPos).flags & SWP_AGG_NOCLIENTCHANGE as u32) != 0
            && ((*WinPos).flags & SWP_SHOWWINDOW as u32) != 0)
    {
        co_IntSendMessage(
            UserHMGetHandle(Window),
            WM_WINDOWPOSCHANGING,
            0,
            WinPos as LPARAM,
        );
    }

    *WindowRect = (*Window).rcWindow;
    *ClientRect = if ((*Window).style & WS_MINIMIZE as u32) != 0 {
        (*Window).rcWindow
    } else {
        (*Window).rcClient
    };

    if ((*WinPos).flags & SWP_NOSIZE as u32) == 0 {
        if ((*Window).style & WS_MINIMIZE as u32) != 0 {
            (*WindowRect).right = (*WindowRect).left + UserGetSystemMetrics(SM_CXMINIMIZED);
            (*WindowRect).bottom = (*WindowRect).top + UserGetSystemMetrics(SM_CYMINIMIZED);
        } else {
            (*WindowRect).right = (*WindowRect).left + (*WinPos).cx;
            (*WindowRect).bottom = (*WindowRect).top + (*WinPos).cy;
        }
    }

    if ((*WinPos).flags & SWP_NOMOVE as u32) == 0 {
        let mut X: i32 = (*WinPos).x;
        let mut Y: i32 = (*WinPos).y;

        let Parent: PWND = (*Window).spwndParent;

        if !Parent.is_null()
            && Parent != (*(*(*Window).head.rpdesk).pDeskInfo).spwnd
        {
            X += (*Parent).rcClient.left;
            Y += (*Parent).rcClient.top;
        }

        (*WindowRect).left = X;
        (*WindowRect).top = Y;
        (*WindowRect).right += X - (*Window).rcWindow.left;
        (*WindowRect).bottom += Y - (*Window).rcWindow.top;

        RECTL_vOffsetRect(
            ClientRect,
            X - (*Window).rcWindow.left,
            Y - (*Window).rcWindow.top,
        );
    }
    (*WinPos).flags |= SWP_NOCLIENTMOVE as u32 | SWP_NOCLIENTSIZE as u32;

    1u8
}

fastcall_fn! {
unsafe fn WinPosDoOwnedPopups(Window: PWND, mut hWndInsertAfter: HWND) -> HWND {
    let mut List: *mut HWND = core::ptr::null_mut();
    let mut Owner: HWND;
    let Style: LONG;
    let mut DesktopWindow: PWND;
    let mut ChildObject: PWND;
    let mut i: i32;

    Style = (*Window).style as LONG;

    if (Style & WS_CHILD as LONG) != 0 {
        return hWndInsertAfter;
    }

    Owner = if !(*Window).spwndOwner.is_null() {
        UserHMGetHandle((*Window).spwndOwner)
    } else {
        core::ptr::null_mut()
    };

    if !Owner.is_null() {
        if hWndInsertAfter != HWND_TOPMOST {
            DesktopWindow = UserGetDesktopWindow();
            List = IntWinListChildren(DesktopWindow);

            if !List.is_null() {
                i = 0;
                while !(*List.offset(i as isize)).is_null() {
                    let mut topmost: u8 = 0u8;

                    ChildObject = ValidateHwndNoErr(*List.offset(i as isize));
                    if !ChildObject.is_null() {
                        topmost = (((*ChildObject).ExStyle & WS_EX_TOPMOST) != 0) as u8;
                    }

                    if *List.offset(i as isize) == Owner {
                        if i > 0 {
                            if *List.offset((i - 1) as isize) != UserHMGetHandle(Window) {
                                hWndInsertAfter = *List.offset((i - 1) as isize);
                            } else {
                                ExFreePoolWithTag(List as *mut _, USERTAG_WINDOWLIST);
                                return hWndInsertAfter;
                            }
                        } else {
                            hWndInsertAfter = if topmost != 0 { HWND_TOPMOST } else { HWND_TOP };
                        }
                        break;
                    }

                    if hWndInsertAfter == HWND_TOP || hWndInsertAfter == HWND_NOTOPMOST {
                        if topmost == 0 {
                            break;
                        }
                    } else if *List.offset(i as isize) == hWndInsertAfter {
                        break;
                    }
                    i += 1;
                }
            } else {
                return hWndInsertAfter;
            }
        }
    }

    'done: {
        if hWndInsertAfter == HWND_BOTTOM {
            if !List.is_null() {
                ExFreePoolWithTag(List as *mut _, USERTAG_WINDOWLIST);
            }
            break 'done;
        }

        if List.is_null() {
            DesktopWindow = UserGetDesktopWindow();
            List = IntWinListChildren(DesktopWindow);
        }

        if !List.is_null() {
            i = 0;

            if hWndInsertAfter == HWND_TOP || hWndInsertAfter == HWND_NOTOPMOST {
                if hWndInsertAfter == HWND_NOTOPMOST || ((*Window).ExStyle & WS_EX_TOPMOST) == 0 {
                    while !(*List.offset(i as isize)).is_null() && {
                        ChildObject = ValidateHwndNoErr(*List.offset(i as isize));
                        !ChildObject.is_null()
                    } && ((*ChildObject).ExStyle & WS_EX_TOPMOST) != 0
                    {
                        i += 1;
                    }
                }
            } else if hWndInsertAfter != HWND_TOPMOST {
                i = 0;
                while !(*List.offset(i as isize)).is_null() {
                    if *List.offset(i as isize) == hWndInsertAfter {
                        break;
                    }
                    if *List.offset(i as isize) == UserHMGetHandle(Window) {
                        ExFreePoolWithTag(List as *mut _, USERTAG_WINDOWLIST);
                        break 'done;
                    }
                    i += 1;
                }
            }

            while !(*List.offset(i as isize)).is_null() {
                let Wnd: PWND;
                let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

                if *List.offset(i as isize) == UserHMGetHandle(Window) {
                    break;
                }

                Wnd = ValidateHwndNoErr(*List.offset(i as isize));
                if Wnd.is_null() {
                    i += 1;
                    continue;
                }

                Owner = if !(*Wnd).spwndOwner.is_null() {
                    UserHMGetHandle((*Wnd).spwndOwner)
                } else {
                    core::ptr::null_mut()
                };

                if Owner != UserHMGetHandle(Window) {
                    i += 1;
                    continue;
                }

                UserRefObjectCo(Wnd, &mut Ref);
                co_WinPosSetWindowPos(
                    Wnd,
                    hWndInsertAfter,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_DEFERERASE,
                );

                UserDerefObjectCo(Wnd);
                hWndInsertAfter = *List.offset(i as isize);
                i += 1;
            }
            ExFreePoolWithTag(List as *mut _, USERTAG_WINDOWLIST);
        }
    }

    hWndInsertAfter
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn WinPosInternalMoveWindow(Window: PWND, MoveX: i32, MoveY: i32) {
    WinPosInternalMoveWindow_impl(Window, MoveX, MoveY)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn WinPosInternalMoveWindow(Window: PWND, MoveX: i32, MoveY: i32) {
    WinPosInternalMoveWindow_impl(Window, MoveX, MoveY)
}

unsafe fn WinPosInternalMoveWindow_impl(Window: PWND, MoveX: i32, MoveY: i32) {
    let mut Child: PWND;

    (*Window).rcWindow.left += MoveX;
    (*Window).rcWindow.right += MoveX;
    (*Window).rcWindow.top += MoveY;
    (*Window).rcWindow.bottom += MoveY;

    (*Window).rcClient.left += MoveX;
    (*Window).rcClient.right += MoveX;
    (*Window).rcClient.top += MoveY;
    (*Window).rcClient.bottom += MoveY;

    Child = (*Window).spwndChild;
    while !Child.is_null() {
        WinPosInternalMoveWindow(Child, MoveX, MoveY);
        Child = (*Child).spwndNext;
    }
}

fastcall_fn! {
unsafe fn WinPosFixupFlags(WinPos: *mut WINDOWPOS, Wnd: PWND) -> u8 {
    let Parent: PWND;
    let mut pt: POINT = core::mem::zeroed();

    if (*WinPos).x < -32768 { (*WinPos).x = -32768; }
    else if (*WinPos).x > 32767 { (*WinPos).x = 32767; }
    if (*WinPos).y < -32768 { (*WinPos).y = -32768; }
    else if (*WinPos).y > 32767 { (*WinPos).y = 32767; }

    (*WinPos).cx = core::cmp::max((*WinPos).cx, 0);
    (*WinPos).cy = core::cmp::max((*WinPos).cy, 0);

    Parent = UserGetAncestor(Wnd, GA_PARENT as u32);
    if IntIsWindowVisible(Parent) == 0
        && ((*WinPos).flags & (SWP_AGG_STATUSFLAGS as u32)) == (SWP_AGG_NOPOSCHANGE as u32)
    {
        (*WinPos).flags |= SWP_NOREDRAW as u32;
    }

    if (*Wnd).style & (WS_VISIBLE as u32) != 0 {
        (*WinPos).flags &= !(SWP_SHOWWINDOW as u32);
    } else {
        (*WinPos).flags &= !(SWP_HIDEWINDOW as u32);
        if (*WinPos).flags & (SWP_SHOWWINDOW as u32) == 0 {
            (*WinPos).flags |= SWP_NOREDRAW as u32;
        }
    }

    if (*Wnd).rcWindow.right - (*Wnd).rcWindow.left == (*WinPos).cx
        && (*Wnd).rcWindow.bottom - (*Wnd).rcWindow.top == (*WinPos).cy
    {
        (*WinPos).flags |= SWP_NOSIZE as u32;
    }

    pt.x = (*WinPos).x;
    pt.y = (*WinPos).y;
    IntClientToScreen(Parent, &mut pt);
    if (*Wnd).rcWindow.left == pt.x && (*Wnd).rcWindow.top == pt.y {
        (*WinPos).flags |= SWP_NOMOVE as u32;
    }

    if (*WinPos).hwnd != UserGetForegroundWindow()
        && ((*Wnd).style & ((WS_POPUP | WS_CHILD) as u32)) != (WS_CHILD as u32)
    {
        if (*WinPos).flags & ((SWP_NOACTIVATE | SWP_HIDEWINDOW) as u32) == 0
            && ((*WinPos).flags & (SWP_NOZORDER as u32) != 0
                || ((*WinPos).hwndInsertAfter != HWND_TOPMOST
                    && (*WinPos).hwndInsertAfter != HWND_NOTOPMOST))
        {
            (*WinPos).flags &= !(SWP_NOZORDER as u32);
            (*WinPos).hwndInsertAfter = if (*Wnd).ExStyle & (WS_EX_TOPMOST as u32) != 0 {
                HWND_TOPMOST
            } else {
                HWND_TOP
            };
        }
    }

    if (*WinPos).flags & (SWP_NOZORDER as u32) == 0 {
        if (*WinPos).hwndInsertAfter == 0xffff as HWND {
            (*WinPos).hwndInsertAfter = HWND_TOPMOST;
        } else if (*WinPos).hwndInsertAfter == 0xfffe as HWND {
            (*WinPos).hwndInsertAfter = HWND_NOTOPMOST;
        }

        if (*WinPos).hwndInsertAfter == HWND_TOP {
            if (*Wnd).ExStyle & (WS_EX_TOPMOST as u32) != 0 {
                (*WinPos).hwndInsertAfter = HWND_TOPMOST;
            }

            if IntGetWindow((*WinPos).hwnd, GW_HWNDFIRST as u32) == (*WinPos).hwnd {
                (*WinPos).flags |= SWP_NOZORDER as u32;
            }
        } else if (*WinPos).hwndInsertAfter == HWND_BOTTOM {
            if (*Wnd).ExStyle & (WS_EX_TOPMOST as u32) == 0
                && IntGetWindow((*WinPos).hwnd, GW_HWNDLAST as u32) == (*WinPos).hwnd
            {
                (*WinPos).flags |= SWP_NOZORDER as u32;
            }
        } else if (*WinPos).hwndInsertAfter == HWND_TOPMOST {
            if (*Wnd).ExStyle & (WS_EX_TOPMOST as u32) != 0
                && IntGetWindow((*WinPos).hwnd, GW_HWNDFIRST as u32) == (*WinPos).hwnd
            {
                (*WinPos).flags |= SWP_NOZORDER as u32;
            }
        } else if (*WinPos).hwndInsertAfter == HWND_NOTOPMOST {
            if (*Wnd).ExStyle & (WS_EX_TOPMOST as u32) == 0 {
                (*WinPos).flags |= SWP_NOZORDER as u32;
            }
        } else {
            let InsAfterWnd: PWND = ValidateHwndNoErr((*WinPos).hwndInsertAfter);
            if InsAfterWnd.is_null() {
                return 1u8;
            }

            if (*InsAfterWnd).spwndParent != (*Wnd).spwndParent {
                return 0u8;
            } else {
                if (*WinPos).hwnd == (*WinPos).hwndInsertAfter
                    || (!(*InsAfterWnd).spwndNext.is_null()
                        && (*WinPos).hwnd == UserHMGetHandle((*InsAfterWnd).spwndNext))
                {
                    (*WinPos).flags |= SWP_NOZORDER as u32;
                }
            }
        }
    }

    1u8
}
}

const HWND_TOP: HWND = 0 as HWND;
const HWND_BOTTOM: HWND = 1 as HWND;
const HWND_TOPMOST: HWND = -1isize as HWND;
const HWND_NOTOPMOST: HWND = -2isize as HWND;

unsafe fn ForceNCPaintErase(Wnd: PWND, hRgn: HRGN, pRgn: PREGION) {
    let hDC: HDC;
    let RgnUpdate: PREGION;
    let RgnType: UINT;
    let mut Create: BOOL = 0;

    if (*Wnd).hrgnUpdate.is_null() {
        (*Wnd).hrgnUpdate = NtGdiCreateRectRgn(0, 0, 0, 0);
        IntGdiSetRegionOwner((*Wnd).hrgnUpdate, GDI_OBJ_HMGR_PUBLIC);
        Create = 1;
    }

    if (*Wnd).hrgnUpdate != HRGN_WINDOW {
        RgnUpdate = REGION_LockRgn((*Wnd).hrgnUpdate);
        if !RgnUpdate.is_null() {
            RgnType = IntGdiCombineRgn(RgnUpdate, RgnUpdate, pRgn, RGN_OR as i32) as UINT;
            REGION_UnlockRgn(RgnUpdate);
            if RgnType == NULLREGION as UINT {
                IntGdiSetRegionOwner((*Wnd).hrgnUpdate, GDI_OBJ_HMGR_POWNED);
                GreDeleteObject((*Wnd).hrgnUpdate as _);
                (*Wnd).hrgnUpdate = core::ptr::null_mut();
                Create = 0;
            }
        }
    }

    IntSendNCPaint(Wnd, hRgn);

    if !(*Wnd).hrgnUpdate.is_null() {
        hDC = UserGetDCEx(
            Wnd,
            (*Wnd).hrgnUpdate as HANDLE,
            (DCX_CACHE | DCX_USESTYLE | DCX_INTERSECTRGN | DCX_KEEPCLIPRGN) as u32,
        );

        (*Wnd).state &= !(WNDS_SENDERASEBACKGROUND | WNDS_ERASEBACKGROUND);
        if co_IntSendMessage(
            UserHMGetHandle(Wnd as _),
            WM_ERASEBKGND,
            hDC as WPARAM,
            0,
        ) == 0
        {
            (*Wnd).state |= WNDS_SENDERASEBACKGROUND | WNDS_ERASEBACKGROUND;
        }
        UserReleaseDC(Wnd, hDC, 0);
    }

    if Create != 0 {
        IntGdiSetRegionOwner((*Wnd).hrgnUpdate, GDI_OBJ_HMGR_POWNED);
        GreDeleteObject((*Wnd).hrgnUpdate as _);
        (*Wnd).hrgnUpdate = core::ptr::null_mut();
    }
}

unsafe fn IntImeWindowPosChanged() {
    let mut phwnd: *mut HWND;
    let mut pwndNode: PWND;
    let pwndDesktop: PWND = UserGetDesktopWindow();
    let pWL: PWINDOWLIST;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    if pwndDesktop.is_null() {
        return;
    }

    pWL = IntBuildHwndList((*pwndDesktop).spwndChild, IACE_LIST as u32, gptiCurrent);
    if pWL.is_null() {
        return;
    }

    phwnd = (*pWL).ahwnd.as_mut_ptr();
    while *phwnd != HWND_TERMINATOR {
        if (*gptiCurrent).TIF_flags & TIF_INCLEANUP != 0 {
            break;
        }

        pwndNode = ValidateHwndNoErr(*phwnd);
        if pwndNode.is_null()
            || head_pti(pwndNode) != gptiCurrent
            || (*(*pwndNode).pcls).atomClassName != (*gpsi).atomSysClass[ICLS_IME as usize]
        {
            phwnd = phwnd.add(1);
            continue;
        }

        UserRefObjectCo(pwndNode, &mut Ref);
        co_IntSendMessage(*phwnd, WM_IME_SYSTEM, IMS_UPDATEIMEUI as WPARAM, 0);
        UserDerefObjectCo(pwndNode);

        phwnd = phwnd.add(1);
    }

    IntFreeHwndList(pWL);
}

fastcall_fn! {
#[no_mangle]
unsafe fn co_WinPosSetWindowPos(
    Window: PWND,
    WndInsertAfter: HWND,
    x: INT,
    y: INT,
    cx: INT,
    cy: INT,
    flags: UINT,
) -> BOOLEAN {
    let mut WinPos: WINDOWPOS = core::mem::zeroed();
    let mut NewWindowRect: RECTL = core::mem::zeroed();
    let mut NewClientRect: RECTL = core::mem::zeroed();
    let mut valid_rects: [RECTL; 2] = core::mem::zeroed();
    let mut VisBefore: PREGION = core::ptr::null_mut();
    let mut VisBeforeJustClient: PREGION = core::ptr::null_mut();
    let mut VisAfter: PREGION = core::ptr::null_mut();
    let mut CopyRgn: PREGION = core::ptr::null_mut();
    let mut WvrFlags: ULONG;
    let mut OldWindowRect: RECTL;
    let mut OldClientRect: RECTL;
    let mut RgnType: i32;
    let mut Dc: HDC;
    let mut CopyRect: RECTL = core::mem::zeroed();
    let Ancestor: PWND;
    let bPointerInWindow: BOOL;
    let mut PosChanged: BOOL = 0;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    ASSERT_REFS_CO(Window);

    #[cfg(DBG)]
    dump_winpos_flags(flags);

    bPointerInWindow = IntPtInWindow(Window, (*pusi(gpsi)).ptCursor.x, (*pusi(gpsi)).ptCursor.y);

    WinPos.hwnd = UserHMGetHandle(Window) as HWND;
    WinPos.hwndInsertAfter = WndInsertAfter;
    WinPos.x = x;
    WinPos.y = y;
    WinPos.cx = cx;
    WinPos.cy = cy;
    WinPos.flags = flags;

    if (flags & SWP_ASYNCWINDOWPOS) != 0 {
        let lRes: LRESULT;
        let ppos: PWINDOWPOS = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<WINDOWPOS>() as SIZE_T,
            USERTAG_SWP,
        ) as PWINDOWPOS;
        if !ppos.is_null() {
            WinPos.flags &= !SWP_ASYNCWINDOWPOS;
            *ppos = WinPos;
            lRes = co_IntSendMessageNoWait(
                WinPos.hwnd,
                WM_ASYNC_SETWINDOWPOS,
                0,
                ppos as LPARAM,
            );
            if lRes == 0 {
                ExFreePoolWithTag(ppos as *mut _, USERTAG_SWP);
                return 0u8;
            }
            return 1u8;
        }
        return 0u8;
    }

    co_WinPosDoWinPosChanging(Window, &mut WinPos, &mut NewWindowRect, &mut NewClientRect);

    if IntIsWindow(WinPos.hwnd) == 0 {
        EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return 0u8;
    }

    if WinPosFixupFlags(&mut WinPos, Window) == 0 {
        return 1u8;
    }

    Ancestor = UserGetAncestor(Window, GA_PARENT);
    if (WinPos.flags & (SWP_NOZORDER | SWP_HIDEWINDOW | SWP_SHOWWINDOW)) != SWP_NOZORDER
        && !Ancestor.is_null()
        && UserHMGetHandle(Ancestor) as HWND == IntGetDesktopWindow()
    {
        WinPos.hwndInsertAfter = WinPosDoOwnedPopups(Window, WinPos.hwndInsertAfter);
    }

    if (WinPos.flags & SWP_NOREDRAW) == 0 {
        if (WinPos.flags & SWP_SHOWWINDOW) == 0
            && (WinPos.flags
                & (SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_HIDEWINDOW | SWP_FRAMECHANGED))
                != (SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER)
        {
            VisBefore = VIS_ComputeVisibleRegion(
                Window,
                0,
                0,
                if ((*Window).style & WS_CLIPSIBLINGS) != 0 { 1 } else { 0 },
            );

            if !VisBefore.is_null() && REGION_Complexity(VisBefore) == NULLREGION as i32 {
                REGION_Delete(VisBefore);
                VisBefore = core::ptr::null_mut();
            } else if !VisBefore.is_null() {
                REGION_bOffsetRgn(
                    VisBefore,
                    -(*Window).rcWindow.left,
                    -(*Window).rcWindow.top,
                );
            }

            if (WinPos.flags & (SWP_NOSIZE | SWP_FRAMECHANGED)) != SWP_NOSIZE {
                VisBeforeJustClient = VIS_ComputeVisibleRegion(
                    Window,
                    1,
                    0,
                    if ((*Window).style & WS_CLIPSIBLINGS) != 0 { 1 } else { 0 },
                );

                if !VisBeforeJustClient.is_null()
                    && REGION_Complexity(VisBeforeJustClient) == NULLREGION as i32
                {
                    REGION_Delete(VisBeforeJustClient);
                    VisBeforeJustClient = core::ptr::null_mut();
                } else if !VisBeforeJustClient.is_null() {
                    REGION_bOffsetRgn(
                        VisBeforeJustClient,
                        -(*Window).rcWindow.left,
                        -(*Window).rcWindow.top,
                    );
                }
            }
        }
    }

    if !(*Window).hrgnNewFrame.is_null() {
        SelectWindowRgn(Window, (*Window).hrgnNewFrame);
        (*Window).hrgnNewFrame = core::ptr::null_mut();
    }

    WvrFlags = co_WinPosDoNCCALCSize(
        Window,
        &mut WinPos,
        &mut NewWindowRect,
        &mut NewClientRect,
        valid_rects.as_mut_ptr(),
    ) as ULONG;

    if (WinPos.flags & SWP_NOZORDER) == 0 && WinPos.hwnd != UserGetShellWindow() {
        IntLinkHwnd(Window, WinPos.hwndInsertAfter);
    }

    OldWindowRect = (*Window).rcWindow;
    OldClientRect = (*Window).rcClient;

    if NewClientRect.left != OldClientRect.left || NewClientRect.top != OldClientRect.top {
        WinPosInternalMoveWindow(
            Window,
            NewClientRect.left - OldClientRect.left,
            NewClientRect.top - OldClientRect.top,
        );
        PosChanged = 1;
    }

    (*Window).rcWindow = NewWindowRect;
    (*Window).rcClient = NewClientRect;

    if (WinPos.flags & SWP_HIDEWINDOW) != 0 {
        co_UserRedrawWindow(
            Window,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            RDW_VALIDATE | RDW_NOFRAME | RDW_NOERASE | RDW_NOINTERNALPAINT | RDW_ALLCHILDREN,
        );

        if UserIsDesktopWindow((*Window).spwndParent) != 0 {
            co_IntShellHookNotify(
                HSHELL_WINDOWDESTROYED as WPARAM,
                UserHMGetHandle(Window) as WPARAM,
                0,
            );
        }

        (*Window).style &= !WS_VISIBLE;
        (*head_pti(Window)).cVisWindows -= 1;
        IntNotifyWinEvent(EVENT_OBJECT_HIDE, Window, OBJID_WINDOW as LONG, CHILDID_SELF as LONG, WEF_SETBYWNDPTI);
    } else if (WinPos.flags & SWP_SHOWWINDOW) != 0 {
        if ((*Window).style & WS_CHILD) != 0 {
            if ((*Window).style & WS_POPUP) != 0 && ((*Window).ExStyle & WS_EX_APPWINDOW) != 0 {
                co_IntShellHookNotify(
                    HSHELL_WINDOWCREATED as WPARAM,
                    UserHMGetHandle(Window) as WPARAM,
                    0,
                );
                if (WinPos.flags & SWP_NOACTIVATE) == 0 {
                    UpdateShellHook(Window);
                }
            }
        } else if ((*Window).ExStyle & WS_EX_APPWINDOW) != 0
            || (((*Window).ExStyle & WS_EX_TOOLWINDOW) == 0
                && (*Window).spwndOwner.is_null()
                && ((*Window).spwndParent.is_null()
                    || UserIsDesktopWindow((*Window).spwndParent) != 0))
        {
            if UserIsDesktopWindow(Window) == 0 {
                co_IntShellHookNotify(
                    HSHELL_WINDOWCREATED as WPARAM,
                    UserHMGetHandle(Window) as WPARAM,
                    0,
                );
                if (WinPos.flags & SWP_NOACTIVATE) == 0 {
                    UpdateShellHook(Window);
                }
            }
        }

        (*Window).style |= WS_VISIBLE;
        (*head_pti(Window)).cVisWindows += 1;
        IntNotifyWinEvent(EVENT_OBJECT_SHOW, Window, OBJID_WINDOW as LONG, CHILDID_SELF as LONG, WEF_SETBYWNDPTI);
    } else {
        IntCheckFullscreen(Window);
    }

    if !(*Window).hrgnUpdate.is_null() && (*Window).hrgnUpdate != HRGN_WINDOW {
        NtGdiOffsetRgn(
            (*Window).hrgnUpdate,
            NewWindowRect.left - OldWindowRect.left,
            NewWindowRect.top - OldWindowRect.top,
        );
    }

    DceResetActiveDCEs(Window);

    if ((*Window).style & WS_VISIBLE) != 0
        && ((WinPos.flags & SWP_STATECHANGED) != 0
            || (((*Window).state2 & WNDS2_WIN31COMPAT) == 0 && (WinPos.flags & SWP_NOREDRAW) != 0))
    {
        (*Window).state |= WNDS_SENDNCPAINT;
    }

    /*
     * Request a full non-client repaint when the frame geometry changes.
     * Otherwise, copied bits can retain parts of the old frame. CORE-20769.
     *
     * NOTE ON ORDER: painting.rs already CONSUMES WNDS2_FORCEFULLNCPAINTCLIPRGN
     * (at ~539) and inherited that consumer from before our fork, but nothing in
     * our tree ever SET the flag -- so that path has been dead the whole time.
     * This is the setter. Upstream added it to fix a regression from b38db9dd238,
     * which we have NOT applied; but it also closes CORE-5877 and CORE-16672,
     * which predate that commit and which we DO carry. Taking it first is also
     * the safe order: if b38db9dd238 is ported later, its regression is already
     * prevented. Upstream-Commit: d09a3600606 (#9495)
     */
    if ((*Window).style & WS_VISIBLE) != 0
        && ((WinPos.flags & SWP_FRAMECHANGED) != 0
            || (OldWindowRect.right - OldWindowRect.left)
                != (NewWindowRect.right - NewWindowRect.left)
            || (OldWindowRect.bottom - OldWindowRect.top)
                != (NewWindowRect.bottom - NewWindowRect.top))
    {
        (*Window).state |= WNDS_SENDNCPAINT;
        (*Window).state2 |= WNDS2_FORCEFULLNCPAINTCLIPRGN;
    }

    if ((WinPos.flags & SWP_NOREDRAW) == 0
        && ((WinPos.flags & SWP_AGG_STATUSFLAGS) != SWP_AGG_NOPOSCHANGE))
        || ((WinPos.flags & SWP_NOZORDER) != 0 && (WinPos.flags & SWP_NOOWNERZORDER) != 0)
    {
        VisAfter = VIS_ComputeVisibleRegion(
            Window,
            0,
            0,
            if ((*Window).style & WS_CLIPSIBLINGS) != 0 { 1 } else { 0 },
        );

        if !VisAfter.is_null() && REGION_Complexity(VisAfter) == NULLREGION as i32 {
            REGION_Delete(VisAfter);
            VisAfter = core::ptr::null_mut();
        } else if !VisAfter.is_null() {
            if !(*Window).hrgnUpdate.is_null() {
                let RgnUpdate: PREGION = REGION_LockRgn((*Window).hrgnUpdate);
                if !RgnUpdate.is_null() {
                    RgnType = IntGdiCombineRgn(RgnUpdate, RgnUpdate, VisAfter, RGN_AND as INT);
                    REGION_UnlockRgn(RgnUpdate);
                }
            }
            REGION_bOffsetRgn(VisAfter, -(*Window).rcWindow.left, -(*Window).rcWindow.top);
        }

        if !VisBefore.is_null()
            && !VisAfter.is_null()
            && (WvrFlags & WVR_REDRAW) == 0
            && (WinPos.flags & SWP_NOCOPYBITS) == 0
            && ((*Window).ExStyle & WS_EX_TRANSPARENT) == 0
        {
            CopyRgn = IntSysCreateRectpRgn(0, 0, 0, 0);
            if (WinPos.flags & SWP_NOSIZE) != 0 && (WinPos.flags & SWP_NOCLIENTSIZE) != 0 {
                RgnType = IntGdiCombineRgn(CopyRgn, VisAfter, VisBefore, RGN_AND as INT);
            } else if !VisBeforeJustClient.is_null() {
                RgnType = IntGdiCombineRgn(CopyRgn, VisAfter, VisBeforeJustClient, RGN_AND as INT);
            }

            if !(*Window).hrgnUpdate.is_null() {
                let RgnUpdate: PREGION = REGION_LockRgn((*Window).hrgnUpdate);
                if !RgnUpdate.is_null() {
                    REGION_bOffsetRgn(CopyRgn, NewWindowRect.left, NewWindowRect.top);
                    IntGdiCombineRgn(CopyRgn, CopyRgn, RgnUpdate, RGN_DIFF as INT);
                    REGION_bOffsetRgn(CopyRgn, -NewWindowRect.left, -NewWindowRect.top);
                    REGION_UnlockRgn(RgnUpdate);
                }
            }

            if REGION_GetRgnBox(CopyRgn, &mut CopyRect) == NULLREGION as i32 {
                REGION_Delete(CopyRgn);
                CopyRgn = core::ptr::null_mut();
            } else if OldWindowRect.left != NewWindowRect.left
                || OldWindowRect.top != NewWindowRect.top
                || (WinPos.flags & SWP_FRAMECHANGED) != 0
            {
                let DcRgn: HRGN = NtGdiCreateRectRgn(0, 0, 0, 0);
                let DcRgnObj: PREGION = REGION_LockRgn(DcRgn);

                IntGdiCombineRgn(DcRgnObj, CopyRgn, core::ptr::null_mut(), RGN_COPY as INT);
                REGION_bOffsetRgn(DcRgnObj, NewWindowRect.left, NewWindowRect.top);
                REGION_UnlockRgn(DcRgnObj);
                Dc = UserGetDCEx(
                    Window,
                    DcRgn as HANDLE,
                    DCX_WINDOW | DCX_CACHE | DCX_INTERSECTRGN | DCX_CLIPSIBLINGS | DCX_KEEPCLIPRGN,
                );
                NtGdiBitBlt(
                    Dc,
                    CopyRect.left,
                    CopyRect.top,
                    CopyRect.right - CopyRect.left,
                    CopyRect.bottom - CopyRect.top,
                    Dc,
                    CopyRect.left + (OldWindowRect.left - NewWindowRect.left),
                    CopyRect.top + (OldWindowRect.top - NewWindowRect.top),
                    SRCCOPY,
                    CLR_INVALID,
                    0,
                );

                UserReleaseDC(Window, Dc, 0);
                IntValidateParent(Window, CopyRgn);
                GreDeleteObject(DcRgn as *mut _);
            }
        } else {
            CopyRgn = core::ptr::null_mut();
        }

        if !VisAfter.is_null() {
            let DirtyRgn: PREGION = IntSysCreateRectpRgn(0, 0, 0, 0);
            if !DirtyRgn.is_null() {
                if !CopyRgn.is_null() {
                    RgnType = IntGdiCombineRgn(DirtyRgn, VisAfter, CopyRgn, RGN_DIFF as INT);
                } else {
                    RgnType = IntGdiCombineRgn(DirtyRgn, VisAfter, core::ptr::null_mut(), RGN_COPY as INT);
                }

                if RgnType != ERROR as i32 && RgnType != NULLREGION as i32 {
                    let Parent: PWND = (*Window).spwndParent;

                    REGION_bOffsetRgn(DirtyRgn, (*Window).rcWindow.left, (*Window).rcWindow.top);

                    if ((*Window).style & WS_CHILD) != 0
                        && !Parent.is_null()
                        && ((*Parent).style & WS_CLIPCHILDREN) == 0
                    {
                        IntInvalidateWindows(Parent, DirtyRgn, RDW_ERASE | RDW_INVALIDATE);
                        co_IntPaintWindows(Parent, RDW_NOCHILDREN, 0);
                    }
                    IntInvalidateWindows(
                        Window,
                        DirtyRgn,
                        RDW_ERASE | RDW_FRAME | RDW_INVALIDATE | RDW_ALLCHILDREN,
                    );
                } else if RgnType != ERROR as i32 && RgnType == NULLREGION as i32 {
                    if PosChanged == 0
                        && (WinPos.flags & SWP_DEFERERASE) == 0
                        && (WinPos.flags & SWP_FRAMECHANGED) != 0
                    {
                        let mut pwnd: PWND = Window;
                        let Parent: PWND = (*Window).spwndParent;

                        if ((*pwnd).style & WS_CHILD) != 0 {
                            pwnd = if !Parent.is_null() { Parent } else { pwnd };
                        }

                        if ((*pwnd).style & WS_CHILD) == 0 {
                            if ((*Window).style & WS_CLIPSIBLINGS) != 0
                                && ((*Window).style
                                    & (WS_POPUP | WS_CLIPCHILDREN | WS_SIZEBOX))
                                    == 0
                            {
                                IntSendNCPaint(pwnd, HRGN_WINDOW);
                            } else {
                                let DcRgn: HRGN = NtGdiCreateRectRgn(0, 0, 0, 0);
                                let DcRgnObj: PREGION = REGION_LockRgn(DcRgn);
                                IntGdiCombineRgn(DcRgnObj, VisBefore, core::ptr::null_mut(), RGN_COPY as INT);
                                REGION_UnlockRgn(DcRgnObj);
                                ForceNCPaintErase(pwnd, DcRgn, DcRgnObj);
                                GreDeleteObject(DcRgn as *mut _);
                            }
                        }
                    }
                }
                REGION_Delete(DirtyRgn);
            }
        }

        if !VisBefore.is_null() {
            let ExposedRgn: PREGION = IntSysCreateRectpRgn(0, 0, 0, 0);
            if !ExposedRgn.is_null() {
                RgnType = IntGdiCombineRgn(ExposedRgn, VisBefore, core::ptr::null_mut(), RGN_COPY as INT);
                REGION_bOffsetRgn(
                    ExposedRgn,
                    OldWindowRect.left - NewWindowRect.left,
                    OldWindowRect.top - NewWindowRect.top,
                );

                if !VisAfter.is_null() {
                    RgnType = IntGdiCombineRgn(ExposedRgn, ExposedRgn, VisAfter, RGN_DIFF as INT);
                }

                if RgnType != ERROR as i32 && RgnType != NULLREGION as i32 {
                    co_VIS_WindowLayoutChanged(Window, ExposedRgn);
                }
                REGION_Delete(ExposedRgn);
            }
        }
    }

    if (WinPos.flags & (SWP_NOACTIVATE | SWP_HIDEWINDOW)) == 0 {
        if ((*Window).style & (WS_CHILD | WS_POPUP)) == WS_CHILD {
            co_IntSendMessageNoWait(WinPos.hwnd, WM_CHILDACTIVATE, 0, 0);
        } else {
            if (*(*pti).MessageQueue).spwndActive != Window
                || (*pti).MessageQueue != gpqForeground
            {
                if ((*Window).state & WNDS_BEINGACTIVATED) == 0 {
                    co_IntSetForegroundWindow(Window);
                }
            }
        }
    }

    if PosChanged == 0
        && (WinPos.flags & SWP_FRAMECHANGED) != 0
        && (WinPos.flags & SWP_DEFERERASE) == 0
        && !VisAfter.is_null()
    {
        let Parent: PWND = (*Window).spwndParent;
        if ((*Window).style & WS_CHILD) == 0
            && !Parent.is_null()
            && ((*Parent).style & WS_CLIPCHILDREN) != 0
        {
            UserSyncAndPaintWindows(Parent, RDW_CLIPCHILDREN);
        }
    }

    if VisBefore.is_null()
        && VisBeforeJustClient.is_null()
        && ((*Window).ExStyle & WS_EX_TOPMOST) == 0
        && (WinPos.flags & SWP_AGG_STATUSFLAGS) == (SWP_AGG_NOPOSCHANGE & !SWP_NOZORDER)
    {
        WinPos.flags |= SWP_NOZORDER | SWP_NOREDRAW;
    }

    if !VisBefore.is_null() {
        REGION_Delete(VisBefore);
        VisBefore = core::ptr::null_mut();
    }
    if !VisBeforeJustClient.is_null() {
        REGION_Delete(VisBeforeJustClient);
        VisBeforeJustClient = core::ptr::null_mut();
    }
    if !VisAfter.is_null() {
        REGION_Delete(VisAfter);
        VisAfter = core::ptr::null_mut();
    }
    if !CopyRgn.is_null() {
        REGION_Delete(CopyRgn);
        CopyRgn = core::ptr::null_mut();
    }

    if (flags & SWP_DEFERERASE) == 0 {
        if (flags & SWP_HIDEWINDOW) != 0
            || ((flags & SWP_SHOWWINDOW) == 0
                && (WinPos.flags & SWP_AGG_STATUSFLAGS) != SWP_AGG_NOGEOMETRYCHANGE)
        {
            let mut Parent: PWND = (*Window).spwndParent;
            if Parent.is_null() || UserIsDesktopWindow(Parent) != 0 {
                Parent = Window;
            }
            UserSyncAndPaintWindows(Parent, RDW_ERASENOW);
        }

        if (WinPos.flags & SWP_AGG_STATUSFLAGS) != SWP_AGG_NOPOSCHANGE
            && (flags & SWP_AGG_NOCLIENTCHANGE) == 0
            && (flags & SWP_SHOWWINDOW) != 0
        {
            UserSyncAndPaintWindows(Window, RDW_ERASENOW);
        }
    }

    if (WinPos.flags & SWP_AGG_STATUSFLAGS) != SWP_AGG_NOPOSCHANGE
        && !((flags & SWP_AGG_NOCLIENTCHANGE) != 0 && (flags & SWP_SHOWWINDOW) != 0)
    {
        WinPos.x = NewWindowRect.left;
        WinPos.y = NewWindowRect.top;
        WinPos.cx = NewWindowRect.right - NewWindowRect.left;
        WinPos.cy = NewWindowRect.bottom - NewWindowRect.top;
        co_IntSendMessageNoWait(
            WinPos.hwnd,
            WM_WINDOWPOSCHANGED,
            0,
            &mut WinPos as *mut WINDOWPOS as LPARAM,
        );
    }

    if (WinPos.flags & SWP_FRAMECHANGED) != 0
        || (WinPos.flags & SWP_STATECHANGED) != 0
        || (WinPos.flags & SWP_NOCLIENTSIZE) == 0
        || (WinPos.flags & SWP_NOCLIENTMOVE) == 0
    {
        let pWnd: PWND = ValidateHwndNoErr(WinPos.hwnd);
        if !pWnd.is_null() {
            IntNotifyWinEvent(EVENT_OBJECT_LOCATIONCHANGE, pWnd, OBJID_WINDOW as LONG, CHILDID_SELF as LONG, WEF_SETBYWNDPTI);
        }
    }

    if (WinPos.flags & (SWP_NOMOVE | SWP_NOSIZE)) != (SWP_NOMOVE | SWP_NOSIZE) {
        if IS_IMM_MODE() != 0 {
            IntImeWindowPosChanged();
        }
    }

    if bPointerInWindow != IntPtInWindow(Window, (*pusi(gpsi)).ptCursor.x, (*pusi(gpsi)).ptCursor.y) {
        let mut msg: MSG = core::mem::zeroed();
        msg.message = WM_MOUSEMOVE;
        msg.wParam = UserGetMouseButtonsState() as WPARAM;
        msg.lParam = MAKELPARAM((*pusi(gpsi)).ptCursor.x, (*pusi(gpsi)).ptCursor.y) as LPARAM;
        msg.pt = (*pusi(gpsi)).ptCursor;
        co_MsqInsertMouseMessage(&mut msg, 0, 0, 1);
    }

    1u8
}
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn co_WinPosGetNonClientSize(
    Window: PWND,
    WindowRect: *mut RECT,
    ClientRect: *mut RECT,
) -> LRESULT {
    let Result: LRESULT;

    *ClientRect = *WindowRect;
    Result = co_IntSendMessageNoWait(
        UserHMGetHandle(Window),
        WM_NCCALCSIZE,
        0 as WPARAM,
        ClientRect as LPARAM,
    );

    FixClientRect(ClientRect as PRECTL, WindowRect as PRECTL);

    Result
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn co_WinPosGetNonClientSize(
    Window: PWND,
    WindowRect: *mut RECT,
    ClientRect: *mut RECT,
) -> LRESULT {
    let Result: LRESULT;

    *ClientRect = *WindowRect;
    Result = co_IntSendMessageNoWait(
        UserHMGetHandle(Window),
        WM_NCCALCSIZE,
        0 as WPARAM,
        ClientRect as LPARAM,
    );

    FixClientRect(ClientRect as PRECTL, WindowRect as PRECTL);

    Result
}

fastcall_fn! {
#[no_mangle]
unsafe fn co_WinPosSendSizeMove(Wnd: PWND) {
    let mut Rect: RECTL = core::mem::zeroed();
    let mut lParam: LPARAM;
    let mut wParam: WPARAM = SIZE_RESTORED as WPARAM;

    IntGetClientRect(Wnd, &mut Rect);
    lParam = MAKELONG((Rect.right - Rect.left) as u32, (Rect.bottom - Rect.top) as u32) as LPARAM;

    (*Wnd).state &= !WNDS_SENDSIZEMOVEMSGS;

    if (*Wnd).style & WS_MAXIMIZE != 0 {
        wParam = SIZE_MAXIMIZED as WPARAM;
    } else if (*Wnd).style & WS_MINIMIZE != 0 {
        wParam = SIZE_MINIMIZED as WPARAM;
        lParam = 0;
    }

    co_IntSendMessageNoWait(UserHMGetHandle(Wnd), WM_SIZE, wParam, lParam);

    if UserIsDesktopWindow((*Wnd).spwndParent) != 0 {
        lParam = MAKELONG((*Wnd).rcClient.left as u32, (*Wnd).rcClient.top as u32) as LPARAM;
    } else {
        lParam = MAKELONG(
            ((*Wnd).rcClient.left - (*(*Wnd).spwndParent).rcClient.left) as u32,
            ((*Wnd).rcClient.top - (*(*Wnd).spwndParent).rcClient.top) as u32,
        ) as LPARAM;
    }

    co_IntSendMessageNoWait(UserHMGetHandle(Wnd), WM_MOVE, 0, lParam);

    IntEngWindowChanged(Wnd, WOC_RGN_CLIENT);
}
}

fastcall_fn! {
#[cfg_attr(target_arch = "x86", export_name = "co_WinPosMinMaximize")]
#[no_mangle]
unsafe fn co_WinPosMinMaximize(
    Wnd: PWND,
    ShowFlag: UINT,
    NewPos: *mut RECTL,
) -> UINT {
    let mut Size: POINT = core::mem::zeroed();
    let mut wpl: WINDOWPLACEMENT = core::mem::zeroed();
    let old_style: LONG;
    let mut SwpFlags: UINT = 0;

    wpl.length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
    IntGetWindowPlacement(Wnd, &mut wpl);

    if co_HOOK_CallHooks(
        WH_CBT as i32,
        HCBT_MINMAX as i32,
        UserHMGetHandle(Wnd) as WPARAM,
        ShowFlag as LPARAM,
    ) != 0
    {
        ERR!("WinPosMinMaximize WH_CBT Call Hook return!\n");
        return SWP_NOSIZE | SWP_NOMOVE;
    }

    if (*Wnd).style & WS_MINIMIZE != 0 {
        match ShowFlag {
            SW_MINIMIZE | SW_SHOWMINNOACTIVE | SW_SHOWMINIMIZED | SW_FORCEMINIMIZE => {
                return SWP_NOSIZE | SWP_NOMOVE;
            }
            _ => {}
        }
        if co_IntSendMessageNoWait(UserHMGetHandle(Wnd), WM_QUERYOPEN, 0, 0) == 0 {
            return SWP_NOSIZE | SWP_NOMOVE;
        }
        SwpFlags |= SWP_NOCOPYBITS;
    }

    match ShowFlag {
        SW_MINIMIZE | SW_SHOWMINNOACTIVE | SW_SHOWMINIMIZED | SW_FORCEMINIMIZE => {
            if (*Wnd).style & WS_MAXIMIZE != 0 {
                (*Wnd).InternalPos.flags |= WPF_RESTORETOMAXIMIZED;
            } else {
                (*Wnd).InternalPos.flags &= !WPF_RESTORETOMAXIMIZED;
            }

            old_style = IntSetStyle(Wnd, WS_MINIMIZE as u32, WS_MAXIMIZE as u32) as LONG;

            co_UserRedrawWindow(
                Wnd,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                RDW_VALIDATE | RDW_NOERASE | RDW_NOINTERNALPAINT,
            );

            if (*Wnd).InternalPos.flags & WPF_SETMINPOSITION == 0 {
                (*Wnd).InternalPos.flags &= !WPF_MININIT;
            }

            WinPosFindIconPos(Wnd, (&mut wpl.ptMinPosition as *mut _POINTL).cast());

            if old_style & WS_MINIMIZE as i32 == 0 {
                SwpFlags |= SWP_STATECHANGED;
                IntShowOwnedPopups(Wnd, 0);
            }

            RECTL_vSetRect(
                NewPos as *mut RECT,
                wpl.ptMinPosition.x,
                wpl.ptMinPosition.y,
                wpl.ptMinPosition.x + UserGetSystemMetrics(SM_CXMINIMIZED as ULONG),
                wpl.ptMinPosition.y + UserGetSystemMetrics(SM_CYMINIMIZED as ULONG),
            );
            SwpFlags |= SWP_NOCOPYBITS;
        }

        SW_MAXIMIZE => {
            IntSetSnapEdge(Wnd, HTNOWHERE);
            if (*Wnd).style & WS_MAXIMIZE != 0 && (*Wnd).style & WS_VISIBLE != 0 {
                SwpFlags = SWP_NOSIZE | SWP_NOMOVE;
            } else {
                co_WinPosGetMinMaxInfo(
                    Wnd,
                    &mut Size,
                    (&mut wpl.ptMaxPosition as *mut _POINTL).cast(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );

                old_style = IntSetStyle(Wnd, WS_MAXIMIZE as u32, WS_MINIMIZE as u32) as LONG;

                if old_style & WS_MAXIMIZE as i32 == 0 {
                    SwpFlags |= SWP_STATECHANGED;
                }
                RECTL_vSetRect(
                    NewPos as *mut RECT,
                    wpl.ptMaxPosition.x,
                    wpl.ptMaxPosition.y,
                    Size.x,
                    Size.y,
                );
            }
        }

        SW_SHOWNOACTIVATE | SW_SHOWNORMAL | SW_RESTORE | SW_SHOWDEFAULT => {
            if ShowFlag == SW_SHOWNOACTIVATE {
                (*Wnd).InternalPos.flags &= !WPF_RESTORETOMAXIMIZED;
            }

            old_style = IntSetStyle(Wnd, 0, (WS_MINIMIZE | WS_MAXIMIZE) as u32) as LONG;
            if old_style & WS_MINIMIZE as i32 != 0 {
                IntShowOwnedPopups(Wnd, 1);

                if (*Wnd).InternalPos.flags & WPF_RESTORETOMAXIMIZED != 0 {
                    co_WinPosGetMinMaxInfo(
                        Wnd,
                        &mut Size,
                        (&mut wpl.ptMaxPosition as *mut _POINTL).cast(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                    IntSetStyle(Wnd, WS_MAXIMIZE as u32, 0);
                    SwpFlags |= SWP_STATECHANGED;
                    RECTL_vSetRect(
                        NewPos as *mut RECT,
                        wpl.ptMaxPosition.x,
                        wpl.ptMaxPosition.y,
                        wpl.ptMaxPosition.x + Size.x,
                        wpl.ptMaxPosition.y + Size.y,
                    );
                } else {
                    *NewPos = core::mem::transmute_copy(&wpl.rcNormalPosition);
                    if ShowFlag != SW_SHOWNORMAL && ShowFlag != SW_SHOWDEFAULT {
                        let edge = IntGetWindowSnapEdge(Wnd);
                        if edge != 0 {
                            co_IntCalculateSnapPosition(Wnd, edge, NewPos as *mut RECT);
                        }
                    }
                    (*NewPos).right -= (*NewPos).left;
                    (*NewPos).bottom -= (*NewPos).top;
                }
            } else {
                if old_style & WS_MAXIMIZE as i32 != 0 {
                    SwpFlags |= SWP_STATECHANGED;
                    (*Wnd).InternalPos.flags &= !WPF_RESTORETOMAXIMIZED;
                    *NewPos = core::mem::transmute_copy(&wpl.rcNormalPosition);
                    (*NewPos).right -= (*NewPos).left;
                    (*NewPos).bottom -= (*NewPos).top;
                }
            }
        }

        _ => {}
    }

    SwpFlags
}
}

unsafe fn IntForceMinimizeWindow(pWnd: PWND) {
    let hRgn: HRGN;
    let pRgn: PREGION;

    if ((*pWnd).style & (WS_MINIMIZE | WS_VISIBLE)) != WS_VISIBLE {
        return;
    }

    if ((*pWnd).state & WNDS_DESTROYED) != 0 {
        return;
    }

    (*pWnd).ExStyle &= !WS_EX_MAKEVISIBLEWHENUNGHOSTED;

    IntSetStyle(pWnd, 0, WS_VISIBLE as u32);

    hRgn = GreCreateRectRgnIndirect(core::ptr::addr_of_mut!((*pWnd).rcWindow));
    pRgn = REGION_LockRgn(hRgn);
    co_UserRedrawWindow(
        UserGetDesktopWindow(),
        core::ptr::null_mut(),
        pRgn,
        (RDW_ALLCHILDREN | RDW_ERASE | RDW_INVALIDATE) as u32,
    );
    REGION_UnlockRgn(pRgn);
    GreDeleteObject(hRgn as _);

    if (*pWnd).spwndParent == (*(*(*pWnd).head.rpdesk).pDeskInfo).spwnd {
        co_WinPosActivateOtherWindow(pWnd);
    }
}

fastcall_fn! {
#[cfg_attr(target_arch = "x86", export_name = "co_WinPosShowWindow")]
#[no_mangle]
unsafe fn co_WinPosShowWindow(Wnd: PWND, mut Cmd: INT) -> BOOLEAN {
    let WasVisible: BOOLEAN;
    let mut Swp: UINT = 0;
    let mut EventMsg: UINT = 0;
    let mut NewPos: RECTL = RECTL { left: 0, top: 0, right: 0, bottom: 0 };
    let ShowFlag: BOOLEAN;
    let style: LONG;
    let mut Parent: PWND;
    let pti: PTHREADINFO;
    let mut ShowOwned: BOOL = 0;
    let mut FirstTime: BOOL = 0;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    WasVisible = if ((*Wnd).style & WS_VISIBLE as u32) != 0 { 1 } else { 0 };
    style = (*Wnd).style as LONG;

    if (*(*pti).ppi).usi.dwFlags & STARTF_USESHOWWINDOW != 0 {
        if ((*Wnd).style & (WS_POPUP | WS_CHILD) as u32) != WS_CHILD as u32 {
            if ((*Wnd).style & WS_CAPTION as u32) == WS_CAPTION as u32 {
                if (*Wnd).spwndOwner.is_null() {
                    if Cmd == SW_SHOWNORMAL as i32 || Cmd == SW_SHOW as i32 {
                        Cmd = SW_SHOWDEFAULT as i32;
                    }
                    FirstTime = 1;
                }
            }
        }
    }

    if Cmd == SW_SHOWDEFAULT as i32 {
        if (*(*pti).ppi).usi.dwFlags & STARTF_USESHOWWINDOW != 0 {
            Cmd = (*(*pti).ppi).usi.wShowWindow as INT;
            FirstTime = 1;
        }
    }

    if FirstTime != 0 {
        (*(*pti).ppi).usi.dwFlags &=
            !(STARTF_USEPOSITION | STARTF_USESIZE | STARTF_USESHOWWINDOW);
    }

    match Cmd as u32 {
        x if x == SW_HIDE as u32 => {
            if WasVisible == 0 {
                return 0;
            }
            Swp |= SWP_HIDEWINDOW | SWP_NOSIZE | SWP_NOMOVE;
            if Wnd != (*(*pti).MessageQueue).spwndActive {
                Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
            }
        }
        x if x == SW_FORCEMINIMIZE as u32 => {
            IntForceMinimizeWindow(Wnd);
            return WasVisible;
        }
        x if x == SW_SHOWMINNOACTIVE as u32
            || x == SW_SHOWMINIMIZED as u32
            || x == SW_MINIMIZE as u32 =>
        {
            if Cmd == SW_SHOWMINNOACTIVE as i32 {
                Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
            }
            Swp |= SWP_SHOWWINDOW;
            Swp |= SWP_NOACTIVATE;
            if (style & WS_MINIMIZE as LONG) == 0 {
                IntShowOwnedPopups(Wnd, 0);
                if Cmd == SW_SHOWMINIMIZED as i32 {
                    if (style & (WS_CHILD | WS_POPUP) as LONG) == WS_CHILD as LONG {
                        co_UserSetFocus((*Wnd).spwndParent);
                    } else {
                        co_UserSetFocus(core::ptr::null_mut());
                    }
                }

                Swp |= co_WinPosMinMaximize(Wnd, Cmd as UINT, &mut NewPos as *mut RECTL);

                EventMsg = EVENT_SYSTEM_MINIMIZESTART;
            } else {
                if WasVisible != 0 {
                    return 1;
                }
                Swp |= SWP_NOSIZE | SWP_NOMOVE;
            }
        }
        x if x == SW_SHOWMAXIMIZED as u32 => {
            Swp |= SWP_SHOWWINDOW;
            if (style & WS_MAXIMIZE as LONG) == 0 {
                ShowOwned = 1;

                Swp |= co_WinPosMinMaximize(Wnd, SW_MAXIMIZE as UINT, &mut NewPos as *mut RECTL);

                EventMsg = EVENT_SYSTEM_MINIMIZEEND;
            } else {
                if WasVisible != 0 {
                    return 1;
                }
                Swp |= SWP_NOSIZE | SWP_NOMOVE;
            }
        }
        x if x == SW_SHOWNA as u32 => {
            Swp |= SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_NOSIZE | SWP_NOMOVE;
            if (style & WS_CHILD as LONG) != 0 && ((*Wnd).ExStyle & WS_EX_MDICHILD) == 0 {
                Swp |= SWP_NOZORDER;
            }
        }
        x if x == SW_SHOW as u32 => {
            if WasVisible != 0 {
                return 1;
            }
            Swp |= SWP_SHOWWINDOW | SWP_NOSIZE | SWP_NOMOVE;
            if (style & WS_CHILD as LONG) != 0 && ((*Wnd).ExStyle & WS_EX_MDICHILD) == 0 {
                Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
            }
        }
        x if x == SW_SHOWNOACTIVATE as u32
            || x == SW_SHOWNORMAL as u32
            || x == SW_SHOWDEFAULT as u32
            || x == SW_RESTORE as u32 =>
        {
            if Cmd == SW_SHOWNOACTIVATE as i32 {
                Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
            }
            if WasVisible == 0 {
                Swp |= SWP_SHOWWINDOW;
            }
            if (style & (WS_MINIMIZE | WS_MAXIMIZE) as LONG) != 0 {
                Swp |= co_WinPosMinMaximize(Wnd, Cmd as UINT, &mut NewPos as *mut RECTL);
                if (style & WS_MINIMIZE as LONG) != 0 {
                    EventMsg = EVENT_SYSTEM_MINIMIZEEND;
                }
            } else {
                if WasVisible != 0 {
                    return 1;
                }
                Swp |= SWP_NOSIZE | SWP_NOMOVE;
            }
            if (style & WS_CHILD as LONG) != 0
                && ((*Wnd).ExStyle & WS_EX_MDICHILD) == 0
                && (Swp & SWP_STATECHANGED) == 0
            {
                Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
            }
        }
        _ => {
            return 0;
        }
    }

    ShowFlag = if Cmd != SW_HIDE as i32 { 1 } else { 0 };

    if (ShowFlag != WasVisible || Cmd == SW_SHOWNA as i32)
        && Cmd != SW_SHOWMAXIMIZED as i32
        && (Swp & SWP_STATECHANGED) == 0
    {
        co_IntSendMessageNoWait(
            UserHMGetHandle(Wnd),
            WM_SHOWWINDOW,
            ShowFlag as WPARAM,
            0,
        );
        if VerifyWnd(Wnd).is_null() {
            return WasVisible;
        }
    }

    if ((*Wnd).style & WS_CHILD as u32) != 0
        && ((*Wnd).ExStyle & WS_EX_MDICHILD) == 0
        && Cmd != SW_SHOWNA as i32
    {
        Swp |= SWP_NOACTIVATE | SWP_NOZORDER;
    }

    if IsChildVisible(Wnd) != 0 || (Swp & SWP_STATECHANGED) != 0 {
        co_WinPosSetWindowPos(
            Wnd,
            if ((*Wnd).ExStyle & WS_EX_TOPMOST) != 0 {
                HWND_TOPMOST
            } else {
                HWND_TOP
            },
            NewPos.left,
            NewPos.top,
            NewPos.right,
            NewPos.bottom,
            LOWORD(Swp) as UINT,
        );
    } else {
        if ShowFlag != 0 {
            IntSetStyle(Wnd, WS_VISIBLE as u32, 0);
        } else {
            IntSetStyle(Wnd, 0, WS_VISIBLE as u32);
        }
    }

    if EventMsg != 0 {
        IntNotifyWinEvent(
            EventMsg,
            Wnd,
            OBJID_WINDOW as LONG,
            CHILDID_SELF as LONG,
            WEF_SETBYWNDPTI,
        );
    }

    if ShowOwned != 0 {
        IntShowOwnedPopups(Wnd, 1);
    }

    if Cmd == SW_HIDE as i32 || Cmd == SW_MINIMIZE as i32 {
        if Wnd == (*(*pti).MessageQueue).spwndActive
            && (*pti).MessageQueue == IntGetFocusMessageQueue()
        {
            if UserIsDesktopWindow((*Wnd).spwndParent) != 0 {
                if ActivateOtherWindowMin(Wnd) == 0 {
                    co_WinPosActivateOtherWindow(Wnd);
                }
            } else {
                co_WinPosActivateOtherWindow(Wnd);
            }
        }

        if Wnd == (*(*pti).MessageQueue).spwndFocus {
            Parent = (*Wnd).spwndParent;
            if UserIsDesktopWindow((*Wnd).spwndParent) != 0 {
                Parent = core::ptr::null_mut();
            }
            co_UserSetFocus(Parent);
        }
        if Cmd == SW_HIDE as i32 {
            return WasVisible;
        }
    }

    if ((*Wnd).state & WNDS_SENDSIZEMOVEMSGS) != 0
        && ((*Wnd).state2 & WNDS2_INDESTROY) == 0
    {
        co_WinPosSendSizeMove(Wnd);
    }

    if (style & WS_MINIMIZE as LONG) != 0 {
        co_UserSetFocus(Wnd);
        if (style & WS_CHILD as LONG) == 0 {
            co_IntSendMessage(
                UserHMGetHandle(Wnd),
                WM_ACTIVATE,
                WA_ACTIVE as WPARAM,
                0,
            );
        }
    }

    WasVisible
}
}

unsafe fn co_WinPosSearchChildren(
    ScopeWin: PWND,
    Point: *mut POINT,
    HitTest: *mut USHORT,
    Ignore: BOOL,
) -> PWND {
    let mut List: *mut HWND;
    let mut phWnd: *mut HWND;
    let mut pwndChild: PWND = core::ptr::null_mut();

    /* not visible */
    if ((*ScopeWin).style & WS_VISIBLE) == 0 {
        return core::ptr::null_mut();
    }

    /* not in window or in window region */
    if IntPtInWindow(ScopeWin, (*Point).x, (*Point).y) == 0 {
        return core::ptr::null_mut();
    }

    /* transparent */
    if ((*ScopeWin).ExStyle & (WS_EX_LAYERED | WS_EX_TRANSPARENT))
        == (WS_EX_LAYERED | WS_EX_TRANSPARENT)
    {
        return core::ptr::null_mut();
    }

    if Ignore == 0 && ((*ScopeWin).style & WS_DISABLED) != 0 {
        /* disabled child */
        if ((*ScopeWin).style & (WS_POPUP | WS_CHILD)) == WS_CHILD {
            return core::ptr::null_mut();
        }
        /* process the hit error */
        *HitTest = HTERROR as USHORT;
        return ScopeWin;
    }

    /* not minimized and check if point is inside the window */
    if ((*ScopeWin).style & WS_MINIMIZE) == 0
        && RECTL_bPointInRect(
            core::ptr::addr_of_mut!((*ScopeWin).rcClient),
            (*Point).x,
            (*Point).y,
        ) != 0
    {
        UserReferenceObject(ScopeWin as *mut _);

        List = IntWinListChildren(ScopeWin);
        if !List.is_null() {
            phWnd = List;
            while !(*phWnd).is_null() {
                pwndChild = ValidateHwndNoErr(*phWnd);
                if pwndChild.is_null() {
                    phWnd = phWnd.add(1);
                    continue;
                }

                pwndChild = co_WinPosSearchChildren(pwndChild, Point, HitTest, Ignore);

                if !pwndChild.is_null() {
                    ExFreePoolWithTag(List as *mut _, USERTAG_WINDOWLIST);
                    UserDereferenceObject(ScopeWin as *mut _);
                    return pwndChild;
                }
                phWnd = phWnd.add(1);
            }
            ExFreePoolWithTag(List as *mut _, USERTAG_WINDOWLIST);
        }
        UserDereferenceObject(ScopeWin as *mut _);
    }

    if head_pti(ScopeWin) == PsGetCurrentThreadWin32Thread() as PTHREADINFO {
        *HitTest = co_IntSendMessage(
            UserHMGetHandle(ScopeWin),
            WM_NCHITTEST,
            0,
            MAKELONG((*Point).x as u32, (*Point).y as u32) as LPARAM,
        ) as USHORT;

        if *HitTest == HTTRANSPARENT as USHORT {
            return core::ptr::null_mut();
        }
    } else {
        if *HitTest == HTNOWHERE as USHORT && pwndChild.is_null() {
            *HitTest = HTCLIENT as USHORT;
        }
    }

    ScopeWin
}

#[no_mangle]
unsafe extern "system" fn co_WinPosWindowFromPoint(
    mut ScopeWin: PWND,
    WinPoint: *mut POINT,
    HitTest: *mut USHORT,
    Ignore: BOOL,
) -> PWND {
    let Window: PWND;
    let mut Point: POINT = *WinPoint;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    if ScopeWin.is_null() {
        ScopeWin = UserGetDesktopWindow();
        if ScopeWin.is_null() {
            return core::ptr::null_mut();
        }
    }

    *HitTest = HTNOWHERE as USHORT;

    UserRefObjectCo(ScopeWin, &mut Ref);

    Window = co_WinPosSearchChildren(ScopeWin, &mut Point, HitTest, Ignore);

    UserDerefObjectCo(ScopeWin);

    Window
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntRealChildWindowFromPoint(Parent: PWND, x: LONG, y: LONG) -> PWND {
    IntRealChildWindowFromPoint_impl(Parent, x, y)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntRealChildWindowFromPoint(Parent: PWND, x: LONG, y: LONG) -> PWND {
    IntRealChildWindowFromPoint_impl(Parent, x, y)
}

unsafe fn IntRealChildWindowFromPoint_impl(Parent: PWND, x: LONG, y: LONG) -> PWND {
    let mut Pt: POINTL = core::mem::zeroed();
    let List: *mut HWND;
    let mut phWnd: *mut HWND;
    let mut pwndHit: PWND = core::ptr::null_mut();

    Pt.x = x;
    Pt.y = y;

    if UserIsDesktopWindow(Parent) == 0 {
        Pt.x += (*Parent).rcClient.left;
        Pt.y += (*Parent).rcClient.top;
    }

    if IntPtInWindow(Parent, Pt.x, Pt.y) == 0 {
        return core::ptr::null_mut();
    }

    List = IntWinListChildren(Parent);
    if !List.is_null() {
        phWnd = List;
        while !(*phWnd).is_null() {
            let Child: PWND = ValidateHwndNoErr(*phWnd);
            if !Child.is_null() {
                if (*Child).style & WS_VISIBLE != 0 && IntPtInWindow(Child, Pt.x, Pt.y) != 0 {
                    if (*(*Child).pcls).atomClassName != (*gpsi).atomSysClass[ICLS_BUTTON as usize]
                        || (*Child).style & BS_TYPEMASK != BS_GROUPBOX
                    {
                        ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
                        return Child;
                    }
                    pwndHit = Child;
                }
            }
            phWnd = phWnd.add(1);
        }
        ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
    }

    if !pwndHit.is_null() {
        pwndHit
    } else {
        Parent
    }
}

#[no_mangle]
pub unsafe extern "system" fn IntChildWindowFromPointEx(
    Parent: PWND,
    x: LONG,
    y: LONG,
    uiFlags: UINT,
) -> PWND {
    let mut Pt: POINTL = core::mem::zeroed();
    let mut pwndHit: PWND = core::ptr::null_mut();

    Pt.x = x;
    Pt.y = y;

    if UserIsDesktopWindow(Parent) == 0 {
        if (*Parent).ExStyle & WS_EX_LAYOUTRTL != 0 {
            Pt.x = (*Parent).rcClient.right - Pt.x;
        } else {
            Pt.x += (*Parent).rcClient.left;
        }
        Pt.y += (*Parent).rcClient.top;
    }

    if IntPtInWindow(Parent, Pt.x, Pt.y) == 0 {
        return core::ptr::null_mut();
    }

    let List = IntWinListChildren(Parent);
    if !List.is_null() {
        let mut phWnd = List;
        while !(*phWnd).is_null() {
            let Child = ValidateHwndNoErr(*phWnd);
            if !Child.is_null() {
                if uiFlags & (CWP_SKIPINVISIBLE | CWP_SKIPDISABLED) != 0 {
                    if (*Child).style & WS_VISIBLE == 0 && (uiFlags & CWP_SKIPINVISIBLE) != 0 {
                        phWnd = phWnd.add(1);
                        continue;
                    }
                    if (*Child).style & WS_DISABLED != 0 && (uiFlags & CWP_SKIPDISABLED) != 0 {
                        phWnd = phWnd.add(1);
                        continue;
                    }
                }

                if uiFlags & CWP_SKIPTRANSPARENT != 0 {
                    if (*Child).ExStyle & WS_EX_TRANSPARENT != 0 {
                        phWnd = phWnd.add(1);
                        continue;
                    }
                }

                if IntPtInWindow(Child, Pt.x, Pt.y) != 0 {
                    pwndHit = Child;
                    break;
                }
            }
            phWnd = phWnd.add(1);
        }
        ExFreePoolWithTag(List as *mut _, USERTAG_WINDOWLIST);
    }

    if !pwndHit.is_null() {
        pwndHit
    } else {
        Parent
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntDeferWindowPos(
    hdwp: HDWP,
    hwnd: HWND,
    hwndAfter: HWND,
    x: INT,
    y: INT,
    cx: INT,
    cy: INT,
    flags: UINT,
) -> HDWP {
    IntDeferWindowPos_impl(hdwp, hwnd, hwndAfter, x, y, cx, cy, flags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntDeferWindowPos(
    hdwp: HDWP,
    hwnd: HWND,
    hwndAfter: HWND,
    x: INT,
    y: INT,
    cx: INT,
    cy: INT,
    flags: UINT,
) -> HDWP {
    IntDeferWindowPos_impl(hdwp, hwnd, hwndAfter, x, y, cx, cy, flags)
}

unsafe fn IntDeferWindowPos_impl(
    hdwp: HDWP,
    hwnd: HWND,
    hwndAfter: HWND,
    x: INT,
    y: INT,
    cx: INT,
    cy: INT,
    flags: UINT,
) -> HDWP {
    let pDWP: PSMWP;
    let mut i: i32;
    let mut retvalue: HDWP = hdwp;

    if (flags
        & !(SWP_NOSIZE
            | SWP_NOMOVE
            | SWP_NOZORDER
            | SWP_NOREDRAW
            | SWP_NOACTIVATE
            | SWP_NOCOPYBITS
            | SWP_NOOWNERZORDER
            | SWP_SHOWWINDOW
            | SWP_HIDEWINDOW
            | SWP_FRAMECHANGED))
        != 0
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    pDWP = UserGetObject(gHandleTable, hdwp as *mut _, TYPE_SETWINDOWPOS as _) as PSMWP;
    if pDWP.is_null() {
        EngSetLastError(ERROR_INVALID_DWP_HANDLE);
        return core::ptr::null_mut();
    }

    'end: {
        i = 0;
        while i < (*pDWP).ccvr {
            if (*pDWP).acvr.offset(i as isize).as_ref().unwrap().pos.hwnd == hwnd {
                let acvr = (*pDWP).acvr.offset(i as isize);
                if (flags & SWP_NOZORDER) == 0 {
                    (*acvr).pos.hwndInsertAfter = hwndAfter;
                }
                if (flags & SWP_NOMOVE) == 0 {
                    (*acvr).pos.x = x;
                    (*acvr).pos.y = y;
                }
                if (flags & SWP_NOSIZE) == 0 {
                    (*acvr).pos.cx = cx;
                    (*acvr).pos.cy = cy;
                }
                (*acvr).pos.flags &= flags
                    | !(SWP_NOSIZE
                        | SWP_NOMOVE
                        | SWP_NOZORDER
                        | SWP_NOREDRAW
                        | SWP_NOACTIVATE
                        | SWP_NOCOPYBITS
                        | SWP_NOOWNERZORDER);
                (*acvr).pos.flags |=
                    flags & (SWP_SHOWWINDOW | SWP_HIDEWINDOW | SWP_FRAMECHANGED);
                break 'end;
            }
            i += 1;
        }

        if (*pDWP).ccvr >= (*pDWP).ccvrAlloc {
            let newpos = ExAllocatePoolWithTag(
                PagedPool,
                ((*pDWP).ccvrAlloc as usize * 2 * core::mem::size_of::<CVR>()) as SIZE_T,
                USERTAG_SWP,
            ) as PCVR;
            if newpos.is_null() {
                retvalue = core::ptr::null_mut();
                break 'end;
            }
            RtlZeroMemory(
                newpos as *mut _,
                (*pDWP).ccvrAlloc as usize * 2 * core::mem::size_of::<CVR>(),
            );
            RtlCopyMemory(
                newpos as *mut _,
                (*pDWP).acvr as *mut _,
                ((*pDWP).ccvrAlloc as usize * core::mem::size_of::<CVR>()) as SIZE_T,
            );
            ExFreePoolWithTag((*pDWP).acvr as *mut _, USERTAG_SWP);
            (*pDWP).ccvrAlloc *= 2;
            (*pDWP).acvr = newpos;
        }

        let acvr = (*pDWP).acvr.offset((*pDWP).ccvr as isize);
        (*acvr).pos.hwnd = hwnd;
        (*acvr).pos.hwndInsertAfter = hwndAfter;
        (*acvr).pos.x = x;
        (*acvr).pos.y = y;
        (*acvr).pos.cx = cx;
        (*acvr).pos.cy = cy;
        (*acvr).pos.flags = flags;
        (*acvr).hrgnClip = core::ptr::null_mut();
        (*acvr).hrgnInterMonitor = core::ptr::null_mut();
        (*pDWP).ccvr += 1;
    }

    retvalue
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntEndDeferWindowPosEx(hdwp: HDWP, bAsync: BOOL) -> BOOL {
    IntEndDeferWindowPosEx_impl(hdwp, bAsync)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntEndDeferWindowPosEx(hdwp: HDWP, bAsync: BOOL) -> BOOL {
    IntEndDeferWindowPosEx_impl(hdwp, bAsync)
}

unsafe fn IntEndDeferWindowPosEx_impl(hdwp: HDWP, bAsync: BOOL) -> BOOL {
    let mut res: BOOL = 1;
    let mut i: INT;

    let pDWP = UserGetObject(gHandleTable, hdwp as _, TYPE_SETWINDOWPOS) as PSMWP;
    if pDWP.is_null() {
        EngSetLastError(ERROR_INVALID_DWP_HANDLE);
        return 0;
    }

    i = 0;
    let mut winpos: PCVR = (*pDWP).acvr;
    while res != 0 && i < (*pDWP).ccvr {
        let pwnd: PWND = ValidateHwndNoErr((*winpos).pos.hwnd);
        if pwnd.is_null() {
            i += 1;
            winpos = winpos.add(1);
            continue;
        }

        let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
        UserRefObjectCo(pwnd, &mut Ref);

        if bAsync != 0 {
            let ppos = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<WINDOWPOS>() as SIZE_T,
                USERTAG_SWP,
            ) as PWINDOWPOS;
            if !ppos.is_null() {
                *ppos = (*winpos).pos;
                let lRes = co_IntSendMessageNoWait(
                    (*winpos).pos.hwnd,
                    WM_ASYNC_SETWINDOWPOS,
                    0,
                    ppos as LPARAM,
                );
                if lRes == 0 {
                    ExFreePoolWithTag(ppos as _, USERTAG_SWP);
                }
            }
        } else {
            res = co_WinPosSetWindowPos(
                pwnd,
                (*winpos).pos.hwndInsertAfter,
                (*winpos).pos.x,
                (*winpos).pos.y,
                (*winpos).pos.cx,
                (*winpos).pos.cy,
                (*winpos).pos.flags,
            ) as BOOL;
        }

        if res != 0
            && ((*winpos).pos.flags & (SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER)) == SWP_NOZORDER
        {
            EngSetLastError(ERROR_SUCCESS);
        }

        UserDerefObjectCo(pwnd);

        i += 1;
        winpos = winpos.add(1);
    }

    ExFreePoolWithTag((*pDWP).acvr as _, USERTAG_SWP);
    UserDereferenceObject(pDWP as _);
    UserDeleteObject(hdwp as _, TYPE_SETWINDOWPOS);
    res
}
#[no_mangle]
pub unsafe extern "system" fn NtUserChildWindowFromPointEx(
    hwndParent: HWND,
    x: LONG,
    y: LONG,
    uiFlags: UINT,
) -> HWND {
    let mut pwndParent: PWND;
    UserEnterExclusive();
    pwndParent = UserGetWindowObject(hwndParent);
    if !pwndParent.is_null() {
        pwndParent = IntChildWindowFromPointEx(pwndParent, x, y, uiFlags);
    }
    UserLeave();
    if !pwndParent.is_null() {
        UserHMGetHandle(pwndParent)
    } else {
        core::ptr::null_mut()
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserEndDeferWindowPosEx(
    WinPosInfo: HDWP,
    bAsync: BOOL,
) -> BOOL {
    let Ret: BOOL;
    UserEnterExclusive();
    Ret = IntEndDeferWindowPosEx(WinPosInfo, bAsync);
    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDeferWindowPos(
    WinPosInfo: HDWP,
    Wnd: HWND,
    WndInsertAfter: HWND,
    x: INT,
    y: INT,
    cx: INT,
    cy: INT,
    Flags: UINT,
) -> HDWP {
    let pWnd: PWND;
    let pWndIA: PWND;
    let mut Ret: HDWP = core::ptr::null_mut();
    let Tmp: UINT = !(SWP_ASYNCWINDOWPOS
        | SWP_DEFERERASE
        | SWP_NOSENDCHANGING
        | SWP_NOREPOSITION
        | SWP_NOCOPYBITS
        | SWP_HIDEWINDOW
        | SWP_SHOWWINDOW
        | SWP_FRAMECHANGED
        | SWP_NOACTIVATE
        | SWP_NOREDRAW
        | SWP_NOZORDER
        | SWP_NOMOVE
        | SWP_NOSIZE);

    UserEnterExclusive();

    'block: {
        if Flags & Tmp != 0 {
            EngSetLastError(ERROR_INVALID_FLAGS);
            break 'block;
        }

        pWnd = UserGetWindowObject(Wnd);
        if pWnd.is_null() || UserIsDesktopWindow(pWnd) != 0 || UserIsMessageWindow(pWnd) != 0 {
            break 'block;
        }

        if !WndInsertAfter.is_null()
            && WndInsertAfter != HWND_BOTTOM
            && WndInsertAfter != HWND_TOPMOST
            && WndInsertAfter != HWND_NOTOPMOST
        {
            pWndIA = UserGetWindowObject(WndInsertAfter);
            if pWndIA.is_null() || UserIsDesktopWindow(pWndIA) != 0 || UserIsMessageWindow(pWndIA) != 0 {
                break 'block;
            }
        }

        Ret = IntDeferWindowPos(WinPosInfo, Wnd, WndInsertAfter, x, y, cx, cy, Flags);
    }

    UserLeave();
    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetInternalWindowPos(
    hWnd: HWND,
    rectWnd: LPRECT,
    ptIcon: LPPOINT,
) -> u32 {
    let Window: PWND;
    let mut Ret: u32 = 0;
    let mut Hit: u8 = 0u8;
    let mut wndpl: WINDOWPLACEMENT = core::mem::zeroed();
    let mut Status: i32;

    UserEnterShared();

    'exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            Hit = 0u8;
            break 'exit;
        }

        Status = STATUS_SUCCESS;
        if !rectWnd.is_null() {
            Status = SafeProbeForWrite(
                rectWnd as *mut core::ffi::c_void,
                core::mem::size_of::<RECT>() as u32,
                1,
            );
        }
        if NT_SUCCESS(Status) && !ptIcon.is_null() {
            Status = SafeProbeForWrite(
                ptIcon as *mut core::ffi::c_void,
                core::mem::size_of::<POINT>() as u32,
                1,
            );
        }
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Hit = 1u8;
        }

        wndpl.length = core::mem::size_of::<WINDOWPLACEMENT>() as u32;

        if IntGetWindowPlacement(Window, &mut wndpl) != 0 && Hit == 0 {
            if !rectWnd.is_null() {
                RtlCopyMemory(
                    rectWnd as *mut core::ffi::c_void,
                    core::ptr::addr_of!(wndpl.rcNormalPosition) as *const core::ffi::c_void,
                    core::mem::size_of::<RECT>() as SIZE_T,
                );
            }
            if !ptIcon.is_null() {
                RtlCopyMemory(
                    ptIcon as *mut core::ffi::c_void,
                    core::ptr::addr_of!(wndpl.ptMinPosition) as *const core::ffi::c_void,
                    core::mem::size_of::<POINT>() as SIZE_T,
                );
            }

            if Hit == 0 {
                Ret = wndpl.showCmd as u32;
            }
        }
    }

    UserLeave();
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetInternalWindowPos(
    hWnd: HWND,
    rectWnd: LPRECT,
    ptIcon: LPPOINT,
) -> u32 {
    let Window: PWND;
    let mut Ret: u32 = 0;
    let mut Hit: u8 = 0u8;
    let mut wndpl: WINDOWPLACEMENT = core::mem::zeroed();
    let mut Status: i32;

    UserEnterShared();

    'exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            Hit = 0u8;
            break 'exit;
        }

        Status = STATUS_SUCCESS;
        if !rectWnd.is_null() {
            Status = SafeProbeForWrite(
                rectWnd as *mut core::ffi::c_void,
                core::mem::size_of::<RECT>() as _,
                1,
            );
        }
        if NT_SUCCESS(Status) && !ptIcon.is_null() {
            Status = SafeProbeForWrite(
                ptIcon as *mut core::ffi::c_void,
                core::mem::size_of::<POINT>() as _,
                1,
            );
        }
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Hit = 1u8;
        }

        wndpl.length = core::mem::size_of::<WINDOWPLACEMENT>() as u32;

        if IntGetWindowPlacement(Window, &mut wndpl) != 0 && Hit == 0 {
            if !rectWnd.is_null() {
                RtlCopyMemory(
                    rectWnd as *mut core::ffi::c_void,
                    core::ptr::addr_of!(wndpl.rcNormalPosition) as *const core::ffi::c_void,
                    core::mem::size_of::<RECT>() as _,
                );
            }
            if !ptIcon.is_null() {
                RtlCopyMemory(
                    ptIcon as *mut core::ffi::c_void,
                    core::ptr::addr_of!(wndpl.ptMinPosition) as *const core::ffi::c_void,
                    core::mem::size_of::<POINT>() as _,
                );
            }

            if Hit == 0 {
                Ret = wndpl.showCmd as u32;
            }
        }
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetWindowPlacement(
    hWnd: HWND,
    lpwndpl: *mut WINDOWPLACEMENT,
) -> i32 {

    let Wnd: PWND;
    let mut Safepl: WINDOWPLACEMENT = core::mem::zeroed();
    let mut Status: i32;
    let mut Ret: i32 = 0;

    UserEnterShared();

    'Exit: {
        Wnd = UserGetWindowObject(hWnd);
        if Wnd.is_null() {
            break 'Exit;
        }

        Status = MmCopyFromCaller(
            core::ptr::addr_of_mut!(Safepl) as *mut core::ffi::c_void,
            lpwndpl as *const core::ffi::c_void,
            core::mem::size_of::<WINDOWPLACEMENT>(),
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'Exit;
        }

        Safepl.length = core::mem::size_of::<WINDOWPLACEMENT>() as u32;

        IntGetWindowPlacement(Wnd, core::ptr::addr_of_mut!(Safepl));

        Status = MmCopyToCaller(
            lpwndpl as *mut core::ffi::c_void,
            core::ptr::addr_of!(Safepl) as *const core::ffi::c_void,
            core::mem::size_of::<WINDOWPLACEMENT>(),
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'Exit;
        }

        Ret = 1;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserMinMaximize(
    hWnd: HWND,
    mut cmd: UINT,
    Hide: BOOL,
) -> DWORD {
    let pWnd: PWND;

    UserEnterExclusive();

    'exit: {
        pWnd = UserGetWindowObject(hWnd);
        if pWnd.is_null() || UserIsDesktopWindow(pWnd) != 0 || UserIsMessageWindow(pWnd) != 0 {
            break 'exit;
        }

        if cmd > SW_MAX as UINT || ((*pWnd).state2 & WNDS2_INDESTROY) != 0 {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'exit;
        }

        cmd |= if Hide != 0 { SW_HIDE as UINT } else { 0 };

        co_WinPosShowWindow(pWnd, cmd as INT);
    }

    UserLeave();
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserMoveWindow(
    hWnd: HWND,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    nWidth: core::ffi::c_int,
    nHeight: core::ffi::c_int,
    bRepaint: BOOL,
) -> BOOL {
    NtUserSetWindowPos(
        hWnd,
        core::ptr::null_mut(),
        X,
        Y,
        nWidth,
        nHeight,
        if bRepaint != 0 {
            SWP_NOZORDER | SWP_NOACTIVATE
        } else {
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOREDRAW
        },
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRealChildWindowFromPoint(
    Parent: HWND,
    x: LONG,
    y: LONG,
) -> HWND {
    let mut pwndParent: PWND;
    UserEnterShared();
    pwndParent = UserGetWindowObject(Parent);
    if !pwndParent.is_null() {
        pwndParent = IntRealChildWindowFromPoint(pwndParent, x, y);
    }
    UserLeave();
    if !pwndParent.is_null() {
        UserHMGetHandle(pwndParent)
    } else {
        core::ptr::null_mut()
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetWindowPos(
    hWnd: HWND,
    hWndInsertAfter: HWND,
    mut X: INT,
    mut Y: INT,
    mut cx: INT,
    mut cy: INT,
    uFlags: UINT,
) -> BOOL {
    let mut Window: PWND;
    let pWndIA: PWND;
    let mut ret: BOOL = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterExclusive();

    'Exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() || UserIsDesktopWindow(Window) != 0 || UserIsMessageWindow(Window) != 0 {
            break 'Exit;
        }

        if hWndInsertAfter != HWND_TOP
            && hWndInsertAfter != HWND_BOTTOM
            && hWndInsertAfter != HWND_TOPMOST
            && hWndInsertAfter != HWND_NOTOPMOST
        {
            pWndIA = UserGetWindowObject(hWndInsertAfter);
            if pWndIA.is_null()
                || UserIsDesktopWindow(pWndIA) != 0
                || UserIsMessageWindow(pWndIA) != 0
            {
                break 'Exit;
            }
        }

        if (uFlags & SWP_NOMOVE) == 0 {
            if X < -32768 {
                X = -32768;
            } else if X > 32767 {
                X = 32767;
            }
            if Y < -32768 {
                Y = -32768;
            } else if Y > 32767 {
                Y = 32767;
            }
        }
        if (uFlags & SWP_NOSIZE) == 0 {
            if cx < 0 {
                cx = 0;
            } else if cx > 32767 {
                cx = 32767;
            }
            if cy < 0 {
                cy = 0;
            } else if cy > 32767 {
                cy = 32767;
            }
        }

        UserRefObjectCo(Window, &mut Ref);
        ret = co_WinPosSetWindowPos(Window, hWndInsertAfter, X, Y, cx, cy, uFlags) as BOOL;
        UserDerefObjectCo(Window);
    }

    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetWindowRgn(
    hWnd: HWND,
    hRgn: HRGN,
    bRedraw: BOOL,
) -> INT {
    let mut hrgnCopy: HRGN = core::ptr::null_mut();
    let Window: PWND;
    let flags: INT = (SWP_NOCLIENTSIZE | SWP_NOCLIENTMOVE | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_NOSIZE | SWP_NOMOVE) as INT;
    let mut Ret: INT = 0;

    UserEnterExclusive();

    'Exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() || UserIsDesktopWindow(Window) != 0 || UserIsMessageWindow(Window) != 0 {
            break 'Exit;
        }

        if !hRgn.is_null() {
            if GreIsHandleValid(hRgn as HGDIOBJ) != 0 {
                hrgnCopy = NtGdiCreateRectRgn(0, 0, 0, 0);
                NtGdiCombineRgn(hrgnCopy, hRgn, core::ptr::null_mut(), RGN_COPY as INT);
            } else {
                break 'Exit;
            }
        }

        if !hrgnCopy.is_null() {
            (*Window).hrgnNewFrame = hrgnCopy;
        } else {
            (*Window).hrgnNewFrame = HRGN_WINDOW;
        }

        Ret = co_WinPosSetWindowPos(
            Window,
            HWND_TOP,
            0,
            0,
            0,
            0,
            if bRedraw != 0 { flags as UINT } else { (flags | SWP_NOREDRAW as INT) as UINT },
        ) as INT;
    }

    UserLeave();
    Ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserSetInternalWindowPos(
    hwnd: HWND,
    showCmd: UINT,
    lprect: LPRECT,
    lppt: LPPOINT,
) -> DWORD {
    let mut wndpl: WINDOWPLACEMENT = core::mem::zeroed();
    let mut flags: UINT;
    let Wnd: PWND;
    let mut rect: RECT = core::mem::zeroed();
    let mut pt: POINT = core::mem::zeroed();
    let mut Ret: BOOL = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Status: NTSTATUS;

    UserEnterExclusive();

    'exit: {
        Wnd = UserGetWindowObject(hwnd);
        if Wnd.is_null() || UserIsDesktopWindow(Wnd) != 0 || UserIsMessageWindow(Wnd) != 0 {
            break 'exit;
        }

        if !lppt.is_null() {
            Status = SafeProbeForRead(lppt as *mut _, core::mem::size_of::<POINT>() as _, 1);
        } else {
            Status = STATUS_SUCCESS;
        }
        if NT_SUCCESS(Status) && !lprect.is_null() {
            Status = SafeProbeForRead(lprect as *mut _, core::mem::size_of::<RECT>() as _, 1);
        }
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'exit;
        } else {
            if !lppt.is_null() {
                if !crate::copy::capture_from_user(
                    core::ptr::addr_of_mut!(pt) as *mut core::ffi::c_void,
                    lppt as *const core::ffi::c_void,
                    core::mem::size_of::<POINT>(),
                ) {
                    SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as _);
                    break 'exit;
                }
            }
            if !lprect.is_null() {
                if !crate::copy::capture_from_user(
                    core::ptr::addr_of_mut!(rect) as *mut core::ffi::c_void,
                    lprect as *const core::ffi::c_void,
                    core::mem::size_of::<RECT>(),
                ) {
                    SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as _);
                    break 'exit;
                }
            }
        }

        wndpl.length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
        wndpl.showCmd = showCmd;
        flags = 0;
        wndpl.flags = 0;

        if !lppt.is_null() {
            flags |= PLACE_MIN as UINT;
            wndpl.flags |= WPF_SETMINPOSITION as UINT;
            wndpl.ptMinPosition = core::mem::transmute_copy(&pt);
        }
        if !lprect.is_null() {
            flags |= PLACE_RECT as UINT;
            wndpl.rcNormalPosition = core::mem::transmute_copy(&rect);
        }

        UserRefObjectCo(Wnd, core::ptr::addr_of_mut!(Ref));
        IntSetWindowPlacement(Wnd, core::ptr::addr_of_mut!(wndpl), flags);
        UserDerefObjectCo(Wnd);
        Ret = 1;
    }

    UserLeave();
    Ret as DWORD
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserSetInternalWindowPos(
    hwnd: HWND,
    showCmd: UINT,
    lprect: LPRECT,
    lppt: LPPOINT,
) -> DWORD {
    let mut wndpl: WINDOWPLACEMENT = core::mem::zeroed();
    let mut flags: UINT;
    let Wnd: PWND;
    let mut rect: RECT = core::mem::zeroed();
    let mut pt: POINT = core::mem::zeroed();
    let mut Ret: BOOL = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut Status: NTSTATUS;

    UserEnterExclusive();

    'exit: {
        Wnd = UserGetWindowObject(hwnd);
        if Wnd.is_null() || UserIsDesktopWindow(Wnd) != 0 || UserIsMessageWindow(Wnd) != 0 {
            break 'exit;
        }

        if !lppt.is_null() {
            Status = SafeProbeForRead(lppt as *mut _, core::mem::size_of::<POINT>() as _, 1);
        } else {
            Status = STATUS_SUCCESS;
        }
        if NT_SUCCESS(Status) && !lprect.is_null() {
            Status = SafeProbeForRead(lprect as *mut _, core::mem::size_of::<RECT>() as _, 1);
        }
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'exit;
        } else {
            if !lppt.is_null() {
                if !crate::copy::capture_from_user(
                    core::ptr::addr_of_mut!(pt) as *mut core::ffi::c_void,
                    lppt as *const core::ffi::c_void,
                    core::mem::size_of::<POINT>(),
                ) {
                    SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as _);
                    break 'exit;
                }
            }
            if !lprect.is_null() {
                if !crate::copy::capture_from_user(
                    core::ptr::addr_of_mut!(rect) as *mut core::ffi::c_void,
                    lprect as *const core::ffi::c_void,
                    core::mem::size_of::<RECT>(),
                ) {
                    SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as _);
                    break 'exit;
                }
            }
        }

        wndpl.length = core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
        wndpl.showCmd = showCmd;
        flags = 0;
        wndpl.flags = 0;

        if !lppt.is_null() {
            flags |= PLACE_MIN as UINT;
            wndpl.flags |= WPF_SETMINPOSITION as UINT;
            wndpl.ptMinPosition = core::mem::transmute(pt);
        }
        if !lprect.is_null() {
            flags |= PLACE_RECT as UINT;
            wndpl.rcNormalPosition = core::mem::transmute(rect);
        }

        UserRefObjectCo(Wnd, core::ptr::addr_of_mut!(Ref));
        IntSetWindowPlacement(Wnd, core::ptr::addr_of_mut!(wndpl), flags);
        UserDerefObjectCo(Wnd);
        Ret = 1;
    }

    UserLeave();
    Ret as DWORD
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetWindowPlacement(hWnd: HWND, lpwndpl: *mut WINDOWPLACEMENT) -> BOOL {
    let mut Wnd: PWND;
    let mut Safepl: WINDOWPLACEMENT = core::mem::zeroed();
    let mut Flags: UINT;
    let mut Ret: BOOL = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterExclusive();

    'exit: {
        let __SehStatus = SafeProbeForRead(lpwndpl as *mut _, core::mem::size_of::<WINDOWPLACEMENT>() as _, 1);
        if !NT_SUCCESS(__SehStatus) {
            SetLastNtError(__SehStatus);
            break 'exit;
        }

        // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(Safepl) as *mut core::ffi::c_void,
            lpwndpl as *const core::ffi::c_void,
            core::mem::size_of::<WINDOWPLACEMENT>(),
        ) {
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            break 'exit;
        }

        if LOWORD((*gptiCurrent).dwExpWinVer as u32) < WINVER_WINNT4 as u32 {
            Safepl.length = core::mem::size_of::<WINDOWPLACEMENT>() as _;
        }

        if Safepl.length as usize != core::mem::size_of::<WINDOWPLACEMENT>() {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'exit;
        }

        Flags = (PLACE_MAX | PLACE_RECT) as UINT;
        if Safepl.flags & WPF_SETMINPOSITION as UINT != 0 {
            Flags |= PLACE_MIN as UINT;
        }

        Wnd = UserGetWindowObject(hWnd);
        if Wnd.is_null() {
            break 'exit;
        }

        UserRefObjectCo(Wnd, &mut Ref);
        if UserIsDesktopWindow(Wnd) == 0 && UserIsMessageWindow(Wnd) == 0 {
            Ret = IntSetWindowPlacement(Wnd, &mut Safepl, Flags);
        }
        UserDerefObjectCo(Wnd);
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserShowWindowAsync(hWnd: HWND, nCmdShow: LONG) -> BOOL {
    let Window: PWND;
    let Result: LRESULT;
    let mut ret: BOOL = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterExclusive();

    'exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() || UserIsDesktopWindow(Window) != 0 || UserIsMessageWindow(Window) != 0 {
            break 'exit;
        }

        if nCmdShow > SW_MAX as LONG {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'exit;
        }

        UserRefObjectCo(Window, &mut Ref);
        Result = co_IntSendMessageNoWait(hWnd, WM_ASYNC_SHOWWINDOW, nCmdShow as WPARAM, 0);
        UserDerefObjectCo(Window);
        if Result != -1 && Result != 0 {
            ret = 1;
        }
    }

    UserLeave();
    ret
}

type BOOL = i32;
type LONG = i32;
#[no_mangle]
pub unsafe extern "system" fn NtUserShowWindow(hWnd: HWND, nCmdShow: LONG) -> BOOL {
    let mut Window: PWND;
    let mut ret: BOOL = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterExclusive();

    'exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null()
            || UserIsDesktopWindow(Window) != 0
            || UserIsMessageWindow(Window) != 0
        {
            break 'exit;
        }

        if nCmdShow > SW_MAX as LONG || ((*Window).state2 & WNDS2_INDESTROY) != 0 {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'exit;
        }

        UserRefObjectCo(Window, &mut Ref);
        ret = co_WinPosShowWindow(Window, nCmdShow) as BOOL;
        UserDerefObjectCo(Window);
    }

    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserWindowFromPoint(X: LONG, Y: LONG) -> HWND {
    let mut pt: POINT = core::mem::zeroed();
    let mut Ret: HWND = core::ptr::null_mut();
    let DesktopWindow: PWND;
    let Window: PWND;
    let mut hittest: USHORT = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterExclusive();

    DesktopWindow = UserGetWindowObject(IntGetDesktopWindow());
    if !DesktopWindow.is_null() {
        pt.x = X;
        pt.y = Y;

        UserRefObjectCo(DesktopWindow, &mut Ref);

        Window = co_WinPosWindowFromPoint(DesktopWindow, &mut pt, &mut hittest, 0);
        if !Window.is_null() {
            Ret = UserHMGetHandle(Window);
        }

        UserDerefObjectCo(DesktopWindow);
    }

    UserLeave();
    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntGetWindowSnapEdge(Wnd: PWND) -> UINT {
    if (*Wnd).ExStyle2 & WS_EX2_VERTICALLYMAXIMIZEDLEFT != 0 {
        return HTLEFT;
    }
    if (*Wnd).ExStyle2 & WS_EX2_VERTICALLYMAXIMIZEDRIGHT != 0 {
        return HTRIGHT;
    }
    HTNOWHERE
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntGetWindowSnapEdge(Wnd: PWND) -> UINT {
    if (*Wnd).ExStyle2 & WS_EX2_VERTICALLYMAXIMIZEDLEFT != 0 {
        return HTLEFT;
    }
    if (*Wnd).ExStyle2 & WS_EX2_VERTICALLYMAXIMIZEDRIGHT != 0 {
        return HTRIGHT;
    }
    HTNOWHERE
}

#[cfg(target_arch = "x86")]
#[cfg_attr(target_arch = "x86", export_name = "co_IntCalculateSnapPosition")]
unsafe extern "fastcall" fn co_IntCalculateSnapPosition(
    Wnd: PWND,
    Edge: UINT,
    pos2: *mut RECT,
) {
    let mut maxs: POINT = core::mem::zeroed();
    let mut mint: POINT = core::mem::zeroed();
    let mut maxt: POINT = core::mem::zeroed();
    let mut width: UINT;
    let mut height: UINT;

    UserSystemParametersInfo(SPI_GETWORKAREA, 0, pos2 as *mut _, 0);

    co_WinPosGetMinMaxInfo(
        Wnd,
        core::ptr::addr_of_mut!(maxs) as *mut _,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(mint) as *mut _,
        core::ptr::addr_of_mut!(maxt) as *mut _,
    );

    width = ((*pos2).right - (*pos2).left) as UINT;
    let half = core::cmp::max(width / 2, mint.x as UINT);
    width = core::cmp::min(core::cmp::min(half, maxt.x as UINT), width);

    height = ((*pos2).bottom - (*pos2).top) as UINT;
    height = core::cmp::min(core::cmp::max(height, mint.y as UINT), maxt.y as UINT);

    match Edge {
        HTTOP => {
            height = core::cmp::min(((*pos2).bottom - (*pos2).top) as UINT, maxs.y as UINT);
        }
        HTLEFT => {
            (*pos2).right = width as _;
        }
        HTRIGHT => {
            (*pos2).left = (*pos2).right - width as i32;
        }
        _ => {
            ERR!("Unexpected snap edge %#x\n", Edge);
        }
    }

    (*pos2).bottom = (*pos2).top + height as i32;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn co_IntCalculateSnapPosition(
    Wnd: PWND,
    Edge: UINT,
    pos2: *mut RECT,
) {
    let mut maxs: POINT = core::mem::zeroed();
    let mut mint: POINT = core::mem::zeroed();
    let mut maxt: POINT = core::mem::zeroed();
    let mut width: UINT;
    let mut height: UINT;

    UserSystemParametersInfo(SPI_GETWORKAREA, 0, pos2 as *mut _, 0);

    co_WinPosGetMinMaxInfo(
        Wnd,
        core::ptr::addr_of_mut!(maxs) as *mut _,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(mint) as *mut _,
        core::ptr::addr_of_mut!(maxt) as *mut _,
    );

    width = ((*pos2).right - (*pos2).left) as UINT;
    let half = core::cmp::max(width / 2, mint.x as UINT);
    width = core::cmp::min(core::cmp::min(half, maxt.x as UINT), width);

    height = ((*pos2).bottom - (*pos2).top) as UINT;
    height = core::cmp::min(core::cmp::max(height, mint.y as UINT), maxt.y as UINT);

    match Edge {
        HTTOP => {
            height = core::cmp::min(((*pos2).bottom - (*pos2).top) as UINT, maxs.y as UINT);
        }
        HTLEFT => {
            (*pos2).right = width as _;
        }
        HTRIGHT => {
            (*pos2).left = (*pos2).right - width as i32;
        }
        _ => {
            ERR!("Unexpected snap edge %#x\n", Edge);
        }
    }

    (*pos2).bottom = (*pos2).top + height as i32;
}

fastcall_fn! {
#[no_mangle]
unsafe fn co_IntSnapWindow(Wnd: PWND, Edge: UINT) {
    let mut newPos: RECT = core::mem::zeroed();
    let wasSnapped: BOOLEAN = IntIsWindowSnapped(Wnd);
    let normal: UINT = (((*Wnd).style & (WS_MAXIMIZE | WS_MINIMIZE)) == 0) as UINT;
    let mut r#ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut hasRef: BOOLEAN = 0u8;

    if Edge == HTTOP {
        co_IntSendMessage(
            UserHMGetHandle(Wnd),
            WM_SYSCOMMAND,
            SC_MAXIMIZE as WPARAM,
            0,
        );
        return;
    } else if Edge != HTNOWHERE {
        UserRefObjectCo(Wnd, &mut r#ref);
        hasRef = 1u8;
        co_IntCalculateSnapPosition(Wnd, Edge, &mut newPos);
        let pos: *const RECT = if wasSnapped != 0 || normal == 0 {
            core::ptr::null()
        } else {
            core::ptr::addr_of!((*Wnd).rcWindow) as *const RECT
        };
        IntSetSnapInfo(Wnd, Edge, pos);
    } else if wasSnapped != 0 {
        if normal == 0 {
            IntSetSnapEdge(Wnd, HTNOWHERE);
            return;
        }
        newPos = core::mem::transmute_copy(&(*Wnd).InternalPos.NormalRect);
        IntSetSnapInfo(Wnd, HTNOWHERE, core::ptr::null());
    } else {
        return;
    }

    co_WinPosSetWindowPos(
        Wnd,
        HWND_TOP,
        newPos.left,
        newPos.top,
        newPos.right - newPos.left,
        newPos.bottom - newPos.top,
        0,
    );

    if hasRef != 0 {
        UserDerefObjectCo(Wnd);
    }
}
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntSetSnapEdge(Wnd: PWND, Edge: UINT) {
    IntSetSnapEdge_impl(Wnd, Edge)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntSetSnapEdge(Wnd: PWND, Edge: UINT) {
    IntSetSnapEdge_impl(Wnd, Edge)
}

unsafe fn IntSetSnapEdge_impl(Wnd: PWND, Edge: UINT) {
    let styleMask: UINT = WS_EX2_VERTICALLYMAXIMIZEDLEFT | WS_EX2_VERTICALLYMAXIMIZEDRIGHT;
    let mut style: UINT = 0;
    match Edge {
        HTNOWHERE => {
            style = 0;
        }
        HTTOP => {
            style = 0;
        }
        HTLEFT => {
            style = WS_EX2_VERTICALLYMAXIMIZEDLEFT;
        }
        HTRIGHT => {
            style = WS_EX2_VERTICALLYMAXIMIZEDRIGHT;
        }
        _ => {}
    }
    (*Wnd).ExStyle2 = ((*Wnd).ExStyle2 & !styleMask) | style;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntSetSnapInfo(Wnd: PWND, Edge: UINT, mut pos2: *const RECT) {
    let mut r: RECT = core::mem::zeroed();
    IntSetSnapEdge(Wnd, Edge);
    if Edge == HTNOWHERE {
        RECTL_vSetEmptyRect(core::ptr::addr_of_mut!(r) as *mut _);
        pos2 = if (*Wnd).style & WS_MINIMIZE != 0 {
            core::ptr::null()
        } else {
            core::ptr::addr_of!(r)
        };
    }
    if !pos2.is_null() {
        (*Wnd).InternalPos.NormalRect = core::mem::transmute_copy(&*pos2);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntSetSnapInfo(Wnd: PWND, Edge: UINT, mut pos2: *const RECT) {
    let mut r: RECT = core::mem::zeroed();
    IntSetSnapEdge(Wnd, Edge);
    if Edge == HTNOWHERE {
        RECTL_vSetEmptyRect(core::ptr::addr_of_mut!(r) as *mut _);
        pos2 = if (*Wnd).style & WS_MINIMIZE != 0 {
            core::ptr::null()
        } else {
            core::ptr::addr_of!(r)
        };
    }
    if !pos2.is_null() {
        (*Wnd).InternalPos.NormalRect = core::mem::transmute(*pos2);
    }
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn UpdateShellHook(Window: PWND);
    fn DceResetActiveDCEs(Window: PWND);
    fn co_IntSendMessageNoWait(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn UpdateShellHook(Window: PWND);
    fn DceResetActiveDCEs(Window: PWND);
    fn co_IntSendMessageNoWait(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SetLastNtError(Status: NTSTATUS);
}
