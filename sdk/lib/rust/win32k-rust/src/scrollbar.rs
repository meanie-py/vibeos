#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{PsGetCurrentThreadWin32Thread};
use core::ffi::c_void;

// ASSERT_REFS_CO(obj) — debug-only ref assert; no-op in retail.
#[inline(always)]
unsafe fn ASSERT_REFS_CO(_Window: PWND) {}

// NT_SUCCESS(Status) — Status >= 0.
#[inline(always)]
fn NT_SUCCESS(Status: NTSTATUS) -> bool { Status >= 0 }

// SafeProbeForRead/Write — NTAPI (stdcall on i386 → extern "system"), per nm
// `_SafeProbeForRead@12`. These RETURN status rather than raising, which is what
// DUMMY_PSEH requires: validate first, then act. See dummy-pseh-verdict.md.
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// max(a,b) macro — C `max`/`abs` from minmax.h / stdlib.h.
#[inline(always)]
fn max<T: PartialOrd>(a: T, b: T) -> T { if a > b { a } else { b } }
#[inline(always)]
fn abs(x: LONG) -> LONG { if x < 0 { -x } else { x } }

// UserHMGetHandle(obj) = obj->head.h (HANDLE at offset 0 of the object HEAD).
#[inline(always)]
unsafe fn UserHMGetHandle(obj: PWND) -> HWND {
    *(obj as *const HWND)
}

// UserIsDesktopWindow(pWnd) (desktop.h macro): pWnd && pWnd->fnid == FNID_DESKTOP.
#[inline(always)]
unsafe fn UserIsDesktopWindow(pWnd: PWND) -> BOOL {
    (!pWnd.is_null() && (*pWnd).fnid == FNID_DESKTOP) as BOOL
}

// UserIsMessageWindow(pWnd) (window.h macro): pWnd && pWnd->fnid == FNID_MESSAGEWND.
#[inline(always)]
unsafe fn UserIsMessageWindow(pWnd: PWND) -> BOOL {
    (!pWnd.is_null() && (*pWnd).fnid == FNID_MESSAGEWND) as BOOL
}

// head_pti(obj) — bindgen flattened THRDESKHEAD (no `pti` field). pti sits at
// 2*sizeof(usize) (8 on i386, 16 on amd64/arm64). (matches ntmouse.rs/event.rs)
#[inline(always)]
unsafe fn head_pti<T>(obj: *const T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}

// TAG_SBARINFO = 'NIBS' (tags.h) — little-endian bytes "SBIN".
const TAG_SBARINFO: u32 = u32::from_le_bytes(*b"SBIN");

// SBOBJ_TO_SBID(Obj) = (Obj) - OBJID_HSCROLL  (scroll.h macro — no symbol).
#[inline(always)]
unsafe fn SBOBJ_TO_SBID(idObject: LONG) -> LONG {
    idObject - OBJID_HSCROLL as LONG
}
// SBID_IS_VALID(id) (scroll.h macro — no symbol).
#[inline(always)]
unsafe fn SBID_IS_VALID(id: LONG) -> BOOL {
    (id == SB_HORZ as LONG || id == SB_VERT as LONG || id == SB_CTL as LONG) as BOOL
}
// IntGetScrollbarInfoFromWindow / IntGetScrollInfoFromWindow (scroll.h macros):
//   &((Window->pSBInfoex + i)->ScrollBarInfo) / ->ScrollInfo
#[inline(always)]
unsafe fn IntGetScrollbarInfoFromWindow(Window: PWND, i: LONG) -> PSCROLLBARINFO {
    core::ptr::addr_of_mut!((*(*Window).pSBInfoex.offset(i as isize)).ScrollBarInfo) as PSCROLLBARINFO
}
#[inline(always)]
unsafe fn IntGetScrollInfoFromWindow(Window: PWND, i: LONG) -> LPSCROLLINFO {
    core::ptr::addr_of_mut!((*(*Window).pSBInfoex.offset(i as isize)).ScrollInfo) as LPSCROLLINFO
}

// RECTL_vOffsetRect / RECTL_bIsEmptyRect (rect.h FORCEINLINE — no symbol).
#[inline(always)]
unsafe fn RECTL_vOffsetRect(prcl: *mut RECTL, cx: INT, cy: INT) {
    (*prcl).left += cx;
    (*prcl).right += cx;
    (*prcl).top += cy;
    (*prcl).bottom += cy;
}
#[inline(always)]
unsafe fn RECTL_bIsEmptyRect(prcl: *const RECTL) -> BOOL {
    ((*prcl).left >= (*prcl).right || (*prcl).top >= (*prcl).bottom) as BOOL
}

// UserRefObjectCo / UserDerefObjectCo (object.h static __inline — no symbol).
#[inline(always)]
unsafe fn UserRefObjectCo(obj: PWND, entry: *mut USER_REFERENCE_ENTRY) {
    let W32Thread = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*entry).obj = obj as PVOID;
    UserReferenceObject(obj as PVOID);
    let head = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    (*entry).Entry.Next = (*head).Next;
    (*head).Next = core::ptr::addr_of_mut!((*entry).Entry);
}
#[inline(always)]
unsafe fn UserDerefObjectCo(obj: PWND) {
    let W32Thread = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let head = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let first = (*head).Next;
    if !first.is_null() {
        (*head).Next = (*first).Next;
    }
    UserDereferenceObject(obj as PVOID);
}

// RtlAllocateHeap / RtlFreeHeap — NTAPI (stdcall i386, C elsewhere).
extern "system" {
    fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn RtlFreeHeap(HeapHandle: PVOID, Flags: ULONG, BaseAddress: PVOID) -> BOOLEAN;
}
const HEAP_NO_SERIALIZE: ULONG = 0x00000001;

// DesktopHeapAlloc / DesktopHeapFree (desktop.h FORCEINLINE — no symbol).
#[inline(always)]
unsafe fn DesktopHeapAlloc(Desktop: PDESKTOP, Bytes: ULONG) -> PVOID {
    RtlAllocateHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, Bytes as SIZE_T)
}
#[inline(always)]
unsafe fn DesktopHeapFree(Desktop: PDESKTOP, lpMem: PVOID) -> BOOL {
    RtlFreeHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, lpMem) as BOOL
}

// Ex pool wrappers (ntkrnl_sys::ddk — kernel imports).
#[inline(always)]
unsafe fn ExAllocatePoolWithTag(pool: POOL_TYPE, n: usize, tag: u32) -> PVOID {
    ntkrnl_sys::ddk::ExAllocatePoolWithTag(pool as _, n as _, tag) as PVOID
}
#[inline(always)]
unsafe fn ExFreePoolWithTag(p: PVOID, tag: u32) {
    ntkrnl_sys::ddk::ExFreePoolWithTag(p as _, tag)
}

// MmCopyFromCaller / MmCopyToCaller — win32k-internal C (cdecl, `_`-prefixed).
extern "C" {
    #[link_name = "_MmCopyFromCaller"]
    fn _MmCopyFromCaller(dst: *mut c_void, src: *const c_void, len: UINT) -> NTSTATUS;
    #[link_name = "_MmCopyToCaller"]
    fn _MmCopyToCaller(dst: *mut c_void, src: *const c_void, len: UINT) -> NTSTATUS;
}
#[inline(always)]
unsafe fn MmCopyFromCaller(dst: *mut c_void, src: *mut c_void, len: ULONG) -> NTSTATUS {
    _MmCopyFromCaller(dst, src, len as UINT)
}
#[inline(always)]
unsafe fn MmCopyToCaller(dst: *mut c_void, src: *mut c_void, len: ULONG) -> NTSTATUS {
    _MmCopyToCaller(dst, src, len as UINT)
}

// RtlCopyMemory / RtlZeroMemory — intrinsic memmove/memset semantics.
#[inline(always)]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy(src as *const u8, dst as *mut u8, len);
}
#[inline(always)]
unsafe fn RtlZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}

// SetLastNtError / co_IntSetCaretPos are cdecl win32k exports.
// DrawEdge / DrawFrameControl are NTAPI (extern "system") win32k exports.
extern "system" {
    fn DrawEdge(hdc: HDC, qrc: *mut RECT, edge: UINT, grfFlags: UINT) -> BOOL;
    fn DrawFrameControl(hDC: HDC, rc: *mut RECT, uType: UINT, uState: UINT) -> BOOL;
}

// co_WinPos* / GetControlBrush / DefWndControlColor are FASTCALL on x86,
// cdecl-as-C elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn co_WinPosShowWindow(Wnd: PWND, Cmd: core::ffi::c_int) -> BOOL;
    fn co_WinPosSetWindowPos(Wnd: PWND, WndInsertAfter: PWND, x: core::ffi::c_int, y: core::ffi::c_int, cx: core::ffi::c_int, cy: core::ffi::c_int, flags: UINT) -> BOOL;
    fn GetControlBrush(pwnd: PWND, hdc: HDC, ctlType: UINT) -> HBRUSH;
    fn DefWndControlColor(hDC: HDC, ctlType: UINT) -> HBRUSH;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn co_WinPosShowWindow(Wnd: PWND, Cmd: core::ffi::c_int) -> BOOL;
    fn co_WinPosSetWindowPos(Wnd: PWND, WndInsertAfter: PWND, x: core::ffi::c_int, y: core::ffi::c_int, cx: core::ffi::c_int, cy: core::ffi::c_int, flags: UINT) -> BOOL;
    fn GetControlBrush(pwnd: PWND, hdc: HDC, ctlType: UINT) -> HBRUSH;
    fn DefWndControlColor(hDC: HDC, ctlType: UINT) -> HBRUSH;
}

// ---- file-scope preamble (translated) ----
pub const SCROLL_NOWHERE: u32 = 0x00;
pub const SCROLL_TOP_ARROW: u32 = 0x01;
pub const SCROLL_TOP_RECT: u32 = 0x02;
pub const SCROLL_THUMB: u32 = 0x03;
pub const SCROLL_BOTTOM_RECT: u32 = 0x04;
pub const SCROLL_BOTTOM_ARROW: u32 = 0x05;
pub const SCROLL_FIRST_DELAY: u32 = 200;
pub const SCROLL_REPEAT_DELAY: u32 = 50;
pub const SCROLL_TIMER: u32 = 0;
pub const SCROLL_MIN_RECT: u32 = 4;
pub const SCROLL_MIN_THUMB: u32 = 6;
pub const SCROLL_ARROW_THUMB_OVERLAP: u32 = 0;
pub const MINTRACKTHUMB: u32 = 8;
pub const SA_SSI_HIDE: i32 = 0x0001;
pub const SA_SSI_SHOW: i32 = 0x0002;
pub const SA_SSI_REFRESH: i32 = 0x0004;
pub const SA_SSI_REPAINT_ARROWS: i32 = 0x0008;
pub const SBRG_SCROLLBAR: u32 = 0;
pub const SBRG_TOPRIGHTBTN: u32 = 1;
pub const SBRG_PAGEUPRIGHT: u32 = 2;
pub const SBRG_SCROLLBOX: u32 = 3;
pub const SBRG_PAGEDOWNLEFT: u32 = 4;
pub const SBRG_BOTTOMLEFTBTN: u32 = 5;

unsafe fn mirror_rect(window_rect: *const RECTL, rect: *mut RECTL) {
    let width = (*window_rect).right - (*window_rect).left;
    let tmp = (*rect).left;
    (*rect).left = width - (*rect).right;
    (*rect).right = width - tmp;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetSBData(pwnd: PWND, Bar: INT) -> PSBDATA {
    IntGetSBData_impl(pwnd, Bar)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetSBData(pwnd: PWND, Bar: INT) -> PSBDATA {
    IntGetSBData_impl(pwnd, Bar)
}

unsafe fn IntGetSBData_impl(pwnd: PWND, Bar: INT) -> PSBDATA {
    let pSBInfo: PSBINFO = (*pwnd).pSBInfo;
    match Bar as u32 {
        SB_HORZ => {
            return core::ptr::addr_of_mut!((*pSBInfo).Horz);
        }
        SB_VERT => {
            return core::ptr::addr_of_mut!((*pSBInfo).Vert);
        }
        SB_CTL => {
            if ((*pwnd).cbwndExtra as usize)
                < (core::mem::size_of::<SBWND>() - core::mem::size_of::<WND>())
            {
                return core::ptr::null_mut();
            }
            let pSBWnd: PSBWND = pwnd as PSBWND;
            return core::ptr::addr_of_mut!((*pSBWnd).SBCalc) as PSBDATA;
        }
        _ => {}
    }
    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetScrollBarRect(Wnd: PWND, nBar: INT, lprect: *mut RECTL) -> BOOL {
    IntGetScrollBarRect_impl(Wnd, nBar, lprect)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetScrollBarRect(Wnd: PWND, nBar: INT, lprect: *mut RECTL) -> BOOL {
    IntGetScrollBarRect_impl(Wnd, nBar, lprect)
}

unsafe fn IntGetScrollBarRect_impl(Wnd: PWND, nBar: INT, lprect: *mut RECTL) -> BOOL {
    let vertical: BOOL;
    *lprect = (*Wnd).rcClient;

    RECTL_vOffsetRect(lprect, -(*Wnd).rcWindow.left, -(*Wnd).rcWindow.top);
    if (*Wnd).ExStyle & WS_EX_LAYOUTRTL != 0 {
        mirror_rect(core::ptr::addr_of!((*Wnd).rcWindow), lprect);
    }

    match nBar {
        x if x == SB_HORZ as INT => {
            (*lprect).top = (*lprect).bottom;
            (*lprect).bottom += UserGetSystemMetrics(SM_CYHSCROLL as _);
            if (*Wnd).style & WS_BORDER != 0 {
                (*lprect).left -= 1;
                (*lprect).right += 1;
            } else if (*Wnd).style & WS_VSCROLL != 0 {
                (*lprect).right += 1;
            }
            vertical = 0;
        }

        x if x == SB_VERT as INT => {
            if (*Wnd).ExStyle & WS_EX_LEFTSCROLLBAR != 0 {
                (*lprect).right = (*lprect).left;
                (*lprect).left -= UserGetSystemMetrics(SM_CXVSCROLL as _);
            } else {
                (*lprect).left = (*lprect).right;
                (*lprect).right += UserGetSystemMetrics(SM_CXVSCROLL as _);
            }
            if (*Wnd).style & WS_BORDER != 0 {
                (*lprect).top -= 1;
                (*lprect).bottom += 1;
            } else if (*Wnd).style & WS_HSCROLL != 0 {
                (*lprect).bottom += 1;
            }
            vertical = 1;
        }

        x if x == SB_CTL as INT => {
            IntGetClientRect(Wnd, lprect);
            vertical = if (*Wnd).style & SBS_VERT != 0 { 1 } else { 0 };
        }

        _ => {
            return 0;
        }
    }

    vertical
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCalculateThumb(
    Wnd: PWND,
    idObject: LONG,
    psbi: PSCROLLBARINFO,
    pSBData: PSBDATA,
) -> BOOL {
    IntCalculateThumb_impl(Wnd, idObject, psbi, pSBData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCalculateThumb(
    Wnd: PWND,
    idObject: LONG,
    psbi: PSCROLLBARINFO,
    pSBData: PSBDATA,
) -> BOOL {
    IntCalculateThumb_impl(Wnd, idObject, psbi, pSBData)
}

unsafe fn IntCalculateThumb_impl(
    Wnd: PWND,
    idObject: LONG,
    psbi: PSCROLLBARINFO,
    pSBData: PSBDATA,
) -> BOOL {
    let mut Thumb: INT;
    let mut ThumbBox: INT;
    let mut ThumbPos: INT;
    let mut cxy: INT;
    let mx: INT;
    let mut ClientRect: RECTL = core::mem::zeroed();

    match idObject {
        x if x == SB_HORZ as LONG => {
            Thumb = UserGetSystemMetrics(SM_CXHSCROLL as _);
            cxy = (*psbi).rcScrollBar.right - (*psbi).rcScrollBar.left;
        }
        x if x == SB_VERT as LONG => {
            Thumb = UserGetSystemMetrics(SM_CYVSCROLL as _);
            cxy = (*psbi).rcScrollBar.bottom - (*psbi).rcScrollBar.top;
        }
        x if x == SB_CTL as LONG => {
            IntGetClientRect(Wnd, core::ptr::addr_of_mut!(ClientRect));
            if (*Wnd).style & (SBS_VERT as u32) != 0 {
                Thumb = UserGetSystemMetrics(SM_CYVSCROLL as _);
                cxy = ClientRect.bottom - ClientRect.top;
            } else {
                Thumb = UserGetSystemMetrics(SM_CXHSCROLL as _);
                cxy = ClientRect.right - ClientRect.left;
            }
        }
        _ => return 0,
    }

    ThumbPos = Thumb;

    if cxy <= (2 * Thumb) {
        Thumb = cxy / 2;
        (*psbi).xyThumbTop = 0;
        (*psbi).xyThumbBottom = 0;
        ThumbPos = Thumb;
    } else if (*psbi).rgstate[SBRG_TOPRIGHTBTN as usize] == STATE_SYSTEM_UNAVAILABLE as u32
        && (*psbi).rgstate[SBRG_BOTTOMLEFTBTN as usize] == STATE_SYSTEM_UNAVAILABLE as u32
        && (*pSBData).posMin >= ((*pSBData).posMax - core::cmp::max((*pSBData).page - 1, 0)) as i32
    {
        (*psbi).xyThumbTop = 0;
        (*psbi).xyThumbBottom = 0;
    } else {
        ThumbBox = if (*pSBData).page != 0 {
            MINTRACKTHUMB as i32
        } else {
            UserGetSystemMetrics(SM_CXHTHUMB as _)
        };
        cxy -= 2 * Thumb;
        if cxy >= ThumbBox {
            if (*pSBData).page != 0 {
                ThumbBox = core::cmp::max(
                    EngMulDiv(cxy, (*pSBData).page, (*pSBData).posMax - (*pSBData).posMin + 1),
                    ThumbBox,
                );
            }

            if cxy > ThumbBox {
                mx = (*pSBData).posMax - core::cmp::max((*pSBData).page - 1, 0);
                if (*pSBData).posMin < mx {
                    ThumbPos = Thumb
                        + EngMulDiv(
                            cxy - ThumbBox,
                            (*pSBData).pos - (*pSBData).posMin,
                            mx - (*pSBData).posMin,
                        );
                } else {
                    ThumbPos = Thumb + ThumbBox;
                }
            }

            (*psbi).xyThumbTop = ThumbPos;
            (*psbi).xyThumbBottom = ThumbPos + ThumbBox;
        } else {
            (*psbi).xyThumbTop = 0;
            (*psbi).xyThumbBottom = 0;
        }
    }
    (*psbi).dxyLineButton = Thumb;

    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntGetScrollInfo(
    Window: PWND,
    nBar: INT,
    pSBData: PSBDATA,
    lpsi: LPSCROLLINFO,
) -> BOOL {
    co_IntGetScrollInfo_impl(Window, nBar, pSBData, lpsi)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntGetScrollInfo(
    Window: PWND,
    nBar: INT,
    pSBData: PSBDATA,
    lpsi: LPSCROLLINFO,
) -> BOOL {
    co_IntGetScrollInfo_impl(Window, nBar, pSBData, lpsi)
}

unsafe fn co_IntGetScrollInfo_impl(
    Window: PWND,
    nBar: INT,
    pSBData: PSBDATA,
    lpsi: LPSCROLLINFO,
) -> BOOL {
    let Mask: UINT;
    let psi: LPSCROLLINFO;

    ASSERT_REFS_CO(Window);

    (*lpsi).fMask &= !SIF_THEMED;
    if SBID_IS_VALID(nBar) == 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        ERR!("Trying to get scrollinfo for unknown scrollbar type %d\n", nBar);
        return 0;
    }

    if (*Window).pSBInfo.is_null() {
        ERR!("IntGetScrollInfo No window scrollbar info\n");
        return 0;
    }

    psi = IntGetScrollInfoFromWindow(Window, nBar);

    if (*lpsi).fMask == SIF_ALL {
        Mask = SIF_PAGE | SIF_POS | SIF_RANGE | SIF_TRACKPOS;
    } else {
        Mask = (*lpsi).fMask;
    }

    if 0 != (Mask & SIF_PAGE) {
        (*lpsi).nPage = (*psi).nPage;
    }

    if 0 != (Mask & SIF_POS) {
        (*lpsi).nPos = (*psi).nPos;
    }

    if 0 != (Mask & SIF_RANGE) {
        (*lpsi).nMin = (*psi).nMin;
        (*lpsi).nMax = (*psi).nMax;
    }

    if 0 != (Mask & SIF_TRACKPOS) {
        (*lpsi).nTrackPos = (*psi).nTrackPos;
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn NEWco_IntGetScrollInfo(
    pWnd: PWND,
    nBar: INT,
    pSBData: PSBDATA,
    lpsi: LPSCROLLINFO,
) -> BOOL {
    NEWco_IntGetScrollInfo_impl(pWnd, nBar, pSBData, lpsi)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NEWco_IntGetScrollInfo(
    pWnd: PWND,
    nBar: INT,
    pSBData: PSBDATA,
    lpsi: LPSCROLLINFO,
) -> BOOL {
    NEWco_IntGetScrollInfo_impl(pWnd, nBar, pSBData, lpsi)
}

unsafe fn NEWco_IntGetScrollInfo_impl(
    pWnd: PWND,
    nBar: INT,
    pSBData: PSBDATA,
    lpsi: LPSCROLLINFO,
) -> BOOL {
    let Mask: UINT;
    let pSBTrack: PSBTRACK = (*head_pti(pWnd)).pSBTrack as PSBTRACK;

    (*lpsi).fMask &= !SIF_THEMED;
    if SBID_IS_VALID(nBar) == 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if (*pWnd).pSBInfo.is_null() || pSBTrack.is_null() {
        return 0;
    }

    Mask = (*lpsi).fMask;

    if 0 != (Mask & SIF_PAGE) {
        (*lpsi).nPage = (*pSBData).page as _;
    }

    if 0 != (Mask & SIF_POS) {
        (*lpsi).nPos = (*pSBData).pos;
    }

    if 0 != (Mask & SIF_RANGE) {
        (*lpsi).nMin = (*pSBData).posMin;
        (*lpsi).nMax = (*pSBData).posMax;
    }

    if 0 != (Mask & SIF_TRACKPOS) {
        if !pSBTrack.is_null() && (*pSBTrack).nBar == nBar && (*pSBTrack).spwndTrack == pWnd {
            (*lpsi).nTrackPos = (*pSBTrack).posNew;
        } else {
            (*lpsi).nTrackPos = (*pSBData).pos;
        }
    }
    ((Mask & SIF_ALL) != 0) as BOOL
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntSetScrollInfo(
    Window: PWND,
    nBar: INT,
    lpsi: LPCSCROLLINFO,
    bRedraw: BOOL,
) -> DWORD {
    co_IntSetScrollInfo_impl(Window, nBar, lpsi, bRedraw)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntSetScrollInfo(
    Window: PWND,
    nBar: INT,
    lpsi: LPCSCROLLINFO,
    bRedraw: BOOL,
) -> DWORD {
    co_IntSetScrollInfo_impl(Window, nBar, lpsi, bRedraw)
}

unsafe fn co_IntSetScrollInfo_impl(
    Window: PWND,
    nBar: INT,
    lpsi: LPCSCROLLINFO,
    bRedraw: BOOL,
) -> DWORD {
    let Info: LPSCROLLINFO;
    let psbi: PSCROLLBARINFO;
    let mut new_flags: INT;
    let mut action: INT = 0;
    let pSBData: PSBDATA;
    let mut OldPos: DWORD = 0;
    let mut bChangeParams: BOOL = 0;
    let MaxPage: UINT;
    let MaxPos: core::ffi::c_int;
    let bVisible: BOOL;

    if SBID_IS_VALID(nBar) == 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if co_IntCreateScrollBars(Window) == 0 {
        return 0;
    }

    if (*lpsi).cbSize as usize != core::mem::size_of::<SCROLLINFO>()
        && (*lpsi).cbSize as usize
            != (core::mem::size_of::<SCROLLINFO>()
                - core::mem::size_of_val(&(*lpsi).nTrackPos))
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if (((*lpsi).fMask & !SIF_THEMED)
        & !(SIF_ALL | SIF_DISABLENOSCROLL | SIF_PREVIOUSPOS))
        != 0
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    psbi = IntGetScrollbarInfoFromWindow(Window, nBar);
    Info = IntGetScrollInfoFromWindow(Window, nBar);
    pSBData = IntGetSBData(Window, nBar);
    if pSBData.is_null() {
        return 0;
    }

    if ((*lpsi).fMask & SIF_THEMED) != 0 && ((*Info).fMask & SIF_THEMED) == 0 {
        (*Info).fMask |= SIF_THEMED;
    }

    if ((*lpsi).fMask & SIF_PAGE) != 0 {
        if (*Info).nPage != (*lpsi).nPage {
            (*Info).nPage = (*lpsi).nPage;
            (*pSBData).page = (*lpsi).nPage as INT;
            bChangeParams = 1;
        }
    }

    if ((*lpsi).fMask & SIF_POS) != 0 {
        OldPos = (*Info).nPos as DWORD;
        if (*Info).nPos != (*lpsi).nPos {
            (*Info).nPos = (*lpsi).nPos;
            (*pSBData).pos = (*lpsi).nPos;
        }
    }

    if ((*lpsi).fMask & SIF_RANGE) != 0 {
        if (*lpsi).nMin > (*lpsi).nMax {
            (*Info).nMin = (*lpsi).nMin;
            (*Info).nMax = (*lpsi).nMin;
            (*pSBData).posMin = (*lpsi).nMin;
            (*pSBData).posMax = (*lpsi).nMin;
            bChangeParams = 1;
        } else if (*Info).nMin != (*lpsi).nMin || (*Info).nMax != (*lpsi).nMax {
            (*Info).nMin = (*lpsi).nMin;
            (*Info).nMax = (*lpsi).nMax;
            (*pSBData).posMin = (*lpsi).nMin;
            (*pSBData).posMax = (*lpsi).nMax;
            bChangeParams = 1;
        }
    }

    MaxPage = (abs((*Info).nMax - (*Info).nMin) + 1) as UINT;
    if (*Info).nPage > MaxPage {
        (*Info).nPage = MaxPage;
        (*pSBData).page = MaxPage as INT;
    }

    MaxPos = (*Info).nMax + 1 - max((*Info).nPage, 1) as core::ffi::c_int;
    if (*Info).nPos < (*Info).nMin {
        (*Info).nPos = (*Info).nMin;
        (*pSBData).pos = (*Info).nMin;
    } else if (*Info).nPos > MaxPos {
        (*Info).nPos = MaxPos;
        (*pSBData).pos = MaxPos;
    }

    if ((*lpsi).fMask & SIF_ALL) == 0 {
        return if ((*lpsi).fMask & SIF_PREVIOUSPOS) != 0 {
            OldPos
        } else {
            (*pSBData).pos as DWORD
        };
    }

    if ((*lpsi).fMask & (SIF_RANGE | SIF_PAGE | SIF_DISABLENOSCROLL)) != 0 {
        new_flags = (*(*Window).pSBInfo).WSBflags;
        if (*Info).nMin + max((*Info).nPage, 1) as core::ffi::c_int > (*Info).nMax {
            if ((*lpsi).fMask & SIF_DISABLENOSCROLL) != 0 {
                new_flags = ESB_DISABLE_BOTH as INT;
                bChangeParams = 1;
            } else if nBar != SB_CTL as INT && bChangeParams != 0 {
                action = SA_SSI_HIDE;
            }
        } else if ((*lpsi).fMask & !SIF_THEMED) != SIF_PAGE {
            if nBar != SB_CTL as INT && bChangeParams != 0 {
                new_flags = ESB_ENABLE_BOTH as INT;
                action |= SA_SSI_SHOW;
            } else if nBar == SB_CTL as INT {
                new_flags = ESB_ENABLE_BOTH as INT;
            }
        }

        if (*(*Window).pSBInfo).WSBflags != new_flags {
            (*(*Window).pSBInfo).WSBflags = new_flags;
            action |= SA_SSI_REPAINT_ARROWS;
        }
    }

    if (action & SA_SSI_HIDE) != 0 {
        co_UserShowScrollBar(Window, nBar, 0, 0);
    } else {
        if (action & SA_SSI_SHOW) != 0 {
            if co_UserShowScrollBar(Window, nBar, 1, 1) != 0 {
                return if ((*lpsi).fMask & SIF_PREVIOUSPOS) != 0 {
                    OldPos
                } else {
                    (*pSBData).pos as DWORD
                };
            }
        }

        bVisible = match nBar {
            x if x == SB_HORZ as INT => ((*Window).style & WS_HSCROLL != 0) as BOOL,
            x if x == SB_VERT as INT => ((*Window).style & WS_VSCROLL != 0) as BOOL,
            x if x == SB_CTL as INT => ((*Window).style & WS_VISIBLE != 0) as BOOL,
            _ => 0,
        };

        if bRedraw != 0 && bVisible != 0 {
            if ((*Info).fMask & SIF_THEMED) == 0 {
                if (action & SA_SSI_REPAINT_ARROWS) != 0 {
                    let mut UpdateRect: RECTL = (*psbi).rcScrollBar;
                    UpdateRect.left -= (*Window).rcClient.left - (*Window).rcWindow.left;
                    UpdateRect.right -= (*Window).rcClient.left - (*Window).rcWindow.left;
                    UpdateRect.top -= (*Window).rcClient.top - (*Window).rcWindow.top;
                    UpdateRect.bottom -= (*Window).rcClient.top - (*Window).rcWindow.top;
                    co_UserRedrawWindow(
                        Window,
                        core::ptr::addr_of_mut!(UpdateRect),
                        core::ptr::null_mut(),
                        RDW_INVALIDATE | RDW_FRAME,
                    );
                } else {
                    IntRefeshScrollInterior(Window, nBar, psbi);
                }
            } else {
                let mut UpdateRect: RECTL = (*psbi).rcScrollBar;
                UpdateRect.left -= (*Window).rcClient.left - (*Window).rcWindow.left;
                UpdateRect.right -= (*Window).rcClient.left - (*Window).rcWindow.left;
                UpdateRect.top -= (*Window).rcClient.top - (*Window).rcWindow.top;
                UpdateRect.bottom -= (*Window).rcClient.top - (*Window).rcWindow.top;
                if bChangeParams != 0 || OldPos != (*pSBData).pos as DWORD {
                    co_UserRedrawWindow(
                        Window,
                        core::ptr::addr_of_mut!(UpdateRect),
                        core::ptr::null_mut(),
                        RDW_INVALIDATE | RDW_FRAME,
                    );
                }
            }
        }
    }

    if bChangeParams != 0
        && (nBar == SB_HORZ as INT || nBar == SB_VERT as INT)
        && ((*lpsi).fMask & SIF_DISABLENOSCROLL) != 0
    {
        IntEnableScrollBar(
            (nBar == SB_HORZ as INT) as BOOL,
            psbi,
            (*(*Window).pSBInfo).WSBflags as _,
        );
    }

    if ((*lpsi).fMask & SIF_PREVIOUSPOS) != 0 {
        OldPos
    } else {
        (*pSBData).pos as DWORD
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntGetScrollBarInfo(
    Window: PWND,
    idObject: LONG,
    psbi: PSCROLLBARINFO,
) -> BOOL {
    co_IntGetScrollBarInfo_impl(Window, idObject, psbi)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntGetScrollBarInfo(
    Window: PWND,
    idObject: LONG,
    psbi: PSCROLLBARINFO,
) -> BOOL {
    co_IntGetScrollBarInfo_impl(Window, idObject, psbi)
}

unsafe fn co_IntGetScrollBarInfo_impl(
    Window: PWND,
    idObject: LONG,
    psbi: PSCROLLBARINFO,
) -> BOOL {
    let Bar: INT;
    let sbi: PSCROLLBARINFO;
    let pSBData: PSBDATA;

    Bar = SBOBJ_TO_SBID(idObject);

    if SBID_IS_VALID(Bar) == 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if co_IntCreateScrollBars(Window) == 0 {
        return 0;
    }

    sbi = IntGetScrollbarInfoFromWindow(Window, Bar);
    pSBData = IntGetSBData(Window, Bar);
    if pSBData.is_null() {
        return 0;
    }

    IntGetScrollBarRect(Window, Bar, core::ptr::addr_of_mut!((*sbi).rcScrollBar));
    IntCalculateThumb(Window, Bar, sbi, pSBData);

    (*psbi).rgstate[0] = 0;
    if (Bar == SB_HORZ as INT && ((*Window).style & WS_HSCROLL as ULONG) == 0)
        || (Bar == SB_VERT as INT && ((*Window).style & WS_VSCROLL as ULONG) == 0)
    {
        (*psbi).rgstate[0] |= STATE_SYSTEM_INVISIBLE as u32;
    }
    if (*pSBData).posMin >= (*pSBData).posMax - max((*pSBData).page - 1, 0) {
        if (*psbi).rgstate[0] & STATE_SYSTEM_INVISIBLE as u32 == 0 {
            (*psbi).rgstate[0] |= STATE_SYSTEM_UNAVAILABLE as u32;
        } else {
            (*psbi).rgstate[0] |= STATE_SYSTEM_OFFSCREEN as u32;
        }
    }
    if Bar == SB_CTL as INT && ((*Window).style & WS_DISABLED as ULONG) == 0 {
        (*psbi).rgstate[0] |= STATE_SYSTEM_UNAVAILABLE as u32;
    }

    RtlCopyMemory(
        psbi as *mut core::ffi::c_void,
        sbi as *const core::ffi::c_void,
        core::mem::size_of::<SCROLLBARINFO>() as usize,
    );

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntSetScrollBarInfo(
    Window: PWND,
    idObject: LONG,
    psbi: PSETSCROLLBARINFO,
) -> BOOL {
    co_IntSetScrollBarInfo_impl(Window, idObject, psbi)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntSetScrollBarInfo(
    Window: PWND,
    idObject: LONG,
    psbi: PSETSCROLLBARINFO,
) -> BOOL {
    co_IntSetScrollBarInfo_impl(Window, idObject, psbi)
}

unsafe fn co_IntSetScrollBarInfo_impl(
    Window: PWND,
    idObject: LONG,
    psbi: PSETSCROLLBARINFO,
) -> BOOL {
    let Bar: INT;
    let sbi: PSCROLLBARINFO;
    let psi: LPSCROLLINFO;
    ASSERT_REFS_CO(Window);

    Bar = SBOBJ_TO_SBID(idObject);

    if SBID_IS_VALID(Bar) == 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        ERR!("Trying to get scrollinfo for unknown scrollbar type %d\n", Bar);
        return 0;
    }

    if co_IntCreateScrollBars(Window) == 0 {
        ERR!("Failed to create scrollbars for window\n");
        return 0;
    }

    sbi = IntGetScrollbarInfoFromWindow(Window, Bar);
    psi = IntGetScrollInfoFromWindow(Window, Bar);

    (*psi).nTrackPos = (*psbi).nTrackPos;
    (*sbi).reserved = (*psbi).reserved;
    RtlCopyMemory(
        core::ptr::addr_of_mut!((*sbi).rgstate) as *mut _,
        core::ptr::addr_of!((*psbi).rgstate) as *const _,
        core::mem::size_of_val(&(*psbi).rgstate),
    );

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntCreateScrollBars(Window: PWND) -> BOOL {
    co_IntCreateScrollBars_impl(Window)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntCreateScrollBars(Window: PWND) -> BOOL {
    co_IntCreateScrollBars_impl(Window)
}

unsafe fn co_IntCreateScrollBars_impl(Window: PWND) -> BOOL {
    let psbi: PSCROLLBARINFO;
    let pSBData: PSBDATA;
    let Size: ULONG;
    let mut s: ULONG;
    let mut i: INT;

    ASSERT_REFS_CO(Window);

    if !(*Window).pSBInfo.is_null() && !(*Window).pSBInfoex.is_null() {
        return 1;
    }

    Size = 3 * (core::mem::size_of::<SBINFOEX>() as ULONG);
    (*Window).pSBInfoex = ExAllocatePoolWithTag(PagedPool, Size as usize, TAG_SBARINFO) as *mut SBINFOEX;
    if (*Window).pSBInfoex.is_null() {
        ERR!(
            "Unable to allocate memory for scrollbar information for window %p\n",
            UserHMGetHandle(Window),
        );
        return 0;
    }

    RtlZeroMemory((*Window).pSBInfoex as *mut core::ffi::c_void, Size as usize);

    (*Window).pSBInfo = DesktopHeapAlloc((*Window).head.rpdesk, core::mem::size_of::<SBINFO>() as ULONG) as *mut SBINFO;
    if (*Window).pSBInfo.is_null() {
        ERR!(
            "Unable to allocate memory for scrollbar information for window %p\n",
            UserHMGetHandle(Window),
        );
        return 0;
    }

    RtlZeroMemory((*Window).pSBInfo as *mut core::ffi::c_void, core::mem::size_of::<SBINFO>());
    (*(*Window).pSBInfo).Vert.posMax = 100;
    (*(*Window).pSBInfo).Horz.posMax = 100;

    co_WinPosGetNonClientSize(
        Window,
        core::ptr::addr_of_mut!((*Window).rcWindow),
        core::ptr::addr_of_mut!((*Window).rcClient),
    );

    s = SB_HORZ as ULONG;
    while s <= SB_VERT as ULONG {
        let psbi_local = IntGetScrollbarInfoFromWindow(Window, s as INT) as PSCROLLBARINFO;
        let _ = psbi;
        let psbi = psbi_local;
        (*psbi).cbSize = core::mem::size_of::<SCROLLBARINFO>() as DWORD;
        i = 0;
        while i < CCHILDREN_SCROLLBAR as INT + 1 {
            (*psbi).rgstate[i as usize] = 0;
            i += 1;
        }

        let pSBData_local = IntGetSBData(Window, s as INT);
        let _ = pSBData;
        let pSBData = pSBData_local;

        IntGetScrollBarRect(Window, s as INT, core::ptr::addr_of_mut!((*psbi).rcScrollBar));
        IntCalculateThumb(Window, s as LONG, psbi, pSBData);

        s += 1;
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntDestroyScrollBars(Window: PWND) -> BOOL {
    IntDestroyScrollBars_impl(Window)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntDestroyScrollBars(Window: PWND) -> BOOL {
    IntDestroyScrollBars_impl(Window)
}

unsafe fn IntDestroyScrollBars_impl(Window: PWND) -> BOOL {
    if !(*Window).pSBInfo.is_null() && !(*Window).pSBInfoex.is_null() {
        DesktopHeapFree((*Window).head.rpdesk, (*Window).pSBInfo as *mut _);
        (*Window).pSBInfo = core::ptr::null_mut();
        ExFreePoolWithTag((*Window).pSBInfoex as *mut _, TAG_SBARINFO);
        (*Window).pSBInfoex = core::ptr::null_mut();
        return 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn IntEnableScrollBar(Horz: BOOL, Info: PSCROLLBARINFO, wArrows: UINT) -> BOOL {
    let mut Chg: BOOL = 0;
    match wArrows {
        ESB_DISABLE_BOTH => {
            if (*Info).rgstate[SBRG_TOPRIGHTBTN as usize] != STATE_SYSTEM_UNAVAILABLE {
                Chg = 1;
            }
            (*Info).rgstate[SBRG_TOPRIGHTBTN as usize] = STATE_SYSTEM_UNAVAILABLE;
            if (*Info).rgstate[SBRG_BOTTOMLEFTBTN as usize] != STATE_SYSTEM_UNAVAILABLE {
                Chg = 1;
            }
            (*Info).rgstate[SBRG_BOTTOMLEFTBTN as usize] = STATE_SYSTEM_UNAVAILABLE;
        }
        ESB_DISABLE_RTDN => {
            if Horz != 0 {
                if (*Info).rgstate[SBRG_BOTTOMLEFTBTN as usize] != STATE_SYSTEM_UNAVAILABLE {
                    Chg = 1;
                }
                (*Info).rgstate[SBRG_BOTTOMLEFTBTN as usize] = STATE_SYSTEM_UNAVAILABLE;
            } else {
                if (*Info).rgstate[SBRG_TOPRIGHTBTN as usize] != STATE_SYSTEM_UNAVAILABLE {
                    Chg = 1;
                }
                (*Info).rgstate[SBRG_TOPRIGHTBTN as usize] = STATE_SYSTEM_UNAVAILABLE;
            }
        }
        ESB_DISABLE_LTUP => {
            if Horz != 0 {
                if (*Info).rgstate[SBRG_TOPRIGHTBTN as usize] != STATE_SYSTEM_UNAVAILABLE {
                    Chg = 1;
                }
                (*Info).rgstate[SBRG_TOPRIGHTBTN as usize] = STATE_SYSTEM_UNAVAILABLE;
            } else {
                if (*Info).rgstate[SBRG_BOTTOMLEFTBTN as usize] != STATE_SYSTEM_UNAVAILABLE {
                    Chg = 1;
                }
                (*Info).rgstate[SBRG_BOTTOMLEFTBTN as usize] = STATE_SYSTEM_UNAVAILABLE;
            }
        }
        ESB_ENABLE_BOTH => {
            if (*Info).rgstate[SBRG_TOPRIGHTBTN as usize] != 0 {
                Chg = 1;
            }
            (*Info).rgstate[SBRG_TOPRIGHTBTN as usize] = 0;
            if (*Info).rgstate[SBRG_BOTTOMLEFTBTN as usize] != 0 {
                Chg = 1;
            }
            (*Info).rgstate[SBRG_BOTTOMLEFTBTN as usize] = 0;
        }
        _ => {}
    }
    Chg
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_UserShowScrollBar(
    Wnd: PWND,
    nBar: core::ffi::c_int,
    fShowH: i32,
    fShowV: i32,
) -> u32 {
    co_UserShowScrollBar_impl(Wnd, nBar, fShowH, fShowV)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_UserShowScrollBar(
    Wnd: PWND,
    nBar: core::ffi::c_int,
    fShowH: i32,
    fShowV: i32,
) -> u32 {
    co_UserShowScrollBar_impl(Wnd, nBar, fShowH, fShowV)
}

unsafe fn co_UserShowScrollBar_impl(
    Wnd: PWND,
    nBar: core::ffi::c_int,
    fShowH: i32,
    fShowV: i32,
) -> u32 {

    let old_style: u32;
    let mut set_bits: u32 = 0;
    let mut clear_bits: u32 = 0;

    loop {
        if nBar == SB_CTL as core::ffi::c_int {
            co_WinPosShowWindow(Wnd, if fShowH != 0 { SW_SHOW } else { SW_HIDE } as core::ffi::c_int);
            return 1;
        } else if nBar == SB_BOTH as core::ffi::c_int || nBar == SB_HORZ as core::ffi::c_int {
            if fShowH != 0 {
                set_bits |= WS_HSCROLL;
            } else {
                clear_bits |= WS_HSCROLL;
            }
            if nBar == SB_HORZ as core::ffi::c_int {
                break;
            }
            // Fall through to SB_VERT
            if fShowV != 0 {
                set_bits |= WS_VSCROLL;
            } else {
                clear_bits |= WS_VSCROLL;
            }
            break;
        } else if nBar == SB_VERT as core::ffi::c_int {
            if fShowV != 0 {
                set_bits |= WS_VSCROLL;
            } else {
                clear_bits |= WS_VSCROLL;
            }
            break;
        } else {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }

    old_style = IntSetStyle(Wnd, set_bits, clear_bits);
    if (old_style & clear_bits) != 0 || (old_style & set_bits) != set_bits {
        co_WinPosSetWindowPos(
            Wnd,
            core::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOZORDER | SWP_FRAMECHANGED,
        );
        return 1;
    }
    0
}

unsafe fn IntDrawScrollInterior(pWnd: PWND, hDC: HDC, nBar: INT, Vertical: BOOL, ScrollBarInfo: PSCROLLBARINFO) {
    let ThumbSize: INT = (*ScrollBarInfo).xyThumbBottom - (*ScrollBarInfo).xyThumbTop;
    let mut ThumbTop: INT = (*ScrollBarInfo).xyThumbTop;
    let mut Rect: RECT = core::mem::zeroed();
    let hSaveBrush: HBRUSH;
    let mut hBrush: HBRUSH;
    let mut TopSelected: BOOL = 0;
    let mut BottomSelected: BOOL = 0;

    if (*ScrollBarInfo).rgstate[SCROLL_TOP_RECT as usize] & STATE_SYSTEM_PRESSED != 0 {
        TopSelected = 1;
    }
    if (*ScrollBarInfo).rgstate[SCROLL_BOTTOM_RECT as usize] & STATE_SYSTEM_PRESSED != 0 {
        BottomSelected = 1;
    }

    if nBar == SB_CTL as INT {
        hBrush = GetControlBrush(pWnd, hDC, WM_CTLCOLORSCROLLBAR);
        if hBrush.is_null() {
            hBrush = IntGetSysColorBrush(COLOR_SCROLLBAR as INT) as HBRUSH;
        }
    } else {
        hBrush = DefWndControlColor(hDC, CTLCOLOR_SCROLLBAR);
    }

    hSaveBrush = NtGdiSelectBrush(hDC, hBrush);

    if Vertical != 0 {
        Rect.top = (*ScrollBarInfo).rcScrollBar.top + (*ScrollBarInfo).dxyLineButton;
        Rect.bottom = (*ScrollBarInfo).rcScrollBar.bottom - (*ScrollBarInfo).dxyLineButton;
        Rect.left = (*ScrollBarInfo).rcScrollBar.left;
        Rect.right = (*ScrollBarInfo).rcScrollBar.right;
    } else {
        Rect.top = (*ScrollBarInfo).rcScrollBar.top;
        Rect.bottom = (*ScrollBarInfo).rcScrollBar.bottom;
        Rect.left = (*ScrollBarInfo).rcScrollBar.left + (*ScrollBarInfo).dxyLineButton;
        Rect.right = (*ScrollBarInfo).rcScrollBar.right - (*ScrollBarInfo).dxyLineButton;
    }

    if (*ScrollBarInfo).xyThumbBottom == 0 {
        NtGdiPatBlt(hDC, Rect.left, Rect.top, Rect.right - Rect.left,
            Rect.bottom - Rect.top, PATCOPY as _);

        NtGdiSelectBrush(hDC, hSaveBrush);
        return;
    }

    ThumbTop -= (*ScrollBarInfo).dxyLineButton;

    if (*ScrollBarInfo).dxyLineButton != 0 {
        if Vertical != 0 {
            if ThumbSize != 0 {
                NtGdiPatBlt(hDC, Rect.left, Rect.top, Rect.right - Rect.left,
                    ThumbTop, if TopSelected != 0 { BLACKNESS as _ } else { PATCOPY as _ });
                Rect.top += ThumbTop;
                NtGdiPatBlt(hDC, Rect.left, Rect.top + ThumbSize, Rect.right - Rect.left,
                    Rect.bottom - Rect.top - ThumbSize, if BottomSelected != 0 { BLACKNESS as _ } else { PATCOPY as _ });
                Rect.bottom = Rect.top + ThumbSize;
            } else {
                if ThumbTop != 0 {
                    NtGdiPatBlt(hDC, Rect.left, (*ScrollBarInfo).dxyLineButton,
                        Rect.right - Rect.left, Rect.bottom - Rect.top, PATCOPY as _);
                }
            }
        } else {
            if ThumbSize != 0 {
                NtGdiPatBlt(hDC, Rect.left, Rect.top, ThumbTop,
                    Rect.bottom - Rect.top, if TopSelected != 0 { BLACKNESS as _ } else { PATCOPY as _ });
                Rect.left += ThumbTop;
                NtGdiPatBlt(hDC, Rect.left + ThumbSize, Rect.top,
                    Rect.right - Rect.left - ThumbSize, Rect.bottom - Rect.top,
                    if BottomSelected != 0 { BLACKNESS as _ } else { PATCOPY as _ });
                Rect.right = Rect.left + ThumbSize;
            } else {
                if ThumbTop != 0 {
                    NtGdiPatBlt(hDC, (*ScrollBarInfo).dxyLineButton, Rect.top,
                        Rect.right - Rect.left, Rect.bottom - Rect.top, PATCOPY as _);
                }
            }
        }
    }

    if ThumbSize != 0 {
        DrawEdge(hDC, core::ptr::addr_of_mut!(Rect), EDGE_RAISED, BF_RECT | BF_MIDDLE);
    }

    NtGdiSelectBrush(hDC, hSaveBrush);
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntDrawScrollArrows(hDC: HDC, ScrollBarInfo: PSCROLLBARINFO, Vertical: i32) {
    IntDrawScrollArrows_impl(hDC, ScrollBarInfo, Vertical)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntDrawScrollArrows(hDC: HDC, ScrollBarInfo: PSCROLLBARINFO, Vertical: i32) {
    IntDrawScrollArrows_impl(hDC, ScrollBarInfo, Vertical)
}

unsafe fn IntDrawScrollArrows_impl(hDC: HDC, ScrollBarInfo: PSCROLLBARINFO, Vertical: i32) {

    let mut RectLT: RECTL = (*ScrollBarInfo).rcScrollBar;
    let mut RectRB: RECTL = (*ScrollBarInfo).rcScrollBar;
    let mut ScrollDirFlagLT: u32;
    let mut ScrollDirFlagRB: u32;

    if Vertical != 0 {
        ScrollDirFlagLT = DFCS_SCROLLUP;
        ScrollDirFlagRB = DFCS_SCROLLDOWN;
        RectLT.bottom = RectLT.top + (*ScrollBarInfo).dxyLineButton;
        RectRB.top = RectRB.bottom - (*ScrollBarInfo).dxyLineButton;
    } else {
        ScrollDirFlagLT = DFCS_SCROLLLEFT;
        ScrollDirFlagRB = DFCS_SCROLLRIGHT;
        RectLT.right = RectLT.left + (*ScrollBarInfo).dxyLineButton;
        RectRB.left = RectRB.right - (*ScrollBarInfo).dxyLineButton;
    }

    if (*ScrollBarInfo).rgstate[0x01] & STATE_SYSTEM_PRESSED as u32 != 0 {
        ScrollDirFlagLT |= DFCS_PUSHED | DFCS_FLAT;
    }

    if (*ScrollBarInfo).rgstate[0x01] & STATE_SYSTEM_UNAVAILABLE as u32 != 0 {
        ScrollDirFlagLT |= DFCS_INACTIVE;
    }

    if (*ScrollBarInfo).rgstate[0x05] & STATE_SYSTEM_PRESSED as u32 != 0 {
        ScrollDirFlagRB |= DFCS_PUSHED | DFCS_FLAT;
    }

    if (*ScrollBarInfo).rgstate[0x05] & STATE_SYSTEM_UNAVAILABLE as u32 != 0 {
        ScrollDirFlagRB |= DFCS_INACTIVE;
    }

    DrawFrameControl(hDC, core::ptr::addr_of_mut!(RectLT) as *mut RECT, DFC_SCROLL, ScrollDirFlagLT);
    DrawFrameControl(hDC, core::ptr::addr_of_mut!(RectRB) as *mut RECT, DFC_SCROLL, ScrollDirFlagRB);
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntScrollGetObjectId(SBType: INT) -> LONG {
    if SBType == SB_VERT as INT {
        return OBJID_VSCROLL as LONG;
    }
    if SBType == SB_HORZ as INT {
        return OBJID_HSCROLL as LONG;
    }
    OBJID_CLIENT as LONG
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntScrollGetObjectId(SBType: INT) -> LONG {
    if SBType == SB_VERT as INT {
        return OBJID_VSCROLL as LONG;
    }
    if SBType == SB_HORZ as INT {
        return OBJID_HSCROLL as LONG;
    }
    OBJID_CLIENT as LONG
}

unsafe fn IntRefeshScrollInterior(pWnd: PWND, nBar: INT, psbi: PSCROLLBARINFO) {
    let hdc: HDC;
    let Vertical: BOOL = if nBar == SB_CTL as INT {
        (((*pWnd).style & SBS_VERT as u32) != 0) as BOOL
    } else {
        (nBar == SB_VERT as INT) as BOOL
    };

    hdc = UserGetDCEx(
        pWnd,
        core::ptr::null_mut(),
        DCX_CACHE | (if nBar == SB_CTL as INT { 0 } else { DCX_WINDOW }),
    );
    if !hdc.is_null() {
        co_IntGetScrollBarInfo(pWnd, IntScrollGetObjectId(nBar), psbi);
        IntDrawScrollInterior(pWnd, hdc, nBar, Vertical, psbi);
        UserReleaseDC(pWnd, hdc, 0);
    }
}

#[no_mangle]
pub unsafe extern "C" fn IntDrawScrollBar(Wnd: PWND, DC: HDC, Bar: INT) {

    let pti: PTHREADINFO;
    let mut Info: SCROLLBARINFO = core::mem::zeroed();
    let Vertical: BOOL;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if Bar == SB_HORZ as INT {
        Vertical = 0;
    } else if Bar == SB_VERT as INT {
        Vertical = 1;
    } else if Bar == SB_CTL as INT {
        Vertical = if (*Wnd).style & SBS_VERT as u32 != 0 { 1 } else { 0 };
    } else {
        return;
    }

    if co_IntGetScrollBarInfo(Wnd, IntScrollGetObjectId(Bar), &mut Info) == 0 {
        return;
    }

    if RECTL_bIsEmptyRect(&Info.rcScrollBar) != 0 {
        return;
    }

    if Info.dxyLineButton != 0 {
        IntDrawScrollArrows(DC, &mut Info, Vertical);
    }

    IntDrawScrollInterior(Wnd, DC, Bar, Vertical, &mut Info);

    if Wnd == (*(*pti).MessageQueue).spwndFocus && Bar == SB_CTL as INT {
        if Vertical != 0 {
            co_IntSetCaretPos(Info.rcScrollBar.top + 1, Info.dxyLineButton + 1);
        } else {
            co_IntSetCaretPos(Info.dxyLineButton + 1, Info.rcScrollBar.top + 1);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ScrollBarWndProc(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let lResult: LRESULT = 0;
    let pWnd: PWND;
    pWnd = UserGetWindowObject(hWnd);
    if pWnd.is_null() {
        return 0;
    }

    match Msg {
        WM_ENABLE => {
            if !(*pWnd).pSBInfo.is_null() {
                (*(*pWnd).pSBInfo).WSBflags = if wParam != 0 {
                    ESB_ENABLE_BOTH as _
                } else {
                    ESB_DISABLE_BOTH as _
                };
            }
        }
        _ => {}
    }
    lResult
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetScrollBarInfo(
    hWnd: HWND,
    idObject: LONG,
    psbi: PSCROLLBARINFO,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut sbi: SCROLLBARINFO = core::mem::zeroed();
    let Window: PWND;
    let mut Ret: BOOL = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    TRACE!(b"Enter NtUserGetScrollBarInfo\n\0".as_ptr() as *const i8);
    UserEnterExclusive();

    'Exit: {
        Status = MmCopyFromCaller(
            core::ptr::addr_of_mut!(sbi) as *mut _,
            psbi as *mut _,
            core::mem::size_of::<SCROLLBARINFO>() as _,
        );
        if !NT_SUCCESS(Status) || (sbi.cbSize as usize != core::mem::size_of::<SCROLLBARINFO>()) {
            SetLastNtError(Status as u32);
            break 'Exit;
        }

        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'Exit;
        }

        UserRefObjectCo(Window, core::ptr::addr_of_mut!(Ref));
        Ret = co_IntGetScrollBarInfo(Window, idObject, core::ptr::addr_of_mut!(sbi));
        UserDerefObjectCo(Window);

        Status = MmCopyToCaller(
            psbi as *mut _,
            core::ptr::addr_of_mut!(sbi) as *mut _,
            core::mem::size_of::<SCROLLBARINFO>() as _,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status as u32);
            Ret = 0;
        }
    }

    TRACE!(b"Leave NtUserGetScrollBarInfo, ret=%i\n\0".as_ptr() as *const i8, Ret);
    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSBGetParms(
    hWnd: HWND,
    fnBar: ::core::ffi::c_int,
    pSBData: PSBDATA,
    lpsi: LPSCROLLINFO,
) -> BOOL {
    let Window: PWND;
    let mut psi: SCROLLINFO = core::mem::zeroed();
    let mut Ret: BOOL = 0;
    let mut SBDataSafe: SBDATA = core::mem::zeroed();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterShared();

    'exit: {
        // lpsi and pSBData come straight from user mode and were captured with no
        // validation at all: the pre-fix upstream C at least wrapped these copies in
        // _SEH2_TRY, and the translation dropped even that, so an ordinary bad pointer
        // faulted unguarded in kernel mode. Probe first (upstream e7e1465812).
        // lpsi is read AND written below, so ProbeForWrite covers both.
        if !NT_SUCCESS(SafeProbeForWrite(
            lpsi as PVOID,
            core::mem::size_of::<SCROLLINFO>() as SIZE_T,
            1,
        )) {
            break 'exit;
        }

        RtlCopyMemory(
            core::ptr::addr_of_mut!(psi) as *mut _,
            lpsi as *const _,
            core::mem::size_of::<SCROLLINFO>() as _,
        );
        if !pSBData.is_null() {
            // Upstream made this copy unconditional and probes pSBData always, which
            // turns a legitimately-NULL optional argument into a failure. Keep our
            // NULL check and probe only when there is something to probe.
            if !NT_SUCCESS(SafeProbeForRead(
                pSBData as PVOID,
                core::mem::size_of::<SBDATA>() as SIZE_T,
                1,
            )) {
                break 'exit;
            }
            RtlCopyMemory(
                core::ptr::addr_of_mut!(SBDataSafe) as *mut _,
                pSBData as *const _,
                core::mem::size_of::<SBDATA>() as _,
            );
        }

        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'exit;
        }

        UserRefObjectCo(Window, core::ptr::addr_of_mut!(Ref));
        Ret = co_IntGetScrollInfo(
            Window,
            fnBar,
            core::ptr::addr_of_mut!(SBDataSafe),
            core::ptr::addr_of_mut!(psi),
        );
        UserDerefObjectCo(Window);

        RtlCopyMemory(
            lpsi as *mut _,
            core::ptr::addr_of!(psi) as *const _,
            core::mem::size_of::<SCROLLINFO>() as _,
        );
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserEnableScrollBar(
    hWnd: HWND,
    wSBflags: UINT,
    wArrows: UINT,
) -> BOOL {
    let OrigArrows: UINT;
    let mut Window: PWND = core::ptr::null_mut();
    let mut InfoV: PSCROLLBARINFO = core::ptr::null_mut();
    let mut InfoH: PSCROLLBARINFO = core::ptr::null_mut();
    let mut Chg: BOOL = 0;
    let mut Ret: BOOL = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterExclusive();

    'cleanup: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() || UserIsDesktopWindow(Window) != 0 || UserIsMessageWindow(Window) != 0 {
            Window = core::ptr::null_mut();
            break 'cleanup;
        }

        UserRefObjectCo(Window, &mut Ref);

        if co_IntCreateScrollBars(Window) == 0 {
            break 'cleanup;
        }

        OrigArrows = (*(*Window).pSBInfo).WSBflags as _;
        (*(*Window).pSBInfo).WSBflags = wArrows as INT;

        if wSBflags == SB_CTL {
            if wArrows == ESB_DISABLE_BOTH || wArrows == ESB_ENABLE_BOTH {
                IntEnableWindow(hWnd, (wArrows == ESB_ENABLE_BOTH) as BOOL);
            }

            Ret = 1;
            break 'cleanup;
        }

        if wSBflags != SB_BOTH && SBID_IS_VALID(wSBflags as LONG) == 0 {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'cleanup;
        }

        match wSBflags {
            SB_BOTH => {
                InfoV = IntGetScrollbarInfoFromWindow(Window, SB_VERT as INT);
                InfoH = IntGetScrollbarInfoFromWindow(Window, SB_HORZ as INT);
            }
            SB_HORZ => {
                InfoH = IntGetScrollbarInfoFromWindow(Window, SB_HORZ as INT);
            }
            SB_VERT => {
                InfoV = IntGetScrollbarInfoFromWindow(Window, SB_VERT as INT);
            }
            _ => {
                break 'cleanup;
            }
        }

        if !InfoV.is_null() {
            Chg = IntEnableScrollBar(0, InfoV, wArrows);
        }

        if !InfoH.is_null() {
            Chg = (IntEnableScrollBar(1, InfoH, wArrows) != 0 || Chg != 0) as BOOL;
        }

        Ret = Chg;
        break 'cleanup;
    }

    if !Window.is_null() {
        UserDerefObjectCo(Window);
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetScrollInfo(
    hWnd: HWND,
    fnBar: ::core::ffi::c_int,
    lpsi: LPCSCROLLINFO,
    bRedraw: BOOL,
) -> DWORD {
    let mut Window: PWND = core::ptr::null_mut();
    let Status: NTSTATUS;
    let mut ScrollInfo: SCROLLINFO = core::mem::zeroed();
    let mut Ret: DWORD = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    TRACE!(b"Enter NtUserSetScrollInfo\n\0".as_ptr() as *const _);
    UserEnterExclusive();

    'cleanup: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null()
            || UserIsDesktopWindow(Window) != 0
            || UserIsMessageWindow(Window) != 0
        {
            break 'cleanup;
        }

        UserRefObjectCo(Window, core::ptr::addr_of_mut!(Ref));

        Status = MmCopyFromCaller(
            core::ptr::addr_of_mut!(ScrollInfo) as *mut _,
            lpsi as *mut _,
            (core::mem::size_of::<SCROLLINFO>() - core::mem::size_of_val(&ScrollInfo.nTrackPos)) as _,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status as u32);
            break 'cleanup;
        }

        Ret = co_IntSetScrollInfo(Window, fnBar, core::ptr::addr_of_mut!(ScrollInfo), bRedraw);
    }

    if !Window.is_null() {
        UserDerefObjectCo(Window);
    }

    TRACE!(b"Leave NtUserSetScrollInfo, ret=%lu\n\0".as_ptr() as *const _, Ret);
    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserShowScrollBar(
    hWnd: HWND,
    nBar: core::ffi::c_int,
    bShow: u32,
) -> u32 {
    let Window: PWND;
    let mut ret: u32 = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    TRACE!(b"Enter NtUserShowScrollBar\n\0".as_ptr());
    UserEnterExclusive();

    Window = UserGetWindowObject(hWnd);
    if !Window.is_null() {
        UserRefObjectCo(Window, &mut Ref);
        ret = co_UserShowScrollBar(
            Window,
            nBar,
            if nBar == SB_VERT as core::ffi::c_int { 0 } else { bShow as i32 },
            if nBar == SB_HORZ as core::ffi::c_int { 0 } else { bShow as i32 },
        );
        UserDerefObjectCo(Window);
    }

    TRACE!(b"Leave NtUserShowScrollBar, ret=%lu\n\0".as_ptr(), ret);
    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetScrollBarInfo(
    hWnd: HWND,
    idObject: LONG,
    info: *mut SETSCROLLBARINFO,
) -> BOOL {
    let mut Window: PWND = core::ptr::null_mut();
    let mut Safeinfo: SETSCROLLBARINFO = core::mem::zeroed();
    let sbi: PSCROLLBARINFO;
    let psi: LPSCROLLINFO;
    let Status: NTSTATUS;
    let Obj: LONG;
    let mut Ret: BOOL = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterExclusive();

    'cleanup: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            break 'cleanup;
        }

        UserRefObjectCo(Window, core::ptr::addr_of_mut!(Ref));

        Obj = SBOBJ_TO_SBID(idObject);
        if SBID_IS_VALID(Obj) == 0 {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'cleanup;
        }

        if co_IntCreateScrollBars(Window) == 0 {
            break 'cleanup;
        }

        Status = MmCopyFromCaller(
            core::ptr::addr_of_mut!(Safeinfo) as *mut _,
            info as *mut _,
            core::mem::size_of::<SETSCROLLBARINFO>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status as u32);
            break 'cleanup;
        }

        sbi = IntGetScrollbarInfoFromWindow(Window, Obj);
        psi = IntGetScrollInfoFromWindow(Window, Obj);

        (*psi).nTrackPos = Safeinfo.nTrackPos;
        (*sbi).reserved = Safeinfo.reserved;
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*sbi).rgstate) as *mut _,
            core::ptr::addr_of!(Safeinfo.rgstate) as *const _,
            core::mem::size_of_val(&Safeinfo.rgstate),
        );

        Ret = 1;
    }

    if !Window.is_null() {
        UserDerefObjectCo(Window);
    }

    UserLeave();
    Ret
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: u32);
    fn co_IntSetCaretPos(X: INT, Y: INT) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: u32);
    fn co_IntSetCaretPos(X: INT, Y: INT) -> BOOL;
}