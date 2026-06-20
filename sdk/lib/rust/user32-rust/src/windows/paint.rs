//! windows/paint.c → Rust. user32 update-region / window-region / frame-draw client.
//!
//! Faithful port of win32ss/user/user32/windows/paint.c. Cohab-linked into
//! user32.dll via libuser32_rust.a; the user32-internal C helpers it references
//! (ValidateHwnd, the UserApiHook trio, guah, gpsi, gfServerProcess, …) still
//! live in the surviving C objects of the same DLL, so they are declared here as
//! plain extern imports with their C calling conventions.

use crate::ntwrapper::*;
use win32k_sys::win32k::*;

// ---------------------------------------------------------------------------
// gdi32.dll exports (stdcall) — see win32ss/gdi/gdi32/gdi32.spec
// ---------------------------------------------------------------------------
extern "system" {
    fn CreateSolidBrush(color: COLORREF) -> HBRUSH;
    fn CreateBitmap(
        nWidth: INT,
        nHeight: INT,
        nPlanes: UINT,
        nBitCount: UINT,
        lpBits: *const ::core::ffi::c_void,
    ) -> HBITMAP;
    fn CreatePatternBrush(hbm: HBITMAP) -> HBRUSH;
    fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    fn SetRectRgn(hrgn: HRGN, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
    fn CombineRgn(
        hrgnDest: HRGN,
        hrgnSrc1: HRGN,
        hrgnSrc2: HRGN,
        iMode: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn GetRgnBox(hrgn: HRGN, lprc: LPRECT) -> ::core::ffi::c_int;
    // gdi32 undocumented exports used by user32 (gdi32.spec: MirrorRgn/PolyPatBlt)
    fn MirrorRgn(hwnd: HWND, hrgn: HRGN) -> BOOL;
    fn PolyPatBlt(hdc: HDC, rop: DWORD, pRects: PPATRECT, cRects: INT, flags: ULONG) -> BOOL;
}

// ---------------------------------------------------------------------------
// kernel32.dll exports (stdcall)
// ---------------------------------------------------------------------------
extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    fn RtlIsThreadWithinLoaderCallout() -> BOOLEAN; // ntdll, NTAPI (stdcall on i386)
}

// ---------------------------------------------------------------------------
// user32-internal C symbols (still in the cohab C objects of this DLL).
// `FASTCALL` → `extern "fastcall"` on i386; data globals → plain extern statics.
// ---------------------------------------------------------------------------
extern "fastcall" {
    fn ValidateHwnd(hwnd: HWND) -> PWND;
    fn BeginIfHookedUserApiHook() -> BOOL;
    fn EndUserApiHook() -> BOOL;
    fn IsInsideUserApiHook() -> BOOL;
}

extern "C" {
    static mut guah: USERAPIHOOK;
    static gfServerProcess: BOOLEAN;
}

// RGB macro: (BYTE r) | (BYTE g << 8) | (BYTE b << 16)
#[inline]
unsafe fn RGB(r: u32, g: u32, b: u32) -> COLORREF {
    ((r & 0xff) | ((g & 0xff) << 8) | ((b & 0xff) << 16)) as COLORREF
}

// `static __inline void LoadUserApiHook()` from include/user_x.h — reproduced
// faithfully (it has no exported symbol to link against).
#[inline]
unsafe fn LoadUserApiHook() {
    if gfServerProcess == 0
        && IsInsideUserApiHook() == 0
        && !gpsi.is_null()
        && ((*gpsi).dwSRVIFlags & SRVINFO_APIHOOK) != 0
        && RtlIsThreadWithinLoaderCallout() == 0
    {
        NtUserCallNoParam(NOPARAM_ROUTINE_LOADUSERAPIHOOK as DWORD);
    }
}

// ---- file-scope statics (translated) ----
static mut FrameBrushes: [HBRUSH; 13] = [core::ptr::null_mut(); 13];
static mut hHatch: HBITMAP = core::ptr::null_mut();
pub static HatchBitmap: [DWORD; 4] = [0x5555AAAA, 0x5555AAAA, 0x5555AAAA, 0x5555AAAA];
pub static MappingTable: [BYTE; 33] = [
    5, 9, 2, 3, 5, 7, 0, 0, 0, 7, 5, 5, 3, 2, 7, 5, 3, 3, 0, 5, 7, 10, 5, 0, 11, 4, 1, 1, 3, 8, 6,
    12, 7,
];

/* FUNCTIONS *****************************************************************/

#[no_mangle]
pub unsafe extern "system" fn MirrorWindowRect(pWnd: PWND, lprc: LPRECT) -> INT {
    let Ret = (*pWnd).rcWindow.right - (*pWnd).rcWindow.left - (*lprc).left;
    (*lprc).left = (*pWnd).rcWindow.right - (*pWnd).rcWindow.left - (*lprc).right;
    (*lprc).right = Ret;
    Ret
}

unsafe fn CreateFrameBrushes() {
    FrameBrushes[0] = CreateSolidBrush(RGB(0, 0, 0));
    FrameBrushes[1] = CreateSolidBrush(RGB(0, 0, 128));
    FrameBrushes[2] = CreateSolidBrush(RGB(10, 36, 106));
    FrameBrushes[3] = CreateSolidBrush(RGB(128, 128, 128));
    FrameBrushes[4] = CreateSolidBrush(RGB(181, 181, 181));
    FrameBrushes[5] = CreateSolidBrush(RGB(212, 208, 200));
    FrameBrushes[6] = CreateSolidBrush(RGB(236, 233, 216));
    FrameBrushes[7] = CreateSolidBrush(RGB(255, 255, 255));
    FrameBrushes[8] = CreateSolidBrush(RGB(49, 106, 197));
    FrameBrushes[9] = CreateSolidBrush(RGB(58, 110, 165));
    FrameBrushes[10] = CreateSolidBrush(RGB(64, 64, 64));
    FrameBrushes[11] = CreateSolidBrush(RGB(255, 255, 225));
    hHatch = CreateBitmap(8, 8, 1, 1, HatchBitmap.as_ptr() as *const ::core::ffi::c_void);
    FrameBrushes[12] = CreatePatternBrush(hHatch);
}

#[allow(dead_code)]
#[no_mangle]
pub unsafe extern "C" fn DeleteFrameBrushes() {
    let count = FrameBrushes.len(); // sizeof(FrameBrushes)/sizeof(HBRUSH) == 13
    for Brush in 0..count {
        if !FrameBrushes[Brush].is_null() {
            DeleteObject(FrameBrushes[Brush] as HGDIOBJ);
            FrameBrushes[Brush] = core::ptr::null_mut();
        }
    }
    if !hHatch.is_null() {
        DeleteObject(hHatch as HGDIOBJ);
        hHatch = core::ptr::null_mut();
    }
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetUpdateRect(Wnd: HWND, Rect: LPRECT, Erase: BOOL) -> BOOL {
    let pWnd: PWND = ValidateHwnd(Wnd);
    if pWnd.is_null() {
        return FALSE as BOOL;
    }

    if !(*pWnd).hrgnUpdate.is_null()
        || ((*pWnd).state
            & (WNDS_SENDERASEBACKGROUND
                | WNDS_SENDNCPAINT
                | WNDS_UPDATEDIRTY
                | WNDS_PAINTNOTPROCESSED))
            != 0
    {
        return NtUserGetUpdateRect(Wnd, Rect as *mut _RECTL, Erase);
    }

    if !Rect.is_null() {
        // Did the Rgn update? No! Back set and shutup!
        (*Rect).left = 0;
        (*Rect).right = 0;
        (*Rect).top = 0;
        (*Rect).bottom = 0;
    }
    FALSE as BOOL // msdn: "If there is no update region, the return value is zero."
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetUpdateRgn(hWnd: HWND, hRgn: HRGN, bErase: BOOL) -> ::core::ffi::c_int {
    if hRgn.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return ERROR as ::core::ffi::c_int;
    }

    let pWnd: PWND = ValidateHwnd(hWnd);
    if pWnd.is_null() {
        return ERROR as ::core::ffi::c_int;
    }

    if !(*pWnd).hrgnUpdate.is_null()
        || ((*pWnd).state
            & (WNDS_SENDERASEBACKGROUND
                | WNDS_SENDNCPAINT
                | WNDS_UPDATEDIRTY
                | WNDS_PAINTNOTPROCESSED))
            != 0
    {
        return NtUserGetUpdateRgn(hWnd, hRgn, bErase);
    }
    SetRectRgn(hRgn, 0, 0, 0, 0);
    NULLREGION as ::core::ffi::c_int
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ScrollDC(
    hDC: HDC,
    dx: ::core::ffi::c_int,
    dy: ::core::ffi::c_int,
    lprcScroll: *const RECT,
    lprcClip: *const RECT,
    hrgnUpdate: HRGN,
    lprcUpdate: LPRECT,
) -> BOOL {
    if hDC.is_null() {
        return FALSE as BOOL;
    }

    if dx == 0 && dy == 0 {
        if !hrgnUpdate.is_null() {
            SetRectRgn(hrgnUpdate, 0, 0, 0, 0);
        }
        if !lprcUpdate.is_null() {
            (*lprcUpdate).left = 0;
            (*lprcUpdate).right = 0;
            (*lprcUpdate).top = 0;
            (*lprcUpdate).bottom = 0;
        }
        return TRUE as BOOL;
    }

    NtUserScrollDC(hDC, dx, dy, lprcScroll as _, lprcClip as _, hrgnUpdate, lprcUpdate as *mut _RECTL)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetWindowRgn(hWnd: HWND, hRgn: HRGN, bRedraw: BOOL) -> ::core::ffi::c_int {
    let Hook: BOOL;
    let mut Ret: ::core::ffi::c_int = 0;

    LoadUserApiHook();

    Hook = BeginIfHookedUserApiHook();

    /* Bypass SEH and go direct. */
    if Hook == 0 {
        Ret = NtUserSetWindowRgn(hWnd, hRgn, bRedraw);
        if Ret != 0 {
            DeleteObject(hRgn as HGDIOBJ);
        }
        return Ret;
    }

    // _SEH2_TRY { ... } _SEH2_EXCEPT(EXECUTE_HANDLER) {} — DUMMY_PSEH: handler is
    // dead code, only the guarded body runs.
    if let Some(f) = guah.SetWindowRgn {
        Ret = f(hWnd, hRgn, bRedraw);
    }

    EndUserApiHook();

    Ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn UpdateWindow(hWnd: HWND) -> BOOL {
    let pWnd: PWND = ValidateHwnd(hWnd);

    if pWnd.is_null() {
        return FALSE as BOOL;
    }

    if !(*pWnd).hrgnUpdate.is_null()
        || ((*pWnd).state & WNDS_INTERNALPAINT) != 0
        || !(*pWnd).spwndChild.is_null()
    {
        return NtUserxUpdateWindow(hWnd);
    }
    TRUE as BOOL
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ValidateRgn(hWnd: HWND, hRgn: HRGN) -> BOOL {
    NtUserxValidateRgn(hWnd, hRgn)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetWindowRgn(hWnd: HWND, hRgn: HRGN) -> ::core::ffi::c_int {
    if hRgn.is_null() {
        return ERROR as ::core::ffi::c_int;
    }

    let pWnd: PWND = ValidateHwnd(hWnd);

    if pWnd.is_null()
        || (*pWnd).hrgnClip.is_null()
        || ((*pWnd).state2 & WNDS2_MAXIMIZEDMONITORREGION) != 0
    {
        return ERROR as ::core::ffi::c_int;
    }

    let Ret = CombineRgn(hRgn, (*pWnd).hrgnClip, core::ptr::null_mut(), RGN_COPY as ::core::ffi::c_int);

    if Ret == 0 {
        return ERROR as ::core::ffi::c_int;
    }
    /*
      if (hWnd != GetDesktopWindow())
         Ret = OffsetRgn(hRgn, -pWnd->rcWindow.left, -pWnd->rcWindow.top);
    */
    if ((*pWnd).ExStyle & WS_EX_LAYOUTRTL) != 0 {
        MirrorRgn(hWnd, hRgn);
    }

    Ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetWindowRgnBox(hWnd: HWND, lprc: LPRECT) -> ::core::ffi::c_int {
    if lprc.is_null() {
        return ERROR as ::core::ffi::c_int;
    }

    let pWnd: PWND = ValidateHwnd(hWnd);

    if pWnd.is_null()
        || (*pWnd).hrgnClip.is_null()
        || ((*pWnd).state2 & WNDS2_MAXIMIZEDMONITORREGION) != 0
    {
        return ERROR as ::core::ffi::c_int;
    }

    let Ret = GetRgnBox((*pWnd).hrgnClip, lprc);

    if Ret == 0 {
        return ERROR as ::core::ffi::c_int;
    }
    /*
      if (hWnd != GetDesktopWindow())
         OffsetRect(lprc, -pWnd->rcWindow.left, -pWnd->rcWindow.top);
    */
    if ((*pWnd).ExStyle & WS_EX_LAYOUTRTL) != 0 {
        MirrorWindowRect(pWnd, lprc);
    }

    Ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn DrawFrame(hDc: HDC, r: *mut RECT, width: DWORD, type_: DWORD) -> BOOL {
    let rop: DWORD;
    let mut brush: DWORD;
    let hbrFrame: HBRUSH;
    let mut p: [PATRECT; 4] = core::mem::zeroed();

    if FrameBrushes[0].is_null() {
        CreateFrameBrushes();
    }
    if type_ & 4 != 0 {
        rop = PATINVERT;
    } else {
        rop = PATCOPY;
    }
    brush = type_ / 8;
    if brush >= 33 {
        brush = 32;
    }
    brush = MappingTable[brush as usize] as DWORD;
    hbrFrame = FrameBrushes[brush as usize];
    p[0].hBrush = hbrFrame;
    p[1].hBrush = hbrFrame;
    p[2].hBrush = hbrFrame;
    p[3].hBrush = hbrFrame;
    p[0].r.left = (*r).left;
    p[0].r.top = (*r).top;
    p[0].r.right = (*r).right - (*r).left;
    p[0].r.bottom = width as LONG;
    p[1].r.left = (*r).left;
    p[1].r.top = (*r).bottom - width as LONG;
    p[1].r.right = (*r).right - (*r).left;
    p[1].r.bottom = width as LONG;
    p[2].r.left = (*r).left;
    p[2].r.top = (*r).top + width as LONG;
    p[2].r.right = width as LONG;
    p[2].r.bottom = (*r).bottom - (*r).top - (width as LONG * 2);
    p[3].r.left = (*r).right - width as LONG;
    p[3].r.top = (*r).top + width as LONG;
    p[3].r.right = width as LONG;
    p[3].r.bottom = (*r).bottom - (*r).top - (width as LONG * 2);
    PolyPatBlt(hDc, rop, p.as_mut_ptr(), 4, 0)
}
