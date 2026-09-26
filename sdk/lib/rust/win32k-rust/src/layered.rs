#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS};
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::ddk::ExAllocatePoolWithTag;


// Not in ntkrnl_sys::base; the tree declares it per-file where a fault has to be
// reported as a status rather than swallowed.
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// USERTAG_REDIRECT: C 'rdrU' (big-endian multi-char) = 0x72647255 → LE bytes "Urdr".
const USERTAG_REDIRECT: u32 = u32::from_le_bytes(*b"Urdr");

// SafeProbeForRead/Write — NTAPI stdcall (`_Name@N`).
extern "system" {
    fn SafeProbeForWrite(Address: *mut core::ffi::c_void, Length: SIZE_T, Alignment: u32) -> i32;
    fn SafeProbeForRead(Address: *mut core::ffi::c_void, Length: usize, Alignment: u32) -> i32;
}

// Cross-file fastcall (`@Name@N` in nm) — fastcall on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_bIntersectRect(dst: *mut RECT, src1: *mut RECT, src2: *mut RECT) -> u32;
    fn co_WinPosSetWindowPos(wnd: PWND, insertAfter: PWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> u32;
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_bIntersectRect(dst: *mut RECT, src1: *mut RECT, src2: *mut RECT) -> u32;
    fn co_WinPosSetWindowPos(wnd: PWND, insertAfter: PWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> u32;
    fn SetLastNtError(Status: NTSTATUS);
}

// RECTL_vOffsetRect is a FORCEINLINE in win32ss/gdi/ntgdi/rect.h (no exported
// symbol) — reimplement inline. Offsets a rect by (cx, cy).
#[inline(always)]
unsafe fn RECTL_vOffsetRect(rect: *mut RECT, cx: i32, cy: i32) {
    (*rect).left += cx;
    (*rect).right += cx;
    (*rect).top += cy;
    (*rect).bottom += cy;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _LRD_PROP {
    pub Key: COLORREF,
    pub Alpha: BYTE,
    pub is_Layered: BYTE,
    pub NoUsed: [BYTE; 2],
    pub Flags: DWORD,
}
pub type LRD_PROP = _LRD_PROP;
pub type PLRD_PROP = *mut _LRD_PROP;

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn GetLayeredStatus(pWnd: PWND) -> BOOL {
    GetLayeredStatus_impl(pWnd)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn GetLayeredStatus(pWnd: PWND) -> BOOL {
    GetLayeredStatus_impl(pWnd)
}

unsafe fn GetLayeredStatus_impl(pWnd: PWND) -> BOOL {
    let pLrdProp = UserGetProp(pWnd, AtomLayer, 1u8) as PLRD_PROP;
    if !pLrdProp.is_null() {
        return (*pLrdProp).is_Layered as BOOL;
    }
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn SetLayeredStatus(pWnd: PWND, set: u8) -> BOOL {
    SetLayeredStatusImpl(pWnd, set)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn SetLayeredStatus(pWnd: PWND, set: u8) -> BOOL {
    SetLayeredStatusImpl(pWnd, set)
}

unsafe fn SetLayeredStatusImpl(pWnd: PWND, set: u8) -> BOOL {
    let pLrdProp = UserGetProp(pWnd, AtomLayer, 1u8) as PLRD_PROP;
    if !pLrdProp.is_null() {
        (*pLrdProp).is_Layered = set;
        return 1;
    }
    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSetLayeredWindowAttributes(
    pWnd: PWND,
    crKey: COLORREF,
    bAlpha: u8,
    dwFlags: u32,
) -> i32 {
    IntSetLayeredWindowAttributesImpl(pWnd, crKey, bAlpha, dwFlags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSetLayeredWindowAttributes(
    pWnd: PWND,
    crKey: COLORREF,
    bAlpha: u8,
    dwFlags: u32,
) -> i32 {
    IntSetLayeredWindowAttributesImpl(pWnd, crKey, bAlpha, dwFlags)
}

unsafe fn IntSetLayeredWindowAttributesImpl(
    pWnd: PWND,
    crKey: COLORREF,
    bAlpha: u8,
    dwFlags: u32,
) -> i32 {
    let mut pLrdProp: *mut LRD_PROP;
    let was_Layered: i32;

    if ((*pWnd).ExStyle & WS_EX_LAYERED) == 0 {
        return 0;
    }

    pLrdProp = UserGetProp(pWnd, AtomLayer, 1u8) as *mut LRD_PROP;

    if pLrdProp.is_null() {
        pLrdProp = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<LRD_PROP>() as _,
            USERTAG_REDIRECT,
        ) as *mut LRD_PROP;
        if pLrdProp.is_null() {
            return 0;
        }
        RtlZeroMemory(pLrdProp as *mut _, core::mem::size_of::<LRD_PROP>());
        UserSetProp(pWnd, AtomLayer, pLrdProp as *mut _, 1u8);
    }

    if !pLrdProp.is_null() {
        was_Layered = (*pLrdProp).is_Layered as i32;

        (*pLrdProp).Key = crKey;

        if (dwFlags & LWA_ALPHA) != 0 {
            (*pLrdProp).Alpha = bAlpha;
        } else {
            if (*pLrdProp).is_Layered == 0 {
                (*pLrdProp).Alpha = 0;
            }
        }

        (*pLrdProp).Flags = dwFlags;

        (*pLrdProp).is_Layered = 1;

        if was_Layered == 0 {
            co_UserRedrawWindow(
                pWnd,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                RDW_INVALIDATE | RDW_ERASE | RDW_FRAME,
            );
        }
    }
    1
}

#[cfg(target_arch = "x86")]
#[allow(non_snake_case)]
unsafe extern "fastcall" fn IntUpdateLayeredWindowI(pWnd: PWND, info: *mut UPDATELAYEREDWINDOWINFO) -> u32 {
    IntUpdateLayeredWindowI_impl(pWnd, info)
}

#[cfg(not(target_arch = "x86"))]
#[allow(non_snake_case)]
unsafe extern "C" fn IntUpdateLayeredWindowI(pWnd: PWND, info: *mut UPDATELAYEREDWINDOWINFO) -> u32 {
    IntUpdateLayeredWindowI_impl(pWnd, info)
}

#[allow(non_snake_case)]
unsafe fn IntUpdateLayeredWindowI_impl(pWnd: PWND, info: *mut UPDATELAYEREDWINDOWINFO) -> u32 {

    let mut Window: RECT;
    let mut Client: RECT;
    let mut Offset: SIZE = core::mem::zeroed();
    let mut ret: u32 = 0;
    let mut flags: u32 =
        SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOREDRAW;

    Window = *(core::ptr::addr_of!((*pWnd).rcWindow) as *const RECT);
    Client = *(core::ptr::addr_of!((*pWnd).rcClient) as *const RECT);

    let Parent: PWND = (*pWnd).spwndParent;
    if ((*pWnd).style & WS_CHILD) != 0 && !Parent.is_null() {
        RECTL_vOffsetRect(
            core::ptr::addr_of_mut!(Window),
            -(*Parent).rcClient.left,
            -(*Parent).rcClient.top,
        );
        RECTL_vOffsetRect(
            core::ptr::addr_of_mut!(Client),
            -(*Parent).rcClient.left,
            -(*Parent).rcClient.top,
        );
    }

    if !(*info).pptDst.is_null() {
        Offset.cx = (*(*info).pptDst).x - Window.left;
        Offset.cy = (*(*info).pptDst).y - Window.top;
        RECTL_vOffsetRect(core::ptr::addr_of_mut!(Client), Offset.cx, Offset.cy);
        RECTL_vOffsetRect(core::ptr::addr_of_mut!(Window), Offset.cx, Offset.cy);
        flags &= !SWP_NOMOVE;
    }
    if !(*info).psize.is_null() {
        Offset.cx = (*(*info).psize).cx - (Window.right - Window.left);
        Offset.cy = (*(*info).psize).cy - (Window.bottom - Window.top);
        if (*(*info).psize).cx <= 0 || (*(*info).psize).cy <= 0 {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
        if ((*info).dwFlags & ULW_EX_NORESIZE) != 0 && (Offset.cx != 0 || Offset.cy != 0) {
            EngSetLastError(ERROR_INCORRECT_SIZE);
            return 0;
        }
        Client.right += Offset.cx;
        Client.bottom += Offset.cy;
        Window.right += Offset.cx;
        Window.bottom += Offset.cy;
        flags &= !SWP_NOSIZE;
    }

    if !(*info).hdcSrc.is_null() {
        let hdc: HDC;
        let hdcBuffer: HDC;
        let mut Rect: RECT;
        let mut blend: BLENDFUNCTION = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: 0,
        };
        let color_key: COLORREF = if ((*info).dwFlags & ULW_COLORKEY) != 0 {
            (*info).crKey
        } else {
            CLR_INVALID
        };
        let hOldBitmap: HBITMAP;
        let hOldBitmap1: HBITMAP;
        let hbmSrc: HBITMAP;
        let hbmDst: HBITMAP;
        let mut dibs: DIBSECTION = core::mem::zeroed();

        Rect = Window;

        RECTL_vOffsetRect(core::ptr::addr_of_mut!(Rect), -Window.left, -Window.top);

        if (*info).hdcDst.is_null() {
            hdc = UserGetDCEx(pWnd, core::ptr::null_mut(), DCX_USESTYLE);
        } else {
            hdc = (*info).hdcDst;
        }

        hbmSrc = NtGdiCreateCompatibleBitmap(
            (*info).hdcSrc,
            Rect.right - Rect.left,
            Rect.bottom - Rect.top,
        );
        hbmDst = NtGdiCreateCompatibleBitmap(
            (*info).hdcSrc,
            Rect.right - Rect.left,
            Rect.bottom - Rect.top,
        );

        GreGetObject(
            hbmSrc as _,
            core::mem::size_of::<DIBSECTION>() as _,
            core::ptr::addr_of_mut!(dibs) as _,
        );

        hdcBuffer = NtGdiCreateCompatibleDC(hdc);

        hOldBitmap = NtGdiSelectBitmap(hdcBuffer, hbmSrc) as HBITMAP;
        hOldBitmap1 = NtGdiSelectBitmap(hdc, hbmDst) as HBITMAP;

        NtGdiStretchBlt(
            hdcBuffer,
            Rect.left,
            Rect.top,
            Rect.right - Rect.left,
            Rect.bottom - Rect.top,
            (*info).hdcSrc,
            Rect.left + if !(*info).pptSrc.is_null() { (*(*info).pptSrc).x } else { 0 },
            Rect.top + if !(*info).pptSrc.is_null() { (*(*info).pptSrc).y } else { 0 },
            Rect.right - Rect.left,
            Rect.bottom - Rect.top,
            SRCCOPY,
            color_key,
        );

        if !(*info).prcDirty.is_null() {
            RECTL_bIntersectRect(
                core::ptr::addr_of_mut!(Rect),
                core::ptr::addr_of_mut!(Rect),
                (*info).prcDirty as *mut _,
            );
            NtGdiPatBlt(
                hdc,
                Rect.left,
                Rect.top,
                Rect.right - Rect.left,
                Rect.bottom - Rect.top,
                BLACKNESS as _,
            );
        }

        if ((*info).dwFlags & ULW_ALPHA) != 0 {
            blend = *(*info).pblend;
        }

        ret = NtGdiAlphaBlend(
            hdc,
            Rect.left,
            Rect.top,
            Rect.right - Rect.left,
            Rect.bottom - Rect.top,
            hdcBuffer,
            Rect.left + if !(*info).pptSrc.is_null() { (*(*info).pptSrc).x } else { 0 },
            Rect.top + if !(*info).pptSrc.is_null() { (*(*info).pptSrc).y } else { 0 },
            Rect.right - Rect.left,
            Rect.bottom - Rect.top,
            blend,
            core::ptr::null_mut(),
        ) as _;

        NtGdiSelectBitmap(hdc, hOldBitmap1);
        NtGdiSelectBitmap(hdcBuffer, hOldBitmap);
        if !hbmSrc.is_null() {
            GreDeleteObject(hbmSrc as _);
        }
        if !hbmDst.is_null() {
            GreDeleteObject(hbmDst as _);
        }
        if !hdcBuffer.is_null() {
            IntGdiDeleteDC(hdcBuffer, 0);
        }
        if (*info).hdcDst.is_null() {
            UserReleaseDC(pWnd, hdc, 0);
        }
    } else {
        ret = 1;
    }

    co_WinPosSetWindowPos(
        pWnd,
        core::ptr::null_mut(),
        Window.left,
        Window.top,
        Window.right - Window.left,
        Window.bottom - Window.top,
        flags,
    );
    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetLayeredWindowAttributes(
    hwnd: HWND,
    pcrKey: *mut COLORREF,
    pbAlpha: *mut u8,
    pdwFlags: *mut u32,
) -> i32 {
    let pLrdProp: PLRD_PROP;
    let pWnd: PWND;
    let mut Ret: i32 = 0;
    let mut Status: i32;

    UserEnterExclusive();

    'block: {
        pWnd = UserGetWindowObject(hwnd);
        if pWnd.is_null() || ((*pWnd).ExStyle & WS_EX_LAYERED) == 0 {
            break 'block;
        }

        pLrdProp = UserGetProp(pWnd, AtomLayer, 1u8) as PLRD_PROP;

        if pLrdProp.is_null() {
            break 'block;
        }

        if (*pLrdProp).is_Layered == 0 {
            break 'block;
        }

        if !pcrKey.is_null() {
            Status = SafeProbeForWrite(
                pcrKey as *mut core::ffi::c_void,
                core::mem::size_of::<COLORREF>() as u32,
                1,
            );
            if NT_SUCCESS(Status) {
                Status = STATUS_SUCCESS;
            }
        } else {
            Status = STATUS_SUCCESS;
        }
        if !pbAlpha.is_null() && NT_SUCCESS(Status) {
            Status = SafeProbeForWrite(
                pbAlpha as *mut core::ffi::c_void,
                core::mem::size_of::<u8>() as u32,
                1,
            );
        }
        if !pdwFlags.is_null() && NT_SUCCESS(Status) {
            Status = SafeProbeForWrite(
                pdwFlags as *mut core::ffi::c_void,
                core::mem::size_of::<u32>() as u32,
                1,
            );
        }
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'block;
        } else {
            if !pcrKey.is_null() {
                // The probes VALIDATE only; this STORE is the access.
                let v = (*pLrdProp).Key;
                if !crate::copy::publish_to_user(
                    pcrKey as *mut core::ffi::c_void,
                    core::ptr::addr_of!(v) as *const core::ffi::c_void,
                    core::mem::size_of_val(&v),
                ) {
                    SetLastNtError(STATUS_ACCESS_VIOLATION);
                    break 'block;
                }
            }
            if !pbAlpha.is_null() {
                // The probes VALIDATE only; this STORE is the access.
                let v = (*pLrdProp).Alpha;
                if !crate::copy::publish_to_user(
                    pbAlpha as *mut core::ffi::c_void,
                    core::ptr::addr_of!(v) as *const core::ffi::c_void,
                    core::mem::size_of_val(&v),
                ) {
                    SetLastNtError(STATUS_ACCESS_VIOLATION);
                    break 'block;
                }
            }
            if !pdwFlags.is_null() {
                // The probes VALIDATE only; this STORE is the access.
                let v = (*pLrdProp).Flags;
                if !crate::copy::publish_to_user(
                    pdwFlags as *mut core::ffi::c_void,
                    core::ptr::addr_of!(v) as *const core::ffi::c_void,
                    core::mem::size_of_val(&v),
                ) {
                    SetLastNtError(STATUS_ACCESS_VIOLATION);
                    break 'block;
                }
            }
        }

        Ret = 1;
    }

    UserLeave();
    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetLayeredWindowAttributes(
    hwnd: HWND,
    pcrKey: *mut COLORREF,
    pbAlpha: *mut u8,
    pdwFlags: *mut u32,
) -> i32 {
    let pLrdProp: PLRD_PROP;
    let pWnd: PWND;
    let mut Ret: i32 = 0;
    let mut Status: i32;

    UserEnterExclusive();

    'block: {
        pWnd = UserGetWindowObject(hwnd);
        if pWnd.is_null() || ((*pWnd).ExStyle & WS_EX_LAYERED) == 0 {
            break 'block;
        }

        pLrdProp = UserGetProp(pWnd, AtomLayer, 1u8) as PLRD_PROP;

        if pLrdProp.is_null() {
            break 'block;
        }

        if (*pLrdProp).is_Layered == 0 {
            break 'block;
        }

        if !pcrKey.is_null() {
            Status = SafeProbeForWrite(
                pcrKey as *mut core::ffi::c_void,
                core::mem::size_of::<COLORREF>() as _,
                1,
            );
            if NT_SUCCESS(Status) {
                Status = STATUS_SUCCESS;
            }
        } else {
            Status = STATUS_SUCCESS;
        }
        if !pbAlpha.is_null() && NT_SUCCESS(Status) {
            Status = SafeProbeForWrite(
                pbAlpha as *mut core::ffi::c_void,
                core::mem::size_of::<u8>() as _,
                1,
            );
        }
        if !pdwFlags.is_null() && NT_SUCCESS(Status) {
            Status = SafeProbeForWrite(
                pdwFlags as *mut core::ffi::c_void,
                core::mem::size_of::<u32>() as _,
                1,
            );
        }
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'block;
        } else {
            if !pcrKey.is_null() {
                // The probes VALIDATE only; this STORE is the access.
                let v = (*pLrdProp).Key;
                if !crate::copy::publish_to_user(
                    pcrKey as *mut core::ffi::c_void,
                    core::ptr::addr_of!(v) as *const core::ffi::c_void,
                    core::mem::size_of_val(&v),
                ) {
                    SetLastNtError(STATUS_ACCESS_VIOLATION);
                    break 'block;
                }
            }
            if !pbAlpha.is_null() {
                // The probes VALIDATE only; this STORE is the access.
                let v = (*pLrdProp).Alpha;
                if !crate::copy::publish_to_user(
                    pbAlpha as *mut core::ffi::c_void,
                    core::ptr::addr_of!(v) as *const core::ffi::c_void,
                    core::mem::size_of_val(&v),
                ) {
                    SetLastNtError(STATUS_ACCESS_VIOLATION);
                    break 'block;
                }
            }
            if !pdwFlags.is_null() {
                // The probes VALIDATE only; this STORE is the access.
                let v = (*pLrdProp).Flags;
                if !crate::copy::publish_to_user(
                    pdwFlags as *mut core::ffi::c_void,
                    core::ptr::addr_of!(v) as *const core::ffi::c_void,
                    core::mem::size_of_val(&v),
                ) {
                    SetLastNtError(STATUS_ACCESS_VIOLATION);
                    break 'block;
                }
            }
        }

        Ret = 1;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetLayeredWindowAttributes(
    hwnd: HWND,
    crKey: COLORREF,
    bAlpha: u8,
    dwFlags: u32,
) -> i32 {
    let pWnd: PWND;
    let mut Ret: i32 = 0;

    UserEnterExclusive();

    'exit: {
        pWnd = UserGetWindowObject(hwnd);
        if pWnd.is_null() || ((*pWnd).ExStyle & WS_EX_LAYERED) == 0 {
            break 'exit;
        }

        Ret = IntSetLayeredWindowAttributes(pWnd, crKey, bAlpha, dwFlags);
    }

    UserLeave();
    Ret
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn NtUserUpdateLayeredWindow(
    hwnd: HWND,
    hdcDst: HDC,
    pptDst: *mut POINT,
    psize: *mut SIZE,
    hdcSrc: HDC,
    pptSrc: *mut POINT,
    crKey: COLORREF,
    pblend: *mut BLENDFUNCTION,
    dwFlags: u32,
    prcDirty: *mut RECT,
) -> i32 {

    let mut info: UPDATELAYEREDWINDOWINFO = core::mem::zeroed();
    let mut Dst: POINT = core::mem::zeroed();
    let mut Src: POINT = core::mem::zeroed();
    let mut Size: SIZE = core::mem::zeroed();
    let mut Dirty: RECT = core::mem::zeroed();
    let mut blend: BLENDFUNCTION = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: 0,
    };
    let pWnd: PWND;
    let mut Ret: i32 = 0;
    let mut Status: i32;

    UserEnterExclusive();

    'exit: loop {
        pWnd = UserGetWindowObject(hwnd);
        if pWnd.is_null() {
            break 'exit;
        }

        Status = STATUS_SUCCESS;
        if !pptDst.is_null() && NT_SUCCESS(Status) {
            Status = SafeProbeForRead(pptDst as *mut core::ffi::c_void, core::mem::size_of::<POINT>(), 1);
        }
        if !pptSrc.is_null() && NT_SUCCESS(Status) {
            Status = SafeProbeForRead(pptSrc as *mut core::ffi::c_void, core::mem::size_of::<POINT>(), 1);
        }
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(psize as *mut core::ffi::c_void, core::mem::size_of::<SIZE>(), 1);
        }
        if !pblend.is_null() && NT_SUCCESS(Status) {
            Status = SafeProbeForRead(pblend as *mut core::ffi::c_void, core::mem::size_of::<BLENDFUNCTION>(), 1);
        }
        if !prcDirty.is_null() && NT_SUCCESS(Status) {
            Status = SafeProbeForRead(prcDirty as *mut core::ffi::c_void, core::mem::size_of::<RECT>(), 1);
        }
        if !NT_SUCCESS(Status) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'exit;
        } else {
            // Probes are validate-only (USE_DUMMY_PSEH); the C wrapped every probe
            // AND every load in one _SEH2_TRY, so the loads must be the
            // fault-tolerant copy.
            macro_rules! grab {
                ($dst:expr, $src:expr, $ty:ty) => {
                    crate::copy::capture_from_user(
                        core::ptr::addr_of_mut!($dst) as *mut core::ffi::c_void,
                        $src as *const core::ffi::c_void,
                        core::mem::size_of::<$ty>(),
                    )
                };
            }
            let ok = (pptDst.is_null() || grab!(Dst, pptDst, POINT))
                && (pptSrc.is_null() || grab!(Src, pptSrc, POINT))
                && grab!(Size, psize, SIZE)
                && (pblend.is_null() || grab!(blend, pblend, BLENDFUNCTION))
                && (prcDirty.is_null() || grab!(Dirty, prcDirty, RECT));
            if !ok {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                break 'exit;
            }
        }

        if GetLayeredStatus(pWnd) != 0
            || (dwFlags & !(ULW_COLORKEY | ULW_ALPHA | ULW_OPAQUE | ULW_EX_NORESIZE)) != 0
            || ((*pWnd).ExStyle & WS_EX_LAYERED) == 0
        {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'exit;
        }

        info.cbSize = core::mem::size_of::<UPDATELAYEREDWINDOWINFO>() as u32;
        info.hdcDst = hdcDst;
        info.pptDst = if !pptDst.is_null() { core::ptr::addr_of_mut!(Dst) as _ } else { core::ptr::null_mut() };
        info.psize = core::ptr::addr_of_mut!(Size);
        info.hdcSrc = hdcSrc;
        info.pptSrc = if !pptSrc.is_null() { core::ptr::addr_of_mut!(Src) as _ } else { core::ptr::null_mut() };
        info.crKey = crKey;
        info.pblend = core::ptr::addr_of_mut!(blend);
        info.dwFlags = dwFlags;
        info.prcDirty = if !prcDirty.is_null() { core::ptr::addr_of_mut!(Dirty) as _ } else { core::ptr::null_mut() };
        Ret = IntUpdateLayeredWindowI(pWnd, core::ptr::addr_of_mut!(info)) as i32;

        break 'exit;
    }

    UserLeave();
    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          Layered window support
//  * FILE:             win32ss/user/ntuser/layered.c
//  * PROGRAMER:
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
// DBG_DEFAULT_CHANNEL(UserMisc);
//
//
// typedef struct _LRD_PROP
// { 
//    COLORREF Key;
//    BYTE     Alpha;
//    BYTE     is_Layered;
//    BYTE     NoUsed[2];
//    DWORD    Flags;
// } LRD_PROP, *PLRD_PROP;
//
// BOOL FASTCALL
// GetLayeredStatus(PWND pWnd)
// {
//    PLRD_PROP pLrdProp = UserGetProp(pWnd, AtomLayer, TRUE);
//    if (pLrdProp)
//    {
//       return pLrdProp->is_Layered;
//    }
//    return FALSE;
// }
//
// BOOL FASTCALL
// SetLayeredStatus(PWND pWnd, BYTE set)
// {
//    PLRD_PROP pLrdProp = UserGetProp(pWnd, AtomLayer, TRUE);
//    if (pLrdProp)
//    {
//       pLrdProp->is_Layered = set;
//       return TRUE;
//    }
//    return FALSE;
// }
//
// BOOL FASTCALL
// IntSetLayeredWindowAttributes(PWND pWnd,
//                               COLORREF crKey,
//                               BYTE bAlpha,
//                               DWORD dwFlags)
// {
//    PLRD_PROP pLrdProp;
//    INT was_Layered;
//
//    if (!(pWnd->ExStyle & WS_EX_LAYERED) )
//    {
//       ERR("Not a Layered Window!\n");
//       return FALSE;
//    }
//
//    pLrdProp = UserGetProp(pWnd, AtomLayer, TRUE);
//
//    if (!pLrdProp)
//    {
//       pLrdProp = ExAllocatePoolWithTag(PagedPool, sizeof(LRD_PROP), USERTAG_REDIRECT);
//       if (pLrdProp == NULL)
//       {
//          ERR("failed to allocate LRD_PROP\n");
//          return FALSE;
//       }
//       RtlZeroMemory(pLrdProp, sizeof(LRD_PROP));
//       UserSetProp(pWnd, AtomLayer, (HANDLE)pLrdProp, TRUE);
//    }
//
//    if (pLrdProp)
//    {
//       was_Layered = pLrdProp->is_Layered;
//
//       pLrdProp->Key = crKey;
//
//       if (dwFlags & LWA_ALPHA)
//       {
//          pLrdProp->Alpha = bAlpha;
//       }
//       else
//       {
//          if (!pLrdProp->is_Layered) pLrdProp->Alpha = 0;
//       }
//
//       pLrdProp->Flags = dwFlags;
//
//       pLrdProp->is_Layered = 1;
//   
//       if (!was_Layered)
//          co_UserRedrawWindow(pWnd, NULL, NULL, RDW_INVALIDATE | RDW_ERASE | RDW_FRAME );
//    }
//    // FIXME: Now set some bits to the Window DC!!!!
//    return TRUE;
// }
//
// BOOL FASTCALL
// IntUpdateLayeredWindowI( PWND pWnd,
//                          UPDATELAYEREDWINDOWINFO *info )
// {
//    PWND Parent;
//    RECT Window, Client;
//    SIZE Offset;
//    BOOL ret = FALSE;
//    DWORD flags = SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOREDRAW;
//
//    Window = pWnd->rcWindow;
//    Client = pWnd->rcClient;
//
//    Parent = pWnd->spwndParent;
//    if (pWnd->style & WS_CHILD && Parent)
//    {
//       RECTL_vOffsetRect(&Window, - Parent->rcClient.left, - Parent->rcClient.top);
//       RECTL_vOffsetRect(&Client, - Parent->rcClient.left, - Parent->rcClient.top);
//    }
//
//    if (info->pptDst)
//    {
//       Offset.cx = info->pptDst->x - Window.left;
//       Offset.cy = info->pptDst->y - Window.top;
//       RECTL_vOffsetRect( &Client, Offset.cx, Offset.cy );
//       RECTL_vOffsetRect( &Window, Offset.cx, Offset.cy );
//       flags &= ~SWP_NOMOVE;
//    }
//    if (info->psize)
//    {
//       Offset.cx = info->psize->cx - (Window.right  - Window.left);
//       Offset.cy = info->psize->cy - (Window.bottom - Window.top);
//       if (info->psize->cx <= 0 || info->psize->cy <= 0)
//       {
//           ERR("Update Layered Invalid Parameters\n");
//           EngSetLastError( ERROR_INVALID_PARAMETER );
//           return FALSE;
//       }
//       if ((info->dwFlags & ULW_EX_NORESIZE) && (Offset.cx || Offset.cy))
//       {
//           ERR("Wrong Size\n");
//           EngSetLastError( ERROR_INCORRECT_SIZE );
//           return FALSE;
//       }
//       Client.right  += Offset.cx;
//       Client.bottom += Offset.cy;
//       Window.right  += Offset.cx;
//       Window.bottom += Offset.cy;
//       flags &= ~SWP_NOSIZE;
//    }
//
//    if (info->hdcSrc)
//    {
//       HDC hdc, hdcBuffer;
//       RECT Rect;
//       BLENDFUNCTION blend = { AC_SRC_OVER, 0, 255, 0 };
//       COLORREF color_key = (info->dwFlags & ULW_COLORKEY) ? info->crKey : CLR_INVALID;
//       HBITMAP hOldBitmap, hOldBitmap1, hbmSrc, hbmDst;
//       DIBSECTION dibs;
//
//       Rect = Window;
//
//       RECTL_vOffsetRect( &Rect, -Window.left, -Window.top );
//
//       TRACE("H %d W %d\n",Rect.bottom - Rect.top,Rect.right - Rect.left);
//
//       if (!info->hdcDst) hdc = UserGetDCEx(pWnd, NULL, DCX_USESTYLE);
//       else hdc = info->hdcDst;
//
//       hbmSrc = NtGdiCreateCompatibleBitmap(info->hdcSrc, Rect.right - Rect.left, Rect.bottom - Rect.top);
//       hbmDst = NtGdiCreateCompatibleBitmap(info->hdcSrc, Rect.right - Rect.left, Rect.bottom - Rect.top);
//
//       GreGetObject(hbmSrc, sizeof(DIBSECTION), &dibs);
//
//       TRACE("Source Bitmap bc %d\n",dibs.dsBmih.biBitCount);
//
//       hdcBuffer = NtGdiCreateCompatibleDC(hdc);
//
//       hOldBitmap = (HBITMAP)NtGdiSelectBitmap(hdcBuffer, hbmSrc);
//       hOldBitmap1 = (HBITMAP)NtGdiSelectBitmap(hdc, hbmDst);
//
//       NtGdiStretchBlt( hdcBuffer,
//                        Rect.left,
//                        Rect.top,
//                        Rect.right - Rect.left,
//                        Rect.bottom - Rect.top,
//                        info->hdcSrc,
//                        Rect.left + (info->pptSrc ? info->pptSrc->x : 0),
//                        Rect.top  + (info->pptSrc ? info->pptSrc->y : 0),
//                        Rect.right - Rect.left,
//                        Rect.bottom - Rect.top,
//                        SRCCOPY,
//                        color_key );
//
//       // Need to test this, Dirty before or after StretchBlt?
//       if (info->prcDirty)
//       {
//          ERR("prcDirty\n");
//          RECTL_bIntersectRect( &Rect, &Rect, info->prcDirty );
//          NtGdiPatBlt( hdc, Rect.left, Rect.top, Rect.right - Rect.left, Rect.bottom - Rect.top, BLACKNESS );
//       }
//
//       if (info->dwFlags & ULW_ALPHA)
//       {
//          blend = *info->pblend;
//          TRACE("ULW_ALPHA bop %d Alpha %d aF %d\n", blend.BlendOp, blend.SourceConstantAlpha, blend.AlphaFormat);
//       }
//
//       ret = NtGdiAlphaBlend( hdc,
//                              Rect.left,
//                              Rect.top,
//                              Rect.right - Rect.left,
//                              Rect.bottom - Rect.top,
//                              hdcBuffer,
//                              Rect.left + (info->pptSrc ? info->pptSrc->x : 0),
//                              Rect.top  + (info->pptSrc ? info->pptSrc->y : 0),
//                              Rect.right - Rect.left,
//                              Rect.bottom - Rect.top,
//                              blend,
//                              0);
//
//       NtGdiSelectBitmap(hdc, hOldBitmap1);
//       NtGdiSelectBitmap(hdcBuffer, hOldBitmap);
//       if (hbmSrc) GreDeleteObject(hbmSrc);
//       if (hbmDst) GreDeleteObject(hbmDst);
//       if (hdcBuffer) IntGdiDeleteDC(hdcBuffer, FALSE);
//       if (!info->hdcDst) UserReleaseDC(pWnd, hdc, FALSE);
//    }
//    else
//       ret = TRUE;
//
//    co_WinPosSetWindowPos(pWnd, 0, Window.left, Window.top, Window.right - Window.left, Window.bottom - Window.top, flags);
//    return ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// __stdcall
// NtUserGetLayeredWindowAttributes(
//     HWND hwnd,
//     COLORREF *pcrKey,
//     BYTE *pbAlpha,
//     DWORD *pdwFlags)
// {
//    PLRD_PROP pLrdProp;
//    PWND pWnd;
//    BOOL Ret = FALSE;
//    NTSTATUS Status;
//
//    TRACE("Enter NtUserGetLayeredWindowAttributes\n");
//    UserEnterExclusive();
//
//    if (!(pWnd = UserGetWindowObject(hwnd)) ||
//        !(pWnd->ExStyle & WS_EX_LAYERED) )
//    {
//       ERR("Not a Layered Window!\n");
//       goto Exit;
//    }
//
//    pLrdProp = UserGetProp(pWnd, AtomLayer, TRUE);
//
//    if (!pLrdProp)
//    {
//       TRACE("No Prop!\n");
//       goto Exit;
//    }
//
//    if (pLrdProp->is_Layered == 0)
//    {
//       goto Exit;
//    }
//
//    if (pcrKey)
//    {
//        Status = SafeProbeForWrite(pcrKey, sizeof(*pcrKey), 1);
//        if (NT_SUCCESS(Status))
//        {
//            Status = STATUS_SUCCESS;
//        }
//    }
//    else
//    {
//        Status = STATUS_SUCCESS;
//    }
//    if (pbAlpha && NT_SUCCESS(Status))
//    {
//        Status = SafeProbeForWrite(pbAlpha, sizeof(*pbAlpha), 1);
//    }
//    if (pdwFlags && NT_SUCCESS(Status))
//    {
//        Status = SafeProbeForWrite(pdwFlags, sizeof(*pdwFlags), 1);
//    }
//    if (!NT_SUCCESS(Status))
//    {
//       SetLastNtError(Status);
//       goto Exit;
//    }
//    else
//    {
//       if (pcrKey)
//       {
//           *pcrKey = pLrdProp->Key;
//       }
//       if (pbAlpha)
//       {
//           *pbAlpha = pLrdProp->Alpha;
//       }
//       if (pdwFlags)
//       {
//           *pdwFlags = pLrdProp->Flags;
//       }
//    }
//
//    Ret = TRUE;
//
// Exit:
//    TRACE("Leave NtUserGetLayeredWindowAttributes, ret=%i\n", Ret);
//    UserLeave();
//    return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL __stdcall
// NtUserSetLayeredWindowAttributes(HWND hwnd,
// 			   COLORREF crKey,
// 			   BYTE bAlpha,
// 			   DWORD dwFlags)
// {
//    PWND pWnd;
//    BOOL Ret = FALSE;
//
//    TRACE("Enter NtUserSetLayeredWindowAttributes\n");
//    UserEnterExclusive();
//
//    if (!(pWnd = UserGetWindowObject(hwnd)) ||
//        !(pWnd->ExStyle & WS_EX_LAYERED) )
//    {
//       ERR("Not a Layered Window!\n");
//       goto Exit;
//    }
//
//    Ret = IntSetLayeredWindowAttributes(pWnd, crKey, bAlpha, dwFlags);
// Exit:
//    TRACE("Leave NtUserSetLayeredWindowAttributes, ret=%i\n", Ret);
//    UserLeave();
//    return Ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// __stdcall
// NtUserUpdateLayeredWindow(
//    HWND hwnd,
//    HDC hdcDst,
//    POINT *pptDst,
//    SIZE *psize,
//    HDC hdcSrc,
//    POINT *pptSrc,
//    COLORREF crKey,
//    BLENDFUNCTION *pblend,
//    DWORD dwFlags,
//    RECT *prcDirty)
// {
//    UPDATELAYEREDWINDOWINFO info;
//    POINT Dst, Src; 
//    SIZE Size;
//    RECT Dirty;
//    BLENDFUNCTION blend = { AC_SRC_OVER, 0, 255, 0 };
//    PWND pWnd;
//    BOOL Ret = FALSE;
//    NTSTATUS Status;
//
//    TRACE("Enter NtUserUpdateLayeredWindow\n");
//    UserEnterExclusive();
//
//    if (!(pWnd = UserGetWindowObject(hwnd)))
//    {
//       goto Exit;
//    }
//
//    Status = STATUS_SUCCESS;
//    if (pptDst && NT_SUCCESS(Status))
//    {
//       Status = SafeProbeForRead(pptDst, sizeof(POINT), 1);
//    }
//    if (pptSrc && NT_SUCCESS(Status))
//    {
//       Status = SafeProbeForRead(pptSrc, sizeof(POINT), 1);
//    }
//    if (NT_SUCCESS(Status))
//    {
//       Status = SafeProbeForRead(psize, sizeof(SIZE), 1);
//    }
//    if (pblend && NT_SUCCESS(Status))
//    {
//       Status = SafeProbeForRead(pblend, sizeof(BLENDFUNCTION), 1);
//    }
//    if (prcDirty && NT_SUCCESS(Status))
//    {
//       Status = SafeProbeForRead(prcDirty, sizeof(RECT), 1);
//    }
//    if (!NT_SUCCESS(Status))
//    {
//       EngSetLastError( ERROR_INVALID_PARAMETER );
//       goto Exit;
//    }
//    else
//    {
//       if (pptDst)
//       {
//          Dst = *pptDst;
//       }
//       if (pptSrc)
//       {
//          Src = *pptSrc;
//       }
//       Size = *psize;
//       if (pblend)
//       {
//          blend = *pblend;
//       }
//       if (prcDirty)
//       {
//          Dirty = *prcDirty;
//       }
//    }
//
//    if ( GetLayeredStatus(pWnd) ||
//         dwFlags & ~(ULW_COLORKEY | ULW_ALPHA | ULW_OPAQUE | ULW_EX_NORESIZE) ||
//        !(pWnd->ExStyle & WS_EX_LAYERED) )
//    {
//       ERR("Layered Window Invalid Parameters\n");
//       EngSetLastError( ERROR_INVALID_PARAMETER );
//       goto Exit;
//    }
//
//    info.cbSize   = sizeof(info);
//    info.hdcDst   = hdcDst;
//    info.pptDst   = pptDst? &Dst : NULL;
//    info.psize    = &Size;
//    info.hdcSrc   = hdcSrc;
//    info.pptSrc   = pptSrc ? &Src : NULL;
//    info.crKey    = crKey;
//    info.pblend   = &blend;
//    info.dwFlags  = dwFlags;
//    info.prcDirty = prcDirty ? &Dirty : NULL;
//    Ret = IntUpdateLayeredWindowI( pWnd, &info );
// Exit:
//    TRACE("Leave NtUserUpdateLayeredWindow, ret=%i\n", Ret);
//    UserLeave();
//    return Ret;
// }
//
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
