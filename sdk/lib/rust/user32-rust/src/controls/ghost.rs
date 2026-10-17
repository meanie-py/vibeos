use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::cursoricon::CreateDIBSection;
use crate::windows::window::HeapAlloc;
use crate::windows::message::HeapReAlloc;
// The crate's own functions: the definition is the declaration.
use crate::windows::class::GetWindowLongPtrW;
use crate::windows::cursoricon::LoadCursorW;
#[allow(unused_imports)]
use core::ffi::c_void;

// ── No-op debug macros (the Wine ERR/WARN/TRACE/FIXME channels compile to nothing). ──

// ── substrate / cross-module imports ──────────────────────────────────────────
use crate::internal::ValidateHwnd;
use crate::winbase::{DeleteObject, GetProcessHeap, NT_SUCCESS, SelectObject};
// These user32 entry points are inline ntwrapper wrappers (NtUser* multiplexers),
// not external link symbols — import them instead of declaring them extern.
use crate::ntwrapper::{
    BeginPaint, EndPaint, GetWindowDC, DestroyWindow, InvalidateRect, KillTimer, SetTimer,
    MoveWindow, SetWindowPos, ShowWindowAsync, SetCursor,
};

// User32Instance lives in dllmain (cross-TU data symbol).
extern "C" {
    static User32Instance: HINSTANCE;
}

// gdi32 exports (stdcall on i386).
extern "system" {
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn DeleteDC(hdc: HDC) -> i32;
    fn BitBlt( hdcDest: HDC, x: INT, y: INT, cx: INT, cy: INT, hdcSrc: HDC, x1: INT, y1: INT, rop: u32, ) -> i32;
    fn GetObjectW(h: HGDIOBJ, c: i32, pv: LPVOID) -> i32;
}

// user32 public exports referenced here (resolve within this DLL).
extern "system" {
    fn ReleaseDC(hwnd: HWND, hdc: HDC) -> i32;
    fn DefWindowProcW(hwnd: HWND, msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcA(hwnd: HWND, msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR;
    fn GetClassLongPtrW(hWnd: HWND, nIndex: INT) -> ULONG_PTR;
    fn IsWindow(hWnd: HWND) -> BOOL;
    fn IsWindowVisible(hWnd: HWND) -> BOOL;
    fn IsHungAppWindow(hWnd: HWND) -> BOOL;
    fn GetPropW(hWnd: HWND, lpString: LPCWSTR) -> *mut core::ffi::c_void;
    fn SetPropW(hWnd: HWND, lpString: LPCWSTR, hData: *mut core::ffi::c_void) -> BOOL;
    fn RemovePropW(hWnd: HWND, lpString: LPCWSTR) -> *mut core::ffi::c_void;
    fn GetWindowRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;
    fn GetWindow(hWnd: HWND, uCmd: UINT) -> HWND;
    fn SetWindowTextW(hWnd: HWND, lpString: LPCWSTR) -> BOOL;
    fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut DWORD) -> DWORD;
    fn PostQuitMessage(nExitCode: INT);
    fn MessageBoxW(hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT) -> INT;
    fn LoadStringW(hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: INT) -> INT;
    fn InternalGetWindowText(hWnd: HWND, pString: LPWSTR, cchMaxCount: INT) -> INT;
}

// kernel32 exports.
extern "system" {
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
    fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> *mut core::ffi::c_void;
    fn TerminateProcess(hProcess: *mut core::ffi::c_void, uExitCode: UINT) -> BOOL;
    fn CloseHandle(hObject: *mut core::ffi::c_void) -> BOOL;
}

// strsafe: StringCchCatW — faithful inline reproduction.
unsafe fn StringCchCatW(psz_dest: LPWSTR, cch_dest: usize, psz_src: LPCWSTR) -> HRESULT {
    if psz_dest.is_null() || cch_dest == 0 {
        return -2147024809; // STG_E_INVALIDPARAMETER (0x80070057)
    }
    let mut i = 0usize;
    while i < cch_dest && *psz_dest.add(i) != 0 {
        i += 1;
    }
    if i >= cch_dest {
        return -2147024809; // STRSAFE_E_INVALID_PARAMETER (0x80070057)
    }
    let mut s = psz_src;
    while i < cch_dest - 1 && !s.is_null() && *s != 0 {
        *psz_dest.add(i) = *s;
        s = s.add(1);
        i += 1;
    }
    *psz_dest.add(i) = 0;
    if !s.is_null() && *s != 0 {
        return -2147024774; // STRSAFE_E_INSUFFICIENT_BUFFER
    }
    0 // S_OK
}

// ---- file-scope preamble (translated) ----
pub const GHOST_TIMER_ID: u32 = 0xFACEDEAD;

pub const GHOST_INTERVAL: u32 = 1000;

// resource IDs (win32ss/user/user32/include/resource.h)
const IDS_NOT_RESPONDING: UINT = 3;
const IDS_ASK_TERMINATE: UINT = 4;
const IDS_HUNG_UP_TITLE: UINT = 5;

// GHOST_PROP — L"GhostProp" (win32ss/include/ghostwnd.h)
static GHOST_PROP_BUF: [u16; 10] = [b'G' as u16, b'h' as u16, b'o' as u16, b's' as u16, b't' as u16, b'P' as u16, b'r' as u16, b'o' as u16, b'p' as u16, 0];
#[allow(non_snake_case)]
unsafe fn GHOST_PROP() -> LPCWSTR { GHOST_PROP_BUF.as_ptr() }

// L"Ghost" class name.
static GHOST_CLASSNAME: [u16; 6] = [b'G' as u16, b'h' as u16, b'o' as u16, b's' as u16, b't' as u16, 0];

// IDC_WAIT cursor ordinal.
const IDC_WAIT: LPCWSTR = 32514usize as LPCWSTR;

// GHOST_DATA — faithful to win32ss/include/ghostwnd.h
#[repr(C)]
pub struct GHOST_DATA {
    pub hwndTarget: HWND,
    pub hbm32bpp: HBITMAP,
    pub bDestroyTarget: BOOL,
}

#[repr(C)]
pub struct builtin_class_descr {
    pub name: *const u16,
    pub style: UINT,
    pub procA: WNDPROC,
    pub procW: WNDPROC,
    pub extra: INT,
    pub cursor: LPCWSTR,
    pub brush: HBRUSH,
}
unsafe impl Sync for builtin_class_descr {}

#[no_mangle]
pub static GHOST_builtin_class: builtin_class_descr = builtin_class_descr {
    name: GHOST_CLASSNAME.as_ptr(),
    style: 0,
    procA: unsafe {
        core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(GhostWndProcA)
    },
    procW: unsafe {
        core::mem::transmute::<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT, WNDPROC>(GhostWndProcW)
    },
    extra: 0,
    cursor: IDC_WAIT,
    brush: core::ptr::null_mut(),
};

unsafe fn IntCreate32BppBitmap(cx: INT, cy: INT) -> HBITMAP {
    let mut hbm: HBITMAP = core::ptr::null_mut();
    let mut bi: BITMAPINFO = core::mem::zeroed();
    let hdc: HDC;
    let mut pvBits: LPVOID = core::ptr::null_mut();

    core::ptr::write_bytes(&mut bi as *mut BITMAPINFO, 0, 1);
    bi.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bi.bmiHeader.biWidth = cx;
    bi.bmiHeader.biHeight = cy;
    bi.bmiHeader.biPlanes = 1;
    bi.bmiHeader.biBitCount = 32;

    hdc = CreateCompatibleDC(core::ptr::null_mut());
    if !hdc.is_null() {
        hbm = CreateDIBSection(
            hdc,
            &mut bi,
            DIB_RGB_COLORS,
            &mut pvBits,
            core::ptr::null_mut(),
            0,
        );
        DeleteDC(hdc);
    }
    hbm
}

unsafe fn IntGetWindowBitmap(hwnd: HWND, cx: INT, cy: INT) -> HBITMAP {
    let mut hbm: HBITMAP = core::ptr::null_mut();
    let hdc: HDC;
    let hdcMem: HDC;
    let hbmOld: HGDIOBJ;

    hdc = GetWindowDC(hwnd);
    if hdc.is_null() {
        return core::ptr::null_mut();
    }

    'earth: {
        hdcMem = CreateCompatibleDC(hdc);
        if hdcMem.is_null() {
            break 'earth;
        }

        hbm = IntCreate32BppBitmap(cx, cy);
        if !hbm.is_null() {
            hbmOld = SelectObject(hdcMem, hbm as HGDIOBJ);
            BitBlt(
                hdcMem,
                0,
                0,
                cx,
                cy,
                hdc,
                0,
                0,
                SRCCOPY | CAPTUREBLT,
            );
            SelectObject(hdcMem, hbmOld);
        }

        DeleteDC(hdcMem);
    }

    ReleaseDC(hwnd, hdc);

    hbm
}

unsafe fn IntMakeGhostImage(hbm: HBITMAP) {
    let mut bm: BITMAP = core::mem::zeroed();

    GetObjectW(hbm as HGDIOBJ, core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bm) as LPVOID);

    if bm.bmBitsPixel != 32 || bm.bmBits.is_null() {
        ERR!("bm.bmBitsPixel == %d, bm.bmBits == %p\n",
            bm.bmBitsPixel as i32, bm.bmBits);
        return;
    }

    let mut pdw = bm.bmBits as *mut u32;
    let mut i: u32 = 0;
    let count = (bm.bmWidth * bm.bmHeight) as u32;
    while i < count {
        *pdw = *pdw | 0x00C0C0C0;
        pdw = pdw.add(1);
        i += 1;
    }
}

unsafe fn Ghost_GetData(hwnd: HWND) -> *mut GHOST_DATA {
    GetWindowLongPtrW(hwnd, GWLP_USERDATA as i32) as *mut GHOST_DATA
}

unsafe fn Ghost_GetTarget(hwnd: HWND) -> HWND {
    let pData: *mut GHOST_DATA = Ghost_GetData(hwnd);
    if pData.is_null() {
        return core::ptr::null_mut();
    }
    (*pData).hwndTarget
}

unsafe fn Ghost_GetText(hwndTarget: HWND, pcchTextW: *mut INT, cchExtra: INT) -> LPWSTR {
    let mut pszTextW: LPWSTR = core::ptr::null_mut();
    let mut pszTextNewW: LPWSTR;
    let mut cchNonExtra: INT;
    let mut cchTextW: INT = *pcchTextW;

    pszTextNewW = HeapAlloc(GetProcessHeap(), 0, (cchTextW as usize * core::mem::size_of::<WCHAR>()) as _) as LPWSTR;
    loop {
        if pszTextNewW.is_null() {
            ERR!("HeapAlloc failed\n");
            if !pszTextW.is_null() {
                HeapFree(GetProcessHeap(), 0, pszTextW as _);
            }
            return core::ptr::null_mut();
        }
        pszTextW = pszTextNewW;

        cchNonExtra = cchTextW - cchExtra;
        if InternalGetWindowText(hwndTarget, pszTextW, cchNonExtra) < cchNonExtra - 1 {
            break;
        }

        cchTextW *= 2;
        pszTextNewW = HeapReAlloc(GetProcessHeap(), 0, pszTextW as _, (cchTextW as usize * core::mem::size_of::<WCHAR>()) as _) as LPWSTR;
    }

    *pcchTextW = cchTextW;
    pszTextW
}

unsafe fn Ghost_OnCreate(hwnd: HWND, lpcs: *mut CREATESTRUCTW) -> BOOL {
    let hbm32bpp: HBITMAP;
    let hwndTarget: HWND;
    let hwndPrev: HWND;
    let pData: *mut GHOST_DATA;
    let mut rc: RECT = core::mem::zeroed();
    let mut style: DWORD;
    let exstyle: DWORD;
    let mut szTextW: [WCHAR; 320] = [0; 320];
    let mut szNotRespondingW: [WCHAR; 32] = [0; 32];
    let mut pszTextW: LPWSTR;
    let mut cchTextW: INT;
    let cchExtraW: INT;
    let cchNonExtraW: INT;

    let pWnd: PWND = ValidateHwnd(hwnd);
    if !pWnd.is_null() {
        if (*pWnd).fnid == 0 {
            NtUserSetWindowFNID(hwnd, FNID_GHOST as u16);
        } else if (*pWnd).fnid != FNID_GHOST as DWORD {
            ERR!("Wrong window class for Ghost! fnId 0x%x\n", (*pWnd).fnid as u32);
            return 0;
        }
    }

    hwndTarget = (*lpcs).lpCreateParams as HWND;
    if IsWindowVisible(hwndTarget) == 0
        || (GetWindowLongPtrW(hwndTarget, GWL_STYLE) & WS_CHILD as LONG_PTR) != 0
        || IsHungAppWindow(hwndTarget) == 0
    {
        return 0;
    }

    if !GetPropW(hwndTarget, GHOST_PROP()).is_null() {
        return 0;
    }

    SetPropW(hwndTarget, GHOST_PROP(), hwnd as *mut core::ffi::c_void);

    pData = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<GHOST_DATA>()) as SIZE_T) as *mut GHOST_DATA;
    if pData.is_null() {
        ERR!("HeapAlloc failed\n");
        return 0;
    }

    GetWindowRect(hwndTarget, &mut rc);
    hbm32bpp = IntGetWindowBitmap(hwndTarget, rc.right - rc.left, rc.bottom - rc.top);
    if hbm32bpp.is_null() {
        ERR!("hbm32bpp was NULL\n");
        HeapFree(GetProcessHeap(), 0, pData as *mut core::ffi::c_void);
        return 0;
    }

    IntMakeGhostImage(hbm32bpp);

    (*pData).hwndTarget = hwndTarget;
    (*pData).hbm32bpp = hbm32bpp;
    (*pData).bDestroyTarget = 0;
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, pData as LONG_PTR);

    style = GetWindowLongPtrW(hwnd, GWL_STYLE) as DWORD;
    style = GetWindowLongPtrW(hwndTarget, GWL_STYLE) as DWORD;
    exstyle = GetWindowLongPtrW(hwndTarget, GWL_EXSTYLE) as DWORD;

    cchTextW = szTextW.len() as INT;
    cchExtraW = szNotRespondingW.len() as INT;
    cchNonExtraW = cchTextW - cchExtraW;
    if InternalGetWindowText(hwndTarget, szTextW.as_mut_ptr(), cchNonExtraW) < cchNonExtraW - 1 {
        pszTextW = szTextW.as_mut_ptr();
    } else {
        cchTextW *= 2;
        pszTextW = Ghost_GetText(hwndTarget, &mut cchTextW, cchExtraW);
        if pszTextW.is_null() {
            ERR!("Ghost_GetText failed\n");
            DeleteObject(hbm32bpp as HGDIOBJ);
            HeapFree(GetProcessHeap(), 0, pData as *mut core::ffi::c_void);
            return 0;
        }
    }

    style &= !(WS_HSCROLL | WS_VSCROLL | WS_VISIBLE);

    SetWindowLongPtrW(hwnd, GWL_STYLE, style as LONG_PTR);
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, exstyle as LONG_PTR);

    LoadStringW(User32Instance, IDS_NOT_RESPONDING, szNotRespondingW.as_mut_ptr(), szNotRespondingW.len() as INT);
    StringCchCatW(pszTextW, cchTextW as usize, szNotRespondingW.as_ptr());
    SetWindowTextW(hwnd, pszTextW);

    if szTextW.as_mut_ptr() != pszTextW {
        HeapFree(GetProcessHeap(), 0, pszTextW as *mut core::ffi::c_void);
    }

    hwndPrev = GetWindow(hwndTarget, GW_HWNDPREV);

    ShowWindowAsync(hwndTarget, SW_HIDE as INT);

    SetWindowPos(hwnd, hwndPrev, 0, 0, 0, 0,
                 (SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_DRAWFRAME) as u32);

    MoveWindow(hwnd, rc.left, rc.top, rc.right - rc.left, rc.bottom - rc.top, 1);

    SetWindowLongPtrW(hwnd, GWL_STYLE, (style | WS_VISIBLE) as LONG_PTR);

    InvalidateRect(hwnd, core::ptr::null_mut(), 1);

    SetTimer(hwnd, 0xFACEDEAD, 1000, None);

    1
}

unsafe fn Ghost_Unenchant(hwnd: HWND, bDestroyTarget: BOOL) {
    let pData = Ghost_GetData(hwnd);
    if pData.is_null() {
        return;
    }

    (*pData).bDestroyTarget |= bDestroyTarget;
    DestroyWindow(hwnd);
}

unsafe fn Ghost_OnDraw(hwnd: HWND, hdc: HDC) {
    let mut bm: BITMAP = core::mem::zeroed();
    let hdcMem: HDC;
    let pData = Ghost_GetData(hwnd);

    if pData.is_null()
        || GetObjectW(
            (*pData).hbm32bpp as HGDIOBJ,
            core::mem::size_of::<BITMAP>() as i32,
            core::ptr::addr_of_mut!(bm) as LPVOID,
        ) == 0
    {
        return;
    }

    hdcMem = CreateCompatibleDC(hdc);
    if !hdcMem.is_null() {
        let hbmOld = SelectObject(hdcMem, (*pData).hbm32bpp as HGDIOBJ);
        BitBlt(
            hdc,
            0,
            0,
            bm.bmWidth,
            bm.bmHeight,
            hdcMem,
            0,
            0,
            SRCCOPY | CAPTUREBLT,
        );
        SelectObject(hdcMem, hbmOld);
        DeleteDC(hdcMem);
    }
}

unsafe fn Ghost_OnNCPaint(hwnd: HWND, hrgn: HRGN, bUnicode: i32) {

    let hdc: HDC;

    if bUnicode != 0 {
        DefWindowProcW(hwnd, WM_NCPAINT, hrgn as WPARAM, 0);
    } else {
        DefWindowProcA(hwnd, WM_NCPAINT, hrgn as WPARAM, 0);
    }

    hdc = GetWindowDC(hwnd);
    if !hdc.is_null() {
        Ghost_OnDraw(hwnd, hdc);
        ReleaseDC(hwnd, hdc);
    }
}

unsafe fn Ghost_OnPaint(hwnd: HWND) {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let hdc = BeginPaint(hwnd, &mut ps);
    if !hdc.is_null() {
        EndPaint(hwnd, &mut ps);
    }
}

unsafe fn Ghost_OnMove(hwnd: HWND, _x: core::ffi::c_int, _y: core::ffi::c_int) {
    let mut rc: RECT = core::mem::zeroed();
    let hwndTarget = Ghost_GetTarget(hwnd);

    GetWindowRect(hwnd, core::ptr::addr_of_mut!(rc));

    SetWindowPos(
        hwndTarget,
        core::ptr::null_mut(),
        rc.left,
        rc.top,
        0,
        0,
        (SWP_NOSIZE | SWP_NOOWNERZORDER | SWP_NOACTIVATE | SWP_NOSENDCHANGING) as u32,
    );
}

unsafe fn Ghost_OnDestroy(hwnd: HWND) {
    KillTimer(hwnd, 0xFACEDEAD);
}

unsafe fn Ghost_DestroyTarget(pData: *mut GHOST_DATA) {
    let hwndTarget = (*pData).hwndTarget;
    let mut pid: DWORD = 0;

    (*pData).hwndTarget = core::ptr::null_mut();
    GetWindowThreadProcessId(hwndTarget, &mut pid);

    let hProcess = OpenProcess(PROCESS_TERMINATE, 0, pid);
    if !hProcess.is_null() {
        TerminateProcess(hProcess, -1i32 as u32);
        CloseHandle(hProcess);
    }

    DestroyWindow(hwndTarget);
}

unsafe fn Ghost_OnNCDestroy(hwnd: HWND) {
    let pData: *mut GHOST_DATA = Ghost_GetData(hwnd);
    if !pData.is_null() {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);

        DeleteObject((*pData).hbm32bpp as HGDIOBJ);
        (*pData).hbm32bpp = core::ptr::null_mut();

        RemovePropW((*pData).hwndTarget, GHOST_PROP());

        ShowWindowAsync((*pData).hwndTarget, SW_SHOWNOACTIVATE as INT);

        if (*pData).bDestroyTarget != 0 {
            Ghost_DestroyTarget(pData);
        }

        HeapFree(GetProcessHeap(), 0, pData as LPVOID);
    }

    NtUserSetWindowFNID(hwnd, FNID_DESTROY as u16);

    PostQuitMessage(0);
}

unsafe fn Ghost_OnClose(hwnd: HWND) {
    let id: INT;
    let mut szAskTerminate: [WCHAR; 128] = [0; 128];
    let mut szHungUpTitle: [WCHAR; 128] = [0; 128];

    // stop timer
    KillTimer(hwnd, GHOST_TIMER_ID as _);

    LoadStringW(
        User32Instance,
        IDS_ASK_TERMINATE,
        szAskTerminate.as_mut_ptr(),
        szAskTerminate.len() as INT,
    );
    LoadStringW(
        User32Instance,
        IDS_HUNG_UP_TITLE,
        szHungUpTitle.as_mut_ptr(),
        szHungUpTitle.len() as INT,
    );

    id = MessageBoxW(
        hwnd,
        szAskTerminate.as_ptr(),
        szHungUpTitle.as_ptr(),
        (MB_ICONINFORMATION | MB_YESNO) as _,
    );
    if id == IDYES as INT {
        // destroy the target
        Ghost_Unenchant(hwnd, 1);
        return;
    }

    // restart timer
    SetTimer(hwnd, GHOST_TIMER_ID as _, GHOST_INTERVAL as _, None);
}

unsafe fn Ghost_OnTimer(hwnd: HWND, id: UINT) {
    let pData = Ghost_GetData(hwnd);

    if id != 0xFACEDEAD || pData.is_null() {
        return;
    }

    KillTimer(hwnd, id as UINT_PTR);

    let hwndTarget = (*pData).hwndTarget;
    if IsWindow(hwndTarget) == 0 || IsHungAppWindow(hwndTarget) == 0 {
        Ghost_Unenchant(hwnd, 0);
        return;
    }

    SetTimer(hwnd, 0xFACEDEAD, 1000, None);
}

unsafe fn Ghost_GetIcon(hwnd: HWND, fType: INT) -> HICON {
    let pData: *mut GHOST_DATA = Ghost_GetData(hwnd);
    let mut hIcon: HICON = core::ptr::null_mut();

    if pData.is_null() {
        return core::ptr::null_mut();
    }

    match fType as u32 {
        ICON_BIG => {
            hIcon = GetClassLongPtrW((*pData).hwndTarget, GCLP_HICON as INT) as HICON;
        }
        ICON_SMALL => {
            hIcon = GetClassLongPtrW((*pData).hwndTarget, GCLP_HICONSM as INT) as HICON;
        }
        _ => {}
    }

    hIcon
}

#[no_mangle]
pub unsafe extern "system" fn GhostWndProc_common(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    unicode: BOOL,
) -> LRESULT {

    match uMsg {
        WM_CREATE => {
            if Ghost_OnCreate(hwnd, lParam as *mut CREATESTRUCTW) == 0 {
                return -1;
            }
        }

        WM_NCPAINT => {
            Ghost_OnNCPaint(hwnd, wParam as HRGN, unicode);
            return 0;
        }

        WM_ERASEBKGND => {
            Ghost_OnDraw(hwnd, wParam as HDC);
            return 1;
        }

        WM_PAINT => {
            Ghost_OnPaint(hwnd);
        }

        WM_MOVE => {
            Ghost_OnMove(
                hwnd,
                (lParam & 0xFFFF) as i16 as core::ffi::c_int,
                ((lParam >> 16) & 0xFFFF) as i16 as core::ffi::c_int,
            );
        }

        WM_SIZE => {}

        WM_SIZING => {
            return 1;
        }

        WM_SYSCOMMAND => {
            match wParam as UINT {
                SC_MAXIMIZE | SC_SIZE => {
                    return 0;
                }
                _ => {}
            }
            if unicode != 0 {
                return DefWindowProcW(hwnd, uMsg, wParam, lParam);
            } else {
                return DefWindowProcA(hwnd, uMsg, wParam, lParam);
            }
        }

        WM_CLOSE => {
            Ghost_OnClose(hwnd);
        }

        WM_TIMER => {
            Ghost_OnTimer(hwnd, wParam as UINT);
        }

        WM_NCMOUSEMOVE => {
            if unicode != 0 {
                DefWindowProcW(hwnd, uMsg, wParam, lParam);
            } else {
                DefWindowProcA(hwnd, uMsg, wParam, lParam);
            }
            SetCursor(LoadCursorW(core::ptr::null_mut(), IDC_WAIT) as HCURSOR);
            return 0;
        }

        WM_GETICON => {
            return Ghost_GetIcon(hwnd, wParam as INT) as LRESULT;
        }

        WM_COMMAND => {
            if (wParam & 0xFFFF) as u16 == 3333 {
                Ghost_Unenchant(hwnd, 0);
            }
        }

        WM_DESTROY => {
            Ghost_OnDestroy(hwnd);
        }

        WM_NCDESTROY => {
            Ghost_OnNCDestroy(hwnd);
        }

        _ => {
            if unicode != 0 {
                return DefWindowProcW(hwnd, uMsg, wParam, lParam);
            } else {
                return DefWindowProcA(hwnd, uMsg, wParam, lParam);
            }
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn GhostWndProcA(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    GhostWndProc_common(hwnd, uMsg, wParam, lParam, 0)
}

unsafe extern "system" fn GhostWndProcW(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    GhostWndProc_common(hwnd, uMsg, wParam, lParam, 1)
}
